use super::{Connection, ModelInstanciation, Reference, Scope, TreatmentInstanciation, IO};
use crate::descriptor::Treatment as TreatmentDescriptor;
use crate::design::{
    Connection as ConnectionDesign, ModelInstanciation as ModelInstanciationDesign,
    Parameter as ParameterDesign, Treatment as TreatmentDesign,
    TreatmentInstanciation as TreatmentInstanciationDesign, IO as IODesign,
};
use crate::error::{LogicError, LogicResult};
use core::fmt::Debug;
use melodium_common::descriptor::{
    Collection, Entry, Identified, Identifier, Parameterized, Treatment as TreatmentTrait,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct Treatment {
    collection: Arc<Collection>,
    descriptor: Weak<TreatmentDescriptor>,

    model_instanciations: HashMap<String, Arc<RwLock<ModelInstanciation>>>,
    treatments: HashMap<String, Arc<RwLock<TreatmentInstanciation>>>,
    connections: Vec<Connection>,

    design_reference: Option<Arc<dyn Reference>>,

    auto_reference: Weak<RwLock<Self>>,
}

impl Treatment {
    pub fn new(
        descriptor: &Arc<TreatmentDescriptor>,
        collection: Arc<Collection>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Arc<RwLock<Self>> {
        Arc::<RwLock<Self>>::new_cyclic(|me| {
            RwLock::new(Self {
                descriptor: Arc::downgrade(descriptor),
                collection,
                model_instanciations: HashMap::new(),
                treatments: HashMap::new(),
                connections: Vec::new(),
                design_reference,
                auto_reference: me.clone(),
            })
        })
    }

    pub fn update_collection(&mut self, collection: Arc<Collection>) -> LogicResult<()> {
        self.collection = collection;

        let mut result = LogicResult::new_success(());
        let mut deletion_list = Vec::new();
        for (name, model_instanciation) in &self.model_instanciations {
            let res = model_instanciation
                .write()
                .unwrap()
                .update_collection(&self.collection);
            if res.is_failure() {
                deletion_list.push(name.clone());
            }
            result = result.and_degrade_failure(res);
        }
        deletion_list.iter().for_each(|d| {
            self.remove_model_instanciation(d);
        });

        let mut deletion_list = Vec::new();
        for (name, treatment) in &self.treatments {
            let res = treatment
                .write()
                .unwrap()
                .update_collection(&self.collection);
            if res.is_failure() {
                deletion_list.push(name.clone());
            }
            result = result.and_degrade_failure(res);
        }
        deletion_list.iter().for_each(|d| {
            self.remove_treatment(d);
        });

        result
    }

    pub fn collection(&self) -> &Arc<Collection> {
        &self.collection
    }

    pub fn descriptor(&self) -> Arc<TreatmentDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn design_reference(&self) -> &Option<Arc<dyn Reference>> {
        &self.design_reference
    }

    pub fn add_model_instanciation(
        &mut self,
        model_identifier: &Identifier,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<Arc<RwLock<ModelInstanciation>>> {
        if let Some(Entry::Model(model_descriptor)) = self.collection.get(model_identifier) {
            let model = ModelInstanciation::new(
                &self.auto_reference.upgrade().unwrap(),
                model_descriptor,
                name,
                design_reference.clone(),
            );
            let rc_model = Arc::new(RwLock::new(model));
            self.model_instanciations
                .insert(name.to_string(), Arc::clone(&rc_model));
            Ok(rc_model).into()
        } else {
            Err(LogicError::unexisting_model(
                41,
                self.descriptor().identifier().clone(),
                model_identifier.clone(),
                design_reference.clone(),
            ))
            .into()
        }
    }

    pub fn remove_model_instanciation(&mut self, name: &str) -> LogicResult<bool> {
        Ok(if let Some(_) = self.model_instanciations.remove(name) {
            for (_, treatment) in &self.treatments {
                let mut treatment = treatment.write().unwrap();
                let to_remove = treatment
                    .models()
                    .iter()
                    .filter_map(|(parametric_name, local_name)| {
                        if local_name == name {
                            Some(parametric_name.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                for name in to_remove {
                    let _ = treatment.remove_model(&name);
                }
            }

            true
        } else {
            false
        })
        .into()
    }

    pub fn add_treatment(
        &mut self,
        treatment_identifier: &Identifier,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<Arc<RwLock<TreatmentInstanciation>>> {
        if let Some(Entry::Treatment(treatment_descriptor)) =
            self.collection.get(treatment_identifier)
        {
            let rc_treatment = TreatmentInstanciation::new(
                &self.auto_reference.upgrade().unwrap(),
                &treatment_descriptor,
                name,
                design_reference.clone(),
            );
            self.treatments
                .insert(name.to_string(), Arc::clone(&rc_treatment));
            Ok(rc_treatment).into()
        } else {
            Err(LogicError::unexisting_treatment(
                40,
                self.descriptor().identifier().clone(),
                treatment_identifier.clone(),
                design_reference.clone(),
            ))
            .into()
        }
    }

    pub fn remove_treatment(&mut self, name: &str) -> LogicResult<bool> {
        Ok(if let Some(ref treatment) = self.treatments.remove(name) {
            self.connections.retain(|conn| {
                !if let IO::Treatment(input_treatment) = &conn.input_treatment {
                    input_treatment.ptr_eq(&Arc::downgrade(treatment))
                } else if let IO::Treatment(output_treatment) = &conn.output_treatment {
                    output_treatment.ptr_eq(&Arc::downgrade(treatment))
                } else {
                    false
                }
            });

            true
        } else {
            false
        })
        .into()
    }

    pub fn add_connection(
        &mut self,
        output_treatment: &str,
        output_name: &str,
        input_treatment: &str,
        input_name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let mut rc_output_treatment = None;
        let mut output = None;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = Some(pos_rc_output_treatment);

            let output_treatment_descriptor = pos_rc_output_treatment.read().unwrap().descriptor();

            if let Some(pos_output) = output_treatment_descriptor.outputs().get(output_name) {
                output = Some(pos_output.clone());
            } else {
                result = result.and_degrade_failure(LogicResult::new_failure(
                    LogicError::connection_output_not_found(
                        36,
                        self.descriptor().identifier().clone(),
                        output_treatment_descriptor.identifier().clone(),
                        output_name.to_string(),
                        design_reference.clone(),
                    ),
                ));
            }
        } else {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::undeclared_treatment(
                    43,
                    self.descriptor().identifier().clone(),
                    output_treatment.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        let mut rc_input_treatment = None;
        let mut input = None;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = Some(pos_rc_input_treatment);

            let input_treatment_descriptor = pos_rc_input_treatment.read().unwrap().descriptor();

            if let Some(pos_input) = input_treatment_descriptor.inputs().get(input_name) {
                input = Some(pos_input.clone());
            } else {
                result = result.and_degrade_failure(LogicResult::new_failure(
                    LogicError::connection_input_not_found(
                        32,
                        self.descriptor().identifier().clone(),
                        input_treatment_descriptor.identifier().clone(),
                        input_name.to_string(),
                        design_reference.clone(),
                    ),
                ));
            }
        } else {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::undeclared_treatment(
                    44,
                    self.descriptor().identifier().clone(),
                    input_treatment.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        if let (Some(rc_output_treatment), Some(output), Some(rc_input_treatment), Some(input)) =
            (rc_output_treatment, output, rc_input_treatment, input)
        {
            if input.matches_output(&output) {
                self.connections.push(Connection::new_internal(
                    output_name,
                    rc_output_treatment,
                    input_name,
                    rc_input_treatment,
                    design_reference.clone(),
                ));
            } else {
                result = result.and_degrade_failure(LogicResult::new_failure(
                    LogicError::unexisting_connexion_type(
                        47,
                        self.descriptor().identifier().clone(),
                        output_treatment.to_string(),
                        output_name.to_string(),
                        input_treatment.to_string(),
                        input_name.to_string(),
                        *output.flow(),
                        *output.datatype(),
                        *input.flow(),
                        *input.datatype(),
                        design_reference,
                    ),
                ));
            }
        }

        result
    }

    pub fn remove_connection(
        &mut self,
        output_treatment: &str,
        output_name: &str,
        input_treatment: &str,
        input_name: &str,
    ) -> LogicResult<bool> {
        let mut found = false;
        self.connections.retain(|connection| {
            if connection.output_name == output_name
                && connection.input_name == input_name
                && match &connection.output_treatment {
                    IO::Treatment(t) => {
                        t.upgrade().unwrap().read().unwrap().name() == output_treatment
                    }
                    _ => false,
                }
                && match &connection.input_treatment {
                    IO::Treatment(t) => {
                        t.upgrade().unwrap().read().unwrap().name() == input_treatment
                    }
                    _ => false,
                }
            {
                found = true;
                false
            } else {
                true
            }
        });

        Ok(found).into()
    }

    pub fn add_self_connection(
        &mut self,
        self_input_name: &str,
        self_output_name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let mut input_self = None;
        if let Some(pos_input) = self.descriptor().inputs().get(self_input_name) {
            input_self = Some(pos_input.clone());
        } else {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::connection_self_input_not_found(
                    34,
                    self.descriptor().identifier().clone(),
                    self_input_name.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        let mut output_self = None;
        if let Some(pos_output) = self.descriptor().outputs().get(self_output_name) {
            output_self = Some(pos_output.clone());
        } else {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::connection_self_output_not_found(
                    38,
                    self.descriptor().identifier().clone(),
                    self_output_name.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        if let (Some(input_self), Some(output_self)) = (input_self, output_self) {
            if input_self.matches_output(&output_self) {
                self.connections.push(Connection::new_self(
                    input_self.name(),
                    output_self.name(),
                    design_reference.clone(),
                ));
            } else {
                result = result.and_degrade_failure(LogicResult::new_failure(
                    LogicError::unexisting_connexion_type(
                        48,
                        self.descriptor().identifier().clone(),
                        "Self".to_string(),
                        self_input_name.to_string(),
                        "Self".to_string(),
                        self_output_name.to_string(),
                        *input_self.flow(),
                        *input_self.datatype(),
                        *output_self.flow(),
                        *output_self.datatype(),
                        design_reference,
                    ),
                ));
            }
        }

        result
    }

    pub fn remove_self_connection(
        &mut self,
        self_input_name: &str,
        self_output_name: &str,
    ) -> LogicResult<bool> {
        let mut found = false;
        self.connections.retain(|connection| {
            if connection.output_name == self_input_name
                && connection.input_name == self_output_name
                && match connection.output_treatment {
                    IO::Sequence() => true,
                    _ => false,
                }
                && match connection.input_treatment {
                    IO::Sequence() => true,
                    _ => false,
                }
            {
                found = true;
                false
            } else {
                true
            }
        });

        Ok(found).into()
    }

    pub fn add_input_connection(
        &mut self,
        self_input_name: &str,
        input_treatment: &str,
        input_name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let mut input_self = None;
        if let Some(pos_input) = self.descriptor().inputs().get(self_input_name) {
            input_self = Some(pos_input.clone());
        } else {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::connection_self_input_not_found(
                    35,
                    self.descriptor().identifier().clone(),
                    self_input_name.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        let mut rc_input_treatment = None;
        let mut input = None;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = Some(pos_rc_input_treatment);

            let input_treatment_descriptor = pos_rc_input_treatment.read().unwrap().descriptor();

            if let Some(pos_input) = input_treatment_descriptor.inputs().get(input_name) {
                input = Some(pos_input.clone());
            } else {
                result = result.and_degrade_failure(LogicResult::new_failure(
                    LogicError::connection_input_not_found(
                        33,
                        self.descriptor().identifier().clone(),
                        input_treatment_descriptor.identifier().clone(),
                        input_name.to_string(),
                        design_reference.clone(),
                    ),
                ));
            }
        } else {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::undeclared_treatment(
                    45,
                    self.descriptor().identifier().clone(),
                    input_treatment.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        if let (Some(input_self), Some(rc_input_treatment), Some(input)) =
            (input_self, rc_input_treatment, input)
        {
            if input_self.matches_input(&input) {
                self.connections.push(Connection::new_self_to_internal(
                    input_self.name(),
                    input.name(),
                    rc_input_treatment,
                    design_reference.clone(),
                ));
            } else {
                result = result.and_degrade_failure(LogicResult::new_failure(
                    LogicError::unexisting_connexion_type(
                        49,
                        self.descriptor().identifier().clone(),
                        "Self".to_string(),
                        self_input_name.to_string(),
                        input_treatment.to_string(),
                        input_name.to_string(),
                        *input_self.flow(),
                        *input_self.datatype(),
                        *input.flow(),
                        *input.datatype(),
                        design_reference,
                    ),
                ));
            }
        }
        result
    }

    pub fn remove_input_connection(
        &mut self,
        self_input_name: &str,
        input_treatment: &str,
        input_name: &str,
    ) -> LogicResult<bool> {
        let mut found = false;
        self.connections.retain(|connection| {
            if connection.output_name == self_input_name
                && connection.input_name == input_name
                && match connection.output_treatment {
                    IO::Sequence() => true,
                    _ => false,
                }
                && match &connection.input_treatment {
                    IO::Treatment(t) => {
                        t.upgrade().unwrap().read().unwrap().name() == input_treatment
                    }
                    _ => false,
                }
            {
                found = true;
                false
            } else {
                true
            }
        });

        Ok(found).into()
    }

    pub fn add_output_connection(
        &mut self,
        self_output_name: &str,
        output_treatment: &str,
        output_name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let mut output_self = None;
        if let Some(pos_output) = self.descriptor().outputs().get(self_output_name) {
            output_self = Some(pos_output.clone());
        } else {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::connection_self_output_not_found(
                    39,
                    self.descriptor().identifier().clone(),
                    self_output_name.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        let mut rc_output_treatment = None;
        let mut output = None;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = Some(pos_rc_output_treatment);

            let output_treatment_descriptor = pos_rc_output_treatment.read().unwrap().descriptor();

            if let Some(pos_output) = output_treatment_descriptor.outputs().get(output_name) {
                output = Some(pos_output.clone());
            } else {
                result = result.and_degrade_failure(LogicResult::new_failure(
                    LogicError::connection_output_not_found(
                        37,
                        self.descriptor().identifier().clone(),
                        output_treatment_descriptor.identifier().clone(),
                        output_name.to_string(),
                        design_reference.clone(),
                    ),
                ));
            }
        } else {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::undeclared_treatment(
                    46,
                    self.descriptor().identifier().clone(),
                    output_treatment.to_string(),
                    design_reference.clone(),
                ),
            ));
        }

        if let (Some(output_self), Some(rc_output_treatment), Some(output)) =
            (output_self, rc_output_treatment, output)
        {
            if output_self.matches_output(&output) {
                self.connections.push(Connection::new_internal_to_self(
                    output.name(),
                    rc_output_treatment,
                    output_self.name(),
                    design_reference.clone(),
                ));
            } else {
                result = result.and_degrade_failure(LogicResult::new_failure(
                    LogicError::unexisting_connexion_type(
                        50,
                        self.descriptor().identifier().clone(),
                        output_treatment.to_string(),
                        output_name.to_string(),
                        "Self".to_string(),
                        self_output_name.to_string(),
                        *output.flow(),
                        *output.datatype(),
                        *output_self.flow(),
                        *output_self.datatype(),
                        design_reference,
                    ),
                ));
            }
        }

        result
    }

    pub fn remove_output_connection(
        &mut self,
        self_output_name: &str,
        output_treatment: &str,
        output_name: &str,
    ) -> LogicResult<bool> {
        let mut found = false;
        self.connections.retain(|connection| {
            if connection.output_name == output_name
                && connection.input_name == self_output_name
                && match &connection.output_treatment {
                    IO::Treatment(t) => {
                        t.upgrade().unwrap().read().unwrap().name() == output_treatment
                    }
                    _ => false,
                }
                && match connection.input_treatment {
                    IO::Sequence() => true,
                    _ => false,
                }
            {
                found = true;
                false
            } else {
                true
            }
        });

        Ok(found).into()
    }

    pub fn model_instanciations(&self) -> &HashMap<String, Arc<RwLock<ModelInstanciation>>> {
        &self.model_instanciations
    }

    pub fn treatments(&self) -> &HashMap<String, Arc<RwLock<TreatmentInstanciation>>> {
        &self.treatments
    }

    pub fn connections(&self) -> &Vec<Connection> {
        &self.connections
    }

    pub fn validate(&self) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        result = self
            .model_instanciations
            .iter()
            .fold(result, |result, (_, model)| {
                result.and_degrade_failure(model.read().unwrap().validate())
            });
        result = self
            .treatments
            .iter()
            .fold(result, |result, (_, treatment)| {
                result.and_degrade_failure(treatment.read().unwrap().validate())
            });

        // TODO Maybe should we check if no circular
        // references in connections there

        // Counting number of outputs connected to self outputs.
        let mut outputs_satisfaction = self
            .descriptor()
            .outputs()
            .iter()
            .map(|(name, _output)| -> (String, usize) { (name.to_string(), 0) })
            .collect::<HashMap<String, usize>>();

        for connection in &self.connections {
            match connection.input_treatment {
                IO::Sequence() => {
                    *(outputs_satisfaction
                        .get_mut(&connection.input_name)
                        .unwrap()) += 1;
                }
                _ => {}
            }
        }

        // Check self outputs are connected to exactly one treatment output.
        for (output, count) in outputs_satisfaction {
            if count < 1 {
                result.errors_mut().push(LogicError::unsatisfied_output(
                    51,
                    self.descriptor().identifier().clone(),
                    output,
                    self.design_reference.clone(),
                ));
            } else if count > 1 {
                result.errors_mut().push(LogicError::overloaded_output(
                    52,
                    self.descriptor().identifier().clone(),
                    output,
                    self.design_reference.clone(),
                ));
            }
        }

        result
    }

    pub fn design(&self) -> LogicResult<TreatmentDesign> {
        let result = self.validate();

        result.and_then(|_| {
            LogicResult::new_success(TreatmentDesign {
                descriptor: self.descriptor.clone(),
                model_instanciations: self
                    .model_instanciations
                    .iter()
                    .map(|(name, model_instanciation)| {
                        let model_instanciation = model_instanciation.read().unwrap();
                        (
                            name.clone(),
                            ModelInstanciationDesign {
                                name: name.clone(),
                                descriptor: Arc::downgrade(&model_instanciation.descriptor()),
                                parameters: model_instanciation
                                    .parameters()
                                    .iter()
                                    .map(|(name, param)| {
                                        (
                                            name.clone(),
                                            ParameterDesign {
                                                name: name.clone(),
                                                value: param
                                                    .read()
                                                    .unwrap()
                                                    .value()
                                                    .as_ref()
                                                    .unwrap()
                                                    .clone(),
                                            },
                                        )
                                    })
                                    .collect(),
                            },
                        )
                    })
                    .collect(),
                treatments: self
                    .treatments
                    .iter()
                    .map(|(name, treatment_instanciation)| {
                        let treatment_instanciation = treatment_instanciation.read().unwrap();
                        (
                            name.clone(),
                            TreatmentInstanciationDesign {
                                name: name.clone(),
                                descriptor: Arc::downgrade(&treatment_instanciation.descriptor()),
                                models: treatment_instanciation.models().clone(),
                                parameters: treatment_instanciation
                                    .parameters()
                                    .iter()
                                    .map(|(name, param)| {
                                        (
                                            name.clone(),
                                            ParameterDesign {
                                                name: name.clone(),
                                                value: param
                                                    .read()
                                                    .unwrap()
                                                    .value()
                                                    .as_ref()
                                                    .unwrap()
                                                    .clone(),
                                            },
                                        )
                                    })
                                    .collect(),
                            },
                        )
                    })
                    .collect(),
                connections: self
                    .connections
                    .iter()
                    .map(|connection| ConnectionDesign {
                        output_treatment: match &connection.output_treatment {
                            IO::Sequence() => IODesign::Sequence(),
                            IO::Treatment(t) => IODesign::Treatment(
                                t.upgrade().unwrap().read().unwrap().name().to_string(),
                            ),
                        },
                        output_name: connection.output_name.clone(),
                        input_treatment: match &connection.input_treatment {
                            IO::Sequence() => IODesign::Sequence(),
                            IO::Treatment(t) => IODesign::Treatment(
                                t.upgrade().unwrap().read().unwrap().name().to_string(),
                            ),
                        },
                        input_name: connection.input_name.clone(),
                    })
                    .collect(),
            })
        })
    }
}

impl Scope for Treatment {
    fn descriptor(&self) -> Arc<dyn Parameterized> {
        Arc::clone(&self.descriptor()) as Arc<dyn Parameterized>
    }

    fn collection(&self) -> Arc<Collection> {
        Arc::clone(&self.collection)
    }

    fn identifier(&self) -> Identifier {
        self.descriptor().identifier().clone()
    }
}
