
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use super::*;
use super::super::error::LogicError;
use super::super::designer::SequenceDesigner;
use super::super::designer::{ConnectionDesigner, ConnectionIODesigner};
use super::super::descriptor::parameterized::Parameterized;
use super::super::descriptor::TreatmentDescriptor;
use super::super::descriptor::IdentifiedDescriptor;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::model::Model;
use super::super::super::executive::value::Value as ExecutiveValue;
use super::super::designer::value::Value;

#[derive(Debug)]
pub struct SequenceBuilder {
    designer: Arc<RwLock<SequenceDesigner>>,
    
    builds: RwLock<Vec<BuildSample>>,
    building_inputs: RwLock<HashMap<(BuildId, u64), FeedingInputs>>
}

#[derive(Debug)]
struct BuildSample {

    genesis_environment: GenesisEnvironment,
    host_treatment: Option<Arc<dyn TreatmentDescriptor>>,
    host_build_id: Option<BuildId>,
    check: Arc<RwLock<CheckBuild>>,
    label: String,
    instancied_models: HashMap<String, Arc<dyn Model>>,
    treatment_build_ids: HashMap<String, u64>,
    root_treatments_build_ids: Vec<(String, u64)>,
    root_connections: Vec<Arc<RwLock<ConnectionDesigner>>>,
    next_treatments_build_ids: HashMap<(String, u64), Vec<(String, u64)>>,
    next_connections: HashMap<(String, u64), Vec<Arc<RwLock<ConnectionDesigner>>>>,
    last_treatments_build_ids: Vec<(String, u64)>,
    last_connections: HashMap<(String, u64), Vec<Arc<RwLock<ConnectionDesigner>>>>,
    direct_connections: Vec<Arc<RwLock<ConnectionDesigner>>>,
}

impl BuildSample {
    pub fn new(host_treatment: &Option<Arc<dyn TreatmentDescriptor>>, host_build: &Option<BuildId>, label: &str, environment: &GenesisEnvironment) -> Self {
        Self {
            genesis_environment: environment.clone(),
            host_treatment: host_treatment.clone(),
            host_build_id: host_build.clone(),
            check: Arc::new(RwLock::new(CheckBuild::new())),
            label: label.to_string(),
            instancied_models: HashMap::new(),
            treatment_build_ids: HashMap::new(),
            root_treatments_build_ids: Vec::new(),
            root_connections: Vec::new(),
            next_treatments_build_ids: HashMap::new(),
            next_connections: HashMap::new(),
            last_treatments_build_ids: Vec::new(),
            last_connections: HashMap::new(),
            direct_connections: Vec::new(),
        }
    }
}

impl SequenceBuilder {

    pub fn new(designer: &Arc<RwLock<SequenceDesigner>>) -> Self {
        Self {
            designer: Arc::clone(designer),
            builds: RwLock::new(Vec::new()),
            building_inputs: RwLock::new(HashMap::new())
        }
    }

    fn get_const_value(value: &Value, genesis_environment: &GenesisEnvironment) -> ExecutiveValue {
        match value {
            Value::Raw(data) => data.clone(),
            Value::Variable(name) => genesis_environment.get_variable(&name).unwrap().clone(),
            Value::Function(descriptor, params) => {

                let executive_values = params.iter().map(|v| Self::get_const_value(v, genesis_environment)).collect();

                descriptor.function()(executive_values)
            }
            // Not possible in constant situation to use context, should have been catched by designer, aborting
            _ => panic!("Impossible data recoverage")
        }
    }

    fn get_value(value: &Value, genesis_environment: &GenesisEnvironment, contextual_environment: &ContextualEnvironment) -> ExecutiveValue {
        match value {
            Value::Raw(data) => data.clone(),
            Value::Variable(name) => {
                if let Some(data) = contextual_environment.get_variable(&name) {
                    data.clone()
                }
                else {
                    genesis_environment.get_variable(&name).unwrap().clone()
                }
            },
            Value::Context((context, name)) => {
                contextual_environment.get_context(context).unwrap().get_value(name).unwrap().clone()
            },
            Value::Function(descriptor, params) => {

                let executive_values = params.iter().map(|v| Self::get_value(v, genesis_environment, contextual_environment)).collect();

                descriptor.function()(executive_values)
            }
        }
    }
}

impl Builder for SequenceBuilder {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        // Make a BuildSample with matching informations
        let mut build_sample = BuildSample::new(&host_treatment, &host_build, &label, environment);
        //let environment = &mut build_sample.genesis_environment;

        let mut builds_writer = self.builds.write().unwrap();
        let idx = builds_writer.len() as BuildId;

        let rc_designer = self.designer.read().unwrap();

        // Assigning missing default values
        for (name, parameter) in rc_designer.descriptor().parameters().iter().filter(|(_, p)| p.default().is_some()) {
            if !environment.variables().contains_key(name) {
                build_sample.genesis_environment.add_variable(name, parameter.default().as_ref().unwrap().clone());
            }
        }

        // Instanciate models
        for (instanciation_name, model_instanciation) in rc_designer.model_instanciations() {

            let mut remastered_environment = environment.base();

            for (_, parameter) in model_instanciation.read().unwrap().parameters() {

                let borrowed_param = parameter.read().unwrap();

                let data = Self::get_const_value(borrowed_param.value().as_ref().unwrap(), &build_sample.genesis_environment);

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

            let instanciation_result = model_instanciation.read().unwrap().descriptor().builder().static_build(
                Some(Arc::clone(rc_designer.descriptor()) as Arc<dyn TreatmentDescriptor>),
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
        for (treatment_name, treatment) in rc_designer.treatments() {

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

                let data = Self::get_const_value(borrowed_param.value().as_ref().unwrap(), &build_sample.genesis_environment);

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

            let build_result = borrowed_treatment.descriptor().builder().static_build(
                Some(Arc::clone(rc_designer.descriptor()) as Arc<dyn TreatmentDescriptor>),
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
        for connection in rc_designer.connections() {

            let borrowed_connection = connection.read().unwrap();

            match borrowed_connection.output_treatment().as_ref().unwrap() {
                ConnectionIODesigner::Sequence() => {
                    
                    match borrowed_connection.input_treatment().as_ref().unwrap() {
                        ConnectionIODesigner::Treatment(t) => {
                            let rc_treatment = t.upgrade().unwrap();
                            let borrowed_treatment = rc_treatment.read().unwrap();
                            let treatment_name = borrowed_treatment.name();

                            build_sample.check.write().unwrap().fed_inputs.insert(
                                connection.read().unwrap().output_name().as_ref().unwrap().clone(),
                                false
                            );

                            build_sample.root_treatments_build_ids.push((
                                treatment_name.to_string(),
                                *build_sample.treatment_build_ids.get(treatment_name).unwrap()
                            ));
                            build_sample.root_connections.push(Arc::clone(connection));
                        },
                        ConnectionIODesigner::Sequence() => {
                            build_sample.direct_connections.push(Arc::clone(connection));
                        }
                    }
                },
                ConnectionIODesigner::Treatment(t) => {
                    let rc_treatment = t.upgrade().unwrap();
                    let borrowed_treatment = rc_treatment.read().unwrap();
                    let treatment_name = borrowed_treatment.name();
                    let treatment_id = build_sample.treatment_build_ids.get(treatment_name).unwrap();
                    let treatment_tuple = (treatment_name.to_string(), *treatment_id);

                    match borrowed_connection.input_treatment().as_ref().unwrap() {
                        ConnectionIODesigner::Treatment(t) => {
                            let rc_next_treatment = t.upgrade().unwrap();
                            let borrowed_next_treatment = rc_next_treatment.read().unwrap();
                            let next_treatment_name = borrowed_next_treatment.name();

                            let mut next_treatments_list: Vec<(String, u64)> = if let Some(list) = build_sample.next_treatments_build_ids.get(&treatment_tuple) {
                                list.clone()
                            }
                            else {
                                Vec::new()
                            };

                            next_treatments_list.push((next_treatment_name.to_string(), *build_sample.treatment_build_ids.get(next_treatment_name).unwrap()));

                            let mut next_connections_list: Vec<Arc<RwLock<ConnectionDesigner>>> = if let Some(list) = build_sample.next_connections.get(&treatment_tuple) {
                                list.clone()
                            }
                            else {
                                Vec::new()
                            };

                            next_connections_list.push(Arc::clone(connection));

                            build_sample.next_treatments_build_ids.insert(treatment_tuple.clone(), next_treatments_list);
                            build_sample.next_connections.insert(treatment_tuple.clone(), next_connections_list);
                        },
                        ConnectionIODesigner::Sequence() => {
                            build_sample.last_treatments_build_ids.push(treatment_tuple.clone());

                            let mut last_connections_list: Vec<Arc<RwLock<ConnectionDesigner>>> = if let Some(list) = build_sample.last_connections.get(&treatment_tuple) {
                                list.clone()
                            }
                            else {
                                Vec::new()
                            };

                            last_connections_list.push(Arc::clone(connection));

                            build_sample.last_connections.insert(treatment_tuple, last_connections_list);
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

        // Get the treatments connected right after self, or first-range treatments
        let mut treatment_build_results: HashMap<String, DynamicBuildResult> = HashMap::new();
        for (treatment_name, treatment_id) in &build_sample.root_treatments_build_ids {
            
            let borrowed_designer = self.designer.read().unwrap();
            let borrowed_treatment = borrowed_designer.treatments().get(treatment_name).unwrap().read().unwrap();
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
            }

            // Setup parameters
            for (_, parameter) in borrowed_treatment.parameters() {

                let borrowed_param = parameter.read().unwrap();

                let data = Self::get_value(borrowed_param.value().as_ref().unwrap(), &build_sample.genesis_environment, environment);

                remastered_environment.add_variable(borrowed_param.name(), data.clone());
            }

            // Call their dynamic_build method with right contextual environment
            treatment_build_results.insert(treatment_name.to_string(), treatment_builder.dynamic_build(*treatment_id, &remastered_environment).unwrap());
        }
        
        let mut result = DynamicBuildResult::new();

        // Take root treatments transmitters and report them accordingly to Self-input (connection output) characteritics
        for root_connection in &build_sample.root_connections {

            let borrowed_connection = root_connection.read().unwrap();

            let treatment_name = match borrowed_connection.input_treatment().as_ref().unwrap() {
                ConnectionIODesigner::Treatment(t) => t.upgrade().unwrap().read().unwrap().name().to_string(),
                _ => panic!("Root connection to (input) treatment expected")
            };

            let input_name = borrowed_connection.input_name().as_ref().unwrap();

            let treatment_build_result = treatment_build_results.get(&treatment_name).unwrap();
            let transmitters = treatment_build_result.feeding_inputs.get(input_name).unwrap().clone();

            result.feeding_inputs.entry(borrowed_connection.output_name().as_ref().unwrap().to_string())
                .or_default().extend(transmitters);
        }

        // Taking all the futures returned by the root treatments
        for (_, treatment_build_result) in treatment_build_results {

            result.prepared_futures.extend(treatment_build_result.prepared_futures);
        }

        // If there are some direct connections, call the give_next host method
        if !build_sample.direct_connections.is_empty() {

            let host_build = build_sample.host_treatment.as_ref().unwrap().builder().give_next(
                build_sample.host_build_id.unwrap(),
                build_sample.label.to_string(),
                &environment.base(),
            ).unwrap();

            for direct_connection in &build_sample.direct_connections {

                let borrowed_connection = direct_connection.read().unwrap();

                let input_name = borrowed_connection.input_name().as_ref().unwrap();

                let transmitters = host_build.feeding_inputs.get(input_name).unwrap().clone();

                result.feeding_inputs.entry(borrowed_connection.output_name().as_ref().unwrap().to_string())
                    .or_default().extend(transmitters);
            }

            result.prepared_futures.extend(host_build.prepared_futures);
        }

        self.building_inputs.write().unwrap().insert((build, environment.track_id()), result.feeding_inputs.clone());
        
        Some(result)
    }

    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(within_build as usize).unwrap();

        let asking_treatment_tuple = (for_label.to_string(), *build_sample.treatment_build_ids.get(&for_label).unwrap());

        let mut result = DynamicBuildResult::new();

        // Get the treatments connected right after the given label in the reffered build
        if let Some(next_treatments) = build_sample.next_treatments_build_ids.get(&asking_treatment_tuple) {
            let mut next_treatments_build_results: HashMap<String, DynamicBuildResult> = HashMap::new();
            for (next_treatment_name, next_treatment_id) in next_treatments {

                let rc_designer = self.designer.read().unwrap();
                let borrowed_next_treatment = rc_designer.treatments().get(next_treatment_name).unwrap().read().unwrap();
                let next_treatment_builder = borrowed_next_treatment.descriptor().builder();
                let mut remastered_environment = environment.base();

                // Make the right contextual environment

                // Setup models
                for (model_treatment_name, _model) in borrowed_next_treatment.descriptor().models() {

                    // model_treatment_name is the name of the model as seen by the treatment,
                    // while model_sequence_name is the name of the model as it exists within the sequence.
                    // Treatment[model_treatment_name = model_sequence_name]

                    let model_sequence_name = borrowed_next_treatment.models().get(model_treatment_name).unwrap();

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
                }

                // Setup parameters
                for (_, parameter) in borrowed_next_treatment.parameters() {

                    let borrowed_param = parameter.read().unwrap();

                    let data = Self::get_value(borrowed_param.value().as_ref().unwrap(), &build_sample.genesis_environment, environment);

                    remastered_environment.add_variable(borrowed_param.name(), data.clone());
                }

                // Call their dynamic_build method with right contextual environment
                next_treatments_build_results.insert(next_treatment_name.to_string(), next_treatment_builder.dynamic_build(*next_treatment_id, &remastered_environment).unwrap());
            }

            // Take transmitters and report them accordingly to _outputs_ characteristics
            if let Some(next_connections) = build_sample.next_connections.get(&asking_treatment_tuple) {
                for next_connection in next_connections {

                    let borrowed_connection = next_connection.read().unwrap();

                    let treatment_name = match borrowed_connection.input_treatment().as_ref().unwrap() {
                        ConnectionIODesigner::Treatment(t) => t.upgrade().unwrap().read().unwrap().name().to_string(),
                        _ => panic!("Connection to treatment expected")
                    };

                    let input_name = borrowed_connection.input_name().as_ref().unwrap();

                    let treatment_build_result = next_treatments_build_results.get(&treatment_name).unwrap();
                    let transmitters = treatment_build_result.feeding_inputs.get(input_name).unwrap().clone();

                    result.feeding_inputs.entry(borrowed_connection.output_name().as_ref().unwrap().to_string())
                        .or_default().extend(transmitters);
                }
            }

            // Taking all the futures returned by the next treatments
            for (_, treatment_build_result) in next_treatments_build_results {

                result.prepared_futures.extend(treatment_build_result.prepared_futures);
            }
        }

        // If the claiming treatment is connected to Self as output, call the give_next host method
        if let Some(last_connections) = build_sample.last_connections.get(&asking_treatment_tuple) {

            let host_build = build_sample.host_treatment.as_ref().unwrap().builder().give_next(
                build_sample.host_build_id.unwrap(),
                build_sample.label.to_string(),
                &environment.base(),
            ).unwrap();

            for last_connection in last_connections {

                let borrowed_connection = last_connection.read().unwrap();

                let input_name = borrowed_connection.input_name().as_ref().unwrap();

                let transmitters = host_build.feeding_inputs.get(input_name).unwrap().clone();

                result.feeding_inputs.entry(borrowed_connection.output_name().as_ref().unwrap().to_string())
                    .or_default().extend(transmitters);
            }

            result.prepared_futures.extend(host_build.prepared_futures);
        }

        Some(result)
    }

    fn check_dynamic_build(&self, build: BuildId, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {

        let mut errors = Vec::new();
        // Check if environment is satisfied
        for (name, _) in self.designer.read().unwrap().descriptor().requirements() {

            let found = environment.contextes.iter().find(|&c| c == name);
            if found.is_none() {
                errors.push(LogicError::unavailable_context());
            }
        }

        // Check if we're not in our own previous steps
        let check_step = CheckStep {
            identifier: self.designer.read().unwrap().descriptor().identifier().clone(),
            build_id: build,
        };
        if let Some(_existing_check_step) = previous_steps.iter().find(|&cs| cs == &check_step) {
            
            errors.push(LogicError::already_included_build_step());
        }
        let mut current_previous_steps = previous_steps.clone();
        current_previous_steps.push(check_step);

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(build as usize).unwrap();
        let check_build = Arc::clone(&build_sample.check);
        
        let mut treatment_build_results: HashMap<String, CheckBuildResult> = HashMap::new();

        if errors.is_empty() {
            // Call each  treatments connected right after self, or first-range treatments
            for (treatment_name, treatment_id) in &build_sample.root_treatments_build_ids {
                
                let borrowed_designer = self.designer.read().unwrap();
                let borrowed_treatment = borrowed_designer.treatments().get(treatment_name).unwrap().read().unwrap();
                let treatment_builder = borrowed_treatment.descriptor().builder();
                
                let check_result = treatment_builder.check_dynamic_build(*treatment_id, environment.clone(), current_previous_steps.clone()).unwrap();

                treatment_build_results.insert(treatment_name.to_string(), check_result);
            }

            for root_connection in &build_sample.root_connections {
                
                let borrowed_connection = root_connection.read().unwrap();

                let treatment_name = match borrowed_connection.input_treatment().as_ref().unwrap() {
                    ConnectionIODesigner::Treatment(t) => t.upgrade().unwrap().read().unwrap().name().to_string(),
                    _ => panic!("Root connection to (input) treatment expected")
                };

                let input_name = borrowed_connection.input_name().as_ref().unwrap();

                let treatment_build_result = treatment_build_results.get(&treatment_name).unwrap();
                let mut borrowed_checked_build = treatment_build_result.build.write().unwrap();
                borrowed_checked_build.fed_inputs.insert(input_name.to_string(), true);
            }
        }

        let mut all_builds = Vec::new();
        let mut all_errors = errors;
        for (_, build_result) in treatment_build_results {
            all_builds.extend(build_result.checked_builds);
            all_errors.extend(build_result.errors);
        }
        all_builds.push(Arc::clone(&check_build));

        // Return checked build result
        let own_checked_build_result = CheckBuildResult {
            checked_builds: all_builds,
            build: check_build,
            errors: all_errors,
        };

        Some(own_checked_build_result)
    }

    fn check_give_next(&self, within_build: BuildId, for_label: String, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {

        let errors = Vec::new();
        // (Check if environment is satisfied)
        //  ↑ actually not because next treatment may not require it -checking evolution to do-

        // (Check if we're not in our own previous steps)
        //  ↑ actually not because we are necessarily in previous steps when give_next is called

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(within_build as usize).unwrap();
        let check_build = Arc::clone(&build_sample.check);
        
        let mut treatment_build_results: HashMap<String, CheckBuildResult> = HashMap::new();

        if errors.is_empty() {
            // Get the treatments connected right after the given label in the reffered build
            // Call their check_dynamic_build

            let asking_treatment_tuple = (for_label.to_string(), *build_sample.treatment_build_ids.get(&for_label).unwrap());

            // Get the treatments connected right after the given label in the reffered build
            if let Some(next_treatments) = build_sample.next_treatments_build_ids.get(&asking_treatment_tuple) {
                for (next_treatment_name, next_treatment_id) in next_treatments {

                    let rc_designer = self.designer.read().unwrap();
                    let borrowed_next_treatment = rc_designer.treatments().get(next_treatment_name).unwrap().read().unwrap();
                    let next_treatment_builder = borrowed_next_treatment.descriptor().builder();

                    let check_result = next_treatment_builder.check_dynamic_build(*next_treatment_id, environment.clone(), previous_steps.clone()).unwrap();

                    treatment_build_results.insert(next_treatment_name.to_string(), check_result);
                }
            }

            if let Some(next_connections) = build_sample.next_connections.get(&asking_treatment_tuple) {
                for next_connection in next_connections {
    
                    let borrowed_connection = next_connection.read().unwrap();
    
                    let treatment_name = match borrowed_connection.input_treatment().as_ref().unwrap() {
                        ConnectionIODesigner::Treatment(t) => t.upgrade().unwrap().read().unwrap().name().to_string(),
                        _ => panic!("Connection to treatment expected")
                    };
    
                    let input_name = borrowed_connection.input_name().as_ref().unwrap();
    
                    let treatment_build_result = treatment_build_results.get(&treatment_name).unwrap();
                    let mut borrowed_checked_build = treatment_build_result.build.write().unwrap();
                    borrowed_checked_build.fed_inputs.insert(input_name.to_string(), true);
                }
            }

            // If the claiming treatment is connected to Self as output, call the check_give_next host method
            if let Some(last_connections) = build_sample.last_connections.get(&asking_treatment_tuple) {

                let host_check_build = build_sample.host_treatment.as_ref().unwrap().builder().check_give_next(
                    build_sample.host_build_id.unwrap(),
                    build_sample.label.to_string(),
                    environment.clone(),
                    previous_steps.clone(),
                ).unwrap();
    
                for last_connection in last_connections {
    
                    let borrowed_connection = last_connection.read().unwrap();
    
                    let input_name = borrowed_connection.input_name().as_ref().unwrap();
    
                    let mut borrowed_checked_build = host_check_build.build.write().unwrap();
                    borrowed_checked_build.fed_inputs.insert(input_name.to_string(), true);
                }
            }
        }

        // Return checked build result
        let mut all_builds = Vec::new();
        let mut all_errors = errors;
        for (_, build_result) in treatment_build_results {
            all_builds.extend(build_result.checked_builds);
            all_errors.extend(build_result.errors);
        }
        all_builds.push(Arc::clone(&check_build));

        // Return checked build result
        let own_checked_build_result = CheckBuildResult {
            checked_builds: all_builds,
            build: check_build,
            errors: all_errors,
        };

        Some(own_checked_build_result)
    }
}


