

use crate::core::prelude::*;

#[derive(Debug)]
pub struct FilesWriterModel {

    helper: ModelHelper,

    #[allow(dead_code)]
    auto_reference: Weak<Self>,
}

impl FilesWriterModel {

    pub fn descriptor() -> Arc<CoreModelDescriptor> {
        
        model_desc!(
            FilesWriterModel,
            core_identifier!("fs","write";"FilesWriter"),
            parameters![
                parameter!("append", Scalar, Bool, Some(Value::Bool(false))),
                parameter!("create", Scalar, Bool, Some(Value::Bool(true))),
                parameter!("new", Scalar, Bool, Some(Value::Bool(false)))
            ],
            model_sources![]
        )
    }

    pub fn new(world: Arc<World>) -> Arc<dyn Model> {

        Arc::new_cyclic(|me| Self {
            helper: ModelHelper::new(Self::descriptor(), world),

            auto_reference: me.clone(),
        })
    }

    pub fn append(&self) -> bool {
        self.helper.get_parameter("append").bool()
    }

    pub fn create(&self) -> bool {
        self.helper.get_parameter("create").bool()
    }

    pub fn create_new(&self) -> bool {
        self.helper.get_parameter("new").bool()
    }
}

model_trait!(FilesWriterModel);
