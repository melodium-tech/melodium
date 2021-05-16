
use std::sync::{Arc, RwLock};
use super::model::Model;

#[derive(Debug)]
pub struct World {

    models: RwLock<Vec<Arc<dyn Model>>>
}

impl World {

    pub fn add_model(&self, model: Arc<dyn Model>) {
        self.models.write().unwrap().push(model);
    }
}
