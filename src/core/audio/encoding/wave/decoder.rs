
use crate::core::prelude::*;
use hound::*;
use itertools::Itertools;
use futures::future::join_all;

#[derive(Debug)]
pub struct WaveDecoderModel {

    helper: ModelHelper,
}

impl WaveDecoderModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        model_desc!(
            WaveDecoderModel,
            core_identifier!("audio","encoding","wave";"WaveDecoder"),
            vec![],
            model_sources![
                ("mono";    "Signal"),
                ("stereo";  "Signal")
            ]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        Arc::new(Self {
            helper: ModelHelper::new(Self::descriptor(), world),
        })
    }

    pub async fn decode(&self, block: Vec<u8>) {

        let reader = WavReader::new(block.as_slice()).unwrap();

        let spec = reader.spec();

        let mut signal_context = Context::new();
        signal_context.set_value("sampleRate", Value::U64(spec.sample_rate.into()));
        signal_context.set_value("channels", Value::U32(spec.channels.into()));

        let mut contextes = HashMap::new();
        contextes.insert("Signal".to_string(), signal_context);

        let data_decoding = |inputs| {
            Self::decode_block(block, spec.channels, inputs)
        };

        let model_id = self.helper.id().unwrap();

        if spec.channels == 1 {
            self.helper.world().create_track(model_id, "mono", contextes, None, Some(data_decoding)).await;
        }
        else if spec.channels == 2 {
            self.helper.world().create_track(model_id, "stereo", contextes, None, Some(data_decoding)).await;
        }
    }

    fn decode_block(block: Vec<u8>, channels: u16, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            let mut reader = WavReader::new(block.as_slice()).unwrap();

            let spec = reader.spec();

            let mono_output = inputs.get("mono");

            let stereo_l_output = inputs.get("left");
            let stereo_r_output = inputs.get("right");

            fn i8sample(sample: i8) -> f32 {
                // Please keep linear conversion explicit
                (
                    (sample as f32 - i8::MIN as f32)
                    / (i8::MAX as f32 - i8::MIN as f32)
                )
                * (1.0 - -1.0) - 1.0
            }

            fn i16sample(sample: i16) -> f32 {
                // Please keep linear conversion explicit
                (
                    (sample as f32 - i16::MIN as f32)
                    / (i16::MAX as f32 - i16::MIN as f32)
                )
                * (1.0 - -1.0) - 1.0
            }

            fn i24sample(sample: i32) -> f32 {
                // Please keep linear conversion explicit
                (
                    (sample as f32 - -8_388_608 as f32)
                    / (8_388_607 as f32 - -8_388_608 as f32)
                )
                * (1.0 - -1.0) - 1.0
            }

            fn i32sample(sample: i32) -> f32 {
                // Please keep linear conversion explicit
                (
                    (sample as f32 - i32::MIN as f32)
                    / (i32::MAX as f32 - i32::MIN as f32)
                )
                * (1.0 - -1.0) - 1.0
            }

            match (spec.sample_format, spec.bits_per_sample) {
                (SampleFormat::Float, 32) => {

                    match channels {
                        1 => for sample in reader.samples::<f32>() {
                            if let Ok(sample) = sample {
                                ok_or_break!(mono_output.unwrap().send_f32(sample).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for samples in reader.samples::<f32>().tuples() {

                            if let (Ok(sample_l), Ok(sample_r)) = samples {
                                ok_or_break!(
                                    join_all(
                                        vec![stereo_l_output.unwrap().send_f32(sample_l),
                                             stereo_r_output.unwrap().send_f32(sample_r)]
                                    ).await.iter().find(|r| r.is_ok()).cloned().transpose()
                                );
                            }
                            else {
                                break;
                            }
                        },
                        _ => ()
                    }
                },
                (SampleFormat::Int, 8) => {
                    match channels {
                        1 => for sample in reader.samples::<i8>() {
                            if let Ok(sample) = sample {
                                ok_or_break!(mono_output.unwrap().send_f32(i8sample(sample)).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for samples in reader.samples::<i8>().tuples() {

                            if let (Ok(sample_l), Ok(sample_r)) = samples {
                                ok_or_break!(
                                    join_all(
                                        vec![stereo_l_output.unwrap().send_f32(i8sample(sample_l)),
                                             stereo_r_output.unwrap().send_f32(i8sample(sample_r))]
                                    ).await.iter().find(|r| r.is_ok()).cloned().transpose()
                                );
                            }
                            else {
                                break;
                            }
                        },
                        _ => ()
                    }
                },
                (SampleFormat::Int, 16) => {
                    match channels {
                        1 => for sample in reader.samples::<i16>() {
                            if let Ok(sample) = sample {
                                ok_or_break!(mono_output.unwrap().send_f32(i16sample(sample)).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for samples in reader.samples::<i16>().tuples() {

                            if let (Ok(sample_l), Ok(sample_r)) = samples {
                                ok_or_break!(
                                    join_all(
                                        vec![stereo_l_output.unwrap().send_f32(i16sample(sample_l)),
                                             stereo_r_output.unwrap().send_f32(i16sample(sample_r))]
                                    ).await.iter().find(|r| r.is_ok()).cloned().transpose()
                                );
                            }
                            else {
                                break;
                            }
                        },
                        _ => ()
                    }
                },
                (SampleFormat::Int, 24) => {
                    match channels {
                        1 => for sample in reader.samples::<i32>() {
                            if let Ok(sample) = sample {
                                ok_or_break!(mono_output.unwrap().send_f32(i24sample(sample)).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for samples in reader.samples::<i32>().tuples() {

                            if let (Ok(sample_l), Ok(sample_r)) = samples {
                                ok_or_break!(
                                    join_all(
                                        vec![stereo_l_output.unwrap().send_f32(i24sample(sample_l)),
                                             stereo_r_output.unwrap().send_f32(i24sample(sample_r))]
                                    ).await.iter().find(|r| r.is_ok()).cloned().transpose()
                                );
                            }
                            else {
                                break;
                            }
                        },
                        _ => ()
                    }
                },
                (SampleFormat::Int, 32) => {
                    match channels {
                        1 => for sample in reader.samples::<i32>() {
                            if let Ok(sample) = sample {
                                ok_or_break!(mono_output.unwrap().send_f32(i32sample(sample)).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for samples in reader.samples::<i32>().tuples() {

                            if let (Ok(sample_l), Ok(sample_r)) = samples {
                                ok_or_break!(
                                    join_all(
                                        vec![stereo_l_output.unwrap().send_f32(i32sample(sample_l)),
                                             stereo_r_output.unwrap().send_f32(i32sample(sample_r))]
                                    ).await.iter().find(|r| r.is_ok()).cloned().transpose()
                                );
                            }
                            else {
                                break;
                            }
                        },
                        _ => ()
                    }
                },
                _ => ()
            }

            match channels {
                1 => mono_output.unwrap().close().await,
                2 => {
                    stereo_l_output.unwrap().close().await;
                    stereo_r_output.unwrap().close().await;
                },
                _ => ()
            }

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }
}

model_trait!(WaveDecoderModel);

source!(mono_decode,
    core_identifier!("audio","encoding","wave";"MonoWave"),
    models![
        ("decoder", crate::core::audio::encoding::wave::decoder::WaveDecoderModel::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::encoding::wave::decoder::WaveDecoderModel::descriptor(), "mono")
    ],
    outputs![
        output!("mono",Scalar,F32,Stream)
    ]
);

source!(stereo_decode,
    core_identifier!("audio","encoding","wave";"StereoWave"),
    models![
        ("decoder", crate::core::audio::encoding::wave::decoder::WaveDecoderModel::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::encoding::wave::decoder::WaveDecoderModel::descriptor(), "stereo")
    ],
    outputs![
        output!("left",Scalar,F32,Stream),
        output!("right",Scalar,F32,Stream)
    ]
);

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(WaveDecoderModel::descriptor() as Arc<dyn ModelDescriptor>));
    mono_decode::register(&mut c);
    stereo_decode::register(&mut c);
}
