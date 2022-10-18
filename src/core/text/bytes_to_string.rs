
use crate::core::prelude::*;

treatment!(byte_to_string,
    core_identifier!("text";"DecodeBytes"),
    indoc!(r#"Decodes a stream of bytes into string.

    The incoming stream of bytes is decoded using the specified encoding.
    If some characters cannot be decoded for some reason (i.e. invalid according to encoding),
    it is replaced by the Unicode Replacement Character (`�`).
    
    The supported encodings and possible values for `encoding` are defined in
    the [Encoding Standard](https://encoding.spec.whatwg.org/#names-and-labels)."#).to_string(),
    models![],
    treatment_sources![],
    parameters![
        parameter!("encoding",Var,Scalar,String,Some(Value::String("utf-8".to_string())))
    ],
    inputs![
        input!("data",Scalar,Byte,Stream)
    ],
    outputs![
        output!("value",Scalar,String,Stream)
    ],
    host {
        let input = host.get_input("data");
        let output = host.get_output("value");

        let encoding = encoding_rs::Encoding::for_label(
            host.get_parameter("encoding").string().as_bytes()
        ).unwrap_or(encoding_rs::UTF_8);
        let mut decoder = encoding.new_decoder();

        let mut finished = false;
        while !finished {

            let bytes;
            if let Ok(data) = input.recv_byte().await {
                bytes = data;
            }
            else {
                bytes = vec![];
                finished = true;
            }

            let mut result = String::with_capacity(bytes.len() * 2);

            let _ = decoder.decode_to_string(&bytes, &mut result, finished);

            ok_or_break!(output.send_string(result).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(string_to_byte,
    core_identifier!("text";"EncodeBytes"),
    indoc!(r#"Encodes a streamed string into bytes.

    The incoming string is encoded and outputted into raw bytes. If some characters cannot
    be encoded into the specified `encoding`, it is turned into coded XML character (`"&#65533;"`).
    
    The supported encodings and possible values for `encoding` are defined in
    the [Encoding Standard](https://encoding.spec.whatwg.org/#names-and-labels)."#).to_string(),
    models![],
    treatment_sources![],
    parameters![
        parameter!("encoding",Var,Scalar,String,Some(Value::String("utf-8".to_string())))
    ],
    inputs![
        input!("value",Scalar,String,Stream)
    ],
    outputs![
        output!("data",Scalar,Byte,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("data");

        let encoding = encoding_rs::Encoding::for_label(
            host.get_parameter("encoding").string().as_bytes()
        ).unwrap_or(encoding_rs::UTF_8);
        let mut encoder = encoding.new_encoder();

        'main: loop {

            if let Ok(strings) = input.recv_string().await {
                for string in strings {

                    // We use 7 times required space because of HTML replacement in case of unmappable chars.
                    let expected_size = 7 * encoder.max_buffer_length_from_utf8_if_no_unmappables(string.len()).unwrap_or(usize::MAX);

                    let mut result = Vec::new();
                    result.reserve(expected_size);

                    let _ = encoder.encode_from_utf8_to_vec(&string, &mut result, false);

                    result.shrink_to_fit();
                    ok_or_break!('main, output.send_multiple_byte(result).await);
                }
            }
            else {
                // Here we finish the encoding as required by encoding_rs
                let expected_size = 7 * encoder.max_buffer_length_from_utf8_if_no_unmappables(0).unwrap_or(usize::MAX);

                let mut result = Vec::new();
                result.reserve(expected_size);

                let _ = encoder.encode_from_utf8_to_vec(&String::new(), &mut result, true);

                result.shrink_to_fit();
                ok_or_break!(output.send_multiple_byte(result).await);
                
                break;
            }
        }
    
        ResultStatus::Ok
    }
);

pub fn register(mut c: &mut CollectionPool) {

    byte_to_string::register(&mut c);
    string_to_byte::register(&mut c);
}
