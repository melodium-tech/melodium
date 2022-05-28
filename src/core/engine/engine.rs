
use crate::core::prelude::*;

#[derive(Debug)]
pub struct EngineModel {

    helper: ModelHelper,

    write_channel: RecvTransmitter<String>,

    auto_reference: Weak<Self>,
}

impl EngineModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        model_desc!(
            EngineModel,
            core_identifier!("engine";"Engine"),
            vec![],
            model_sources![
                ("ready"; ),
                ("read"; )
            ]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        Arc::new_cyclic(|me| EngineModel {
            helper: ModelHelper::new(EngineModel::descriptor(), world),

            write_channel: RecvTransmitter::new(),

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

        futures::join!(
            self.helper.world().create_track(model_id, "ready", HashMap::new(), None, Some(|i| self.ready(i))),
            self.helper.world().create_track(model_id, "read", HashMap::new(), None, Some(|i| self.read(i))),
            self.write()
        )
    }

    pub fn writer(&self) -> &RecvTransmitter<String> {
        &self.write_channel
    }

    fn ready(&self, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            let ready_output = inputs.get("_ready").unwrap();

            let _ = ready_output.send_void(()).await;

            ready_output.close().await;

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }

    fn read(&self, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            let line_output = inputs.get("_line").unwrap();

            let stdin = async_std::io::stdin();
            let mut line = String::new();

            while let Ok(_) = stdin.read_line(&mut line).await {

                ok_or_break!(line_output.send_string(line).await);

                line = String::new();
            }

            line_output.close().await;

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }

    async fn write(&self) {

        let receiver = &self.write_channel;

        while let Ok(text) = receiver.receive_multiple().await {

            for part in text {
                print!("{}", part);
            }
        }
    }
}

model_trait!(EngineModel, initialize);
