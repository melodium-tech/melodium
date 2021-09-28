
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use super::*;
use super::super::error::LogicError;
use super::super::designer::SequenceDesigner;
use super::super::designer::TreatmentDesigner;
use super::super::designer::{ConnectionDesigner, ConnectionIODesigner};
use super::super::descriptor::parameterized::Parameterized;
use super::super::descriptor::TreatmentDescriptor;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::model::Model;
use super::super::super::executive::future::Future;
use super::super::super::executive::transmitter::Transmitter;
use super::super::designer::value::Value;

#[derive(Debug)]
pub struct SequenceBuilder {
    designer: Arc<RwLock<SequenceDesigner>>,
    
    builds: RwLock<Vec<BuildSample>>,
    building_tracks: RwLock<HashMap<(BuildId, u64), DynamicBuildResult>>
}

#[derive(Debug)]
struct BuildSample {

    genesis_environment: GenesisEnvironment,
    host_treatment: Option<Arc<dyn TreatmentDescriptor>>,
    host_build_id: Option<BuildId>,
    label: String,
    instancied_models: HashMap<String, Arc<dyn Model>>,
    treatment_build_ids: HashMap<String, u64>,
    root_treatments_build_ids: Vec<(String, u64)>,
    next_treatments_build_ids: HashMap<(String, u64), Vec<(String, u64)>>,
    last_treatments_build_ids: Vec<(String, u64)>,
}

impl BuildSample {
    pub fn new(host_treatment: &Option<Arc<dyn TreatmentDescriptor>>, host_build: &Option<BuildId>, label: &str, environment: &GenesisEnvironment) -> Self {
        Self {
            genesis_environment: environment.clone(),
            host_treatment: host_treatment.clone(),
            host_build_id: host_build.clone(),
            label: label.to_string(),
            instancied_models: HashMap::new(),
            treatment_build_ids: HashMap::new(),
            root_treatments_build_ids: Vec::new(),
            next_treatments_build_ids: HashMap::new(),
            last_treatments_build_ids: Vec::new(),
        }
    }
}

impl SequenceBuilder {
    pub fn new(designer: &Arc<RwLock<SequenceDesigner>>) -> Self {
        Self {
            designer: Arc::clone(designer),
            builds: RwLock::new(Vec::new()),
            building_tracks: RwLock::new(HashMap::new())
        }
    }
}

impl Builder for SequenceBuilder {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        // Make a BuildSample with matching informations
        let mut build_sample = BuildSample::new(&host_treatment, &host_build, &label, environment);

        let mut builds_writer = self.builds.write().unwrap();
        let idx = builds_writer.len() as BuildId;

        // Instanciate models
        for (instanciation_name, model_instanciation) in self.designer.read().unwrap().model_instanciations() {

            let mut remastered_environment = environment.base();

            for (_, parameter) in model_instanciation.read().unwrap().parameters() {

                let borrowed_param = parameter.read().unwrap();

                let data = match borrowed_param.value().as_ref().unwrap() {
                    Value::Raw(data) => data,
                    Value::Variable(name) => {
                        environment.get_variable(&name).unwrap()
                    },
                    // Not possible in model instanciation to use context, should have been catched by designer, aborting
                    _ => panic!("Impossible data recoverage")
                };

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

            let instanciation_result = model_instanciation.read().unwrap().descriptor().builder().static_build(
                Some(Arc::clone(self.designer.read().unwrap().descriptor()) as Arc<dyn TreatmentDescriptor>),
                Some(idx),
                instanciation_name.to_string(),
                &remastered_environment
            );

            let instancied_model = match instanciation_result.unwrap() {
                StaticBuildResult::Model(m) => m,
                _ => panic!("Model instanciation expected")
            };

            build_sample.instancied_models.insert(instanciation_name.to_string(), instancied_model);

        }

        // Make the internal treatments being built
        for (treatment_name, treatment) in self.designer.read().unwrap().treatments() {

            let borrowed_treatment = treatment.read().unwrap();
            let mut remastered_environment = environment.base();

            // Setup models
            for (model_treatment_name, _model) in borrowed_treatment.descriptor().models() {

                // model_treatment_name is the name of the model as seen by the treatment,
                // while model_sequence_name is the name of the model as it exists within the sequence.
                // Treatment[model_treatment_name = model_sequence_name]

                let model_sequence_name = borrowed_treatment.models().get(model_treatment_name).unwrap();

                let mut executive_model = None;
                
                if let Some(sequence_parameter_given_model) = environment.get_model(model_sequence_name) {
                    executive_model = Some(Arc::clone(sequence_parameter_given_model));
                }
                else {                    
                    if let Some(instancied_model) = build_sample.instancied_models.get(model_sequence_name) {
                        executive_model = Some(Arc::clone(instancied_model));
                    }
                }

                if let Some(executive_model) = executive_model {

                    remastered_environment.add_model(model_treatment_name, executive_model);
                }
                else {
                    // We should have a model there, should have been catched by designer, aborting
                    panic!("Impossible model recoverage")
                }
            }

            // Setup parameters
            for (_, parameter) in borrowed_treatment.parameters() {

                let borrowed_param = parameter.read().unwrap();

                let data = match borrowed_param.value().as_ref().unwrap() {
                    Value::Raw(data) => data,
                    Value::Variable(name) => {
                        environment.get_variable(&name).unwrap()
                    }
                };

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

            let build_result = borrowed_treatment.descriptor().builder().static_build(
                Some(Arc::clone(self.designer.read().unwrap().descriptor()) as Arc<dyn TreatmentDescriptor>),
                Some(idx),
                treatment_name.to_string(),
                &remastered_environment
            );

            let id = match build_result.unwrap() {
                StaticBuildResult::Build(id) => id,
                _ => panic!("Build id expected")
            };

            build_sample.treatment_build_ids.insert(treatment_name.to_string(), id);
        }

        // Order the internal treatments
        for connection in self.designer.read().unwrap().connections() {

            let borrowed_connection = connection.read().unwrap();

            match borrowed_connection.output_treatment().unwrap() {
                ConnectionIODesigner::Sequence() => {
                    let treatment_name = match borrowed_connection.input_treatment().unwrap() { ConnectionIODesigner::Treatment(t) => t.upgrade().unwrap().read().unwrap().name() };

                    build_sample.root_treatments_build_ids.push((
                        treatment_name.to_string(),
                        *build_sample.treatment_build_ids.get(treatment_name).unwrap()
                    ));
                },
                ConnectionIODesigner::Treatment(t) => {
                    let treatment_name = t.upgrade().unwrap().read().unwrap().name();
                    let treatment_id = build_sample.treatment_build_ids.get(treatment_name).unwrap();
                    let treatment_tuple = (treatment_name.to_string(), *treatment_id);

                    match borrowed_connection.input_treatment().unwrap() {
                        ConnectionIODesigner::Treatment(t) => {
                            let next_treatment_name = t.upgrade().unwrap().read().unwrap().name();

                            let next_treatments_list: Vec<(String, u64)> = if let Some(list) = build_sample.next_treatments_build_ids.get(&treatment_tuple) {
                                list.clone()
                            }
                            else {
                                Vec::new()
                            };

                            next_treatments_list.push((next_treatment_name.to_string(), *build_sample.treatment_build_ids.get(next_treatment_name).unwrap()));

                            build_sample.next_treatments_build_ids.insert(treatment_tuple, next_treatments_list);
                        },
                        ConnectionIODesigner::Sequence() => {
                            build_sample.last_treatments_build_ids.push(treatment_tuple);
                        }
                    }
                }
            }
        }

        builds_writer.push(build_sample);

        Ok(StaticBuildResult::Build(idx))
    }

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {


        // Look for existing build
        {
            let borrowed_building_tracks = self.building_tracks.read().unwrap();

            if let Some(existing_building_track) = borrowed_building_tracks.get(&(build, environment.track_id())) {
                return Some(*existing_building_track);
            }
        }

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(build as usize).unwrap();

        // Get the treatments connected right after self, or first-range treatments
        let mut treatment_build_results: HashMap<String, DynamicBuildResult> = HashMap::new();
        for (treatment_name, treatment_id) in build_sample.root_treatments_build_ids {
            
            let borrowed_treatment = self.designer.read().unwrap().treatments().get(&treatment_name).unwrap().read().unwrap();
            let treatment_builder = borrowed_treatment.descriptor().builder();
            let mut remastered_environment = environment.base();

            // Make the right contextual environment

            // Setup models
            for (model_treatment_name, _model) in borrowed_treatment.descriptor().models() {

                // model_treatment_name is the name of the model as seen by the treatment,
                // while model_sequence_name is the name of the model as it exists within the sequence.
                // Treatment[model_treatment_name = model_sequence_name]

                let model_sequence_name = borrowed_treatment.models().get(model_treatment_name).unwrap();

                let mut executive_model = None;
                
                if let Some(sequence_parameter_given_model) = environment.get_model(model_sequence_name) {
                    executive_model = Some(Arc::clone(sequence_parameter_given_model));
                }
                else if let Some(instancied_model) = build_sample.instancied_models.get(model_sequence_name) {
                    executive_model = Some(Arc::clone(instancied_model));
                }

                if let Some(executive_model) = executive_model {

                    remastered_environment.add_model(model_treatment_name, executive_model);
                }
                else {
                    // We should have a model there, should have been catched by designer, aborting
                    panic!("Impossible model recoverage")
                }

                // Setup parameters
                for (_, parameter) in borrowed_treatment.parameters() {

                    let borrowed_param = parameter.read().unwrap();

                    let data = match borrowed_param.value().as_ref().unwrap() {
                        Value::Raw(data) => data,
                        Value::Variable(name) => {
                            environment.get_variable(&name).unwrap()
                        },
                        Value::Context((context, name)) => {
                            environment.get_context(context).unwrap().get_value(name).unwrap()
                        }
                    };

                    remastered_environment.add_variable(borrowed_param.name(), data.clone());
                }

                // Call their dynamic_build method with right contextual environment
                treatment_build_results.insert(treatment_name, treatment_builder.dynamic_build(treatment_id, &remastered_environment).unwrap());
            }

            // Take their transmitters and report them accordingly to _inputs_ characteritics

        }

        

        

        return None;
    }

    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {

        // Look for existing build

        // Get the treatments connected right after the given label in the reffered build

        // Call their dynamic_build method and give them the right contextual environment
        // -or-
        // if the claiming treatment is connected to Self as output, call the give_next host method the same way

        // Take transmitters and report them accordingly to _outputs_ characteristics
    }

    fn check_dynamic_build(&self, build: BuildId, ) -> Vec<LogicError> {

    }

    fn check_give_next(&self, within_build: BuildId, for_label: String, ) -> Vec<LogicError> {

    }
}

/*impl Builder for SequenceBuilder {

    fn static_build(&self, environment: &dyn GenesisEnvironment) -> Option<Arc<dyn Model>> {

        for (instanciation_name, model_instanciation) in self.designer.read().unwrap().model_instanciations() {

            let mut remastered_environment = environment.base();

            for (_, parameter) in model_instanciation.read().unwrap().parameters() {

                let borrowed_param = parameter.read().unwrap();

                let data = match borrowed_param.value().as_ref().unwrap() {
                    Value::Raw(data) => data,
                    Value::Variable(name) => {
                        environment.get_variable(&name).unwrap()
                    },
                    // Not possible in model instanciation to use context, should have been catched by designer, aborting
                    _ => panic!("Impossible data recoverage")
                };

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

            let instancied_model = model_instanciation.read().unwrap().descriptor().builder().static_build(&*remastered_environment);
            self.instancied_models.write().unwrap().insert(instanciation_name.to_string(), instancied_model.unwrap());

        }

        // Esthablishing the order of creation of treatments.
        let mut ordered_treatments: Vec<Arc<RwLock<TreatmentDesigner>>> = self.designer.read().unwrap().treatments().values().cloned().collect();
        ordered_treatments.sort_by(
            |a, b|
            a.read().unwrap().partial_cmp(&b.read().unwrap()).unwrap()
        );
        *self.ordered_treatments.write().unwrap() = ordered_treatments;

        None
    }

    fn dynamic_build(&self, environment: &dyn ContextualEnvironment) -> Option<HashMap<String, Transmitter>> {

        let mut treatments_outputs: HashMap<String, HashMap<String, Transmitter>> = HashMap::new();

        // Invoke all treatments builders
        for treatment in &*self.ordered_treatments.read().unwrap() {

            let borrowed_treatment = treatment.read().unwrap();
            let mut remastered_environment = environment.base();

            // Setup models
            for (model_treatment_name, _model) in borrowed_treatment.descriptor().models() {

                // model_treatment_name is the name of the model as seen by the treatment,
                // while model_sequence_name is the name of the model as it exists within the sequence.
                // Treatment[model_treatment_name = model_sequence_name]

                let model_sequence_name = borrowed_treatment.models().get(model_treatment_name).unwrap();

                let mut executive_model = None;
                
                if let Some(sequence_parameter_given_model) = environment.get_model(model_sequence_name) {
                    executive_model = Some(Arc::clone(sequence_parameter_given_model));
                }
                else {
                    let instancied_models = self.instancied_models.read().unwrap();
                    
                    if let Some(instancied_model) = instancied_models.get(model_sequence_name) {
                        executive_model = Some(Arc::clone(instancied_model));
                    }
                }

                if let Some(executive_model) = executive_model {

                    remastered_environment.add_model(model_treatment_name, executive_model);
                }
                else {
                    // We should have a model there, should have been catched by designer, aborting
                    panic!("Impossible model recoverage")
                }
            }

            // Setup parameters
            for (_, parameter) in borrowed_treatment.parameters() {

                let borrowed_param = parameter.read().unwrap();

                let data = match borrowed_param.value().as_ref().unwrap() {
                    Value::Raw(data) => data,
                    Value::Variable(name) => {
                        environment.get_variable(&name).unwrap()
                    },
                    Value::Context((context, name)) => {
                        environment.get_context(context).unwrap().get_value(name).unwrap()
                    }
                };

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

            // Setup inputs

            //We get all connections that have the treatment as input (end point).
            let input_connections: Vec<Arc<RwLock<ConnectionDesigner>>> = self.designer.read().unwrap().connections().iter().filter_map(
                |conn|
                if conn.read().unwrap().input_treatment() == &Some(ConnectionIODesigner::Treatment(Arc::downgrade(treatment))) {
                    Some(Arc::clone(conn))
                }
                else {
                    None
                }
            ).collect();

            // We get all the inputs required by the treatment
            for input_connection in input_connections {

                let borrowed_connection = input_connection.read().unwrap();
                match borrowed_connection.output_treatment().as_ref().unwrap() {
                    ConnectionIODesigner::Sequence() => {

                        remastered_environment.add_input(
                            &borrowed_connection.input_name().as_ref().unwrap(),
                            environment.get_input(borrowed_connection.output_name().as_ref().unwrap()).unwrap().clone()
                        );
                    },
                    ConnectionIODesigner::Treatment(output_treatment) => {

                        let rc_output_treatment = output_treatment.upgrade().unwrap();
                        let borrowed_output_treatment = rc_output_treatment.read().unwrap();

                        remastered_environment.add_input(
                            &borrowed_connection.input_name().as_ref().unwrap(),
                            treatments_outputs.get(borrowed_output_treatment.name()).unwrap()
                                .get(borrowed_connection.output_name().as_ref().unwrap()).unwrap().clone()
                        );
                    },
                };
            }

            let treatment_outputs = borrowed_treatment.descriptor().builder().dynamic_build(&*remastered_environment).unwrap();
            treatments_outputs.insert(borrowed_treatment.name().to_string(), treatment_outputs);

        }


        // We get all connections that are to sequence output.
        let self_output_connections: Vec<Arc<RwLock<ConnectionDesigner>>> = self.designer.read().unwrap().connections().iter().filter_map(
            |conn|
            match conn.read().unwrap().input_treatment().as_ref().unwrap() {
                ConnectionIODesigner::Sequence() => Some(Arc::clone(conn)),
                _ => None,
            }
        ).collect();

        // We fill the 'Self' outputs that will be returned by the builder.
        let mut outputs: HashMap<String, Transmitter> = HashMap::new();
        for connection in self_output_connections {

            let borrowed_connection = connection.read().unwrap();
            match borrowed_connection.output_treatment().as_ref().unwrap() {
                ConnectionIODesigner::Sequence() => {

                    outputs.insert(
                        borrowed_connection.input_name().as_ref().unwrap().to_string(),
                        environment.get_input(borrowed_connection.output_name().as_ref().unwrap()).unwrap().clone()
                    );
                },
                ConnectionIODesigner::Treatment(output_treatment) => {

                    let rc_output_treatment = output_treatment.upgrade().unwrap();
                    let borrowed_output_treatment = rc_output_treatment.read().unwrap();

                    outputs.insert(
                        borrowed_connection.input_name().as_ref().unwrap().to_string(),
                        treatments_outputs.get(borrowed_output_treatment.name()).unwrap()
                            .get(borrowed_connection.output_name().as_ref().unwrap()).unwrap().clone()
                    );
                },
            };
        }

        Some(outputs)
    }
}*/


