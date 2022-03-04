
use std::future::Future;
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock, Mutex};
use futures::future::{JoinAll, join, join_all};
use async_std::task::block_on;
use async_std::channel::*;
use super::future::*;
use super::model::{Model, ModelId};
use super::transmitter::Transmitter;
use super::input::Input;
use super::output::Output;
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

struct Track {
    pub id: u64,
    pub future: JoinAll<TrackFuture>,
}

impl Track {
    pub fn new(id: u64, future: JoinAll<TrackFuture>) -> Self {
        Self {
            id,
            future,
        }
    }
}

pub struct World {

    auto_reference: RwLock<Weak<Self>>,

    models: RwLock<Vec<Arc<dyn Model>>>,
    sources: RwLock<HashMap<ModelId, HashMap<String, Vec<SourceEntry>>>>,

    errors: RwLock<Vec<LogicError>>,
    main_build_id: RwLock<BuildId>,

    continuous_tasks: RwLock<Vec<ContinuousFuture>>,

    tracks_counter: Mutex<u64>,
    tracks_sender: Sender<Track>,
    tracks_receiver: Receiver<Track>,
    //tracks: RwLock<Vec<JoinAll<TrackFuture>>>,
}

impl Debug for World {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
         .field("models", &self.models.read().unwrap().len())
         .field("sources", &self.sources.read().unwrap().len())
         .field("errors", &self.errors)
         .field("main_build_id", &self.main_build_id)
         .field("continuous_tasks", &self.continuous_tasks.read().unwrap().len())
         //.field("tracks", &self.tracks.read().unwrap().len())
         .finish()
    }
}

impl World {

    pub fn new() -> Arc<Self> {

        let tracks_channel = unbounded();

        let world = Arc::new(Self {
            auto_reference: RwLock::new(Weak::default()),
            models: RwLock::new(Vec::new()),
            sources: RwLock::new(HashMap::new()),
            errors: RwLock::new(Vec::new()),
            main_build_id: RwLock::new(0),
            continuous_tasks: RwLock::new(Vec::new()),
            tracks_counter: Mutex::new(0),
            tracks_sender: tracks_channel.0,
            tracks_receiver: tracks_channel.1,
            //tracks: RwLock::new(Vec::new()),
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

    pub fn errors(&self) -> &RwLock<Vec<LogicError>> {
        &self.errors
    }

    pub fn add_continuous_task(&self, task: ContinuousFuture) {

        let mut borrowed_continuous_tasks = self.continuous_tasks.write().unwrap();

        borrowed_continuous_tasks.push(task);
    }

    // Special syntax for workaroud of async fn implementation
    // https://stackoverflow.com/questions/68591843/async-fn-reports-hidden-type-for-impl-trait-captures-lifetime-that-does-not-a
    // https://github.com/rust-lang/rust/issues/63033#issuecomment-521234696
    pub async fn create_track(&self, id: ModelId, source: &str, contexts: HashMap<String, Context>, parent_track: Option<u64>, callback: Option<impl FnOnce(HashMap<String, Output>) -> Vec<TrackFuture>>) {

        let track_id;
        {
            let mut counter = self.tracks_counter.lock().unwrap();
            track_id = *counter;
            *counter = track_id + 1;
        }

        let mut track_futures: Vec<TrackFuture> = Vec::new();
        let mut inputs: HashMap<String, Vec<Input>> = HashMap::new();

        {
            let borrowed_sources = self.sources.read().unwrap();

            let model_sources = borrowed_sources.get(&id).unwrap();

            let entries = model_sources.get(source).unwrap();

            let mut contextual_environment = ContextualEnvironment::new(
                Weak::upgrade(&self.auto_reference.read().unwrap()).unwrap(),
                track_id
            );

            contexts.iter().for_each(|(name, context)| contextual_environment.add_context(name, context.clone()));

            for entry in entries {

                let build_result = entry.descriptor.builder().dynamic_build(entry.id, &contextual_environment).unwrap();

                track_futures.extend(build_result.prepared_futures);

                for (input_name, input_transmitters) in build_result.feeding_inputs {

                    inputs.entry(input_name).or_default().extend(input_transmitters);
                }
            }
        }

        let model_futures = if let Some(callback) = callback {
            callback(inputs)
        }
        else { Vec::new() };

        track_futures.extend(model_futures);

        let track = Track::new(track_id, join_all(track_futures));
        self.tracks_sender.send(track).await.unwrap();
    }

    pub fn live(&self) {

        let mut borrowed_continuous_tasks = self.continuous_tasks.write().unwrap();

        let model_futures = join_all(borrowed_continuous_tasks.iter_mut());

        let continuum = async move {

            model_futures.await;
            self.tracks_sender.close();
        };

        block_on(join(self.run_track(), continuum));
    }

    async fn run_track(&self) {
        
        while let Ok(track) = self.tracks_receiver.recv().await {

            let _result = track.future.await;
        }
    }
}
