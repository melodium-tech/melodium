
use crate::core::prelude::*;
use async_std::path::PathBuf;
use async_std::fs::{File, OpenOptions};
use async_std::io::BufWriter;

#[derive(Debug)]
pub struct FileWriterModel {

    world: Arc<World>,
    id: RwLock<Option<ModelId>>,

    path: RwLock<String>,
    append: RwLock<bool>,
    create: RwLock<bool>,
    new: RwLock<bool>,

    write_channel: (Sender<u8>, Receiver<u8>),

    auto_reference: RwLock<Weak<Self>>,
}

impl FileWriterModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {
                
                let builder = CoreModelBuilder::new(FileWriterModel::new);

                let descriptor = CoreModelDescriptor::new(
                    core_identifier!("fs","direct";"FileWriter"),
                    vec![
                        parameter!("path", Scalar, String, None),
                        parameter!("append", Scalar, Bool, Some(Value::Bool(false))),
                        parameter!("create", Scalar, Bool, Some(Value::Bool(true))),
                        parameter!("new", Scalar, Bool, Some(Value::Bool(false))),
                    ],
                    model_sources![],
                    Box::new(builder)
                );

                let rc_descriptor = Arc::new(descriptor);
                rc_descriptor.set_autoref(&rc_descriptor);

                rc_descriptor
            };
        }

        Arc::clone(&DESCRIPTOR)
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {
        let model = Arc::new(Self {
            world,
            id: RwLock::new(None),

            path: RwLock::new(String::new()),
            append: RwLock::new(false),
            create: RwLock::new(true),
            new: RwLock::new(false),

            write_channel: bounded(1048576),

            auto_reference: RwLock::new(Weak::new()),
        });

        *model.auto_reference.write().unwrap() = Arc::downgrade(&model);

        model
    }

    pub fn path(&self) -> String {
        self.path.read().unwrap().clone()
    }

    pub fn append(&self) -> bool {
        *self.append.read().unwrap()
    }

    pub fn create(&self) -> bool {
        *self.create.read().unwrap()
    }

    pub fn create_new(&self) -> bool {
        *self.new.read().unwrap()
    }

    pub fn writer(&self) -> &Sender<u8> {
        &self.write_channel.0
    }

    async fn write(&self) {

        let os_path = PathBuf::from(self.path());

        let mut open_options = OpenOptions::new();
        open_options
            .write(true)
            .append(self.append())
            .create(self.create())
            .create_new(self.create_new());

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

impl Model for FileWriterModel {
    
    fn descriptor(&self) -> Arc<CoreModelDescriptor> {
        Self::descriptor()
    }

    fn id(&self) -> Option<ModelId> {
        *self.id.read().unwrap()
    }

    fn set_id(&self, id: ModelId) {
        *self.id.write().unwrap() = Some(id);
    }

    fn set_parameter(&self, param: &str, value: &Value) {

        match param {
            "path" => {
                match value {
                    Value::String(path) => *self.path.write().unwrap() = path.to_string(),
                    _ => panic!("Unexpected value type for 'path'."),
                }
            },
            "append" => {
                match value {
                    Value::Bool(append) => *self.append.write().unwrap() = *append,
                    _ => panic!("Unexpected value type for 'append'."),
                }
            },
            "create" => {
                match value {
                    Value::Bool(create) => *self.create.write().unwrap() = *create,
                    _ => panic!("Unexpected value type for 'create'."),
                }
            },
            "new" => {
                match value {
                    Value::Bool(new) => *self.new.write().unwrap() = *new,
                    _ => panic!("Unexpected value type for 'new'."),
                }
            },
            _ => panic!("No parameter '{}' exists.", param)
        }
    }

    fn initialize(&self) {

        let auto_self = self.auto_reference.read().unwrap().upgrade().unwrap();
        let future_write = Box::pin(async move { auto_self.write().await });

        self.world.add_continuous_task(Box::new(future_write));
    }

    fn shutdown(&self) {

    }
}
