use super::{
    Connection, GenericInstanciation, ModelInstanciation, Reference, Scope, TreatmentInstanciation,
    IO,
};
use crate::descriptor::Treatment as TreatmentDescriptor;
use crate::design::{
    Connection as ConnectionDesign, ModelInstanciation as ModelInstanciationDesign,
    Parameter as ParameterDesign, Treatment as TreatmentDesign,
    TreatmentInstanciation as TreatmentInstanciationDesign, IO as IODesign,
};
use crate::error::{LogicError, LogicResult};
use core::fmt::Debug;
use melodium_common::descriptor::{
    Attribuable, Attributes, Collection, DescribedType, Entry, Generics, Identified, Identifier,
    IdentifierRequirement, Parameterized, Treatment as TreatmentTrait,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};

#[derive(Debug)]
pub struct Treatment {
    collection: Arc<Collection>,
    descriptor: Weak<TreatmentDescriptor>,

    generics: Arc<RwLock<HashMap<String, DescribedType>>>,
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
                generics: Arc::new(RwLock::new(
                    // This is required to satisfy type/generics comparison for inner instanciantions.
                    descriptor
                        .generics()
                        .iter()
                        .map(|generic| {
                            (
                                generic.name.clone(),
                                DescribedType::Generic(Box::new(generic.clone())),
                            )
                        })
                        .collect(),
                )),
                model_instanciations: HashMap::new(),
                treatments: HashMap::new(),
                connections: Vec::new(),
                design_reference,
                auto_reference: me.clone(),
            })
        })
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

    pub fn import_design(
        &mut self,
        design: &TreatmentDesign,
        replace: &HashMap<Identifier, Identifier>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        for (name, model_instanciation_design) in &design.model_instanciations {
            let model_identifier = model_instanciation_design
                .descriptor
                .upgrade()
                .unwrap()
                .identifier()
                .clone();
            let model_identifier = replace.get(&model_identifier).unwrap_or(&model_identifier);
            if let Some(model_instanciation) =
                result.merge_degrade_failure(self.add_model_instanciation(
                    &model_identifier.into(),
                    name,
                    design_reference.clone(),
                ))
            {
                result.merge_degrade_failure(model_instanciation.write().unwrap().import_design(
                    model_instanciation_design,
                    &self.collection,
                    replace,
                ));
            }
        }

        for (name, treatment_instanciation_design) in &design.treatments {
            let treatment_identifier = treatment_instanciation_design
                .descriptor
                .upgrade()
                .unwrap()
                .identifier()
                .clone();
            let treatment_identifier = replace
                .get(&treatment_identifier)
                .unwrap_or(&treatment_identifier);
            if let Some(treatment_instanciation) = result.merge_degrade_failure(self.add_treatment(
                &treatment_identifier.into(),
                name,
                design_reference.clone(),
            )) {
                result.merge_degrade_failure(
                    treatment_instanciation.write().unwrap().import_design(
                        treatment_instanciation_design,
                        &self.collection,
                        replace,
                    ),
                );
            }
        }

        for connection in &design.connections {
            match (&connection.output_treatment, &connection.input_treatment) {
                (IODesign::Sequence(), IODesign::Sequence()) => {
                    result.merge_degrade_failure(self.add_self_connection(
                        &connection.output_name,
                        &connection.input_name,
                        connection.attributes().clone(),
                        design_reference.clone(),
                    ))
                }
                (IODesign::Sequence(), IODesign::Treatment(input_treatment)) => result
                    .merge_degrade_failure(self.add_input_connection(
                        &connection.output_name,
                        input_treatment,
                        &connection.input_name,
                        connection.attributes().clone(),
                        design_reference.clone(),
                    )),
                (IODesign::Treatment(output_treatment), IODesign::Sequence()) => result
                    .merge_degrade_failure(self.add_output_connection(
                        &connection.input_name,
                        output_treatment,
                        &connection.output_name,
                        connection.attributes().clone(),
                        design_reference.clone(),
                    )),
                (IODesign::Treatment(output_treatment), IODesign::Treatment(input_treatment)) => {
                    result.merge_degrade_failure(self.add_connection(
                        output_treatment,
                        &connection.output_name,
                        input_treatment,
                        &connection.input_name,
                        connection.attributes().clone(),
                        design_reference.clone(),
                    ))
                }
            };
        }

        result
    }

    pub fn add_model_instanciation(
        &mut self,
        model_identifier: &IdentifierRequirement,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<Arc<RwLock<ModelInstanciation>>> {
        if self.model_instanciations.contains_key(name) {
            return Err(LogicError::already_declared_model(
                209,
                self.descriptor().identifier().clone(),
                name.to_string(),
                design_reference.clone(),
            ))
            .into();
        }
        if let Some(Entry::Model(model_descriptor)) = self.collection.get(model_identifier) {
            let model = ModelInstanciation::new(
                &(self.descriptor() as Arc<dyn TreatmentTrait>),
                &self.auto_reference.upgrade().unwrap(),
                self.identifier(),
                model_descriptor,
                name,
                design_reference.clone(),
            );
            self.model_instanciations
                .insert(name.to_string(), Arc::clone(&model));
            Ok(model).into()
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

    pub fn rename_model_instanciation(
        &mut self,
        actual_name: &str,
        new_name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());
        if self.model_instanciations.contains_key(new_name) {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::already_declared_model(
                    211,
                    self.descriptor().identifier().clone(),
                    new_name.to_string(),
                    design_reference.clone(),
                ),
            ));
        }
        if let Some(model_instanciation) = self.model_instanciations.remove(actual_name) {
            model_instanciation
                .write()
                .unwrap()
                .set_name(new_name.to_string());
            self.model_instanciations
                .insert(new_name.to_string(), model_instanciation);

            for (_, treatment) in &self.treatments {
                let mut treatment = treatment.write().unwrap();
                let to_replace = treatment
                    .models()
                    .iter()
                    .filter_map(|(parametric_name, local_name)| {
                        if local_name == actual_name {
                            Some(parametric_name.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                for name in to_replace {
                    let _ = treatment.add_model(&name, new_name);
                }
            }
            result
        } else {
            result.and_degrade_failure(LogicResult::new_failure(LogicError::undeclared_model(
                212,
                self.descriptor().identifier().clone(),
                actual_name.to_string(),
                design_reference.clone(),
            )))
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
        treatment_identifier: &IdentifierRequirement,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<Arc<RwLock<TreatmentInstanciation>>> {
        if self.treatments.contains_key(name) {
            return Err(LogicError::already_declared_treatment(
                210,
                self.descriptor().identifier().clone(),
                name.to_string(),
                design_reference.clone(),
            ))
            .into();
        }
        if let Some(Entry::Treatment(treatment_descriptor)) =
            self.collection.get(treatment_identifier)
        {
            let rc_treatment = TreatmentInstanciation::new(
                &(self.descriptor() as Arc<dyn TreatmentTrait>),
                &self.auto_reference.upgrade().unwrap(),
                &self.generics,
                self.identifier(),
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

    pub fn rename_treatment(
        &mut self,
        actual_name: &str,
        new_name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());
        if self.treatments.contains_key(new_name) {
            result = result.and_degrade_failure(LogicResult::new_failure(
                LogicError::already_declared_treatment(
                    212,
                    self.descriptor().identifier().clone(),
                    new_name.to_string(),
                    design_reference.clone(),
                ),
            ));
        }
        if let Some(treatment) = self.treatments.remove(actual_name) {
            treatment.write().unwrap().set_name(new_name.to_string());
            self.treatments.insert(new_name.to_string(), treatment);

            result
        } else {
            result.and_degrade_failure(LogicResult::new_failure(LogicError::undeclared_treatment(
                213,
                self.descriptor().identifier().clone(),
                actual_name.to_string(),
                design_reference.clone(),
            )))
        }
    }

    pub fn remove_treatment(&mut self, name: &str) -> LogicResult<bool> {
        Ok(if let Some(ref treatment) = self.treatments.remove(name) {
            self.connections.retain(|conn| {
                let mut result = true;
                if let IO::Treatment(input_treatment) = &conn.input_treatment {
                    result &= !input_treatment.ptr_eq(&Arc::downgrade(treatment));
                }
                if let IO::Treatment(output_treatment) = &conn.output_treatment {
                    result &= !output_treatment.ptr_eq(&Arc::downgrade(treatment));
                }

                result
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
        attributes: Attributes,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let mut rc_output_treatment = None;
        let mut output = None;
        let mut output_treatment_generics = None;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = Some(pos_rc_output_treatment);

            let output_treatment = pos_rc_output_treatment.read().unwrap();
            let output_treatment_descriptor = output_treatment.descriptor();
            output_treatment_generics = Some(Arc::clone(output_treatment.access_generics()));

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
        let mut input_treatment_generics = None;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = Some(pos_rc_input_treatment);

            let input_treatment = pos_rc_input_treatment.read().unwrap();
            let input_treatment_descriptor = input_treatment.descriptor();
            input_treatment_generics = Some(Arc::clone(input_treatment.access_generics()));

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
            if input.matches_output(
                &input_treatment_generics.unwrap().read().unwrap(),
                &output,
                &output_treatment_generics.unwrap().read().unwrap(),
            ) {
                self.connections.push(Connection::new_internal(
                    output_name,
                    rc_output_treatment,
                    input_name,
                    rc_input_treatment,
                    attributes,
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
                        output.described_type().clone(),
                        *input.flow(),
                        input.described_type().clone(),
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
        attributes: Attributes,
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
            if input_self.matches_output(&self.generics(), &output_self, &self.generics()) {
                self.connections.push(Connection::new_self(
                    input_self.name(),
                    output_self.name(),
                    attributes,
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
                        input_self.described_type().clone(),
                        *output_self.flow(),
                        output_self.described_type().clone(),
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
        attributes: Attributes,
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
        let mut input_treatment_generics = None;
        if let Some(pos_rc_input_treatment) = self.treatments.get(input_treatment) {
            rc_input_treatment = Some(pos_rc_input_treatment);

            let input_treatment = pos_rc_input_treatment.read().unwrap();
            let input_treatment_descriptor = input_treatment.descriptor();
            input_treatment_generics = Some(Arc::clone(input_treatment.access_generics()));

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
            if input_self.matches_input(
                &self.generics(),
                &input,
                &input_treatment_generics.unwrap().read().unwrap(),
            ) {
                self.connections.push(Connection::new_self_to_internal(
                    input_self.name(),
                    input.name(),
                    rc_input_treatment,
                    attributes,
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
                        input_self.described_type().clone(),
                        *input.flow(),
                        input.described_type().clone(),
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
        attributes: Attributes,
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
        let mut output_treatment_generics = None;
        if let Some(pos_rc_output_treatment) = self.treatments.get(output_treatment) {
            rc_output_treatment = Some(pos_rc_output_treatment);

            let output_treatment = pos_rc_output_treatment.read().unwrap();
            let output_treatment_descriptor = output_treatment.descriptor();
            output_treatment_generics = Some(Arc::clone(output_treatment.access_generics()));

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
            if output_self.matches_output(
                &self.generics(),
                &output,
                &output_treatment_generics.unwrap().read().unwrap(),
            ) {
                self.connections.push(Connection::new_internal_to_self(
                    output.name(),
                    rc_output_treatment,
                    output_self.name(),
                    attributes,
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
                        output.described_type().clone(),
                        *output_self.flow(),
                        output_self.described_type().clone(),
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

        // Checking all inputs are satisfied.
        for (treatment_name, arc_treatment) in &self.treatments {
            let treatment = arc_treatment.read().unwrap();
            for (input_name, _) in treatment.descriptor().inputs() {
                let mut satisfaction = 0;
                for connection in &self.connections {
                    if let IO::Treatment(treatment_ref) = &connection.input_treatment {
                        if Arc::<_>::downgrade(&arc_treatment).ptr_eq(treatment_ref)
                            && input_name == &connection.input_name
                        {
                            satisfaction += 1;
                        }
                    }
                }

                if satisfaction == 0 {
                    result = result.and_degrade_failure(LogicResult::new_failure(
                        LogicError::unsatisfied_input(
                            243,
                            Some(self.descriptor().identifier().clone()),
                            treatment_name.clone(),
                            input_name.clone(),
                            self.design_reference.clone(),
                        ),
                    ));
                } else if satisfaction > 1 {
                    result = result.and_degrade_failure(LogicResult::new_failure(
                        LogicError::overloaded_input(
                            244,
                            self.descriptor().identifier().clone(),
                            treatment_name.clone(),
                            input_name.clone(),
                            self.design_reference.clone(),
                        ),
                    ));
                }
            }
        }

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

    pub fn make_use(&self, identifier: &Identifier) -> bool {
        self.unvalidated_design()
            .success()
            .map(|design| design.make_use(identifier))
            .unwrap_or(false)
    }

    pub fn uses(&self) -> Vec<Identifier> {
        self.unvalidated_design()
            .success()
            .map(|design| design.uses())
            .unwrap_or_default()
    }

    pub fn unvalidated_design(&self) -> LogicResult<TreatmentDesign> {
        let result = LogicResult::new_success(());

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
                                attributes: model_instanciation.attributes().clone(),
                                descriptor: Arc::downgrade(&model_instanciation.descriptor()),
                                parameters: model_instanciation
                                    .parameters()
                                    .iter()
                                    .filter_map(|(name, param)| {
                                        if let Some(value) = param.read().unwrap().value().as_ref()
                                        {
                                            Some((
                                                name.clone(),
                                                ParameterDesign {
                                                    name: name.clone(),
                                                    value: value.clone(),
                                                },
                                            ))
                                        } else {
                                            None
                                        }
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
                                attributes: treatment_instanciation.attributes().clone(),
                                descriptor: Arc::downgrade(&treatment_instanciation.descriptor()),
                                generics: {
                                    let clone = treatment_instanciation.generics().clone();
                                    clone
                                },
                                models: treatment_instanciation.models().clone(),
                                parameters: treatment_instanciation
                                    .parameters()
                                    .iter()
                                    .filter_map(|(name, param)| {
                                        if let Some(value) = param.read().unwrap().value().as_ref()
                                        {
                                            Some((
                                                name.clone(),
                                                ParameterDesign {
                                                    name: name.clone(),
                                                    value: value.clone(),
                                                },
                                            ))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect(),
                            },
                        )
                    })
                    .collect(),
                connections: self
                    .connections
                    .iter()
                    .filter_map(|connection| {
                        Some(ConnectionDesign {
                            attributes: connection.attributes().clone(),
                            output_treatment: match &connection.output_treatment {
                                IO::Sequence() => IODesign::Sequence(),
                                IO::Treatment(t) => IODesign::Treatment(
                                    t.upgrade()?.read().unwrap().name().to_string(),
                                ),
                            },
                            output_name: connection.output_name.clone(),
                            input_treatment: match &connection.input_treatment {
                                IO::Sequence() => IODesign::Sequence(),
                                IO::Treatment(t) => IODesign::Treatment(
                                    t.upgrade()?.read().unwrap().name().to_string(),
                                ),
                            },
                            input_name: connection.input_name.clone(),
                        })
                    })
                    .collect(),
            })
        })
    }

    pub fn design(&self) -> LogicResult<TreatmentDesign> {
        self.validate().and_then(|_| self.unvalidated_design())
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

impl GenericInstanciation for Treatment {
    fn generics(&self) -> RwLockReadGuard<HashMap<String, DescribedType>> {
        self.generics.read().unwrap()
    }

    fn set_generic(&mut self, _generic_name: String, _type: DescribedType) -> LogicResult<()> {
        LogicResult::new_success(())
    }
}
