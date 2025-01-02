use super::{ExecutionTrack, InfoTrack, SourceEntry, TrackResult};
use crate::building::HostTreatment;
use crate::building::{
    model::get_builder as get_builder_model, treatment::get_builder as get_builder_treatment,
    BuildId, Builder, CheckEnvironment, ContextualEnvironment, FeedingInputs, GenesisEnvironment,
    StaticBuildResult,
};
use crate::engine::Engine;
use crate::error::{LogicError, LogicErrors, LogicResult};
use crate::transmission::{Input, Output, Outputs};
use async_std::channel::{unbounded, Receiver, Sender};
use async_std::sync::{Barrier, Mutex};
use async_std::task::block_on;
use async_trait::async_trait;
use core::fmt::Debug;
use futures::future::join;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::{pin_mut, select, FutureExt};
use melodium_common::descriptor::{
    Collection, Entry as CollectionEntry, Identified, Identifier, Treatment,
};
use melodium_common::executive::{
    Context as ExecutiveContext, ContinuousFuture, DirectCreationCallback, Input as ExecutiveInput,
    Model, ModelId, Output as ExecutiveOutput, ResultStatus, TrackCreationCallback, TrackFuture,
    TrackId, Value, World as ExecutiveWorld,
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

    errors: RwLock<LogicErrors>,
    main: RwLock<Option<Arc<dyn Treatment>>>,
    main_id: RwLock<Option<Identifier>>,
    main_build_id: RwLock<BuildId>,
    main_tracks: RwLock<HashMap<TrackId, FeedingInputs>>,

    continuous_tasks_sender: Sender<ContinuousFuture>,
    continuous_tasks_receiver: Receiver<ContinuousFuture>,

    tracks_counter: Mutex<TrackId>,
    tracks_info: Mutex<HashMap<TrackId, InfoTrack>>,
    tracks_sender: Sender<ExecutionTrack>,
    tracks_receiver: Receiver<ExecutionTrack>,

    close_at_continuous_end: AtomicBool,
    continous_ended: AtomicBool,
    continous_ended_barrier: Barrier,
    closing: AtomicBool,
    closing_barrier: Barrier,
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("models", &self.models.read().unwrap().len())
            .field("sources", &self.sources.read().unwrap().len())
            .field("errors", &self.errors)
            .field("main_build_id", &self.main_build_id)
            .field("continuous_tasks", &self.continuous_tasks_receiver.len())
            .finish()
    }
}

impl World {
    pub fn new(collection: Arc<Collection>) -> Arc<Self> {
        let (tracks_sender, tracks_receiver) = unbounded();
        let (continuous_tasks_sender, continuous_tasks_receiver) = unbounded();

        Arc::new_cyclic(|me| Self {
            collection,
            auto_reference: me.clone(),
            models: RwLock::new(Vec::new()),
            sources: RwLock::new(HashMap::new()),
            builders: RwLock::new(HashMap::new()),
            errors: RwLock::new(Vec::new()),
            main: RwLock::new(None),
            main_id: RwLock::new(None),
            main_build_id: RwLock::new(0),
            main_tracks: RwLock::new(HashMap::new()),
            continuous_tasks_sender,
            continuous_tasks_receiver,
            tracks_counter: Mutex::new(0),
            tracks_info: Mutex::new(HashMap::new()),
            tracks_sender,
            tracks_receiver,
            close_at_continuous_end: AtomicBool::new(true),
            continous_ended: AtomicBool::new(false),
            continous_ended_barrier: Barrier::new(2),
            closing: AtomicBool::new(false),
            closing_barrier: Barrier::new(2),
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
        params: HashMap<String, Value>,
        build_id: BuildId,
    ) {
        let mut sources = self.sources.write().unwrap();

        let model_sources = sources.get_mut(&model_id).unwrap();

        let sources = model_sources.get_mut(&name.to_string()).unwrap();

        sources.push(SourceEntry {
            descriptor,
            id: build_id,
            params: params.clone(),
        });

        let model = self.models.read().unwrap()[model_id].clone();

        model.invoke_source(name, params);
    }

    pub fn direct(&self, id: &TrackId) -> LogicResult<FeedingInputs> {
        let possible_build_result;
        {
            possible_build_result = self
                .main_tracks
                .read()
                .unwrap()
                .get(id)
                .map(|feeding_inputs| feeding_inputs.clone());
        }

        if let Some(dbr) = possible_build_result {
            LogicResult::new_success(dbr)
        } else {
            LogicResult::new_failure(LogicError::no_direct_track(242, id.clone()))
        }
    }

    pub fn builder(&self, identifier: &Identifier) -> LogicResult<Arc<dyn Builder>> {
        let possible_builder;
        {
            possible_builder = self.builders.read().unwrap().get(identifier).cloned();
        }
        if let Some(builder) = possible_builder {
            LogicResult::new_success(builder)
        } else {
            if let Some(entry) = self.collection.get(&identifier.into()) {
                let builder = match entry {
                    CollectionEntry::Model(model) => get_builder_model(
                        self.auto_reference.upgrade().unwrap(),
                        &model.as_buildable(),
                    ),
                    CollectionEntry::Treatment(treatment) => get_builder_treatment(
                        self.auto_reference.upgrade().unwrap(),
                        &treatment.as_buildable(),
                    ),
                    _ => LogicResult::new_failure(
                        LogicError::unavailable_design(19, identifier.clone(), None).into(),
                    ),
                };

                if let Some(builder) = builder.success() {
                    self.builders
                        .write()
                        .unwrap()
                        .insert(identifier.clone(), Arc::clone(builder));
                }

                builder
            } else {
                LogicResult::new_failure(
                    LogicError::unavailable_design(20, identifier.clone(), None).into(),
                )
            }
        }
    }

    pub fn new_input(&self) -> Input {
        Input::new()
    }

    pub fn new_blocked_input(&self) -> Input {
        let input = Input::new();
        input.close();
        input
    }

    pub fn new_output(&self) -> Output {
        Output::new()
    }

    async fn run_tracks(&self) {
        let mut futures = FuturesUnordered::new();

        let mut tracks_receiver = self.tracks_receiver.clone();
        let closing_barrier = self.closing_barrier.wait().fuse();
        pin_mut!(closing_barrier);
        let continous_ended_barrier = self.continous_ended_barrier.wait().fuse();
        pin_mut!(continous_ended_barrier);

        async fn track_future(mut track: ExecutionTrack) -> TrackResult {
            let mut non_ok: Vec<ResultStatus> = Vec::new();
            while let Some(r) = track.future.next().await {
                // The `_ =>` is unreachable for now, but will ResultStatus will be complexified.
                #[allow(unreachable_patterns)]
                match r {
                    ResultStatus::Ok => {}
                    _ => non_ok.push(r.clone()),
                }
            }

            if non_ok.is_empty() {
                TrackResult::AllOk(track.id)
            } else {
                TrackResult::NotAllOk(track.id, non_ok)
            }
        }

        loop {
            select! {
                received_track = tracks_receiver.select_next_some() => {
                    futures.push(track_future(received_track));
                },
                result = futures.select_next_some() => {
                    match result {
                        TrackResult::AllOk(id) => {
                            self.tracks_info.lock().await.get_mut(&id).unwrap().results = Some(result);
                        }
                        TrackResult::NotAllOk(id, _) => {
                            self.tracks_info.lock().await.get_mut(&id).unwrap().results = Some(result);
                        }
                    }
                },
                _result = continous_ended_barrier => {
                    self.check_closing().await;
                },
                _result = closing_barrier => {},
                complete => break,
            }
        }
    }

    async fn check_closing(&self) {
        if self.auto_end()
            && self.continous_ended.load(Ordering::Relaxed)
            && self.tracks_receiver.len() == 0
        {
            self.end().await;
        }
    }
}

#[async_trait]
impl Engine for World {
    fn collection(&self) -> Arc<Collection> {
        Arc::clone(&self.collection)
    }

    fn genesis(&self, entry: &Identifier, mut params: HashMap<String, Value>) -> LogicResult<()> {
        let mut gen_env = GenesisEnvironment::new();

        {
            if self.main_id.read().unwrap().is_some() {
                return LogicResult::new_success(());
            }
        }

        let descriptor = if let Some(CollectionEntry::Treatment(descriptor)) =
            self.collection.get(&entry.into())
        {
            for (name, param) in descriptor.parameters() {
                if let Some(value) = params.remove(name).filter(|val| {
                    param
                        .described_type()
                        .to_datatype(&HashMap::new())
                        .map(|dt| dt == val.datatype())
                        .unwrap_or(false)
                }) {
                    gen_env.add_variable(name, value);
                } else if param.default().is_some() {
                    continue;
                } else {
                    return LogicResult::new_failure(LogicError::launch_wrong_parameter(
                        225,
                        name.clone(),
                    ));
                }
            }
            Arc::clone(descriptor)
        } else {
            return LogicResult::new_failure(LogicError::launch_expect_treatment(
                224,
                entry.clone(),
            ));
        };

        let result = self.builder(entry).and_then(|builder| {
            builder.static_build(HostTreatment::Direct, None, "main".to_string(), &gen_env)
        });
        if let Some(failure) = result.failure() {
            let mut errors = self.errors.write().unwrap();
            errors.append(&mut result.errors().clone());
            errors.push(failure.clone());
            return result.and(LogicResult::new_success(()));
        }

        match result.success().unwrap() {
            StaticBuildResult::Build(b) => *self.main_build_id.write().unwrap() = *b,
            _ => panic!("Cannot make a genesis with something else than a treatment"),
        };

        // Check all the tracks/paths
        let mut result = result.and_degrade_failure(LogicResult::new_success(()));
        let models = self.models.read().unwrap();
        let mut builds = Vec::new();
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
                    result = result.and_degrade_failure(
                        self.builder(entry.descriptor.identifier())
                            .and_then(|builder| {
                                let check = builder
                                    .check_dynamic_build(
                                        entry.id,
                                        check_environment.clone(),
                                        Vec::new(),
                                    )
                                    .unwrap();
                                builds.extend(check.checked_builds);
                                let mut result = LogicResult::new_success(());
                                result.errors_mut().extend(check.errors);
                                result
                            }),
                    );
                }
            }
        }

        let mut borrowed_errors = self.errors.write().unwrap();
        borrowed_errors.extend(result.errors().clone());

        if result.is_success() {
            if result.errors().is_empty() {
                {
                    self.main.write().unwrap().replace(descriptor);
                    self.main_id.write().unwrap().replace(entry.clone());
                }
                self.models
                    .read()
                    .unwrap()
                    .iter()
                    .for_each(|m| m.initialize());
                Ok(()).into()
            } else {
                result.and(LogicResult::new_failure(LogicError::erroneous_checks(
                    69, None,
                )))
            }
        } else {
            result
        }
    }

    fn errors(&self) -> LogicErrors {
        self.errors.read().unwrap().clone()
    }

    fn set_auto_end(&self, auto_end: bool) {
        self.close_at_continuous_end
            .store(auto_end, Ordering::Relaxed);
    }

    fn auto_end(&self) -> bool {
        self.close_at_continuous_end.load(Ordering::Relaxed)
    }

    async fn live(&self) {
        let me = self.auto_reference.upgrade().unwrap();
        let continuum = {
            let me = Arc::clone(&me);
            async move {
                let mut continuous = FuturesUnordered::new();

                while let Ok(c) = me.continuous_tasks_receiver.recv().await {
                    continuous.push(c);
                }

                while let Some(_) = continuous.next().await {}

                me.continous_ended.store(true, Ordering::Relaxed);
                me.continous_ended_barrier.wait().await;
            }
        };

        self.continuous_tasks_sender.close();

        join(continuum, async move { me.run_tracks().await }).await;
    }

    async fn instanciate(&self, callback: Option<DirectCreationCallback>) {
        let main;
        let main_id;
        let main_build_id;
        {
            if let (Some(descriptor), Some(identifier)) = (
                self.main.read().unwrap().as_ref(),
                self.main_id.read().unwrap().as_ref(),
            ) {
                main = Arc::clone(descriptor);
                main_id = identifier.clone();
                main_build_id = *self.main_build_id.read().unwrap();
            } else {
                return;
            }
        }

        let track_id;
        {
            let mut counter = self.tracks_counter.lock().await;
            track_id = *counter;
            *counter = track_id + 1;
        }

        let mut inputs = HashMap::new();
        for (name, _) in main.inputs() {
            let input = self.new_input();
            inputs.insert(name.clone(), input);
        }
        self.main_tracks.write().unwrap().insert(
            track_id,
            inputs
                .iter()
                .map(|(name, input)| (name.clone(), vec![input.clone()]))
                .collect(),
        );

        let mut track_futures: Vec<TrackFuture> = Vec::new();
        let mut outputs: HashMap<String, Output> = HashMap::new();
        {
            let contextual_environment = ContextualEnvironment::new(track_id);

            let build_result = self
                .builder(&main_id)
                .success()
                .unwrap()
                .dynamic_build(main_build_id, &contextual_environment)
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

        let super_futures = if let Some(callback) = callback {
            let outputs = outputs
                .into_iter()
                .map(|(name, output)| (name, Box::new(output) as Box<dyn ExecutiveOutput>))
                .collect();
            let inputs = inputs
                .into_iter()
                .map(|(name, input)| (name, Box::new(input) as Box<dyn ExecutiveInput>))
                .collect();
            callback(outputs, inputs)
        } else {
            Vec::new()
        };

        track_futures.extend(super_futures);

        let info_track = InfoTrack::new(track_id, None, 0);
        let execution_track = ExecutionTrack::new(track_id, 0, track_futures.into_iter().collect());
        self.tracks_info.lock().await.insert(track_id, info_track);
        let _ = self.tracks_sender.send(execution_track).await;
    }

    /*
        TODO
        Probably prepare a distinction between "closing", that ends the program and let all stored tracks to finish (even create new ones?),
        and termination, that jut allows already running tracks to finish and ends models.
    */
    async fn end(&self) {
        if !self.closing.load(Ordering::Relaxed) {
            self.models
                .read()
                .unwrap()
                .iter()
                .for_each(|m| m.shutdown());
            self.tracks_sender.close();
            self.closing.store(true, Ordering::Relaxed);
            self.closing_barrier.wait().await;
        }
    }
}

#[async_trait]
impl ExecutiveWorld for World {
    fn collection(&self) -> Arc<Collection> {
        Arc::clone(&self.collection)
    }

    fn add_continuous_task(&self, task: ContinuousFuture) {
        let me = self.auto_reference.upgrade().unwrap();
        block_on(async move {
            let _ = me.continuous_tasks_sender.send(task).await;
        });
    }

    async fn create_track(
        &self,
        id: ModelId,
        source: &str,
        params: &HashMap<String, Value>,
        contexts: Vec<Arc<dyn ExecutiveContext>>,
        parent_track: Option<TrackId>,
        callback: Option<TrackCreationCallback>,
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
                if &entry.params == params {
                    let build_result = self
                        .builder(entry.descriptor.identifier())
                        .success()
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
        }

        let model_futures = if let Some(callback) = callback {
            callback(Box::new(Outputs::new(outputs)))
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
        let execution_track =
            ExecutionTrack::new(track_id, ancestry, track_futures.into_iter().collect());
        self.tracks_info.lock().await.insert(track_id, info_track);
        let _ = self.tracks_sender.send(execution_track).await;
    }
}
