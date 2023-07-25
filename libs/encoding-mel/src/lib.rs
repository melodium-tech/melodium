#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::*;
use melodium_macro::{check, mel_package, mel_treatment};

/// Decodes stream of bytes into string.
///
/// The incoming stream of bytes is decoded using the specified encoding.
/// If some characters cannot be decoded for some reason (i.e. invalid according to encoding),
/// it is replaced by the `U+FFFD REPLACEMENT CHARACTER` (�).
///
/// The supported encodings and possible values for `encoding` are defined in
/// the [Encoding Standard](https://encoding.spec.whatwg.org/#names-and-labels).
/// If `encoding` is not recognized, UTF-8 is assumed instead.
#[mel_treatment(
    default encoding "utf-8"
    input data Stream<byte>
    output text Stream<string>
)]
pub async fn decode(encoding: string) {
    let encoding =
        encoding_rs::Encoding::for_label(encoding.as_bytes()).unwrap_or(encoding_rs::UTF_8);
    let mut decoder = encoding.new_decoder();

    let mut finished = false;
    while !finished {
        let bytes;
        if let Ok(data) = data.recv_byte().await {
            bytes = data;
        } else {
            bytes = vec![];
            finished = true;
        }

        let mut result = String::with_capacity(bytes.len() * 2);

        let _ = decoder.decode_to_string(&bytes, &mut result, finished);

        result.shrink_to_fit();

        check!(text.send_one_string(result).await);
    }
}

/// Encodes streamed text with specified encoding.
///
/// The incoming string is encoded and outputted into raw bytes. If some characters cannot
/// be encoded into the specified `encoding`, the behavior is set by `replace`:
/// - `false`: the character is dropped;
/// - `true`: the character is replaced with coded XML character (such as `&#65533;`).
///
/// The supported encodings and possible values for `encoding` are defined in
/// the [Encoding Standard](https://encoding.spec.whatwg.org/#names-and-labels).
/// If `encoding` is not recognized, UTF-8 is assumed instead.
#[mel_treatment(
    default encoding "utf-8"
    default replace false
    input text Stream<string>
    output data Stream<byte>
)]
pub async fn encode(encoding: string, replace: bool) {
    let encoding =
        encoding_rs::Encoding::for_label(encoding.as_bytes()).unwrap_or(encoding_rs::UTF_8);
    let mut encoder = encoding.new_encoder();

    'main: while let Ok(text) = text.recv_string().await {
        for text in text {
            let expected_size = if replace {
                7 * encoder
                    .max_buffer_length_from_utf8_if_no_unmappables(text.len())
                    .unwrap_or(2_usize.pow(20))
            } else {
                encoder
                    .max_buffer_length_from_utf8_without_replacement(text.len())
                    .unwrap_or(2_usize.pow(20))
            };

            let mut result = Vec::new();
            result.reserve(expected_size);

            if replace {
                let _ = encoder.encode_from_utf8_to_vec(&text, &mut result, false);
            } else {
                let _ =
                    encoder.encode_from_utf8_to_vec_without_replacement(&text, &mut result, false);
            }

            result.shrink_to_fit();
            check!('main, data.send_byte(result).await);
        }
    }

    let expected_size = if replace {
        7 * encoder
            .max_buffer_length_from_utf8_if_no_unmappables(0)
            .unwrap_or(2_usize.pow(6))
    } else {
        encoder
            .max_buffer_length_from_utf8_without_replacement(0)
            .unwrap_or(2_usize.pow(6))
    };

    let mut result = Vec::new();
    result.reserve(expected_size);

    if replace {
        let _ = encoder.encode_from_utf8_to_vec(&String::new(), &mut result, false);
    } else {
        let _ =
            encoder.encode_from_utf8_to_vec_without_replacement(&String::new(), &mut result, false);
    }

    result.shrink_to_fit();
    let _ = data.send_byte(result).await;
}

mel_package!();
