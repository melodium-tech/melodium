
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use super::Builder;
use super::super::descriptor::CoreModelDescriptor;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::model::Model;
use super::super::super::executive::value::Value;

/*#[derive(Debug)]
pub struct CoreModelBuilder<T: Model> {
}

impl<T: Model> Builder for CoreModelBuilder<T> {

    fn static_build(&self, environment: &dyn GenesisEnvironment, params: &HashMap<String, Value>) {

        //let arc_model = Arc::new();
        //environment.register_model(model: Arc<dyn Model>)
    }

    fn dynamic_build(&self,  environment: &dyn ContextualEnvironment) {

    }
}*/

pub struct CoreModelBuilder {

}


