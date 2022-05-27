
use crate::core::prelude::*;

#[derive(Debug)]
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

        Arc::new_cyclic(|me| EngineModel {
            helper: ModelHelper::new(EngineModel::descriptor(), world),

            auto_reference: me.clone(),
        })
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

            let ready_output = inputs.get("_ready").unwrap();

            let _ = ready_output.send_void(());

            ready_output.close().await;

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }
}

model_trait!(EngineModel, initialize);
