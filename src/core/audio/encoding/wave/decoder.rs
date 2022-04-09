
use crate::core::prelude::*;
use hound::*;
use itertools::Itertools;

#[derive(Debug)]
pub struct WaveDecoderModel {

    helper: ModelHelper,

    auto_reference: RwLock<Weak<Self>>,
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

        let model = Arc::new(Self {
            helper: ModelHelper::new(Self::descriptor(), world),

            auto_reference: RwLock::new(Weak::new()),
        });

        *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

        model
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

    fn decode_block(block: Vec<u8>, channels: u16, inputs: HashMap<String, Vec<Input>>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            let mut reader = WavReader::new(block.as_slice()).unwrap();

            let spec = reader.spec();

            let mono_output = Output::F32(Arc::new(SendTransmitter::new()));
            if let Some(mono) = inputs.get("_mono"){
                mono.iter().for_each(|i| mono_output.add_input(i));
            }

            let stereo_l_output = Output::F32(Arc::new(SendTransmitter::new()));
            let stereo_r_output = Output::F32(Arc::new(SendTransmitter::new()));
            if let Some(stereo_l) = inputs.get("_stereo_l"){
                stereo_l.iter().for_each(|i| stereo_l_output.add_input(i));
            }
            if let Some(stereo_r) = inputs.get("_stereo_r"){
                stereo_r.iter().for_each(|i| stereo_r_output.add_input(i));
            }

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
                                ok_or_break!(mono_output.send_f32(sample).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for (sample_l, sample_r) in reader.samples::<f32>().tuples() {

                            if let Ok(sample_l) = sample_l {
                                ok_or_break!(stereo_l_output.send_f32(sample_l).await);
                            }
                            else {
                                break;
                            }

                            if let Ok(sample_r) = sample_r {
                                ok_or_break!(stereo_r_output.send_f32(sample_r).await);
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
                                ok_or_break!(mono_output.send_f32(i8sample(sample)).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for (sample_l, sample_r) in reader.samples::<i8>().tuples() {
                            
                            if let Ok(sample_l) = sample_l {
                                ok_or_break!(stereo_l_output.send_f32(i8sample(sample_l)).await);
                            }
                            else {
                                break;
                            }

                            if let Ok(sample_r) = sample_r {
                                ok_or_break!(stereo_r_output.send_f32(i8sample(sample_r)).await);
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
                                ok_or_break!(mono_output.send_f32(i16sample(sample)).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for (sample_l, sample_r) in reader.samples::<i16>().tuples() {
                            
                            if let Ok(sample_l) = sample_l {
                                ok_or_break!(stereo_l_output.send_f32(i16sample(sample_l)).await);
                            }
                            else {
                                break;
                            }

                            if let Ok(sample_r) = sample_r {
                                ok_or_break!(stereo_r_output.send_f32(i16sample(sample_r)).await);
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
                                ok_or_break!(mono_output.send_f32(i24sample(sample)).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for (sample_l, sample_r) in reader.samples::<i32>().tuples() {
                            
                            if let Ok(sample_l) = sample_l {
                                ok_or_break!(stereo_l_output.send_f32(i24sample(sample_l)).await);
                            }
                            else {
                                break;
                            }

                            if let Ok(sample_r) = sample_r {
                                ok_or_break!(stereo_r_output.send_f32(i24sample(sample_r)).await);
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
                                ok_or_break!(mono_output.send_f32(i32sample(sample)).await);
                            }
                            else {
                                break;
                            }
                        },
                        2 => for (sample_l, sample_r) in reader.samples::<i32>().tuples() {
                            
                            if let Ok(sample_l) = sample_l {
                                ok_or_break!(stereo_l_output.send_f32(i32sample(sample_l)).await);
                            }
                            else {
                                break;
                            }

                            if let Ok(sample_r) = sample_r {
                                ok_or_break!(stereo_r_output.send_f32(i32sample(sample_r)).await);
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

            mono_output.close().await;
            stereo_l_output.close().await;
            stereo_r_output.close().await;

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }
}

model_trait!(WaveDecoderModel);

treatment!(mono_decode,
    core_identifier!("audio","encoding","wave";"MonoWave"),
    models![
        ("decoder", crate::core::audio::encoding::wave::decoder::WaveDecoderModel::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::encoding::wave::decoder::WaveDecoderModel::descriptor(), "mono")
    ],
    parameters![],
    inputs![
        input!("_mono",Scalar,F32,Stream)
    ],
    outputs![
        output!("mono",Scalar,F32,Stream)
    ],
    host {

        let input = host.get_input("_mono");
        let output = host.get_output("mono");
    
        while let Ok(signal) = input.recv_f32().await {

            ok_or_break!(output.send_multiple_f32(signal).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(stereo_decode,
    core_identifier!("audio","encoding","wave";"StereoWave"),
    models![
        ("decoder", crate::core::audio::encoding::wave::decoder::WaveDecoderModel::descriptor())
    ],
    treatment_sources![
        (crate::core::audio::encoding::wave::decoder::WaveDecoderModel::descriptor(), "stereo")
    ],
    parameters![],
    inputs![
        input!("_stereo_l",Scalar,F32,Stream),
        input!("_stereo_r",Scalar,F32,Stream)
    ],
    outputs![
        output!("left",Scalar,F32,Stream),
        output!("right",Scalar,F32,Stream)
    ],
    host {

        let input_stereo_l = host.get_input("_stereo_l");
        let input_stereo_r = host.get_input("_stereo_r");
        let output_stereo_l = host.get_output("left");
        let output_stereo_r = host.get_output("right");

        loop {
            if let Ok(signal) = input_stereo_l.recv_f32().await {
                ok_or_break!(output_stereo_l.send_multiple_f32(signal).await);
            }
            else {
                break;
            }
            if let Ok(signal) = input_stereo_r.recv_f32().await {
                ok_or_break!(output_stereo_r.send_multiple_f32(signal).await);
            }
            else {
                break;
            }
        }
    
        ResultStatus::Ok
    }
);

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(WaveDecoderModel::descriptor() as Arc<dyn ModelDescriptor>));
    mono_decode::register(&mut c);
    stereo_decode::register(&mut c);
}
