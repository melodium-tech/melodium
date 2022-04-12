
use crate::core::prelude::*;
use async_std::path::PathBuf;
use async_std::fs::OpenOptions;
use async_std::io::BufWriter;

#[derive(Debug)]
pub struct FileWriterModel {

    helper: ModelHelper,

    write_channel: (Sender<u8>, Receiver<u8>),

    auto_reference: RwLock<Weak<Self>>,
}

impl FileWriterModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        model_desc!(
            FileWriterModel,
            core_identifier!("fs","direct";"FileWriter"),
            vec![
                parameter!("path", Scalar, String, None),
                parameter!("append", Scalar, Bool, Some(Value::Bool(false))),
                parameter!("create", Scalar, Bool, Some(Value::Bool(true))),
                parameter!("new", Scalar, Bool, Some(Value::Bool(false))),
            ],
            model_sources![]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {
        let model = Arc::new(Self {
            helper: ModelHelper::new(Self::descriptor(), world),

            write_channel: bounded(1048576),

            auto_reference: RwLock::new(Weak::new()),
        });

        *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

        model
    }

    pub fn writer(&self) -> &Sender<u8> {
        &self.write_channel.0
    }

    fn initialize(&self) {

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future_write = Box::pin(async move { auto_self.write().await });

        self.helper.world().add_continuous_task(Box::new(future_write));
    }

    async fn write(&self) {

        let os_path = PathBuf::from(self.helper.get_parameter("path").string());

        let mut open_options = OpenOptions::new();
        open_options
            .write(true)
            .append(self.helper.get_parameter("append").bool())
            .create(self.helper.get_parameter("create").bool())
            .create_new(self.helper.get_parameter("new").bool());

        let open_result = open_options.open(&os_path).await;

        if let Ok(file) = open_result {

            let receiver = &self.write_channel.1;

            let mut writer = BufWriter::with_capacity(1048576, file);

            // We don't handle the recv_error case as it means everything is empty and closed
            while let Ok(data) = receiver.recv().await {
                if let Err(write_err) = writer.write(&[data]).await {

                    // Todo handle error
                    panic!("Writing error: {}", write_err)
                }

            }

            if let Err(write_err) = writer.flush().await {

                // Todo handle error
                panic!("Writing (flush) error: {}", write_err)
            }

        }
        else if let Err(error) = open_result {
            panic!("Unable to write file: {}", error)
        }

        // Todo manage failures
    }
}

model_trait!(FileWriterModel, initialize);
