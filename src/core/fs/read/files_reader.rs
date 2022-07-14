
use crate::core::prelude::*;
use async_std::path::PathBuf;
use async_std::fs::File;

#[derive(Debug)]
pub struct FileReaderModel {

    helper: ModelHelper,

    #[allow(dead_code)]
    auto_reference: Weak<Self>,
}

impl FileReaderModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        model_desc!(
            FileReaderModel,
            core_identifier!("fs","read";"FileReader"),
            parameters![],
            model_sources![
                ("read"; "File"),
                ("unaccessible"; "File")
            ]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        Arc::new_cyclic(|me| Self {
            helper: ModelHelper::new(Self::descriptor(), world),

            auto_reference: me.clone(),
        })
    }

    pub async fn read(&self, path: &String) {

        let model_id = self.helper.id().unwrap();

        let os_path = PathBuf::from(path);
        let open_result = File::open(&os_path).await;

        let mut file_context = Context::new();

        let path = if let Ok(os_string) = os_path.canonicalize().await {
            os_string.into_os_string().into_string().unwrap_or_default()
        } else { "".to_string() };
        file_context.set_value("path", Value::String(path));

        let directory = if let Some(path) = os_path.parent() {
            if let Some(path) = path.to_str() {
                path.to_string()
            }
            else { "".to_string() }
        }
        else { "".to_string() };
        file_context.set_value("directory", Value::String(directory));

        let name = if let Some(name) = os_path.file_name() {
            if let Some(name) = name.to_str() {
                name.to_string()
            }
            else { "".to_string() }
        }
        else { "".to_string() };
        file_context.set_value("name", Value::String(name));

        let stem = if let Some(stem) = os_path.file_stem() {
            if let Some(stem) = stem.to_str() {
                stem.to_string()
            }
            else { "".to_string() }
        }
        else { "".to_string() };
        file_context.set_value("stem", Value::String(stem));

        let extension = if let Some(extension) = os_path.file_stem() {
            if let Some(extension) = extension.to_str() {
                extension.to_string()
            }
            else { "".to_string() }
        }
        else { "".to_string() };
        file_context.set_value("extension", Value::String(extension));

        let mut contextes = HashMap::new();
        contextes.insert("File".to_string(), file_context);

        match open_result {
            Ok(file) => {

                let reader = |inputs| {
                    self.read_file(file, inputs)
                };
                self.helper.world().create_track(model_id, "read", contextes, None, Some(reader)).await;
            },
            Err(err) => {

                let failer = |inputs| {
                    self.fail_file(err, inputs)
                };
                self.helper.world().create_track(model_id, "unaccessible", contextes, None, Some(failer)).await;
            },
        }
    }

    fn read_file(&self, mut file: File, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            let data_output = inputs.get("_data").unwrap();
            let failure_output = inputs.get("_failure").unwrap();
            let message_output = inputs.get("_message").unwrap();

            let mut buf = vec![0; 1048576];
            loop {

                match file.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }

                        ok_or_break!(data_output.send_multiple_byte(buf.get(0..n).unwrap().to_vec()).await);
                    },
                    Err(err) => {

                        let _ = futures::join!(failure_output.send_void(()),message_output.send_string(format!("{:?}", err.kind())));
                    },
                }
            }

            futures::join!(data_output.close(), failure_output.close(), message_output.close());

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }

    fn fail_file(&self, err: async_std::io::Error, inputs: HashMap<String, Output>) -> Vec<TrackFuture> {

        let future = Box::new(Box::pin(async move {

            let failure_output = inputs.get("_failure").unwrap();
            let message_output = inputs.get("_message").unwrap();

            let _ = futures::join!(failure_output.send_void(()),message_output.send_string(format!("{:?}", err.kind())));

            futures::join!(failure_output.close(), message_output.close());

            ResultStatus::Ok
        })) as TrackFuture;

        vec![future]
    }
}

model_trait!(FileReaderModel);
