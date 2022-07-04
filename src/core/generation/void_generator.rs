
use crate::core::prelude::*;
use async_std::channel::*;

#[derive(Debug)]
struct ModelGenerator {

    helper: ModelHelper,

    generation_sender: Sender<(u64, Option<u128>)>,
    generation_receiver: Receiver<(u64, Option<u128>)>,

    auto_reference: Weak<Self>,
}

impl ModelGenerator {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {

        model_desc!(
            ModelGenerator,
            core_identifier!("generation", "scalar", "void";"TrackGenerator"),
            parameters![],
            model_sources![
                ("generated";)
            ]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        let (generation_sender, generation_receiver) = unbounded();

        Arc::new_cyclic(|me| Self {
            helper: ModelHelper::new(Self::descriptor(), world),

            generation_sender,
            generation_receiver,

            auto_reference: me.clone(),
        })
    }

    fn initialize(&self) {

        let auto_self = self.auto_reference.upgrade().unwrap();
        let future_generate = Box::pin(async move { auto_self.generate().await });

        self.helper.world().add_continuous_task(Box::new(future_generate));
    }

    pub async fn generate_finite(&self, tracks: u64, length: u128) {
        let _ = self.generation_sender.send((tracks, Some(length))).await;
    }

    pub async fn generate_infinite(&self, tracks: u64) {
        let _ = self.generation_sender.send((tracks, None)).await;
    }

    pub async fn generate(&self) {

        let model_id = self.helper.id().unwrap();

        while let Ok((tracks, length)) = self.generation_receiver.recv().await {

            let generation = |inputs| {
                Self::track_generation(length, inputs)
            };

            for _ in 0..tracks {
                self.helper.world().create_track(model_id, "generated", HashMap::new(), None, Some(&generation)).await;
            }
        }
    }

    fn track_generation(length: Option<u128>, inputs: HashMap<String, Output>) -> Vec<TrackFuture>  {
        let future = Box::new(Box::pin(async move {

            let iter_output = inputs.get("_iter").unwrap();

            if let Some(length) = length {
                for _ in 0..length {
                    ok_or_break!(iter_output.send_void(()).await);
                }
            }
            else {
                loop {
                    ok_or_break!(iter_output.send_void(()).await);
                }
            }

            iter_output.close().await;

            ResultStatus::Ok
        }));

        vec![future]
    }

    pub fn shutdown(&self) {
        self.generation_receiver.close();
    }
}

model_trait!(ModelGenerator, initialize, shutdown);

treatment!(treatment_generate,
    core_identifier!("generation","scalar","void";"Generate"),
    models![("generator".to_string(), super::ModelGenerator::descriptor())],
    treatment_sources![],
    parameters![],
    inputs![
        input!("tracks",Scalar,U64,Stream),
        input!("length",Scalar,U128,Stream)
    ],
    outputs![],
    host {
        let generator = Arc::clone(&host.get_model("generator")).downcast_arc::<crate::core::generation::void_generator::ModelGenerator>().unwrap();
        let input_tracks = host.get_input("tracks");
        let input_length = host.get_input("length");
    
        while let (Ok(tracks), Ok(length)) = futures::join!(input_tracks.recv_one_u64(), input_length.recv_one_u128()) {
    
            generator.generate_finite(tracks, length).await;
        }
    
        ResultStatus::Ok
    }
);

treatment!(treatment_generate_infinite,
    core_identifier!("generation","scalar","void";"GenerateInfinite"),
    models![("generator".to_string(), super::ModelGenerator::descriptor())],
    treatment_sources![],
    parameters![],
    inputs![
        input!("tracks",Scalar,U64,Stream)
    ],
    outputs![],
    host {
        let generator = Arc::clone(&host.get_model("generator")).downcast_arc::<crate::core::generation::void_generator::ModelGenerator>().unwrap();
        let input_tracks = host.get_input("tracks");
    
        while let Ok(tracks) = input_tracks.recv_one_u64().await {
    
            generator.generate_infinite(tracks).await;
        }
    
        ResultStatus::Ok
    }
);

treatment!(treatment_generated,
    core_identifier!("generation","scalar","void";"Generated"),
    models![("generator".to_string(), super::ModelGenerator::descriptor())],
    treatment_sources![
        (super::ModelGenerator::descriptor(), "generated")
    ],
    parameters![],
    inputs![
        input!("_iter",Scalar,Void,Stream)
    ],
    outputs![
        output!("iter",Scalar,Void,Stream)
    ],
    host {
        let input = host.get_input("_iter");
        let output = host.get_output("iter");
    
        while let Ok(data) = input.recv_void().await {
    
            ok_or_break!(output.send_multiple_void(data).await);
        }
    
        ResultStatus::Ok
    }
);

pub fn register(mut c: &mut CollectionPool) {
    c.models.insert(&(ModelGenerator::descriptor() as Arc<dyn ModelDescriptor>));
    treatment_generate::register(&mut c);
    treatment_generate_infinite::register(&mut c);
    treatment_generated::register(&mut c);
}
