
use crate::building::Builder as BuilderTrait;
use crate::descriptor;
use melodium_common::descriptor::{Treatment as TreatmentDescriptor, Parameterized, Identified};
use melodium_common::executive::{Model, Treatment as ExecutiveTreatment, World as ExecutiveWorld, TrackId, Value as ExecutiveValue, Context as ExecutiveContext};
use crate::error::LogicError;
use crate::building::{FeedingInputs, BuildId, ContextualEnvironment, GenesisEnvironment, StaticBuildResult, DynamicBuildResult, CheckBuildResult, CheckEnvironment, CheckStep, CheckBuild};
use crate::world::World;
use crate::design::{Connection, IO, Treatment, Value};
use std::sync::{Arc, Weak, RwLock};
use std::collections::{HashMap};
use core::fmt::Debug;

#[derive(Debug)]
struct BuildSample {

    genesis_environment: GenesisEnvironment,
    host_treatment: Option<Arc<dyn TreatmentDescriptor>>,
    host_build_id: Option<BuildId>,
    check: Arc<RwLock<CheckBuild>>,
    label: String,
    instancied_models: HashMap<String, Arc<dyn Model>>,
    treatment_build_ids: HashMap<String, BuildId>,
    root_treatments_build_ids: Vec<(String, BuildId)>,
    root_connections: Vec<Connection>,
    next_treatments_build_ids: HashMap<(String, BuildId), Vec<(String, BuildId)>>,
    next_connections: HashMap<(String, BuildId), Vec<Connection>>,
    last_treatments_build_ids: Vec<(String, BuildId)>,
    last_connections: HashMap<(String, BuildId), Vec<Connection>>,
    direct_connections: Vec<Connection>,
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

#[derive(Debug)]
pub struct Builder {

    world: Weak<World>,

    design: Arc<Treatment>,
    
    builds: RwLock<Vec<BuildSample>>,
    building_inputs: RwLock<HashMap<(BuildId, TrackId), FeedingInputs>>
}

impl Builder {

    pub fn new(world: Weak<World>, design: Arc<Treatment>) -> Self {
        Self {
            world,
            design,
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
            Value::Context(context, entry) => {
                contextual_environment.get_context(context.name()).unwrap().get_value(entry).clone()
            },
            Value::Function(descriptor, params) => {

                let executive_values = params.iter().map(|v| Self::get_value(v, genesis_environment, contextual_environment)).collect();

                descriptor.function()(executive_values)
            }
        }
    }
}

impl BuilderTrait for Builder {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        let world = self.world.upgrade().unwrap();
        // Make a BuildSample with matching informations
        let mut build_sample = BuildSample::new(&host_treatment, &host_build, &label, environment);
        //let environment = &mut build_sample.genesis_environment;

        let mut builds_writer = self.builds.write().unwrap();
        let idx = builds_writer.len() as BuildId;

        let descriptor = self.design.descriptor.upgrade().unwrap();

        // Assigning missing default values
        for (name, parameter) in descriptor.parameters().iter().filter(|(_, p)| p.default().is_some()) {
            if !environment.variables().contains_key(name) {
                build_sample.genesis_environment.add_variable(name, parameter.default().as_ref().unwrap().clone());
            }
        }

        // Instanciate models
        for (instanciation_name, model_instanciation) in self.design.model_instanciations {

            let mut remastered_environment = environment.base();

            for (name, parameter) in model_instanciation.parameters {

                let data = Self::get_const_value(&parameter.value, &build_sample.genesis_environment);

                remastered_environment.add_variable(&name, data);
            }

            let instanciation_result = world.builder(model_instanciation.descriptor.upgrade().unwrap().identifier()).unwrap().static_build(
                Some(Arc::clone(&descriptor) as Arc<dyn TreatmentDescriptor>),
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
        for (treatment_name, treatment) in self.design.treatments {

            let mut remastered_environment = environment.base();
            let treatment_descriptor = treatment.descriptor.upgrade().unwrap();

            // Setup models
            for (model_treatment_name, _model) in treatment_descriptor.models() {

                // model_treatment_name is the name of the model as seen by the treatment,
                // while model_scope_name is the name of the model as it exists within the treatment.
                // Treatment[model_treatment_name = model_scope_name]

                let model_scope_name = treatment.models.get(model_treatment_name).unwrap();

                let mut executive_model = None;
                
                if let Some(sequence_parameter_given_model) = environment.get_model(model_scope_name) {
                    executive_model = Some(Arc::clone(sequence_parameter_given_model));
                }
                else {                    
                    if let Some(instancied_model) = build_sample.instancied_models.get(model_scope_name) {
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
            for (name, parameter) in treatment.parameters {

                let data = Self::get_const_value(&parameter.value, &build_sample.genesis_environment);

                remastered_environment.add_variable(&name, data);
            }

            let build_result = world.builder(treatment_descriptor.identifier())?.static_build(
                Some(Arc::clone(&descriptor) as Arc<dyn TreatmentDescriptor>),
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
        for connection in self.design.connections {

            match connection.output_treatment {
                IO::Sequence() => {
                    
                    match connection.input_treatment {
                        IO::Treatment(treatment_name) => {

                            build_sample.check.write().unwrap().fed_inputs.insert(
                                connection.output_name.clone(),
                                false
                            );

                            build_sample.root_treatments_build_ids.push((
                                treatment_name.to_string(),
                                *build_sample.treatment_build_ids.get(&treatment_name).unwrap()
                            ));
                            build_sample.root_connections.push(connection.clone());
                        },
                        IO::Sequence() => {
                            build_sample.direct_connections.push(connection.clone());
                        }
                    }
                },
                IO::Treatment(treatment_name) => {
                    let treatment_id = build_sample.treatment_build_ids.get(&treatment_name).unwrap();
                    let treatment_tuple = (treatment_name.to_string(), *treatment_id);

                    match connection.input_treatment {
                        IO::Treatment(next_treatment_name) => {
                            
                            let mut next_treatments_list: Vec<(String, BuildId)> = if let Some(list) = build_sample.next_treatments_build_ids.get(&treatment_tuple) {
                                list.clone()
                            }
                            else {
                                Vec::new()
                            };

                            next_treatments_list.push((next_treatment_name.to_string(), *build_sample.treatment_build_ids.get(&next_treatment_name).unwrap()));

                            let mut next_connections_list: Vec<Connection> = if let Some(list) = build_sample.next_connections.get(&treatment_tuple) {
                                list.clone()
                            }
                            else {
                                Vec::new()
                            };

                            next_connections_list.push(connection.clone());

                            build_sample.next_treatments_build_ids.insert(treatment_tuple.clone(), next_treatments_list);
                            build_sample.next_connections.insert(treatment_tuple.clone(), next_connections_list);
                        },
                        IO::Sequence() => {
                            build_sample.last_treatments_build_ids.push(treatment_tuple.clone());

                            let mut last_connections_list: Vec<Connection> = if let Some(list) = build_sample.last_connections.get(&treatment_tuple) {
                                list.clone()
                            }
                            else {
                                Vec::new()
                            };

                            last_connections_list.push(connection.clone());

                            build_sample.last_connections.insert(treatment_tuple, last_connections_list);
                        }
                    }
                }
            }
        }

        // We remove multiple identical entries from treatments build ids
        // This is important to avoid multiple further calls to the same 
        // treatment during one build, sometimes erasing build w/ and w/o
        // executive future.
        build_sample.root_treatments_build_ids.sort_unstable();
        build_sample.root_treatments_build_ids.dedup();
        build_sample.last_treatments_build_ids.sort_unstable();
        build_sample.last_treatments_build_ids.dedup();
        for v in build_sample.next_treatments_build_ids.values_mut() {
            v.sort_unstable();
            v.dedup();
        }

        builds_writer.push(build_sample);

        Ok(StaticBuildResult::Build(idx))
    }

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {

        let world = self.world.upgrade().unwrap();

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
        let build_sample = borrowed_builds.get(build).unwrap();

        // Get the treatments connected right after self, or first-range treatments
        let mut treatment_build_results: HashMap<String, DynamicBuildResult> = HashMap::new();
        for (treatment_name, treatment_id) in &build_sample.root_treatments_build_ids {
            
            let treatment = self.design.treatments.get(treatment_name).unwrap();
            let treatment_descriptor = treatment.descriptor.upgrade().unwrap();
            let treatment_builder = world.builder(treatment_descriptor.identifier()).unwrap();
            let mut remastered_environment = environment.base();

            // Make the right contextual environment

            // Setup models
            for (model_treatment_name, _model) in treatment_descriptor.models() {

                // model_treatment_name is the name of the model as seen by the treatment,
                // while model_scope_name is the name of the model as it exists within the scope.
                // Treatment[model_treatment_name = model_scope_name]

                let model_scope_name = treatment.models.get(model_treatment_name).unwrap();

                let mut executive_model = None;
                
                if let Some(sequence_parameter_given_model) = environment.get_model(model_scope_name) {
                    executive_model = Some(Arc::clone(sequence_parameter_given_model));
                }
                else if let Some(instancied_model) = build_sample.instancied_models.get(model_scope_name) {
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
            for (name, parameter) in treatment.parameters {

                let data = Self::get_value(&parameter.value, &build_sample.genesis_environment, environment);

                remastered_environment.add_variable(&name, data);
            }

            // Call their dynamic_build method with right contextual environment
            treatment_build_results.insert(treatment_name.to_string(), treatment_builder.dynamic_build(*treatment_id, &remastered_environment).unwrap());
        }
        
        let mut result = DynamicBuildResult::new();

        // Take root treatments transmitters and report them accordingly to Self-input (connection output) characteritics
        for root_connection in &build_sample.root_connections {

            let treatment_name = match root_connection.input_treatment {
                IO::Treatment(t) => t.clone(),
                _ => panic!("Root connection to (input) treatment expected")
            };

            let treatment_build_result = treatment_build_results.get(&treatment_name).unwrap();

            if let Some(transmitters) = treatment_build_result.feeding_inputs.get(&root_connection.input_name) {
                result.feeding_inputs.entry(root_connection.output_name.clone())
                .or_default().extend(transmitters.clone());
            }
        }

        // Taking all the futures returned by the root treatments
        for (_, treatment_build_result) in treatment_build_results {

            result.prepared_futures.extend(treatment_build_result.prepared_futures);
        }

        // If there are some direct connections, call the give_next host method
        if !build_sample.direct_connections.is_empty() {

            let host_descriptor = build_sample.host_treatment.unwrap();
            let host_build = world.builder(host_descriptor.identifier()).unwrap().give_next(
                build_sample.host_build_id.unwrap(),
                build_sample.label.to_string(),
                &environment.base(),
            ).unwrap();

            for direct_connection in &build_sample.direct_connections {

                if let Some(transmitters) = host_build.feeding_inputs.get(&direct_connection.input_name) {
                    result.feeding_inputs.entry(direct_connection.output_name.clone())
                    .or_default().extend(transmitters.clone());
                }
            }

            result.prepared_futures.extend(host_build.prepared_futures);
        }

        self.building_inputs.write().unwrap().insert((build, environment.track_id()), result.feeding_inputs.clone());
        
        Some(result)
    }

    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {

        let world = self.world.upgrade().unwrap();

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(within_build).unwrap();

        let asking_treatment_tuple = (for_label.to_string(), *build_sample.treatment_build_ids.get(&for_label).unwrap());

        let mut result = DynamicBuildResult::new();

        // Get the treatments connected right after the given label in the reffered build
        if let Some(next_treatments) = build_sample.next_treatments_build_ids.get(&asking_treatment_tuple) {
            let mut next_treatments_build_results: HashMap<String, DynamicBuildResult> = HashMap::new();
            for (next_treatment_name, next_treatment_id) in next_treatments {

                let next_treatment = self.design.treatments.get(next_treatment_name).unwrap();
                let next_treatment_descriptor = next_treatment.descriptor.upgrade().unwrap();
                let next_treatment_builder = world.builder(next_treatment_descriptor.identifier()).unwrap();
                let mut remastered_environment = environment.base();

                // Make the right contextual environment

                // Setup models
                for (model_treatment_name, _model) in next_treatment_descriptor.models() {

                    // model_treatment_name is the name of the model as seen by the treatment,
                    // while model_scope_name is the name of the model as it exists within the scope.
                    // Treatment[model_treatment_name = model_scope_name]

                    let model_scope_name = next_treatment.models.get(model_treatment_name).unwrap();

                    let mut executive_model = None;
                    
                    if let Some(sequence_parameter_given_model) = environment.get_model(model_scope_name) {
                        executive_model = Some(Arc::clone(sequence_parameter_given_model));
                    }
                    else if let Some(instancied_model) = build_sample.instancied_models.get(model_scope_name) {
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
                for (name, parameter) in next_treatment.parameters {

                    let data = Self::get_value(&parameter.value, &build_sample.genesis_environment, environment);

                    remastered_environment.add_variable(&name, data);
                }

                // Call their dynamic_build method with right contextual environment
                next_treatments_build_results.insert(next_treatment_name.to_string(), next_treatment_builder.dynamic_build(*next_treatment_id, &remastered_environment).unwrap());
            }

            // Take transmitters and report them accordingly to _outputs_ characteristics
            if let Some(next_connections) = build_sample.next_connections.get(&asking_treatment_tuple) {
                for next_connection in next_connections {

                    let treatment_name = match next_connection.input_treatment {
                        IO::Treatment(t) => t.clone(),
                        _ => panic!("Connection to treatment expected")
                    };

                    let treatment_build_result = next_treatments_build_results.get(&treatment_name).unwrap();

                    if let Some(transmitters) = treatment_build_result.feeding_inputs.get(&next_connection.input_name) {
                        result.feeding_inputs.entry(next_connection.output_name.clone())
                        .or_default().extend(transmitters.clone());
                    }
                }
            }

            // Taking all the futures returned by the next treatments
            for (_, treatment_build_result) in next_treatments_build_results {

                result.prepared_futures.extend(treatment_build_result.prepared_futures);
            }
        }

        // If the claiming treatment is connected to Self as output, call the give_next host method
        if let Some(last_connections) = build_sample.last_connections.get(&asking_treatment_tuple) {

            let host_descriptor = build_sample.host_treatment.unwrap();
            let host_build = world.builder(host_descriptor.identifier()).unwrap().give_next(
                build_sample.host_build_id.unwrap(),
                build_sample.label.to_string(),
                &environment.base(),
            ).unwrap();

            for last_connection in last_connections {

                if let Some(transmitters) = host_build.feeding_inputs.get(&last_connection.input_name) {
                    result.feeding_inputs.entry(last_connection.output_name.clone())
                    .or_default().extend(transmitters.clone());
                }
            }

            result.prepared_futures.extend(host_build.prepared_futures);
        }

        Some(result)
    }

    fn check_dynamic_build(&self, build: BuildId, environment: CheckEnvironment, previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {

        let world = self.world.upgrade().unwrap();
        let descriptor = self.design.descriptor.upgrade().unwrap();

        let mut errors = Vec::new();
        // Check if environment is satisfied
        for (name, _) in descriptor.contexts() {

            let found = environment.contextes.iter().find(|&c| c == name);
            if found.is_none() {
                errors.push(LogicError::unavailable_context());
            }
        }

        // Check if we're not in our own previous steps
        let check_step = CheckStep {
            identifier: descriptor.identifier().clone(),
            build_id: build,
        };
        if let Some(_existing_check_step) = previous_steps.iter().find(|&cs| cs == &check_step) {
            
            errors.push(LogicError::already_included_build_step());
        }
        let mut current_previous_steps = previous_steps.clone();
        current_previous_steps.push(check_step);

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(build).unwrap();
        let check_build = Arc::clone(&build_sample.check);
        
        let mut treatment_build_results: HashMap<String, CheckBuildResult> = HashMap::new();

        if errors.is_empty() {
            // Call each  treatments connected right after self, or first-range treatments
            for (treatment_name, treatment_id) in &build_sample.root_treatments_build_ids {
                
                let treatment = self.design.treatments.get(treatment_name).unwrap();
                let treatment_descriptor = treatment.descriptor.upgrade().unwrap();
                let treatment_builder = world.builder(treatment_descriptor.identifier()).unwrap();
                
                let check_result = treatment_builder.check_dynamic_build(*treatment_id, environment.clone(), current_previous_steps.clone()).unwrap();

                treatment_build_results.insert(treatment_name.to_string(), check_result);
            }

            for root_connection in &build_sample.root_connections {

                let treatment_name = match root_connection.input_treatment {
                    IO::Treatment(t) => t.clone(),
                    _ => panic!("Root connection to (input) treatment expected")
                };

                let treatment_build_result = treatment_build_results.get(&treatment_name).unwrap();
                let mut borrowed_checked_build = treatment_build_result.build.write().unwrap();
                borrowed_checked_build.fed_inputs.insert(root_connection.input_name.clone(), true);
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

        let world = self.world.upgrade().unwrap();

        let errors = Vec::new();
        // (Check if environment is satisfied)
        //  ↑ actually not because next treatment may not require it -checking evolution to do-

        // (Check if we're not in our own previous steps)
        //  ↑ actually not because we are necessarily in previous steps when give_next is called

        // Get build
        let borrowed_builds = self.builds.read().unwrap();
        let build_sample = borrowed_builds.get(within_build).unwrap();
        let check_build = Arc::clone(&build_sample.check);
        
        let mut treatment_build_results: HashMap<String, CheckBuildResult> = HashMap::new();

        if errors.is_empty() {
            // Get the treatments connected right after the given label in the reffered build
            // Call their check_dynamic_build

            let asking_treatment_tuple = (for_label.to_string(), *build_sample.treatment_build_ids.get(&for_label).unwrap());

            // Get the treatments connected right after the given label in the reffered build
            if let Some(next_treatments) = build_sample.next_treatments_build_ids.get(&asking_treatment_tuple) {
                for (next_treatment_name, next_treatment_id) in next_treatments {

                    let next_treatment = self.design.treatments.get(next_treatment_name).unwrap();
                    let next_treatment_descriptor = next_treatment.descriptor.upgrade().unwrap();
                    let next_treatment_builder = world.builder(next_treatment_descriptor.identifier()).unwrap();

                    let check_result = next_treatment_builder.check_dynamic_build(*next_treatment_id, environment.clone(), previous_steps.clone()).unwrap();

                    treatment_build_results.insert(next_treatment_name.to_string(), check_result);
                }
            }

            if let Some(next_connections) = build_sample.next_connections.get(&asking_treatment_tuple) {
                for next_connection in next_connections {
    
                    let treatment_name = match next_connection.input_treatment {
                        IO::Treatment(t) => t.clone(),
                        _ => panic!("Connection to treatment expected")
                    };
    
                    let treatment_build_result = treatment_build_results.get(&treatment_name).unwrap();
                    let mut borrowed_checked_build = treatment_build_result.build.write().unwrap();
                    borrowed_checked_build.fed_inputs.insert(next_connection.input_name.clone(), true);
                }
            }

            // If the claiming treatment is connected to Self as output, call the check_give_next host method
            if let Some(last_connections) = build_sample.last_connections.get(&asking_treatment_tuple) {

                let host_descriptor = build_sample.host_treatment.unwrap();
                let host_check_build = world.builder(host_descriptor.identifier()).unwrap().check_give_next(
                    build_sample.host_build_id.unwrap(),
                    build_sample.label.to_string(),
                    environment.clone(),
                    previous_steps.clone(),
                ).unwrap();
    
                for last_connection in last_connections {
    
                    let mut borrowed_checked_build = host_check_build.build.write().unwrap();
                    borrowed_checked_build.fed_inputs.insert(last_connection.input_name.clone(), true);
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



