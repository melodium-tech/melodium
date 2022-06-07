
use std::fmt;
use std::sync::Mutex;
use crate::core::prelude::*;

pub struct EngineModel {

    helper: ModelHelper,

    read_channel: SendTransmitter<String>,
    write_channel: RecvTransmitter<String>,

    sighup_channel: SendTransmitter<()>,
    sigterm_channel: SendTransmitter<()>,

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
                ("read"; ),
                ("sighup"; ),
                ("sigterm"; )
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
    
                read_channel: SendTransmitter::new(),
                write_channel: RecvTransmitter::new(),

                sighup_channel: SendTransmitter::new(),
                sigterm_channel: SendTransmitter::new(),
    
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

        futures::join!(
            self.helper.world().create_track(model_id, "ready", HashMap::new(), None, Some(|i| self.ready(i))),
            self.helper.world().create_track(model_id, "read", HashMap::new(), None, Some(|i| self.read(i))),
            self.helper.world().create_track(model_id, "sighup", HashMap::new(), None, Some(|i| self.sighup(i))),
            self.helper.world().create_track(model_id, "sigterm", HashMap::new(), None, Some(|i| self.sigterm(i))),
            self.signals(),
            self.stdin(),
            // TODO enable this once engine have end trigger
            //self.write()
        );
    }

    pub fn writer(&self) -> &RecvTransmitter<String> {
        &self.write_channel
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

    fn read(&self, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let stdin = RecvTransmitter::new();
        self.read_channel.add_transmitter(&stdin);

        let future = Box::new(Box::pin(async move {

            if let Some(line_output) = inputs.get("_line") {

                while let Ok(lines) = stdin.receive_multiple().await {

                    ok_or_break!(line_output.send_multiple_string(lines).await);
                }

                line_output.close().await;
            }

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }

    fn sighup(&self, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let sighup = RecvTransmitter::new();
        self.sighup_channel.add_transmitter(&sighup);

        let future = Box::new(Box::pin(async move {

            if let Some(sighup_output) = inputs.get("_sighup") {

                while let Ok(_) = sighup.receive_one().await {

                    ok_or_break!(sighup_output.send_void(()).await);
                }

                sighup_output.close().await;
            }

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }

    fn sigterm(&self, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let sigterm = RecvTransmitter::new();
        self.sigterm_channel.add_transmitter(&sigterm);

        let future = Box::new(Box::pin(async move {

            if let Some(sigterm_output) = inputs.get("_sigterm") {

                while let Ok(_) = sigterm.receive_one().await {

                    ok_or_break!(sigterm_output.send_void(()).await);
                }

                sigterm_output.close().await;
            }

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }

    async fn stdin(&self) {

        let stdin = async_std::io::stdin();
        let mut line = String::new();

        while let Ok(n) = stdin.read_line(&mut line).await {

            ok_or_break!(self.read_channel.send(line).await);

            line = String::new();

            // Meaning EOF is reached
            if n == 0 {
                break;
            }
        }

        self.read_channel.close().await;
    }

    async fn signals(&self) {
        
        // No support of Windows signals
        self.sighup_channel.close().await;
        self.sigterm_channel.close().await;
    }

    // TODO enable this once engine have end trigger
    #[allow(dead_code)]
    async fn write(&self) {

        let receiver = &self.write_channel;

        while let Ok(text) = receiver.receive_multiple().await {

            for part in text {
                print!("{}", part);
            }
        }
    }

    pub fn end(&self) {
        
        self.helper.world().end();
    }

    pub fn close(&self) {
        self.write_channel.close();
    }
}

impl fmt::Debug for EngineModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EngineModel")
         .field("helper", &self.helper)
         .finish()
    }
}

model_trait!(EngineModel, initialize, close);
