
use crate::core::prelude::*;

mod decoder;

pub fn register(mut c: &mut CollectionPool) {

    decoder::register(&mut c);
    decode::register(&mut c);
    encode::register(&mut c);
}

treatment!(decode,
    core_identifier!("audio","encoding","wave";"DecodeWave"),
    models![
        ("decoder", crate::core::audio::encoding::wave::decoder::model_host::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("data",Vector,Byte,Block)
    ],
    outputs![
        output!("signal",Scalar,F32,Stream)
    ],
    host {

        use crate::core::audio::encoding::wave::decoder::WaveDecoderModel;

        let input = host.get_input("data");
        let decoder = host.get_hosted_model("decoder").downcast_arc::<WaveDecoderModel>().unwrap();
    
        if let Ok(data) = input.recv_vec_byte().await {

            // We're taking a block
            let data = data.get(0).unwrap();
            input.close();

            decoder.decode(data.to_vec()).await;
        }
    
        ResultStatus::Ok
    }
);

treatment!(encode,
    core_identifier!("audio","encoding","wave";"EncodeWave"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("signal",Scalar,F32,Stream)
    ],
    outputs![
        output!("data",Vector,Byte,Block)
    ],
    host {

        use hound::*;

        let input = host.get_input("signal");
        let output = host.get_output("data");

        let mut cursor_writer = std::io::Cursor::new(Vec::new());
        {
            let mut wav_writer = WavWriter::new(
                &mut cursor_writer,
                WavSpec {
                    channels: 1,
                    sample_rate: 44100,
                    bits_per_sample: 32,
                    sample_format: SampleFormat::Float,
                }
            ).unwrap();
    
            while let Ok(signal_chunks) = input.recv_f32().await {
    
                for sample in signal_chunks {
                    wav_writer.write_sample(sample).unwrap();
                }
            }
        }

        let data_written = cursor_writer.into_inner();

        let _ = output.send_vec_byte(data_written).await;
    
        ResultStatus::Ok
    }
);

