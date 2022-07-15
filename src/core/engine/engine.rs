
use std::fmt;
use std::sync::Mutex;
use crate::core::prelude::*;

pub struct EngineModel {

    helper: ModelHelper,

    auto_reference: Weak<Self>,
}

impl EngineModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        model_desc!(
            EngineModel,
            core_identifier!("engine";"Engine"),
            vec![],
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
                helper: ModelHelper::new(EngineModel::descriptor(), world),
    
                auto_reference: me.clone(),
            }));

            Arc::clone(&optionnal_engine.as_ref().unwrap()) as Arc<dyn crate::executive::model::Model>
        }
    }

    fn initialize(&self) {

        let auto_self = self.auto_reference.upgrade().unwrap();
        let future = Box::pin(async move { auto_self.run().await });

        self.helper.world().add_continuous_task(Box::new(future));
    }

    async fn run(&self) {

        let model_id = self.helper.id().unwrap();

        self.helper.world().create_track(model_id, "ready", HashMap::new(), None, Some(|i| self.ready(i))).await;
    }

    fn ready(&self, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            if let Some(ready_output) = inputs.get("_ready") {

                let _ = ready_output.send_void(()).await;

                ready_output.close().await;
            }

            ResultStatus::Ok

            
        })) as TrackFuture;

        vec![future]
    }

    pub fn end(&self) {
        
        self.helper.world().end();
    }
}

impl fmt::Debug for EngineModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EngineModel")
         .field("helper", &self.helper)
         .finish()
    }
}

model_trait!(EngineModel, initialize);
