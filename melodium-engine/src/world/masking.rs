use aho_corasick::{AhoCorasick, MatchKind};
use base64::Engine;
use melodium_common::executive::{PackedArray, Value};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::RwLock;

/// Shortest text masked, shorter ones would mask ordinary words.
pub const MIN_MASKED_LENGTH: usize = 8;

/// Every character except the unreserved ones of RFC 3986 (`A-Z a-z 0-9 - . _ ~`),
/// as `std/secret::|url_encode` does.
const URL_COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

#[derive(Default)]
struct Patterns {
    /// Masked texts, and the name of the secret they come from.
    names: HashMap<String, String>,
    matcher: Option<(AhoCorasick, Vec<String>)>,
}

/// Registry of revealed values, masked in text outputs such as log messages.
///
/// Each value is masked as is, line by line when it has several lines,
/// and in its base64 and URL-encoded forms.
#[derive(Default)]
pub struct Masking {
    patterns: RwLock<Patterns>,
}

impl Masking {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a value revealed from the secret named `name`.
    ///
    /// Only text and bytes values are registered.
    pub fn add(&self, name: &str, value: &Value) {
        let texts = Self::forms(value);
        if texts.is_empty() {
            return;
        }

        let mut patterns = self.patterns.write().unwrap();
        let mut added = false;
        for text in texts {
            if !patterns.names.contains_key(&text) {
                patterns.names.insert(text, name.to_string());
                added = true;
            }
        }
        if added {
            let (texts, replacements): (Vec<&String>, Vec<String>) = patterns
                .names
                .iter()
                .map(|(text, name)| (text, format!("<secret {name:?}>")))
                .unzip();
            let matcher = AhoCorasick::builder()
                .match_kind(MatchKind::LeftmostLongest)
                .build(texts)
                .ok()
                .map(|matcher| (matcher, replacements));
            patterns.matcher = matcher;
        }
    }

    /// Gives `text` with every registered value replaced by a placeholder naming its secret.
    pub fn mask<'a>(&self, text: &'a str) -> Cow<'a, str> {
        let patterns = self.patterns.read().unwrap();
        match &patterns.matcher {
            Some((matcher, replacements)) if matcher.is_match(text) => {
                Cow::Owned(matcher.replace_all(text, replacements))
            }
            _ => Cow::Borrowed(text),
        }
    }

    /// Forgets every registered value.
    pub fn clear(&self) {
        *self.patterns.write().unwrap() = Patterns::default();
    }

    /// Gives the texts to mask for `value`.
    fn forms(value: &Value) -> Vec<String> {
        let bytes: Cow<'_, [u8]> = match value {
            Value::String(text) => Cow::Borrowed(text.as_bytes()),
            Value::Packed(PackedArray::Byte(bytes)) | Value::Packed(PackedArray::U8(bytes)) => {
                Cow::Borrowed(bytes.as_slice())
            }
            Value::Vec(values) => {
                let bytes: Option<Vec<u8>> = values
                    .iter()
                    .map(|value| match value {
                        Value::Byte(byte) | Value::U8(byte) => Some(*byte),
                        _ => None,
                    })
                    .collect();
                match bytes {
                    Some(bytes) => Cow::Owned(bytes),
                    None => return Vec::new(),
                }
            }
            _ => return Vec::new(),
        };

        let mut forms = Vec::new();
        // Without padding, so that padded occurrences are masked too.
        forms.push(base64::engine::general_purpose::STANDARD_NO_PAD.encode(&bytes));
        if let Ok(text) = std::str::from_utf8(&bytes) {
            forms.push(text.to_string());
            forms.push(utf8_percent_encode(text, URL_COMPONENT).to_string());
            if text.contains('\n') {
                forms.extend(
                    text.lines()
                        .map(|line| line.trim_end_matches('\r').to_string()),
                );
            }
        }
        forms.retain(|form| form.len() >= MIN_MASKED_LENGTH);
        forms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn masks_values_and_their_encoded_forms() {
        let masking = Masking::new();
        masking.add("token", &Value::String("t0k:en/value@1".to_string()));

        assert_eq!(
            masking.mask("Authorization: Bearer t0k:en/value@1 refused"),
            "Authorization: Bearer <secret \"token\"> refused"
        );
        // base64 of "t0k:en/value@1", padded
        assert_eq!(
            masking.mask("basic dDBrOmVuL3ZhbHVlQDE="),
            "basic <secret \"token\">="
        );
        assert_eq!(
            masking.mask("https://ci:t0k%3Aen%2Fvalue%401@host"),
            "https://ci:<secret \"token\">@host"
        );
    }

    #[test]
    fn masks_lines_of_multiline_values() {
        let masking = Masking::new();
        masking.add(
            "key",
            &Value::String("-----BEGIN KEY-----\r\nMIIEvQIBADANBgkq\nshort\n".to_string()),
        );
        assert_eq!(
            masking.mask("parse error near MIIEvQIBADANBgkq"),
            "parse error near <secret \"key\">"
        );
        assert_eq!(masking.mask("a short line"), "a short line");
    }

    #[test]
    fn masks_bytes_values() {
        let masking = Masking::new();
        masking.add(
            "bytes",
            &Value::Packed(PackedArray::Byte(Arc::new(b"binary-secret".to_vec()))),
        );
        assert_eq!(masking.mask("got binary-secret"), "got <secret \"bytes\">");
    }

    #[test]
    fn ignores_short_and_non_text_values() {
        let masking = Masking::new();
        masking.add("short", &Value::String("abc".to_string()));
        masking.add("number", &Value::U64(12345678901234));
        assert!(matches!(
            masking.mask("abc 12345678901234"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn prefers_the_longest_value() {
        let masking = Masking::new();
        masking.add("inner", &Value::String("password".to_string()));
        masking.add("outer", &Value::String("Bearer password-long".to_string()));
        assert_eq!(
            masking.mask("Bearer password-long and password"),
            "<secret \"outer\"> and <secret \"inner\">"
        );
    }

    /// Measures the cost of masking log lines, for registries of growing size.
    /// Run with `cargo test --release -p melodium-engine masking_overhead -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn masking_overhead() {
        const LINES: usize = 1_000_000;
        let lines: Vec<String> = (0..LINES)
            .map(|i| {
                format!(
                    "[track {i}] request to https://gitlab.com/group/project/-/jobs/{i} finished with status 200 in {}ms",
                    i % 977
                )
            })
            .collect();

        for secrets in [0, 1, 10, 100] {
            let masking = Masking::new();
            for i in 0..secrets {
                masking.add(
                    &format!("secret_{i}"),
                    &Value::String(format!("glpat-{i:04}-Xk9vQ2mTzR7wLp3N")),
                );
            }
            let start = std::time::Instant::now();
            let mut masked = 0;
            for line in &lines {
                if let Cow::Owned(_) = masking.mask(line) {
                    masked += 1;
                }
            }
            let elapsed = start.elapsed();
            println!(
                "{secrets:>3} secrets: {:>6.1} ns per line ({LINES} lines, {masked} masked)",
                elapsed.as_nanos() as f64 / LINES as f64
            );
        }
    }

    #[test]
    fn clear_forgets_values() {
        let masking = Masking::new();
        masking.add("token", &Value::String("t0k:en/value@1".to_string()));
        masking.clear();
        assert_eq!(masking.mask("t0k:en/value@1"), "t0k:en/value@1");
    }
}
