
use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    decode::register(&mut c);
}

treatment!(decode,
    core_identifier!("audio","encoding","wave";"DecodeWave"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("data",Vector,Byte,Block)
    ],
    outputs![
        output!("signal",Scalar,F32,Stream)
    ],
    host {

        use hound::*;

        let input = host.get_input("data");
        let output = host.get_output("signal");
    
        while let Ok(data) = input.recv_vec_byte().await {

            // We're taking a block
            let data = data.get(0).unwrap();
            input.close();

            let mut reader = WavReader::new(data.as_slice()).unwrap();

            let spec = reader.spec();

            match (spec.sample_format, spec.bits_per_sample) {
                (SampleFormat::Float, 32) => {
                    for sample in reader.samples::<f32>() {
                        if let Ok(sample) = sample {
                            ok_or_break!(output.send_f32(sample).await);
                        }
                        else {
                            break;
                        }
                    }
                },
                (SampleFormat::Int, 8) => {
                    for sample in reader.samples::<i8>() {
                        if let Ok(sample) = sample {
                            // Please keep linear conversion explicit
                            let sample = (
                                (sample as f32 - i8::MIN as f32)
                                / (i8::MAX as f32 - i8::MIN as f32)
                            )
                            * (1.0 - -1.0) - 1.0;

                            ok_or_break!(output.send_f32(sample).await);
                        }
                        else {
                            break;
                        }
                    }
                },
                (SampleFormat::Int, 16) => {
                    for sample in reader.samples::<i16>() {
                        if let Ok(sample) = sample {
                            // Please keep linear conversion explicit
                            let sample = (
                                (sample as f32 - i16::MIN as f32)
                                / (i16::MAX as f32 - i16::MIN as f32)
                            )
                            * (1.0 - -1.0) - 1.0;
                            
                            ok_or_break!(output.send_f32(sample).await);
                        }
                        else {
                            break;
                        }
                    }
                },
                (SampleFormat::Int, 24) => {
                    for sample in reader.samples::<i32>() {
                        if let Ok(sample) = sample {
                            // Please keep linear conversion explicit
                            let sample = (
                                (sample as f32 - -8_388_608 as f32)
                                / (8_388_607 as f32 - -8_388_608 as f32)
                            )
                            * (1.0 - -1.0) - 1.0;
                            
                            ok_or_break!(output.send_f32(sample).await);
                        }
                        else {
                            break;
                        }
                    }
                },
                (SampleFormat::Int, 32) => {
                    for sample in reader.samples::<i32>() {
                        if let Ok(sample) = sample {
                            // Please keep linear conversion explicit
                            let sample = (
                                (sample as f32 - i32::MIN as f32)
                                / (i32::MAX as f32 - i32::MIN as f32)
                            )
                            * (1.0 - -1.0) - 1.0;
                            
                            ok_or_break!(output.send_f32(sample).await);
                        }
                        else {
                            break;
                        }
                    }
                },
                _ => ()
            }

            

            //ok_or_break!(output.send_multiple_f32(signal).await);
        }
    
        ResultStatus::Ok
    }
);
