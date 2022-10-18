
use std::fmt;
use std::sync::{Mutex, Arc, Weak};
use std::collections::HashMap;
use async_std::task::block_on;
use futures::future::abortable;
use futures::stream::AbortHandle;
use crate::core::prelude::*;

pub struct StdinModel {

    id: Mutex<Option<ModelId>>,

    world: Arc<World>,

    read_abort: Mutex<Option<AbortHandle>>,
    read_channel: SendTransmitter<String>,

    auto_reference: Weak<Self>,
}

impl StdinModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        model_desc!(
            StdinModel,
            core_identifier!("engine";"Stdin"),
            "Standard input model".to_string(),
            parameters![],
            model_sources![
                ("read"; )
            ]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        lazy_static! {
            static ref SINGLE_STDIN: Mutex<Option<Arc<StdinModel>>> = Mutex::new(None);
        }

        let mut optionnal_stdin = SINGLE_STDIN.lock().unwrap();

        if let Some(rc_stdin) = &*optionnal_stdin {
            Arc::clone(&rc_stdin) as Arc<dyn crate::executive::model::Model>
        }
        else {

            *optionnal_stdin = Some(Arc::new_cyclic(|me| StdinModel {
                id: Mutex::new(None),
                world,
                read_abort: Mutex::new(None),
                read_channel: SendTransmitter::new(),
                auto_reference: me.clone(),
            }));

            Arc::clone(&optionnal_stdin.as_ref().unwrap()) as Arc<dyn crate::executive::model::Model>
        }
    }

    async fn run(&self) {

        let model_id = self.id.lock().unwrap().unwrap();

        let (stdin, abort_stdin) = abortable(self.stdin());

        *self.read_abort.lock().unwrap() = Some(abort_stdin);

        let _ = futures::join!(
            self.world.create_track(model_id, "read", HashMap::new(), None, Some(|i| self.read(i))),
            stdin
        );
    }

    fn read(&self, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let stdin = RecvTransmitter::new();
        self.read_channel.add_transmitter(&stdin);

        let future = Box::new(Box::pin(async move {

            if let Some(line_output) = inputs.get("line") {

                while let Ok(lines) = stdin.receive_multiple().await {

                    ok_or_break!(line_output.send_multiple_string(lines).await);
                }

                line_output.close().await;
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

    pub fn close(&self) {
        block_on(self.read_channel.close());

        if let Some(abort_handle) = &*self.read_abort.lock().unwrap() {
            abort_handle.abort();
        }
    }
}

impl fmt::Debug for StdinModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StdinModel")
         .field("id", &self.id)
         .finish()
    }
}

impl Model for StdinModel {
    
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

    fn shutdown(&self) {
        self.close()
    }
}

source!(stdin_read_source,
    core_identifier!("engine";"Read"),
    indoc!(r#"Read process standard input.

    Stream the lines received as input on stdin.
    
    ⚠️ When using `Read()` user may have to call `Close()` at some point,
    because unless program is explicitly ended or standard input stream closed
    by connected process, listening on stream will continue indefinitely."#).to_string(),
    models![
        ("stdin", crate::core::engine::stdin::StdinModel::descriptor())
    ],
    treatment_sources![
        (crate::core::engine::stdin::StdinModel::descriptor(), "read")
    ],
    outputs![
        output!("line",Scalar,String,Stream)
    ]
);

treatment!(stdin_close_treatment,
    core_identifier!("engine";"Close"),
    indoc!(r#"Closes the process standard input.

    ℹ️ This does not end program execution, see `End()` sequence for that."#).to_string(),
    models![
        ("stdin", crate::core::engine::stdin::StdinModel::descriptor())
    ],
    treatment_sources![],
    parameters![],
    inputs![
        input!("close",Scalar,Void,Block)
    ],
    outputs![],
    host {

        let stdin = std::sync::Arc::clone(&host.get_model("stdin")).downcast_arc::<crate::core::engine::stdin::StdinModel>().unwrap();

        let input = host.get_input("close");

        if let Ok(_) = input.recv_void().await {
            stdin.close();
        }
    
        ResultStatus::Ok
    }
);
