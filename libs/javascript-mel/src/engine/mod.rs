use async_std::channel::{unbounded, Receiver, Sender};
use boa_engine::{Context, JsValue, Source};
use serde_json::Value;
use std::sync::Mutex;
use std::thread::JoinHandle;

#[derive(Debug)]
pub struct Engine {
    join: Mutex<Option<JoinHandle<()>>>,
    sender: Sender<(Value, String)>,
    receiver: Receiver<Result<Value, String>>,
}

impl Engine {
    pub fn new(
        stack_size: u64,
        recursion_limit: u64,
        loop_iteration_limit: u64,
        strict: bool,
        code: String,
    ) -> Self {
        let (sender, js_receiver) = unbounded();
        let (js_sender, receiver) = unbounded();

        Self {
            join: Mutex::new(Some(std::thread::spawn(move || {
                Self::inner_run(
                    stack_size,
                    recursion_limit,
                    loop_iteration_limit,
                    strict,
                    code,
                    js_receiver,
                    js_sender,
                )
            }))),
            sender,
            receiver,
        }
    }

    fn inner_run(
        stack_size: u64,
        recursion_limit: u64,
        loop_iteration_limit: u64,
        strict: bool,
        code: String,
        receiver: Receiver<(Value, String)>,
        sender: Sender<Result<Value, String>>,
    ) {
        let mut context = Context::default();

        context
            .runtime_limits_mut()
            .set_stack_size_limit(stack_size as usize);
        context
            .runtime_limits_mut()
            .set_recursion_limit(recursion_limit as usize);
        context
            .runtime_limits_mut()
            .set_loop_iteration_limit(loop_iteration_limit);
        context.strict(strict);

        match context.eval(Source::from_bytes(code.as_bytes())) {
            Ok(_v) => {}
            Err(_err) => {}
        }

        while let Ok((val, code)) = receiver.recv_blocking() {
            match JsValue::from_json(&val, &mut context) {
                Ok(value) => {
                    let _ = context
                        .global_object()
                        .delete_property_or_throw("value", &mut context);
                    let _ =
                        context
                            .global_object()
                            .create_data_property("value", value, &mut context);

                    let result = context.eval(Source::from_bytes(code.as_bytes()));

                    let result = match result {
                        Ok(val) => {
                            if !val.is_undefined() {
                                val.to_json(&mut context).map_err(|err| err.to_string())
                            } else {
                                Err("result is `undefined`".to_string())
                            }
                        }
                        Err(err) => Err(err.to_string()),
                    };

                    if sender.send_blocking(result).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    if sender.send_blocking(Err(err.to_string())).is_err() {
                        break;
                    }
                }
            }
        }

        receiver.close();
        sender.close();
    }

    pub async fn process(&self, value: Value, code: String) -> Result<Result<Value, String>, ()> {
        match self.sender.send((value, code)).await {
            Ok(_) => match self.receiver.recv().await {
                Ok(result) => Ok(result),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub fn stop(&self) {
        self.sender.close();
        self.receiver.close();
        if let Some(jh) = self.join.lock().unwrap().take() {
            let _ = jh.join();
        }
    }
}
