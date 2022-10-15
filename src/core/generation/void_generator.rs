
use std::sync::{Arc, Weak};
use std::collections::HashMap;
use crate::core::prelude::*;

#[derive(Debug)]
struct ModelGenerator {

    host: Weak<ModelHost>,
}

impl ModelGenerator {

    pub fn new(host: Weak<ModelHost>) -> Arc<dyn HostedModel> {

        Arc::new(Self {
            host
        })
    }

    pub async fn generate_finite(&self, tracks: u64, length: u128) {

        let host = self.host.upgrade().unwrap();

        let model_id = host.id().unwrap();

        let generation = |inputs| {
            Self::track_generation(Some(length), inputs)
        };

        for _ in 0..tracks {
            host.world().create_track(model_id, "generated", HashMap::new(), None, Some(&generation)).await;
        }
    }

    pub async fn generate_infinite(&self, tracks: u64) {

        let host = self.host.upgrade().unwrap();

        let model_id = host.id().unwrap();

        let generation = |inputs| {
            Self::track_generation(None, inputs)
        };

        for _ in 0..tracks {
            host.world().create_track(model_id, "generated", HashMap::new(), None, Some(&generation)).await;
        }
    }

    fn track_generation(length: Option<u128>, inputs: HashMap<String, Output>) -> Vec<TrackFuture>  {
        let future = Box::new(Box::pin(async move {

            let iter_output = inputs.get("iter").unwrap();

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
}

impl HostedModel for ModelGenerator {

    fn initialize(&self) {}
    fn shutdown(&self) {}
}

model!(
    ModelGenerator,
    core_identifier!("generation", "scalar", "void";"TrackGenerator"),
    "Generates arbitrary tracks".to_string(),
    parameters![],
    model_sources![
        ("generated";)
    ]
);

treatment!(treatment_generate,
    core_identifier!("generation","scalar","void";"Generate"),
    r#"Trigger generation of tracks
    
    Makes `generator` generating number of `tracks` with given `length`."#.to_string(),
    models![("generator".to_string(), super::model_host::descriptor())],
    treatment_sources![],
    parameters![],
    inputs![
        input!("tracks",Scalar,U64,Stream),
        input!("length",Scalar,U128,Stream)
    ],
    outputs![],
    host {
        let generator = host.get_hosted_model("generator").downcast_arc::<crate::core::generation::void_generator::ModelGenerator>().unwrap();
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
    r#"Trigger generation of tracks
    
    Makes `generator` generating number of `tracks` with infinite length."#.to_string(),
    models![("generator".to_string(), super::model_host::descriptor())],
    treatment_sources![],
    parameters![],
    inputs![
        input!("tracks",Scalar,U64,Stream)
    ],
    outputs![],
    host {
        let generator = host.get_hosted_model("generator").downcast_arc::<crate::core::generation::void_generator::ModelGenerator>().unwrap();
        let input_tracks = host.get_input("tracks");
    
        while let Ok(tracks) = input_tracks.recv_one_u64().await {
    
            generator.generate_infinite(tracks).await;
        }
    
        ResultStatus::Ok
    }
);

source!(source_generated,
    core_identifier!("generation","scalar","void";"Generated"),
    r#"Generated track"#.to_string(),
    models![("generator".to_string(), super::model_host::descriptor())],
    treatment_sources![
        (super::model_host::descriptor(), "generated")
    ],
    outputs![
        output!("iter",Scalar,Void,Stream)
    ]
);

pub fn register(mut c: &mut CollectionPool) {
    model_host::register(&mut c);
    treatment_generate::register(&mut c);
    treatment_generate_infinite::register(&mut c);
    source_generated::register(&mut c);
}
