use super::{GenericInstanciation, Reference, Value};
use crate::{LogicError, LogicResult};
use melodium_common::descriptor::{
    DescribedType, Function as FunctionDescriptor, Identifier, Parameterized, Variability,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, Weak},
};

#[derive(Debug)]
pub struct FunctionInstanciation {
    descriptor: Weak<dyn FunctionDescriptor>,
    scope_descriptor: Weak<dyn Parameterized>,
    scope_id: Identifier,
    parameter_name: String,
    generics: Arc<RwLock<HashMap<String, DescribedType>>>,
    design_reference: Option<Arc<dyn Reference>>,
}

impl FunctionInstanciation {
    pub fn new(
        descriptor: &Arc<dyn FunctionDescriptor>,
        scope_descriptor: &Arc<dyn Parameterized>,
        scope_id: Identifier,
        parameter_name: &str,
        generics: Arc<RwLock<HashMap<String, DescribedType>>>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            descriptor: Arc::downgrade(descriptor),
            scope_descriptor: Arc::downgrade(scope_descriptor),
            scope_id,
            parameter_name: parameter_name.to_string(),
            generics,
            design_reference,
        }
    }

    pub fn descriptor(&self) -> Arc<dyn FunctionDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn check_function_return(
        &self,
        parameters: &Vec<Value>,
    ) -> LogicResult<(Variability, DescribedType)> {
        let descriptor = self.descriptor();

        let mut result = LogicResult::new_success(Variability::Const);

        if descriptor.parameters().len() != parameters.len() {
            result
                .errors_mut()
                .push(LogicError::unmatching_number_of_parameters(
                    64,
                    self.scope_id.clone(),
                    descriptor.identifier().clone(),
                    self.design_reference.clone(),
                ));
        }

        for i in 0..usize::min(descriptor.parameters().len(), parameters.len()) {
            let param_descriptor = &descriptor.parameters()[i];
            match &parameters[i] {
                Value::Raw(data) => {
                    if !param_descriptor
                        .described_type()
                        .is_datatype(&data.datatype(), &self.generics.read().unwrap())
                    {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            16,
                            self.scope_id.clone(),
                            descriptor.identifier().clone(),
                            param_descriptor.name().to_string(),
                            parameters[i].clone(),
                            param_descriptor.described_type().clone(),
                            data.datatype().into(),
                            self.design_reference.clone(),
                        ));
                    }
                }
                Value::Variable(name) => {
                    if let Some(scope_variable) = self
                        .scope_descriptor
                        .upgrade()
                        .unwrap()
                        .parameters()
                        .get(name)
                    {
                        if *scope_variable.variability() != Variability::Const {
                            result
                                .success_mut()
                                .map(|variability| *variability = Variability::Var);
                        }

                        if !param_descriptor.described_type().is_compatible(
                            scope_variable.described_type(),
                            &self.generics.read().unwrap(),
                        ) {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                17,
                                self.scope_id.clone(),
                                descriptor.identifier().clone(),
                                param_descriptor.name().to_string(),
                                parameters[i].clone(),
                                param_descriptor.described_type().clone(),
                                scope_variable.described_type().clone(),
                                self.design_reference.clone(),
                            ));
                        }
                    } else {
                        result.errors_mut().push(LogicError::unexisting_variable(
                            7,
                            self.scope_id.clone(),
                            self.parameter_name.to_string(),
                            name.to_string(),
                            self.design_reference.clone(),
                        ));
                    }
                }
                Value::Context(context, name) => {
                    result
                        .success_mut()
                        .map(|variability| *variability = Variability::Var);

                    if let Some(context_variable_datatype) = context.values().get(name) {
                        if !param_descriptor
                            .described_type()
                            .is_datatype(context_variable_datatype, &self.generics.read().unwrap())
                        {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                18,
                                self.scope_id.clone(),
                                descriptor.identifier().clone(),
                                param_descriptor.name().to_string(),
                                parameters[i].clone(),
                                param_descriptor.described_type().clone(),
                                context_variable_datatype.into(),
                                self.design_reference.clone(),
                            ));
                        }
                    } else {
                        result
                            .errors_mut()
                            .push(LogicError::unexisting_context_variable(
                                9,
                                self.scope_id.clone(),
                                self.parameter_name.clone(),
                                context.identifier().clone(),
                                name.clone(),
                                self.design_reference.clone(),
                            ));
                    }
                }
                Value::Function(descriptor, generics, parameters) => {
                    let function_instanciation = FunctionInstanciation::new(
                        descriptor,
                        &self.scope_descriptor.upgrade().unwrap(),
                        self.scope_id.clone(),
                        &self.parameter_name,
                        Arc::new(RwLock::new(generics.clone())),
                        self.design_reference.clone(),
                    );

                    result = result.clone().and_degrade_failure(
                        function_instanciation
                            .check_function_return(parameters)
                            .and_then(|(sub_variability, sub_return_type)| {
                                if !param_descriptor
                                    .described_type()
                                    .is_compatible(&sub_return_type, &self.generics.read().unwrap())
                                {
                                    LogicResult::new_failure(LogicError::unmatching_datatype(
                                        214,
                                        self.scope_id.clone(),
                                        descriptor.identifier().clone(),
                                        param_descriptor.name().to_string(),
                                        parameters[i].clone(),
                                        param_descriptor.described_type().clone(),
                                        sub_return_type,
                                        self.design_reference.clone(),
                                    ))
                                } else if sub_variability != Variability::Const {
                                    LogicResult::new_success(Variability::Var)
                                } else {
                                    // The unwrap_or default value has no importance as if none the whole result will be turned into failure anyway.
                                    LogicResult::new_success(
                                        result.success().cloned().unwrap_or(Variability::Const),
                                    )
                                }
                            }),
                    );
                }
            }
        }

        result.and_then(|variability| {
            if let Some(described_type) = descriptor
                .return_type()
                .as_defined(&self.generics.read().unwrap())
            {
                LogicResult::new_success((variability, described_type))
            } else {
                LogicResult::new_failure(LogicError::undefined_generic(
                    215,
                    self.scope_id.clone(),
                    descriptor.identifier().clone(),
                    descriptor.return_type().clone(),
                    self.design_reference.clone(),
                ))
            }
        })
    }
}

impl GenericInstanciation for FunctionInstanciation {
    fn generics(&self) -> RwLockReadGuard<HashMap<String, DescribedType>> {
        self.generics.read().unwrap()
    }

    fn set_generic(&mut self, generic: String, r#type: DescribedType) -> LogicResult<()> {
        let descriptor = self.descriptor();
        if descriptor.generics().contains(&generic) {
            self.generics.write().unwrap().insert(generic, r#type);
            LogicResult::new_success(())
        } else {
            LogicResult::new_failure(LogicError::unexisting_generic(
                218,
                self.scope_id.clone(),
                descriptor.identifier().clone(),
                generic,
                r#type,
                self.design_reference.clone(),
            ))
        }
    }
}
