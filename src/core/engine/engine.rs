
use std::fmt;
use std::sync::{Mutex, Arc, Weak};
use std::collections::HashMap;
use crate::core::prelude::*;

pub struct EngineModel {

    id: Mutex<Option<ModelId>>,

    world: Arc<World>,
    auto_reference: Weak<Self>,
}

impl EngineModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        model_desc!(
            EngineModel,
            core_identifier!("engine";"Engine"),
            parameters![],
            model_sources![
                ("ready"; )
            ]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        lazy_static! {
            static ref SINGLE_ENGINE: Mutex<Option<Arc<EngineModel>>> = Mutex::new(None);
        }

        let mut optionnal_engine = SINGLE_ENGINE.lock().unwrap();

        if let Some(rc_engine) = &*optionnal_engine {
            Arc::clone(&rc_engine) as Arc<dyn crate::executive::model::Model>
        }
        else {

            *optionnal_engine = Some(Arc::new_cyclic(|me| EngineModel {
                id: Mutex::new(None),
                world,
                auto_reference: me.clone(),
            }));

            Arc::clone(&optionnal_engine.as_ref().unwrap()) as Arc<dyn crate::executive::model::Model>
        }
    }

    async fn run(&self) {

        let model_id = self.id.lock().unwrap().unwrap();

        self.world.create_track(model_id, "ready", HashMap::new(), None, Some(|i| self.ready(i))).await;
    }

    fn ready(&self, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            if let Some(ready_output) = inputs.get("ready") {

                let _ = ready_output.send_void(()).await;

                ready_output.close().await;
            }

            ResultStatus::Ok

            
        })) as TrackFuture;

        vec![future]
    }

    pub fn end(&self) {
        
        self.world.end();
    }
}

impl Model for EngineModel {
    
    fn descriptor(&self) -> std::sync::Arc<CoreModelDescriptor> {
        Self::descriptor()
    }

    fn id(&self) -> Option<ModelId> {
        *self.id.lock().unwrap()
    }

    fn set_id(&self, id: ModelId) {
        *self.id.lock().unwrap() = Some(id);
    }

    fn set_parameter(&self, _param: &str, _value: &Value) {}

    fn initialize(&self) {

        let auto_self = self.auto_reference.upgrade().unwrap();
        let future = Box::pin(async move { auto_self.run().await });

        self.world.add_continuous_task(Box::new(future));
    }

    fn shutdown(&self) {}
}

impl fmt::Debug for EngineModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EngineModel")
         .field("id", &self.id)
         .finish()
    }
}

source!(engine_ready_source,
    core_identifier!("engine";"Ready"),
    models![
        ("engine", crate::core::engine::engine::EngineModel::descriptor())
    ],
    treatment_sources![
        (crate::core::engine::engine::EngineModel::descriptor(), "ready")
    ],
    outputs![
        output!("ready",Scalar,Void,Block)
    ]
);

treatment!(engine_end_treatment,
    core_identifier!("engine";"End"),
    models![
        ("engine", crate::core::engine::engine::EngineModel::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("end",Scalar,Void,Block)
    ],
    outputs![],
    host {

        let engine = std::sync::Arc::clone(&host.get_model("engine")).downcast_arc::<crate::core::engine::engine::EngineModel>().unwrap();

        let input = host.get_input("end");
    
        if let Ok(_) = input.recv_one_void().await {

            engine.end();
        }
    
        ResultStatus::Ok
    }
);

