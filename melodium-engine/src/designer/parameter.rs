use super::{FunctionInstanciation, Reference, Scope, Value};
use crate::design::Parameter as ParameterDesign;
use crate::error::{LogicError, LogicResult};
use melodium_common::descriptor::{
    Collection, DescribedType, Entry, Identifier, Parameterized, Variability,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct Parameter {
    scope: Weak<RwLock<dyn Scope>>,
    scope_descriptor: Weak<dyn Parameterized>,
    scope_generics: Arc<RwLock<HashMap<String, DescribedType>>>,
    scope_id: Identifier,
    parent_descriptor: Weak<dyn Parameterized>,
    parent_generics: Arc<RwLock<HashMap<String, DescribedType>>>,
    name: String,
    value: Option<Value>,
    design_reference: Option<Arc<dyn Reference>>,
}

impl Parameter {
    pub fn new(
        scope: &Arc<RwLock<dyn Scope>>,
        scope_descriptor: &Arc<dyn Parameterized>,
        scope_generics: &Arc<RwLock<HashMap<String, DescribedType>>>,
        scope_id: Identifier,
        parent_descriptor: &Arc<dyn Parameterized>,
        parent_generics: &Arc<RwLock<HashMap<String, DescribedType>>>,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            scope: Arc::downgrade(scope),
            scope_descriptor: Arc::downgrade(scope_descriptor),
            scope_generics: Arc::clone(scope_generics),
            scope_id,
            parent_descriptor: Arc::downgrade(parent_descriptor),
            parent_generics: Arc::clone(parent_generics),
            name: name.to_string(),
            value: None,
            design_reference,
        }
    }

    pub fn scope(&self) -> &Weak<RwLock<dyn Scope>> {
        &self.scope
    }

    pub fn parent_descriptor(&self) -> &Weak<dyn Parameterized> {
        &self.parent_descriptor
    }

    pub fn design_reference(&self) -> &Option<Arc<dyn Reference>> {
        &self.design_reference
    }

    pub(crate) fn import_design(
        &mut self,
        design: &ParameterDesign,
        collection: &Arc<Collection>,
        replace: &HashMap<Identifier, Identifier>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let value = match &design.value {
            Value::Raw(executive_value) => Some(Value::Raw(executive_value.clone())),
            Value::Variable(variable) => Some(Value::Variable(variable.clone())),
            Value::Context(former_context, entry) => {
                if let Some(Entry::Context(new_context)) = collection.get(
                    replace
                        .get(former_context.identifier())
                        .unwrap_or_else(|| former_context.identifier()),
                ) {
                    Some(Value::Context(new_context.clone(), entry.clone()))
                } else {
                    result = result.and(LogicResult::new_failure(LogicError::unexisting_context(
                        204,
                        self.scope_id.clone(),
                        former_context.identifier().clone(),
                        self.design_reference.clone(),
                    )));
                    None
                }
            }
            Value::Function(former_function, generics, values) => {
                if let Some(Entry::Function(new_function)) = collection.get(
                    replace
                        .get(former_function.identifier())
                        .unwrap_or_else(|| former_function.identifier()),
                ) {
                    Some(Value::Function(
                        new_function.clone(),
                        generics.clone(),
                        values.clone(),
                    ))
                } else {
                    result = result.and(LogicResult::new_failure(LogicError::unexisting_function(
                        205,
                        self.scope_id.clone(),
                        former_function.identifier().clone(),
                        self.design_reference.clone(),
                    )));
                    None
                }
            }
        };

        if let Some(value) = value {
            result = result.and(self.set_value(value));
        }

        result
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_value(&mut self, value: Value) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());
        let parent_descriptor = self.parent_descriptor.upgrade().unwrap();
        let parameter = parent_descriptor.parameters().get(&self.name);
        match &value {
            Value::Raw(data) => {
                self.value = Some(value.clone());

                if let Some(parameter) = parameter {
                    if !parameter
                        .described_type()
                        .is_datatype(&data.datatype(), &self.parent_generics.read().unwrap())
                    {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            13,
                            self.scope_id.clone(),
                            parent_descriptor.identifier().clone(),
                            self.name.clone(),
                            value.clone(),
                            parameter.described_type().clone(),
                            data.datatype().into(),
                            self.design_reference.clone(),
                        ));
                    }
                }
            }
            Value::Variable(name) => {
                self.value = Some(value.clone());

                if let Some(scope_variable) = self
                    .scope_descriptor
                    .upgrade()
                    .unwrap()
                    .parameters()
                    .get(name)
                {
                    if let Some(parameter) = parameter {
                        if *parameter.variability() == Variability::Const
                            && *scope_variable.variability() != Variability::Const
                        {
                            result
                                .errors_mut()
                                .push(LogicError::const_required_var_provided(
                                    60,
                                    self.scope_id.clone(),
                                    parent_descriptor.identifier().clone(),
                                    self.name.clone(),
                                    name.to_string(),
                                    self.design_reference.clone(),
                                ));
                        }

                        if !parameter.described_type().is_compatible(
                            &self.parent_generics.read().unwrap(),
                            scope_variable.described_type(),
                            &self.scope_generics.read().unwrap(),
                        ) {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                14,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                value.clone(),
                                parameter.described_type().clone(),
                                scope_variable.described_type().clone(),
                                self.design_reference.clone(),
                            ));
                        }
                    }
                } else {
                    result.errors_mut().push(LogicError::unexisting_variable(
                        6,
                        self.scope_id.clone(),
                        self.name.to_string(),
                        name.to_string(),
                        self.design_reference.clone(),
                    ));
                }
            }
            Value::Context(context, name) => {
                self.value = Some(value.clone());

                if let Some(parameter) = parameter {
                    if *parameter.variability() == Variability::Const {
                        result
                            .errors_mut()
                            .push(LogicError::const_required_context_provided(
                                61,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                context.identifier().clone(),
                                name.to_string(),
                                self.design_reference.clone(),
                            ));
                    }
                }

                if let Some(context_variable_datatype) = context.values().get(name) {
                    if let Some(parameter) = parameter {
                        if !parameter.described_type().is_datatype(
                            context_variable_datatype,
                            &self.parent_generics.read().unwrap(),
                        ) {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                15,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                value.clone(),
                                parameter.described_type().clone(),
                                context_variable_datatype.into(),
                                self.design_reference.clone(),
                            ));
                        }
                    }
                } else {
                    result
                        .errors_mut()
                        .push(LogicError::unexisting_context_variable(
                            8,
                            self.scope_id.clone(),
                            self.name.clone(),
                            context.identifier().clone(),
                            name.clone(),
                            self.design_reference.clone(),
                        ));
                }
            }
            Value::Function(descriptor, generics, parameters) => {
                self.value = Some(value.clone());

                let function_instanciation = FunctionInstanciation::new(
                    descriptor,
                    &self.scope_descriptor.upgrade().unwrap(),
                    &self.scope_generics,
                    self.scope_id.clone(),
                    &self.name,
                    Arc::new(RwLock::new(generics.clone())),
                    self.design_reference.clone(),
                );

                if let Some(parameter) = parameter {
                    if let Some((sub_variability, sub_return_type)) = result.merge_degrade_failure(
                        function_instanciation.check_function_return(parameters),
                    ) {
                        if !parameter.described_type().is_compatible(
                            &self.parent_generics.read().unwrap(),
                            &sub_return_type,
                            &self.scope_generics.read().unwrap(),
                        ) {
                            result = result.and_degrade_failure(LogicResult::new_failure(
                                LogicError::unmatching_datatype(
                                    216,
                                    self.scope_id.clone(),
                                    descriptor.identifier().clone(),
                                    parameter.name().to_string(),
                                    value.clone(),
                                    parameter.described_type().clone(),
                                    sub_return_type,
                                    self.design_reference.clone(),
                                ),
                            ));
                        }
                        if *parameter.variability() == Variability::Const
                            && sub_variability != Variability::Const
                        {
                            result = result.and_degrade_failure(LogicResult::new_failure(
                                LogicError::const_required_function_returns_var(
                                    63,
                                    self.scope_id.clone(),
                                    parent_descriptor.identifier().clone(),
                                    self.name.clone(),
                                    descriptor.identifier().clone(),
                                    self.design_reference.clone(),
                                ),
                            ));
                        }
                    }
                }
            }
        }

        result
    }

    pub fn value(&self) -> &Option<Value> {
        &self.value
    }

    pub fn described_type(&self) -> LogicResult<Option<DescribedType>> {
        let parent_descriptor = self.parent_descriptor.upgrade().unwrap();
        let parameter = parent_descriptor.parameters().get(&self.name);
        match parameter.map(|parameter| {
            parameter
                .described_type()
                .as_defined(&self.parent_generics.read().unwrap())
                .ok_or_else(|| -> LogicResult<Option<DescribedType>> {
                    LogicResult::new_failure(LogicError::undefined_generic(
                        221,
                        self.scope_id.clone(),
                        parent_descriptor.identifier().clone(),
                        parameter.described_type().clone(),
                        self.design_reference.clone(),
                    ))
                })
        }) {
            Some(described_type) => match described_type {
                Ok(described_type) => LogicResult::new_success(Some(described_type)),
                Err(err) => err,
            },
            None => LogicResult::new_success(None),
        }
    }

    pub fn validate(&self) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let parent_descriptor = self.parent_descriptor.upgrade().unwrap();
        let parameter = parent_descriptor.parameters().get(&self.name);

        if self.value.is_none() {
            result.errors_mut().push(LogicError::no_value(
                27,
                self.scope_id.clone(),
                parent_descriptor.identifier().clone(),
                self.name.clone(),
                self.design_reference.clone(),
            ));
        }

        // Check parameter exists
        if let Some(parameter) = parameter {
            // Check datatype
            match &self.value {
                Some(Value::Raw(data)) => {
                    if !parameter
                        .described_type()
                        .is_datatype(&data.datatype(), &self.parent_generics.read().unwrap())
                    {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            195,
                            self.scope_id.clone(),
                            parent_descriptor.identifier().clone(),
                            self.name.clone(),
                            self.value.as_ref().unwrap().clone(),
                            parameter.described_type().clone(),
                            data.datatype().into(),
                            self.design_reference.clone(),
                        ));
                    }
                }
                Some(Value::Variable(name)) => {
                    if let Some(scope_variable) = self
                        .scope_descriptor
                        .upgrade()
                        .unwrap()
                        .parameters()
                        .get(name)
                    {
                        if *parameter.variability() == Variability::Const
                            && *scope_variable.variability() != Variability::Const
                        {
                            result
                                .errors_mut()
                                .push(LogicError::const_required_var_provided(
                                    196,
                                    self.scope_id.clone(),
                                    parent_descriptor.identifier().clone(),
                                    self.name.clone(),
                                    name.to_string(),
                                    self.design_reference.clone(),
                                ));
                        }

                        if !parameter.described_type().is_compatible(
                            &self.parent_generics.read().unwrap(),
                            scope_variable.described_type(),
                            &self.scope_generics.read().unwrap(),
                        ) {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                197,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                self.value.as_ref().unwrap().clone(),
                                parameter.described_type().clone(),
                                scope_variable.described_type().clone(),
                                self.design_reference.clone(),
                            ));
                        }
                    } else {
                        result.errors_mut().push(LogicError::unexisting_variable(
                            198,
                            self.scope_id.clone(),
                            self.name.to_string(),
                            name.to_string(),
                            self.design_reference.clone(),
                        ));
                    }
                }
                Some(Value::Context(context, name)) => {
                    if *parameter.variability() == Variability::Const {
                        result
                            .errors_mut()
                            .push(LogicError::const_required_context_provided(
                                199,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                context.identifier().clone(),
                                name.to_string(),
                                self.design_reference.clone(),
                            ));
                    }

                    if let Some(context_variable_datatype) = context.values().get(name) {
                        if !parameter.described_type().is_datatype(
                            context_variable_datatype,
                            &self.parent_generics.read().unwrap(),
                        ) {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                200,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                self.value.as_ref().unwrap().clone(),
                                parameter.described_type().clone(),
                                context_variable_datatype.into(),
                                self.design_reference.clone(),
                            ));
                        }
                    } else {
                        result
                            .errors_mut()
                            .push(LogicError::unexisting_context_variable(
                                201,
                                self.scope_id.clone(),
                                self.name.clone(),
                                context.identifier().clone(),
                                name.clone(),
                                self.design_reference.clone(),
                            ));
                    }
                }
                Some(Value::Function(descriptor, generics, parameters)) => {
                    let function_instanciation = FunctionInstanciation::new(
                        descriptor,
                        &self.scope_descriptor.upgrade().unwrap(),
                        &self.scope_generics,
                        self.scope_id.clone(),
                        &self.name,
                        Arc::new(RwLock::new(generics.clone())),
                        self.design_reference.clone(),
                    );

                    if let Some((sub_variability, sub_return_type)) = result.merge_degrade_failure(
                        function_instanciation.check_function_return(parameters),
                    ) {
                        if !parameter.described_type().is_compatible(
                            &self.parent_generics.read().unwrap(),
                            &sub_return_type,
                            &self.scope_generics.read().unwrap(),
                        ) {
                            result = result.and_degrade_failure(LogicResult::new_failure(
                                LogicError::unmatching_datatype(
                                    217,
                                    self.scope_id.clone(),
                                    descriptor.identifier().clone(),
                                    parameter.name().to_string(),
                                    self.value.as_ref().unwrap().clone(),
                                    parameter.described_type().clone(),
                                    sub_return_type,
                                    self.design_reference.clone(),
                                ),
                            ));
                        }
                        if *parameter.variability() == Variability::Const
                            && sub_variability != Variability::Const
                        {
                            result = result.and_degrade_failure(LogicResult::new_failure(
                                LogicError::const_required_function_returns_var(
                                    202,
                                    self.scope_id.clone(),
                                    parent_descriptor.identifier().clone(),
                                    self.name.clone(),
                                    descriptor.identifier().clone(),
                                    self.design_reference.clone(),
                                ),
                            ));
                        }
                    }
                }

                None => {
                    result.errors_mut().push(LogicError::no_value(
                        27,
                        self.scope_id.clone(),
                        self.parent_descriptor
                            .upgrade()
                            .unwrap()
                            .identifier()
                            .clone(),
                        self.name.clone(),
                        self.design_reference.clone(),
                    ));
                }
            }
        } else {
            result.errors_mut().push(LogicError::unexisting_parameter(
                194,
                self.scope_id.clone(),
                parent_descriptor.identifier().clone(),
                self.name.clone(),
                self.design_reference.clone(),
            ));
        }

        result
    }
}
