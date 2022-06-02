
use std::fmt::Debug;
use std::collections::{HashMap, hash_map::Entry};
use std::sync::{Arc, Weak, RwLock, atomic::{AtomicBool, Ordering}};
use futures::future::{JoinAll, join, join_all};
use futures::stream::{FuturesUnordered, StreamExt};
use async_std::task::block_on;
use async_std::sync::Mutex;
use async_std::channel::*;
use super::future::*;
use super::model::{Model, ModelId};
use super::output::Output;
use super::environment::{ContextualEnvironment, GenesisEnvironment};
use super::context::Context;
use super::result_status::ResultStatus;
use super::super::logic::descriptor::BuildableDescriptor;
use super::super::logic::descriptor::ModelDescriptor;
use super::super::logic::error::LogicError;
use super::super::logic::builder::*;

#[derive(Debug)]
struct SourceEntry {
    pub descriptor: Arc<dyn BuildableDescriptor>,
    pub id: BuildId,
}

pub type TrackId = u64;

// We don't use id nor parent_id for now, but might be useful for reporting implementations.
#[allow(dead_code)]
struct InfoTrack {
    pub id: TrackId,
    pub parent_id: Option<TrackId>,
    pub ancestry_level: u64,
    pub results: Option<TrackResult>,
}

impl InfoTrack {
    pub fn new(id: TrackId, parent_id: Option<TrackId>, ancestry_level: u64) -> Self {
        Self {
            id,
            parent_id,
            ancestry_level,
            results: None,
        }
    }
}

// We don't use ancestry_level for now, but might be useful for scheduling implementations.
#[allow(dead_code)]
struct ExecutionTrack {
    pub id: TrackId,
    pub ancestry_level: u64,
    pub future: JoinAll<TrackFuture>,
}

impl ExecutionTrack {
    pub fn new(id: TrackId, ancestry_level: u64, future: JoinAll<TrackFuture>) -> Self {
        Self {
            id,
            ancestry_level,
            future,
        }
    }
}

enum TrackResult {
    AllOk(TrackId),
    NotAllOk(TrackId, Vec<ResultStatus>),
}

pub struct World {

    auto_reference: Weak<Self>,

    models: RwLock<Vec<Arc<dyn Model>>>,
    sources: RwLock<HashMap<ModelId, HashMap<String, Vec<SourceEntry>>>>,

    errors: RwLock<Vec<LogicError>>,
    main_build_id: RwLock<BuildId>,

    continuous_tasks: RwLock<Vec<ContinuousFuture>>,

    tracks_counter: Mutex<u64>,
    tracks_info: Mutex<HashMap<TrackId, InfoTrack>>,
    tracks_sender: Sender<ExecutionTrack>,
    tracks_receiver: Receiver<ExecutionTrack>,

    closing: AtomicBool,
}

impl Debug for World {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
         .field("models", &self.models.read().unwrap().len())
         .field("sources", &self.sources.read().unwrap().len())
         .field("errors", &self.errors)
         .field("main_build_id", &self.main_build_id)
         .field("continuous_tasks", &self.continuous_tasks.read().unwrap().len())
         .finish()
    }
}

impl World {

    pub fn new() -> Arc<Self> {

        let (sender, receiver) = unbounded();

        Arc::new_cyclic(|me| Self {
            auto_reference: me.clone(),
            models: RwLock::new(Vec::new()),
            sources: RwLock::new(HashMap::new()),
            errors: RwLock::new(Vec::new()),
            main_build_id: RwLock::new(0),
            continuous_tasks: RwLock::new(Vec::new()),
            tracks_counter: Mutex::new(0),
            tracks_info: Mutex::new(HashMap::new()),
            tracks_sender: sender,
            tracks_receiver: receiver,
            closing: AtomicBool::new(false),
        })
    }

    pub fn add_model(&self, model: Arc<dyn Model>) -> ModelId {
        let mut models = self.models.write().unwrap();

        if let Some(position) = models.iter().position(|m| Arc::ptr_eq(m, &model)) {
            return position as u64;
        }

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

        let gen_env = GenesisEnvironment::new(Weak::upgrade(&self.auto_reference).unwrap());

        let result = beginning.builder().static_build(None, None, "main".to_string(), &gen_env);
        if result.is_err() {

            let error = result.unwrap_err();
            self.errors.write().unwrap().push(error);
            return false;
        }

        match result.unwrap() {
            StaticBuildResult::Build(b) => *self.main_build_id.write().unwrap() = b,
            _ => panic!("Cannot make a genesis with something else than a treatment")
        };

        
        // Check all the tracks/paths
        let models = self.models.read().unwrap();
        let mut builds = Vec::new();
        let mut errors = Vec::new();
        for (model_id, model_sources) in self.sources.read().unwrap().iter() {

            let model = models.get(*model_id as usize).unwrap();

            for (source, entries) in model_sources {

                let check_environment = CheckEnvironment {
                    contextes: model.descriptor().sources().get(source).unwrap().iter().map(|context| context.name().to_string()).collect()
                };

                for entry in entries {
                    let result = entry.descriptor.builder().check_dynamic_build(
                        entry.id,
                        check_environment.clone(),
                        Vec::new()
                    ).unwrap();

                    builds.extend(result.checked_builds);
                    errors.extend(result.errors);
                }
            }
        }

        // Check that all inputs are satisfied.
        for rc_check_build in builds {

            let borrowed_check_build = rc_check_build.read().unwrap();
            for (_input_name, input_satisfied) in &borrowed_check_build.fed_inputs {

                if !input_satisfied {
                    errors.push(LogicError::unsatisfied_input());
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

    pub async fn create_track(&self, id: ModelId, source: &str, contexts: HashMap<String, Context>, parent_track: Option<TrackId>, callback: Option<impl FnOnce(HashMap<String, Output>) -> Vec<TrackFuture>>) {

        let track_id;
        {
            let mut counter = self.tracks_counter.lock().await;
            track_id = *counter;
            *counter = track_id + 1;
        }

        let mut track_futures: Vec<TrackFuture> = Vec::new();
        let mut outputs: HashMap<String, Output> = HashMap::new();

        {
            let borrowed_sources = self.sources.read().unwrap();

            let model_sources = borrowed_sources.get(&id).unwrap();

            let entries = model_sources.get(source).unwrap();

            let mut contextual_environment = ContextualEnvironment::new(
                Weak::upgrade(&self.auto_reference).unwrap(),
                track_id
            );

            contexts.iter().for_each(|(name, context)| contextual_environment.add_context(name, context.clone()));

            for entry in entries {

                let build_result = entry.descriptor.builder().dynamic_build(entry.id, &contextual_environment).unwrap();

                track_futures.extend(build_result.prepared_futures);

                for (input_name, mut input_transmitters) in build_result.feeding_inputs {

                    match outputs.entry(input_name) {
                        Entry::Vacant(e) => {
                            let e = e.insert(Output::from(input_transmitters.pop().unwrap()));
                            input_transmitters.iter().for_each(|i| e.add_input(i));
                        },
                        Entry::Occupied(e) => {
                            input_transmitters.iter().for_each(|i| e.get().add_input(i));
                        }
                    }
                }
            }
        }

        let model_futures = if let Some(callback) = callback {
            callback(outputs)
        }
        else { Vec::new() };

        track_futures.extend(model_futures);

        let ancestry = if let Some(parent) = parent_track {
            self.tracks_info.lock().await.get(&parent).unwrap().ancestry_level + 1
        }
        else {
            0
        };

        let info_track = InfoTrack::new(track_id, parent_track, ancestry);
        let execution_track = ExecutionTrack::new(track_id, ancestry, join_all(track_futures));
        self.tracks_info.lock().await.insert(track_id, info_track);
        self.tracks_sender.send(execution_track).await.unwrap();
    }

    pub fn live(&self) {

        let mut borrowed_continuous_tasks = self.continuous_tasks.write().unwrap();

        let model_futures = join_all(borrowed_continuous_tasks.iter_mut());

        let continuum = async move {

            model_futures.await;

            self.tracks_sender.close();
            self.end();
        };

        block_on(join(continuum, self.run_tracks()));
    }

    /*
        TODO
        Probably prepare a distinction between "closing", that ends the program and let all stored tracks to finish (even create new ones?),
        and termination, that jut allows already running tracks to finish and ends models.
    */
    pub fn end(&self) {

        if !self.closing.load(Ordering::Relaxed) {
            self.models.read().unwrap().iter().for_each(|m| m.shutdown());
        }
        self.closing.store(true, Ordering::Relaxed);
    }

    async fn run_tracks(&self) {

        let mut futures = FuturesUnordered::new();

        async fn track_future(track: ExecutionTrack) -> TrackResult {

            let non_ok: Vec<ResultStatus> = track.future.await.iter().filter_map(
                // The `_ =>` is unreachable for now, but will ResultStatus will be complexified.
                #[allow(unreachable_patterns)]
                |r| match r { ResultStatus::Ok => None, _ => Some(r.clone()) }
            ).collect();

            if non_ok.is_empty() {
                TrackResult::AllOk(track.id)
            }
            else {
                TrackResult::NotAllOk(track.id, non_ok)
            }
        }

        while !self.closing.load(Ordering::Relaxed) {

            if !self.tracks_receiver.is_empty() {

                while let Ok(track) = self.tracks_receiver.try_recv() {

                    futures.push(track_future(track));
                }
            }
            else if futures.is_empty() {

                if let Ok(track) = self.tracks_receiver.recv().await {

                    futures.push(track_future(track));
                }
            }

            while let Some(result) = futures.next().await {

                match result {
                    TrackResult::AllOk(id) => {
                        self.tracks_info.lock().await.get_mut(&id).unwrap().results = Some(result);
                    },
                    TrackResult::NotAllOk(id, _) => {
                        self.tracks_info.lock().await.get_mut(&id).unwrap().results = Some(result);
                    },
                }
            }
        }
    }
}
