use aho_corasick::{AhoCorasick, MatchKind};
use base64::Engine;
use melodium_common::executive::{count_wiped, PackedArray, RevealedValues, Value, Wipe};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::RwLock;
use zeroize::Zeroize;

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
    /// Values searched in data leaving the engine, and the name of the secret they come from.
    searched: HashMap<Vec<u8>, String>,
    searcher: Option<Searcher>,
}

/// Search of the revealed values in data, giving the name of their secret.
struct Searcher {
    automaton: AhoCorasick,
    names: Vec<String>,
    longest: usize,
}

/// Registry of revealed values, masked in text outputs such as log messages.
///
/// Each value is masked as is, line by line when it has several lines,
/// and in its base64 and URL-encoded forms.
///
/// The same values are searched in data leaving the engine, such as uploaded files,
/// as is, in base64 and URL-encoded, but never line by line.
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
        let (texts, searched) = Self::forms(value);
        if texts.is_empty() && searched.is_empty() {
            return;
        }

        let mut patterns = self.patterns.write().unwrap();
        let mut added = false;
        for mut text in texts {
            if patterns.names.contains_key(&text) {
                text.zeroize();
            } else {
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

        let mut added = false;
        for mut bytes in searched {
            if patterns.searched.contains_key(&bytes) {
                bytes.zeroize();
            } else {
                patterns.searched.insert(bytes, name.to_string());
                added = true;
            }
        }
        if added {
            let (values, names): (Vec<&Vec<u8>>, Vec<String>) = patterns
                .searched
                .iter()
                .map(|(bytes, name)| (bytes, name.clone()))
                .unzip();
            let longest = values.iter().map(|bytes| bytes.len()).max().unwrap_or(0);
            let searcher = AhoCorasick::builder()
                .build(values)
                .ok()
                .map(|automaton| Searcher {
                    automaton,
                    names,
                    longest,
                });
            patterns.searcher = searcher;
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

    /// Forgets every registered value, overwriting them.
    ///
    /// The matcher built from them is dropped without being overwritten,
    /// its memory being out of reach.
    pub fn clear(&self) {
        self.wipe();
    }

    /// Gives the texts to mask for `value`, and the bytes to search for it in data.
    fn forms(value: &Value) -> (Vec<String>, Vec<Vec<u8>>) {
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
                    None => return (Vec::new(), Vec::new()),
                }
            }
            _ => return (Vec::new(), Vec::new()),
        };

        let mut forms = Vec::new();
        let mut lines = Vec::new();
        // Without padding, so that padded occurrences are masked too.
        forms.push(base64::engine::general_purpose::STANDARD_NO_PAD.encode(&bytes));
        let raw = match std::str::from_utf8(&bytes) {
            Ok(text) => {
                forms.push(text.to_string());
                forms.push(utf8_percent_encode(text, URL_COMPONENT).to_string());
                if text.contains('\n') {
                    lines.extend(
                        text.lines()
                            .map(|line| line.trim_end_matches('\r').to_string()),
                    );
                }
                None
            }
            Err(_) => Some(bytes.to_vec()),
        };
        if let Cow::Owned(mut bytes) = bytes {
            bytes.zeroize();
        }

        // Lines are not searched: ordinary lines of a multi-line value, such as
        // `apiVersion: v1` or the armor of a key, would be found in any data.
        let searched = forms
            .iter()
            .map(|form| form.as_bytes().to_vec())
            .chain(raw)
            .filter_map(Self::long_enough)
            .collect();
        let masked = forms
            .into_iter()
            .chain(lines)
            .filter_map(Self::long_enough)
            .collect();
        (masked, searched)
    }

    /// Gives `form` if it is long enough to be masked or searched, overwriting it otherwise.
    fn long_enough<T: AsRef<[u8]> + Zeroize>(mut form: T) -> Option<T> {
        if form.as_ref().len() >= MIN_MASKED_LENGTH {
            Some(form)
        } else {
            form.zeroize();
            None
        }
    }
}

impl Drop for Masking {
    fn drop(&mut self) {
        self.wipe();
    }
}

impl Wipe for Masking {
    fn wipe(&self) -> bool {
        let mut patterns = self.patterns.write().unwrap();
        let wiped = !patterns.names.is_empty() || !patterns.searched.is_empty();
        for (mut text, _) in patterns.names.drain() {
            text.zeroize();
        }
        for (mut bytes, _) in patterns.searched.drain() {
            bytes.zeroize();
        }
        patterns.matcher = None;
        patterns.searcher = None;
        if wiped {
            count_wiped();
        }
        wiped
    }
}

impl RevealedValues for Masking {
    fn find(&self, data: &[u8]) -> Option<String> {
        let patterns = self.patterns.read().unwrap();
        let searcher = patterns.searcher.as_ref()?;
        searcher
            .automaton
            .find(data)
            .map(|found| searcher.names[found.pattern().as_usize()].clone())
    }

    fn longest(&self) -> usize {
        self.patterns
            .read()
            .unwrap()
            .searcher
            .as_ref()
            .map_or(0, |searcher| searcher.longest)
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
    fn searches_whole_values_in_data() {
        let masking = Masking::new();
        masking.add(
            "kubeconfig",
            &Value::String("apiVersion: v1\nkind: Config\ntoken: kube-token-value\n".to_string()),
        );
        masking.add(
            "key",
            &Value::Packed(PackedArray::Byte(std::sync::Arc::new(vec![
                0xff, 0x00, 0xfe, 0x01, 0xfd, 0x02, 0xfc, 0x03,
            ]))),
        );

        assert_eq!(
            masking.find(b"file: apiVersion: v1\nkind: Config\ntoken: kube-token-value\n"),
            Some("kubeconfig".to_string())
        );
        // Lines alone are masked in text, never searched in data.
        assert_eq!(masking.find(b"apiVersion: v1\nkind: Pod\n"), None);
        assert_eq!(
            masking.find(&[0x10, 0xff, 0x00, 0xfe, 0x01, 0xfd, 0x02, 0xfc, 0x03, 0x10]),
            Some("key".to_string())
        );
        assert!(masking.longest() >= 50);

        masking.wipe();
        assert_eq!(
            masking.find(b"apiVersion: v1\nkind: Config\ntoken: kube-token-value\n"),
            None
        );
        assert_eq!(masking.longest(), 0);
    }

    #[test]
    fn wiping_forgets_values() {
        let masking = Masking::new();
        assert!(!masking.wipe());
        masking.add("token", &Value::String("t0k:en/value@1".to_string()));
        assert!(masking.wipe());
        assert_eq!(masking.mask("t0k:en/value@1"), "t0k:en/value@1");
    }

    #[test]
    fn clear_forgets_values() {
        let masking = Masking::new();
        masking.add("token", &Value::String("t0k:en/value@1".to_string()));
        masking.clear();
        assert_eq!(masking.mask("t0k:en/value@1"), "t0k:en/value@1");
    }
}
