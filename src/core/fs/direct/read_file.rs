
use std::collections::HashMap;
use super::file_reader::FileReaderModel;
use crate::executive::model::{Model, ModelId};
use crate::executive::value::Value;
use crate::executive::transmitter::Transmitter;
use crate::executive::treatment::Treatment;
use crate::executive::world::World;
use crate::executive::environment::{ContextualEnvironment, GenesisEnvironment};
use crate::logic::builder::*;
use async_std::future::Future;
use crate::executive::result_status::ResultStatus;
use crate::logic::descriptor::{ParameterDescriptor, CoreModelDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, TreatmentDescriptor};
use std::sync::{Arc, Weak, RwLock};
use crate::logic::error::LogicError;

#[derive(Debug)]
struct FileReadBuilder {

    builds: RwLock<Vec<BuildSample>>,
    building_tracks: RwLock<HashMap<(BuildId, u64), DynamicBuildResult>>
}

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

impl FileReadBuilder {

    pub fn new() -> Self {
        todo!()
    }
}

impl Builder for FileReadBuilder {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        // Make a BuildSample with matching informations
        let build_sample = BuildSample::new(&host_treatment, &host_build, &label, environment);

        let mut builds_writer = self.builds.write().unwrap();
        let idx = builds_writer.len() as BuildId;

        builds_writer.push(build_sample);

        Ok(StaticBuildResult::Build(idx))

    }

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        
        //todo!()

        // Look for existing build
        {
            let borrowed_building_tracks = self.building_tracks.read().unwrap();

            if let Some(existing_building_track) = borrowed_building_tracks.get(&(build, environment.track_id())) {

                let mut dynamic_result = DynamicBuildResult::new();
                // We only copy transmitters because prepared futures were already included the first time
                // dynamic_build were called.
                dynamic_result.feeding_inputs.extend(existing_building_track.feeding_inputs.clone());

                return Some(dynamic_result);
            }
        }

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(build as usize).unwrap();

        let mut result = DynamicBuildResult::new();

        let treatment = ReadFileTreatment::new();

        for (name, model) in build_sample.genesis_environment.models() {
            treatment.set_model(name, model);
        }

        for (name, value) in environment.variables() {
            treatment.set_parameter(name, value);
        }

        // TODO check why result is not currently added in sequence builder?

        Some(result)
    }

    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        
        todo!()
    }

    fn check_dynamic_build(&self, build: BuildId, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {
        
        todo!()
    }

    fn check_give_next(&self, within_build: BuildId, for_label: String, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {
        
        todo!()
    }
}

pub struct ReadFileTreatment {

}

impl ReadFileTreatment {

    pub fn new() -> Arc<Self> {
        Arc::new(Self{})
    }
}

impl Treatment for ReadFileTreatment {

    fn set_parameter(&self, param: &str, value: &Value) {
        todo!()
    }

    fn set_model(&self, name: &str, model: &Arc<dyn Model>) {
        todo!()
    }

    fn get_output(&self, output_name: &str) -> Option<Transmitter> {
        todo!()
    }
    
    fn set_input(&self, input_name: &str, transmitter: Transmitter) -> Result<(), ()> {
        todo!()
    }

    fn prepare(&self) -> Vec<Box<dyn Future<Output = ResultStatus>>> {
        todo!()
    }
    
}
