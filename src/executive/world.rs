
use std::sync::{Arc, Weak, RwLock};
use super::model::Model;
use super::environment::Environment;
use super::super::logic::descriptor::BuildableDescriptor;

#[derive(Debug)]
pub struct World {

    models: RwLock<Vec<Arc<dyn Model>>>,
    auto_reference: RwLock<Weak<Self>>,
}

impl World {

    pub fn new() -> Arc<Self> {
        let world = Arc::new(Self {
            models: RwLock::new(Vec::new()),
            auto_reference: RwLock::new(Weak::default()),
        });

        *world.auto_reference.write().unwrap() = Arc::downgrade(&world);

        world
    }

    pub fn add_model(&self, model: Arc<dyn Model>) {
        self.models.write().unwrap().push(model);
    }

    pub fn genesis(&self, beginning: &dyn BuildableDescriptor) {

        let gen_env = Environment::new(Weak::upgrade(&self.auto_reference.read().unwrap()).unwrap());

        beginning.builder().static_build(&gen_env);

        self.models.read().unwrap().iter().for_each(|m| m.initialize());
    }
}
