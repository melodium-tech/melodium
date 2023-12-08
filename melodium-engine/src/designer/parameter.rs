use super::{Reference, Scope, Value};
use crate::design::Parameter as ParameterDesign;
use crate::error::{LogicError, LogicResult};
use melodium_common::descriptor::{
    Collection, Entry, Function, Identifier, Parameterized, Variability,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct Parameter {
    scope: Weak<RwLock<dyn Scope>>,
    scope_descriptor: Weak<dyn Parameterized>,
    scope_id: Identifier,
    parent_descriptor: Weak<dyn Parameterized>,
    name: String,
    value: Option<Value>,
    design_reference: Option<Arc<dyn Reference>>,
}

impl Parameter {
    pub fn new(
        scope: &Arc<RwLock<dyn Scope>>,
        scope_descriptor: &Arc<dyn Parameterized>,
        scope_id: Identifier,
        parent_descriptor: &Arc<dyn Parameterized>,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            scope: Arc::downgrade(scope),
            scope_descriptor: Arc::downgrade(scope_descriptor),
            scope_id,
            parent_descriptor: Arc::downgrade(parent_descriptor),
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
            Value::Function(former_function, values) => {
                if let Some(Entry::Function(new_function)) = collection.get(
                    replace
                        .get(former_function.identifier())
                        .unwrap_or_else(|| former_function.identifier()),
                ) {
                    Some(Value::Function(new_function.clone(), values.clone()))
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
                    if !parameter.described_type().is_compatible(data) {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            13,
                            self.scope_id.clone(),
                            parent_descriptor.identifier().clone(),
                            self.name.clone(),
                            value.clone(),
                            parameter.described_type().clone(),
                            data.datatype().clone(),
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

                        if scope_variable.datatype() != parameter.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                14,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                value.clone(),
                                parameter.datatype().clone(),
                                scope_variable.datatype().clone(),
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
                        if context_variable_datatype != parameter.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                15,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                value.clone(),
                                parameter.datatype().clone(),
                                context_variable_datatype.clone(),
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
            Value::Function(descriptor, parameters) => {
                self.value = Some(value.clone());
                result =
                    self.check_function_return(descriptor, parameters)
                        .and_then(|variability| {
                            let mut res = LogicResult::new_success(());
                            if let Some(parameter) = parameter {
                                if *parameter.variability() == Variability::Const
                                    && variability != Variability::Const
                                {
                                    res.errors_mut().push(
                                        LogicError::const_required_function_returns_var(
                                            63,
                                            self.scope_id.clone(),
                                            parent_descriptor.identifier().clone(),
                                            self.name.clone(),
                                            descriptor.identifier().clone(),
                                            self.design_reference.clone(),
                                        ),
                                    )
                                }
                            }
                            res
                        });
            }
        }

        result
    }

    fn check_function_return(
        &self,
        descriptor: &Arc<dyn Function>,
        parameters: &Vec<Value>,
    ) -> LogicResult<Variability> {
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
                    if !param_descriptor.datatype().is_compatible(&data) {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            16,
                            self.scope_id.clone(),
                            descriptor.identifier().clone(),
                            param_descriptor.name().to_string(),
                            parameters[i].clone(),
                            param_descriptor.datatype().clone(),
                            data.datatype(),
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

                        if scope_variable.datatype() != param_descriptor.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                17,
                                self.scope_id.clone(),
                                descriptor.identifier().clone(),
                                param_descriptor.name().to_string(),
                                parameters[i].clone(),
                                param_descriptor.datatype().clone(),
                                scope_variable.datatype().clone(),
                                self.design_reference.clone(),
                            ));
                        }
                    } else {
                        result.errors_mut().push(LogicError::unexisting_variable(
                            7,
                            self.scope_id.clone(),
                            self.name.to_string(),
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
                        if context_variable_datatype != param_descriptor.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                18,
                                self.scope_id.clone(),
                                descriptor.identifier().clone(),
                                param_descriptor.name().to_string(),
                                parameters[i].clone(),
                                param_descriptor.datatype().clone(),
                                context_variable_datatype.clone(),
                                self.design_reference.clone(),
                            ));
                        }
                    } else {
                        result
                            .errors_mut()
                            .push(LogicError::unexisting_context_variable(
                                9,
                                self.scope_id.clone(),
                                self.name.clone(),
                                context.identifier().clone(),
                                name.clone(),
                                self.design_reference.clone(),
                            ));
                    }
                }
                Value::Function(descriptor, parameters) => {
                    result = result.clone().and_degrade_failure(
                        self.check_function_return(descriptor, parameters).and_then(
                            |sub_variability| {
                                if sub_variability != Variability::Const {
                                    LogicResult::new_success(Variability::Var)
                                } else {
                                    // The unwrap_or default value has no importance as if none the whole result will be turned into failure anyway.
                                    LogicResult::new_success(
                                        result.success().cloned().unwrap_or(Variability::Const),
                                    )
                                }
                            },
                        ),
                    );
                }
            }
        }

        result
    }

    pub fn value(&self) -> &Option<Value> {
        &self.value
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
                    if !parameter.datatype().is_compatible(data) {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            195,
                            self.scope_id.clone(),
                            parent_descriptor.identifier().clone(),
                            self.name.clone(),
                            self.value.as_ref().unwrap().clone(),
                            parameter.datatype().clone(),
                            data.datatype().clone(),
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

                        if scope_variable.datatype() != parameter.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                197,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                self.value.as_ref().unwrap().clone(),
                                parameter.datatype().clone(),
                                scope_variable.datatype().clone(),
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
                        if context_variable_datatype != parameter.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                200,
                                self.scope_id.clone(),
                                parent_descriptor.identifier().clone(),
                                self.name.clone(),
                                self.value.as_ref().unwrap().clone(),
                                parameter.datatype().clone(),
                                context_variable_datatype.clone(),
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
                Some(Value::Function(descriptor, parameters)) => {
                    result =
                        result.and(self.check_function_return(descriptor, parameters).and_then(
                            |variability| {
                                let mut res = LogicResult::new_success(());
                                if *parameter.variability() == Variability::Const
                                    && variability != Variability::Const
                                {
                                    res.errors_mut().push(
                                        LogicError::const_required_function_returns_var(
                                            202,
                                            self.scope_id.clone(),
                                            parent_descriptor.identifier().clone(),
                                            self.name.clone(),
                                            descriptor.identifier().clone(),
                                            self.design_reference.clone(),
                                        ),
                                    )
                                }

                                res
                            },
                        ));
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
