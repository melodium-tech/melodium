
use std::collections::HashMap;
use crate::executive::model::{Model, ModelId};
use crate::executive::value::Value;
use crate::executive::transmitter::Transmitter;
use crate::executive::treatment::Treatment;
use crate::executive::world::World;
use crate::executive::environment::{ContextualEnvironment, GenesisEnvironment};
use crate::logic::builder::*;
use async_std::future::Future;
use crate::executive::result_status::ResultStatus;
use crate::logic::descriptor::{ParameterDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor, BuildableDescriptor};
use std::sync::{Arc, Weak, RwLock};
use crate::logic::error::LogicError;

#[derive(Debug)]
struct BuildSample {
    genesis_environment: GenesisEnvironment,
    host_treatment: Option<Arc<dyn TreatmentDescriptor>>,
    host_build_id: Option<BuildId>,
    check: Arc<RwLock<CheckBuild>>,
    label: String,
}

impl BuildSample {
    pub fn new(host_treatment: &Option<Arc<dyn TreatmentDescriptor>>, host_build: &Option<BuildId>, label: &str, environment: &GenesisEnvironment) -> Self {
        Self {
            genesis_environment: environment.clone(),
            host_treatment: host_treatment.clone(),
            host_build_id: host_build.clone(),
            check: Arc::new(RwLock::new(CheckBuild::new())),
            label: label.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CoreTreatmentBuilder {

    new_treatment: fn(Arc<World>) -> Arc<dyn Treatment>,
    descriptor: Weak<dyn TreatmentDescriptor>,

    builds: RwLock<Vec<BuildSample>>,
    building_inputs: RwLock<HashMap<(BuildId, u64), FeedingInputs>>
}

impl CoreTreatmentBuilder {

    pub fn new(descriptor: &Arc<dyn TreatmentDescriptor>, new_treatment: fn(Arc<World>) -> Arc<dyn Treatment>) -> Self {
        Self {
            new_treatment,
            descriptor: Arc::downgrade(descriptor),
            builds: RwLock::new(Vec::new()),
            building_inputs: RwLock::new(HashMap::new()),
        }
    }
}

impl Builder for CoreTreatmentBuilder {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        // Make a BuildSample with matching informations
        let build_sample = BuildSample::new(&host_treatment, &host_build, &label, environment);

        let mut builds_writer = self.builds.write().unwrap();
        let idx = builds_writer.len() as BuildId;

        let rc_descriptor = self.descriptor.upgrade().unwrap();
        for (model_descriptor, sources) in rc_descriptor.source_from() {

            let (_, matching_model) = environment.models().iter().find(|(_,model)| &model.descriptor() == model_descriptor).unwrap();

            for source in sources {;
                environment.world().add_source(matching_model.id().unwrap(), source, self.descriptor.upgrade().unwrap().as_buildable(), idx);
            }
            
        }

        builds_writer.push(build_sample);

        Ok(StaticBuildResult::Build(idx))

    }

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {

        // Look for existing build
        {
            let borrowed_building_inputs = self.building_inputs.read().unwrap();

            if let Some(existing_building_inputs) = borrowed_building_inputs.get(&(build, environment.track_id())) {

                let mut dynamic_result = DynamicBuildResult::new();
                dynamic_result.feeding_inputs.extend(existing_building_inputs.clone());

                return Some(dynamic_result);
            }
        }

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(build as usize).unwrap();

        let mut result = DynamicBuildResult::new();

        let treatment = (self.new_treatment)(build_sample.genesis_environment.world());

        for (name, model) in build_sample.genesis_environment.models() {
            treatment.set_model(name, model);
        }

        for (name, value) in environment.variables() {
            treatment.set_parameter(name, value);
        }

        let host_build = build_sample.host_treatment.as_ref().unwrap().builder().give_next(
            build_sample.host_build_id.unwrap(),
            build_sample.label.to_string(),
            &environment.base(),
        ).unwrap();

        for (io_name, inputs) in host_build.feeding_inputs {
            treatment.set_output(&io_name, inputs);
        }

        let inputs = treatment.get_inputs();

        result.feeding_inputs = inputs;

        let prepared_futures = treatment.prepare();
        result.prepared_futures.extend(prepared_futures);
        result.prepared_futures.extend(host_build.prepared_futures);

        self.building_inputs.write().unwrap().insert((build, environment.track_id()), result.feeding_inputs.clone());

        Some(result)
    }

    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        
        // A core treatment cannot have sub-treatments (its not a sequence), so nothing to ever return.
        None
    }

    fn check_dynamic_build(&self, build: BuildId, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {
        
        //todo!()
        Some(CheckBuildResult::new())
    }

    fn check_give_next(&self, within_build: BuildId, for_label: String, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {
        
        // A core treatment cannot have sub-treatments (its not a sequence), so nothing to ever return.
        None
    }
}
