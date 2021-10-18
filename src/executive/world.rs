
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use super::model::{Model, ModelId};
use super::transmitter::Transmitter;
use super::environment::GenesisEnvironment;
use super::context::Context;
use super::super::logic::descriptor::BuildableDescriptor;
use super::super::logic::error::LogicError;
use super::super::logic::builder::*;

#[derive(Debug)]
struct SourceEntry {
    pub descriptor: Arc<dyn BuildableDescriptor>,
    pub id: BuildId,
}

#[derive(Debug)]
pub struct World {

    auto_reference: RwLock<Weak<Self>>,

    models: RwLock<Vec<Arc<dyn Model>>>,
    sources: RwLock<HashMap<ModelId, HashMap<String, Vec<SourceEntry>>>>,

    errors: RwLock<Vec<LogicError>>,
    main_build_id: RwLock<BuildId>,
}

impl World {

    pub fn new() -> Arc<Self> {
        let world = Arc::new(Self {
            auto_reference: RwLock::new(Weak::default()),
            models: RwLock::new(Vec::new()),
            sources: RwLock::new(HashMap::new()),
            errors: RwLock::new(Vec::new()),
            main_build_id: RwLock::new(0),
        });

        *world.auto_reference.write().unwrap() = Arc::downgrade(&world);

        world
    }

    pub fn add_model(&self, model: Arc<dyn Model>) -> ModelId {
        let mut models = self.models.write().unwrap();

        let id: ModelId = models.len() as u64;
        models.push(model);

        self.sources.write().unwrap().insert(id, HashMap::new());

        id
    }

    pub fn add_source(&self, model_id: ModelId, name: &str, descriptor: Arc<dyn BuildableDescriptor>, build_id: BuildId) {

        let mut sources = self.sources.write().unwrap();

        let model_sources = sources.get_mut(&model_id).unwrap();

        if !model_sources.contains_key(&name.to_string()) {
            model_sources.insert(name.to_string(), Vec::new());
        }

        let sources = model_sources.get_mut(&name.to_string()).unwrap();

        sources.push(SourceEntry {
            descriptor,
            id: build_id
        });
    }

    pub fn genesis(&self, beginning: &dyn BuildableDescriptor) -> bool {

        let gen_env = GenesisEnvironment::new(Weak::upgrade(&self.auto_reference.read().unwrap()).unwrap());

        let result = beginning.builder().static_build(None, None, "main".to_string(), &gen_env);
        if result.is_err() {

            let error = result.unwrap_err();
            self.errors.write().unwrap().push(error);
            return false;
        }

        match result.unwrap() {
            StaticBuildResult::Build(b) => *self.main_build_id.write().unwrap() = b,
            _ => panic!("Cannot make a genesis with anything else than a treatment")
        };

        
        // Check all the tracks/paths
        let models = self.models.read().unwrap();
        for (model_id, model_sources) in self.sources.read().unwrap().iter() {

            let model = models.get(*model_id as usize).unwrap();

            for (source, entries) in model_sources {

                let check_environment = CheckEnvironment {
                    contextes: model.get_context_for(source)
                };

                for entry in entries {
                    let result = entry.descriptor.builder().check_dynamic_build(
                        entry.id,
                        check_environment.clone(),
                        Vec::new()
                    ).unwrap();

                    let errors = result.errors;

                    // TODO check also that all inputs are satisfied.
                }
            }
        }

        self.models.read().unwrap().iter().for_each(|m| m.initialize());

        true
    }

    pub fn create_track(&self, id: ModelId, source: &str, contexts: HashMap<String, Context>, parent_track: Option<u64>) -> Option<HashMap<String, Transmitter>> {

        // TODO instanciate track
        None
    }
}
