
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use futures::future::{BoxFuture, JoinAll, join_all};
use async_std::task::block_on;
use super::future::*;
use super::model::{Model, ModelId};
use super::transmitter::Transmitter;
use super::environment::{ContextualEnvironment, GenesisEnvironment};
use super::context::Context;
use super::super::logic::descriptor::BuildableDescriptor;
use super::super::logic::descriptor::ModelDescriptor;
use super::super::logic::error::LogicError;
use super::super::logic::builder::*;

#[derive(Debug)]
struct SourceEntry {
    pub descriptor: Arc<dyn BuildableDescriptor>,
    pub id: BuildId,
}

pub struct World {

    auto_reference: RwLock<Weak<Self>>,

    models: RwLock<Vec<Arc<dyn Model>>>,
    sources: RwLock<HashMap<ModelId, HashMap<String, Vec<SourceEntry>>>>,

    errors: RwLock<Vec<LogicError>>,
    main_build_id: RwLock<BuildId>,

    continuous_tasks: RwLock<Vec<ContinuousFuture>>,
    tracks: RwLock<Vec<JoinAll<TrackFuture>>>,
}

impl Debug for World {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
         .field("models", &self.models.read().unwrap().len())
         .field("sources", &self.sources.read().unwrap().len())
         .field("errors", &self.errors)
         .field("main_build_id", &self.main_build_id)
         .field("continuous_tasks", &self.continuous_tasks.read().unwrap().len())
         .field("tracks", &self.tracks.read().unwrap().len())
         .finish()
    }
}

impl World {

    pub fn new() -> Arc<Self> {
        let world = Arc::new(Self {
            auto_reference: RwLock::new(Weak::default()),
            models: RwLock::new(Vec::new()),
            sources: RwLock::new(HashMap::new()),
            errors: RwLock::new(Vec::new()),
            main_build_id: RwLock::new(0),
            continuous_tasks: RwLock::new(Vec::new()),
            tracks: RwLock::new(Vec::new()),
        });

        *world.auto_reference.write().unwrap() = Arc::downgrade(&world);

        world
    }

    pub fn add_model(&self, model: Arc<dyn Model>) -> ModelId {
        let mut models = self.models.write().unwrap();

        let mut sources = HashMap::new();
        for (name, _) in model.descriptor().sources() {
            sources.insert(name.to_owned(), Vec::new());
        }

        let id: ModelId = models.len() as u64;
        models.push(model);

        self.sources.write().unwrap().insert(id, sources);

        id
    }

    pub fn add_source(&self, model_id: ModelId, name: &str, descriptor: Arc<dyn BuildableDescriptor>, build_id: BuildId) {

        let mut sources = self.sources.write().unwrap();

        let model_sources = sources.get_mut(&model_id).unwrap();

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
        let mut errors = Vec::new();
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

                    errors.extend(result.errors);

                    // Check that all inputs are satisfied.
                    for rc_check_build in result.checked_builds {

                        let borrowed_check_build = rc_check_build.read().unwrap();
                        for (input_name, input_satisfied) in &borrowed_check_build.fed_inputs {

                            if !input_satisfied {
                                errors.push(LogicError::unsatisfied_input());
                            }
                        }
                    }
                }
            }
        }

        let mut borrowed_errors = self.errors.write().unwrap();
        borrowed_errors.extend(errors);

        if !borrowed_errors.is_empty() {
            return false
        }

        self.models.read().unwrap().iter().for_each(|m| m.initialize());

        true
    }

    pub fn add_continuous_task(&self, task: ContinuousFuture) {

        let mut borrowed_continuous_tasks = self.continuous_tasks.write().unwrap();

        borrowed_continuous_tasks.push(task);
    }

    pub fn create_track(&self, id: ModelId, source: &str, contexts: HashMap<String, Context>, parent_track: Option<u64>) -> HashMap<String, Vec<Transmitter>> {

        let borrowed_sources = self.sources.read().unwrap();

        let model_sources = borrowed_sources.get(&id).unwrap();

        println!("Model #{} source '{}'", id, source);
        println!("{:?}", model_sources.keys());
        let entries = model_sources.get(source).unwrap();

        let mut borrowed_tracks = self.tracks.write().unwrap();

        let mut contextual_environment = ContextualEnvironment::new(
            Weak::upgrade(&self.auto_reference.read().unwrap()).unwrap(),
            borrowed_tracks.len() as u64
        );

        contexts.iter().for_each(|(name, context)| contextual_environment.add_context(name, context.clone()));
        
        let mut track_futures: Vec<TrackFuture> = Vec::new();
        let mut inputs: HashMap<String, Vec<Transmitter>> = HashMap::new();

        for entry in entries {

            let build_result = entry.descriptor.builder().dynamic_build(entry.id, &contextual_environment).unwrap();

            track_futures.extend(build_result.prepared_futures);

            for (input_name, input_transmitters) in build_result.feeding_inputs {

                inputs.entry(input_name).or_default().extend(input_transmitters);
            }
        }

        let track = join_all(track_futures);
        borrowed_tracks.push(track);

        inputs
    }

    pub fn live(&self) {

        let mut borrowed_continuous_tasks = self.continuous_tasks.write().unwrap();

        let continuum = join_all(borrowed_continuous_tasks.iter_mut());

        block_on(continuum);
    }

    async fn run_track(&self) {
        
    }
}
