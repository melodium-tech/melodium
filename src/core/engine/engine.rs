
use std::fmt;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use async_std::sync::Mutex as AsyncMutex;
use async_std::task::block_on;
use futures::future::abortable;
use futures::stream::AbortHandle;
use signal_hook::consts::signal::*;
use signal_hook_async_std::{Signals, Handle};
use crate::core::prelude::*;

pub struct EngineModel {

    helper: ModelHelper,

    read_abort: Mutex<Option<AbortHandle>>,
    read_channel: SendTransmitter<String>,
    write_channel: RecvTransmitter<String>,

    signals: Arc<AsyncMutex<Signals>>,
    signals_handle: Arc<Handle>,
    sigterm_handled: AtomicBool,

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

            let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT]).unwrap();
            let handle = signals.handle();

            *optionnal_engine = Some(Arc::new_cyclic(|me| EngineModel {
                helper: ModelHelper::new(EngineModel::descriptor(), world),
    
                read_abort: Mutex::new(None),
                read_channel: SendTransmitter::new(),
                write_channel: RecvTransmitter::new(),

                signals: Arc::new(AsyncMutex::new(signals)),
                signals_handle: Arc::new(handle),
                sigterm_handled: AtomicBool::new(false),

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

        let (stdin, abort_stdin) = abortable(self.stdin());

        *self.read_abort.lock().unwrap() = Some(abort_stdin);

        let _ = futures::join!(
            self.helper.world().create_track(model_id, "ready", HashMap::new(), None, Some(|i| self.ready(i))),
            self.helper.world().create_track(model_id, "read", HashMap::new(), None, Some(|i| self.read(i))),
            self.helper.world().create_track(model_id, "sighup", HashMap::new(), None, Some(|i| self.sighup(i))),
            self.helper.world().create_track(model_id, "sigterm", HashMap::new(), None, Some(|i| self.sigterm(i))),
            self.signals(),
            stdin,
            self.write()
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

        self.sigterm_handled.store(true, Ordering::Relaxed);

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

            // Meaning EOF is reached
            if n == 0 {
                break;
            }

            ok_or_break!(self.read_channel.send(line).await);

            line = String::new();
        }

        self.read_channel.close().await;
    }

    async fn signals(&self) {
        while let Some(signal) = self.signals.lock().await.next().await {
            match signal {
                SIGHUP => {
                    let _ = self.sighup_channel.send(()).await;
                },
                SIGTERM | SIGINT | SIGQUIT => {
                    if self.sigterm_handled.load(Ordering::Relaxed) {
                        let _ = self.sigterm_channel.send(()).await;
                    }
                    else {
                        self.end();
                    }
                },
                _ => unreachable!(),
            }
        }

        self.sighup_channel.close().await;
        self.sigterm_channel.close().await;
    }

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
        self.signals_handle.close();
        block_on(self.read_channel.close());
        self.write_channel.close();

        if let Some(abort_handle) = &*self.read_abort.lock().unwrap() {
            abort_handle.abort();
        }
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
