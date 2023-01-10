use super::{ExecutionTrack, InfoTrack, SourceEntry, TrackResult};
use crate::building::{
    model::get_builder as get_builder_model, treatment::get_builder as get_builder_treatment,
    BuildId, Builder, CheckEnvironment, ContextualEnvironment, GenesisEnvironment,
    StaticBuildResult,
};
use crate::engine::Engine;
use crate::error::LogicError;
use crate::executive::Context;
use crate::transmission::{Input, Output};
use async_std::channel::{unbounded, Receiver, Sender};
use async_std::sync::Mutex;
use async_std::task::block_on;
use async_trait::async_trait;
use core::fmt::Debug;
use core::marker::Send;
use futures::future::{join, join_all};
use futures::stream::{FuturesUnordered, StreamExt};
use melodium_common::descriptor::{
    Collection, Entry as CollectionEntry, Identified, Identifier, Input as InputDescriptor,
    Output as OutputDescriptor,
};
use melodium_common::executive::{
    Context as ExecutiveContext, ContinuousFuture, Model, ModelId, Output as ExecutiveOutput,
    ResultStatus, TrackFuture, TrackId, World as ExecutiveWorld,
};
use std::collections::{hash_map::Entry, HashMap};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock, Weak,
};

pub struct World {
    collection: Arc<Collection>,
    auto_reference: Weak<Self>,

    models: RwLock<Vec<Arc<dyn Model>>>,
    sources: RwLock<HashMap<ModelId, HashMap<String, Vec<SourceEntry>>>>,

    builders: RwLock<HashMap<Identifier, Arc<dyn Builder>>>,

    errors: RwLock<Vec<LogicError>>,
    main_build_id: RwLock<BuildId>,

    continuous_tasks: RwLock<Vec<ContinuousFuture>>,

    tracks_counter: Mutex<TrackId>,
    tracks_info: Mutex<HashMap<TrackId, InfoTrack>>,
    tracks_sender: Sender<ExecutionTrack>,
    tracks_receiver: Receiver<ExecutionTrack>,

    continous_ended: AtomicBool,
    closing: AtomicBool,
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("models", &self.models.read().unwrap().len())
            .field("sources", &self.sources.read().unwrap().len())
            .field("errors", &self.errors)
            .field("main_build_id", &self.main_build_id)
            .field(
                "continuous_tasks",
                &self.continuous_tasks.read().unwrap().len(),
            )
            .finish()
    }
}

impl World {
    pub fn new(collection: Arc<Collection>) -> Arc<Self> {
        let (sender, receiver) = unbounded();

        Arc::new_cyclic(|me| Self {
            collection,
            auto_reference: me.clone(),
            models: RwLock::new(Vec::new()),
            sources: RwLock::new(HashMap::new()),
            builders: RwLock::new(HashMap::new()),
            errors: RwLock::new(Vec::new()),
            main_build_id: RwLock::new(0),
            continuous_tasks: RwLock::new(Vec::new()),
            tracks_counter: Mutex::new(0),
            tracks_info: Mutex::new(HashMap::new()),
            tracks_sender: sender,
            tracks_receiver: receiver,
            continous_ended: AtomicBool::new(false),
            closing: AtomicBool::new(false),
        })
    }

    pub fn add_model(&self, model: Arc<dyn Model>) -> ModelId {
        let mut models = self.models.write().unwrap();

        if let Some(position) = models.iter().position(|m| Arc::ptr_eq(m, &model)) {
            return position;
        }

        let mut sources = HashMap::new();
        for (name, _) in model.descriptor().sources() {
            sources.insert(name.to_owned(), Vec::new());
        }

        let id: ModelId = models.len();
        models.push(model);

        self.sources.write().unwrap().insert(id, sources);

        id
    }

    pub fn add_source(
        &self,
        model_id: ModelId,
        name: &str,
        descriptor: Arc<dyn Identified>,
        build_id: BuildId,
    ) {
        let mut sources = self.sources.write().unwrap();

        let model_sources = sources.get_mut(&model_id).unwrap();

        let sources = model_sources.get_mut(&name.to_string()).unwrap();

        sources.push(SourceEntry {
            descriptor,
            id: build_id,
        });
    }

    pub fn builder(&self, identifier: &Identifier) -> Result<Arc<dyn Builder>, LogicError> {
        if let Some(builder) = self.builders.read().unwrap().get(identifier) {
            Ok(Arc::clone(builder))
        } else {
            if let Some(entry) = self.collection.get(identifier) {
                let builder = match entry {
                    CollectionEntry::Model(model) => get_builder_model(
                        self.auto_reference.upgrade().unwrap(),
                        &model.as_buildable(),
                    ),
                    CollectionEntry::Treatment(treatment) => get_builder_treatment(
                        self.auto_reference.upgrade().unwrap(),
                        &treatment.as_buildable(),
                    ),
                    _ => Err(LogicError::unavailable_design()),
                }?;

                self.builders
                    .write()
                    .unwrap()
                    .insert(identifier.clone(), Arc::clone(&builder));

                Ok(builder)
            } else {
                Err(LogicError::unavailable_design())
            }
        }
    }

    pub fn new_input(&self, descriptor: &InputDescriptor) -> Input {
        Input::new(descriptor)
    }

    pub fn new_output(&self, descriptor: &OutputDescriptor) -> Output {
        Output::new(descriptor)
    }

    async fn run_tracks(&self) {
        let mut futures = FuturesUnordered::new();

        async fn track_future(track: ExecutionTrack) -> TrackResult {
            let non_ok: Vec<ResultStatus> = track
                .future
                .await
                .iter()
                .filter_map(
                    // The `_ =>` is unreachable for now, but will ResultStatus will be complexified.
                    #[allow(unreachable_patterns)]
                    |r| match r {
                        ResultStatus::Ok => None,
                        _ => Some(r.clone()),
                    },
                )
                .collect();

            if non_ok.is_empty() {
                TrackResult::AllOk(track.id)
            } else {
                TrackResult::NotAllOk(track.id, non_ok)
            }
        }

        while !self.closing.load(Ordering::Relaxed) {
            if !self.tracks_receiver.is_empty() {
                while let Ok(track) = self.tracks_receiver.try_recv() {
                    futures.push(track_future(track));
                }
            } else if futures.is_empty() {
                self.check_closing();

                if let Ok(track) = self.tracks_receiver.recv().await {
                    futures.push(track_future(track));
                }
            }

            while let Some(result) = futures.next().await {
                match result {
                    TrackResult::AllOk(id) => {
                        self.tracks_info.lock().await.get_mut(&id).unwrap().results = Some(result);
                    }
                    TrackResult::NotAllOk(id, _) => {
                        self.tracks_info.lock().await.get_mut(&id).unwrap().results = Some(result);
                    }
                }
            }
        }
    }

    fn check_closing(&self) {
        if self.continous_ended.load(Ordering::Relaxed) && self.tracks_receiver.len() == 0 {
            self.tracks_sender.close();
            self.end();
        }
    }
}

impl Engine for World {
    fn collection(&self) -> Arc<Collection> {
        Arc::clone(&self.collection)
    }

    fn genesis(&self, beginning: &Identifier) -> Result<(), Vec<LogicError>> {
        let gen_env = GenesisEnvironment::new();

        let result =
            self.builder(beginning)
                .unwrap()
                .static_build(None, None, "main".to_string(), &gen_env);
        if result.is_err() {
            let error = result.unwrap_err();
            let mut errors = self.errors.write().unwrap();
            errors.push(error);
            return Err(errors.clone());
        }

        match result.unwrap() {
            StaticBuildResult::Build(b) => *self.main_build_id.write().unwrap() = b,
            _ => panic!("Cannot make a genesis with something else than a treatment"),
        };

        // Check all the tracks/paths
        let models = self.models.read().unwrap();
        let mut builds = Vec::new();
        let mut errors = Vec::new();
        for (model_id, model_sources) in self.sources.read().unwrap().iter() {
            let model = models.get(*model_id as usize).unwrap();

            for (source, entries) in model_sources {
                let check_environment = CheckEnvironment {
                    contextes: model
                        .descriptor()
                        .sources()
                        .get(source)
                        .unwrap()
                        .iter()
                        .map(|context| context.name().to_string())
                        .collect(),
                };

                for entry in entries {
                    let result = self
                        .builder(entry.descriptor.identifier())
                        .unwrap()
                        .check_dynamic_build(entry.id, check_environment.clone(), Vec::new())
                        .unwrap();

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
            return Err(borrowed_errors.clone());
        }

        self.models
            .read()
            .unwrap()
            .iter()
            .for_each(|m| m.initialize());

        Ok(())
    }

    fn errors(&self) -> Vec<LogicError> {
        self.errors.read().unwrap().clone()
    }

    fn live(&self) {
        let mut borrowed_continuous_tasks = self.continuous_tasks.write().unwrap();

        let model_futures = join_all(borrowed_continuous_tasks.iter_mut());

        let continuum = async move {
            model_futures.await;

            self.continous_ended.store(true, Ordering::Relaxed);
        };

        block_on(join(continuum, self.run_tracks()));
    }

    /*
        TODO
        Probably prepare a distinction between "closing", that ends the program and let all stored tracks to finish (even create new ones?),
        and termination, that jut allows already running tracks to finish and ends models.
    */
    fn end(&self) {
        if !self.closing.load(Ordering::Relaxed) {
            self.models
                .read()
                .unwrap()
                .iter()
                .for_each(|m| m.shutdown());
        }
        self.closing.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl ExecutiveWorld for World {
    fn new_context(&self, identifier: &Identifier) -> Box<dyn ExecutiveContext> {
        let descriptor = match self.collection.get(identifier).expect("Unknown identifier") {
            CollectionEntry::Context(id) => Arc::clone(id),
            _ => panic!("Identifier `{}` doesn't refer to context.", identifier),
        };
        Box::new(Context::new(descriptor))
    }

    fn add_continuous_task(&self, task: ContinuousFuture) {
        let mut borrowed_continuous_tasks = self.continuous_tasks.write().unwrap();

        borrowed_continuous_tasks.push(task);
    }

    async fn create_track(
        &self,
        id: ModelId,
        source: &str,
        contexts: Vec<Box<dyn ExecutiveContext>>,
        parent_track: Option<TrackId>,
        callback: Option<
            impl FnOnce(HashMap<String, Box<dyn ExecutiveOutput>>) -> Vec<TrackFuture> + Send,
        >,
    ) {
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

            let mut contextual_environment = ContextualEnvironment::new(track_id);

            contexts.into_iter().for_each(|context| {
                contextual_environment.add_context(context.descriptor().name(), context)
            });

            for entry in entries {
                let build_result = self
                    .builder(entry.descriptor.identifier())
                    .unwrap()
                    .dynamic_build(entry.id, &contextual_environment)
                    .unwrap();

                track_futures.extend(build_result.prepared_futures);

                for (input_name, mut input_transmitters) in build_result.feeding_inputs {
                    match outputs.entry(input_name) {
                        Entry::Vacant(e) => {
                            let e = e.insert(Output::from(input_transmitters.pop().unwrap()));
                            e.add_transmission(&input_transmitters);
                        }
                        Entry::Occupied(e) => {
                            e.get().add_transmission(&input_transmitters);
                        }
                    }
                }
            }
        }

        let model_futures = if let Some(callback) = callback {
            callback(
                outputs
                    .into_iter()
                    .map(|(name, output)| (name, Box::new(output) as Box<dyn ExecutiveOutput>))
                    .collect(),
            )
        } else {
            Vec::new()
        };

        track_futures.extend(model_futures);

        let ancestry = if let Some(parent) = parent_track {
            self.tracks_info
                .lock()
                .await
                .get(&parent)
                .unwrap()
                .ancestry_level
                + 1
        } else {
            0
        };

        let info_track = InfoTrack::new(track_id, parent_track, ancestry);
        let execution_track = ExecutionTrack::new(track_id, ancestry, join_all(track_futures));
        self.tracks_info.lock().await.insert(track_id, info_track);
        self.tracks_sender.send(execution_track).await.unwrap();
    }
}
