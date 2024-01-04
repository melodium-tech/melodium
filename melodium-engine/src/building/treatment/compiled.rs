use crate::building::Builder as BuilderTrait;
use crate::building::{
    BuildId, CheckBuild, CheckBuildResult, CheckEnvironment, CheckStep, ContextualEnvironment,
    DynamicBuildResult, FeedingInputs, GenesisEnvironment, StaticBuildResult,
};
use crate::error::{LogicError, LogicResult};
use crate::world::World;
use core::fmt::Debug;
use melodium_common::descriptor::{Status, Treatment as TreatmentDescriptor};
use melodium_common::executive::{TrackId, Treatment};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
struct BuildSample {
    genesis_environment: GenesisEnvironment,
    host_treatment: Option<Arc<dyn TreatmentDescriptor>>,
    host_build_id: Option<BuildId>,
    check: Arc<RwLock<CheckBuild>>,
    label: String,
}

impl BuildSample {
    pub fn new(
        host_treatment: &Option<Arc<dyn TreatmentDescriptor>>,
        host_build: &Option<BuildId>,
        label: &str,
        environment: &GenesisEnvironment,
    ) -> Self {
        Self {
            genesis_environment: environment.clone(),
            host_treatment: host_treatment.clone(),
            host_build_id: host_build.clone(),
            check: Arc::new(RwLock::new(CheckBuild::new(
                host_treatment
                    .as_ref()
                    .map(|descriptor| descriptor.identifier())
                    .cloned(),
                label,
            ))),
            label: label.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Builder {
    world: Weak<World>,

    build_fn: fn() -> Arc<dyn Treatment>,
    descriptor: Weak<dyn TreatmentDescriptor>,

    builds: RwLock<Vec<BuildSample>>,
    building_inputs: RwLock<HashMap<(BuildId, TrackId), FeedingInputs>>,
}

impl Builder {
    pub fn new(
        world: Weak<World>,
        descriptor: Weak<dyn TreatmentDescriptor>,
        build_fn: fn() -> Arc<dyn Treatment>,
    ) -> Self {
        Self {
            world,
            build_fn,
            descriptor,
            builds: RwLock::new(Vec::new()),
            building_inputs: RwLock::new(HashMap::new()),
        }
    }
}

impl BuilderTrait for Builder {
    fn static_build(
        &self,
        host_treatment: Option<Arc<dyn TreatmentDescriptor>>,
        host_build: Option<BuildId>,
        label: String,
        environment: &GenesisEnvironment,
    ) -> LogicResult<StaticBuildResult> {
        let world = self.world.upgrade().unwrap();
        // Make a BuildSample with matching informations
        let build_sample = BuildSample::new(&host_treatment, &host_build, &label, environment);

        let mut builds_writer = self.builds.write().unwrap();
        let idx = builds_writer.len() as BuildId;

        let rc_descriptor = self.descriptor.upgrade().unwrap();
        for (model_name, sources) in rc_descriptor.source_from() {
            let (_, matching_model) = environment
                .models()
                .iter()
                .find(|(name, _)| name == &model_name)
                .unwrap();

            for source in sources {
                world.add_source(
                    matching_model.id().unwrap(),
                    source,
                    TreatmentDescriptor::as_identified(&*self.descriptor.upgrade().unwrap()),
                    idx,
                );
            }
        }

        builds_writer.push(build_sample);

        Status::new_success(StaticBuildResult::Build(idx))
    }

    fn dynamic_build(
        &self,
        build: BuildId,
        environment: &ContextualEnvironment,
    ) -> Option<DynamicBuildResult> {
        let world = self.world.upgrade().unwrap();

        // Look for existing build
        {
            let borrowed_building_inputs = self.building_inputs.read().unwrap();

            if let Some(existing_building_inputs) =
                borrowed_building_inputs.get(&(build, environment.track_id()))
            {
                let mut dynamic_result = DynamicBuildResult::new();
                dynamic_result
                    .feeding_inputs
                    .extend(existing_building_inputs.clone());

                return Some(dynamic_result);
            }
        }

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(build as usize).unwrap();
        let descriptor = self.descriptor.upgrade().unwrap();

        let mut result = DynamicBuildResult::new();

        let treatment = (self.build_fn)();

        for (name, model) in build_sample.genesis_environment.models() {
            treatment.set_model(name, Arc::clone(model));
        }

        for (name, value) in environment.variables() {
            treatment.set_parameter(name, value.clone());
        }

        let host_descriptor = build_sample.host_treatment.as_ref().unwrap();
        let host_build = world
            .builder(host_descriptor.identifier())
            .success()
            .unwrap()
            .give_next(
                build_sample.host_build_id.unwrap(),
                build_sample.label.to_string(),
                &environment.base_on(),
            )
            .unwrap();

        let mut inputs = HashMap::new();
        for (name, _) in descriptor.inputs() {
            let input = world.new_input();
            treatment.assign_input(name, Box::new(input.clone()));
            inputs.insert(name.clone(), input);
        }
        for (name, _) in descriptor.outputs() {
            let output = world.new_output();
            if let Some(inputs) = host_build.feeding_inputs.get(name) {
                output.add_transmission(inputs);
            }
            treatment.assign_output(name, Box::new(output));
        }

        result.feeding_inputs = inputs
            .iter()
            .map(|(name, input)| (name.to_string(), vec![input.clone()]))
            .collect();

        let prepared_futures = treatment.prepare();
        result.prepared_futures.extend(prepared_futures);
        result.prepared_futures.extend(host_build.prepared_futures);

        self.building_inputs.write().unwrap().insert(
            (build, environment.track_id()),
            result.feeding_inputs.clone(),
        );

        Some(result)
    }

    fn give_next(
        &self,
        _within_build: BuildId,
        _for_label: String,
        _environment: &ContextualEnvironment,
    ) -> Option<DynamicBuildResult> {
        // A core treatment cannot have sub-treatments (it is not a sequence), so nothing to ever return.
        None
    }

    fn check_dynamic_build(
        &self,
        build: BuildId,
        environment: CheckEnvironment,
        previous_steps: Vec<CheckStep>,
    ) -> Option<CheckBuildResult> {
        let world = self.world.upgrade().unwrap();
        let descriptor = self.descriptor.upgrade().unwrap();

        let mut errors = Vec::new();
        // Check if we're not in our own previous steps
        let check_step = CheckStep {
            identifier: descriptor.identifier().clone(),
            build_id: build,
        };
        if let Some(_existing_check_step) = previous_steps.iter().find(|&cs| cs == &check_step) {
            errors.push(LogicError::already_included_build_step(
                56,
                descriptor.identifier().clone(),
                check_step.clone(),
                previous_steps.clone(),
                None,
            ));
        }
        let mut current_previous_steps = previous_steps.clone();
        current_previous_steps.push(check_step);

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(build as usize).unwrap();

        let mut all_builds = Vec::new();
        if errors.is_empty() {
            let host_descriptor = build_sample.host_treatment.as_ref().unwrap();
            let build_result = world
                .builder(host_descriptor.identifier())
                .success()
                .unwrap()
                .check_give_next(
                    build_sample.host_build_id.unwrap(),
                    build_sample.label.to_string(),
                    environment.clone(),
                    current_previous_steps,
                )
                .unwrap();

            all_builds.extend(build_result.checked_builds);
            all_builds.push(Arc::clone(&build_sample.check));

            errors.extend(build_result.errors);
        }

        // Return checked build result
        let own_checked_build_result = CheckBuildResult {
            checked_builds: all_builds,
            build: Arc::clone(&build_sample.check),
            errors,
        };

        Some(own_checked_build_result)
    }

    fn check_give_next(
        &self,
        _within_build: BuildId,
        _for_label: String,
        _environment: CheckEnvironment,
        _previous_steps: Vec<CheckStep>,
    ) -> Option<CheckBuildResult> {
        // A core treatment cannot have sub-treatments (its not a sequence), so nothing to ever return.
        None
    }
}
