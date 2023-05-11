use super::{Reference, Scope, Value};
use crate::error::{LogicError, LogicResult};
use melodium_common::descriptor::{Collection, Entry, Function, Parameterized, Variability};
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct Parameter {
    scope: Weak<RwLock<dyn Scope>>,
    parent_descriptor: Weak<dyn Parameterized>,
    name: String,
    value: Option<Value>,
    design_reference: Option<Arc<dyn Reference>>,
}

impl Parameter {
    pub fn new(
        scope: &Arc<RwLock<dyn Scope>>,
        parent_descriptor: &Arc<dyn Parameterized>,
        name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            scope: Arc::downgrade(scope),
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

    pub fn update_collection(&mut self, collection: &Arc<Collection>) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());
        match &mut self.value {
            Some(Value::Context(context, _)) => {
                if let Some(Entry::Context(new_context)) = collection.get(context.identifier()) {
                    *context = new_context.clone();
                } else {
                    result = result.and(LogicResult::new_failure(LogicError::unexisting_context(
                        204,
                        self.scope.upgrade().unwrap().read().unwrap().identifier(),
                        context.identifier().clone(),
                        self.design_reference.clone(),
                    )));
                }
            }
            Some(Value::Function(function, _)) => {
                if let Some(Entry::Function(new_function)) = collection.get(function.identifier()) {
                    *function = new_function.clone();
                } else {
                    result = result.and(LogicResult::new_failure(LogicError::unexisting_function(
                        205,
                        self.scope.upgrade().unwrap().read().unwrap().identifier(),
                        function.identifier().clone(),
                        self.design_reference.clone(),
                    )));
                }
            }
            _ => {}
        }

        result.and(self.validate())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_value(&mut self, value: Value) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());
        let rc_scope = self.scope.upgrade().unwrap();
        let scope = rc_scope.read().unwrap();
        let parent_descriptor = self.parent_descriptor.upgrade().unwrap();
        let parameter = parent_descriptor.parameters().get(&self.name);
        match &value {
            Value::Raw(data) => {
                self.value = Some(value.clone());

                if let Some(parameter) = parameter {
                    if !parameter.datatype().is_compatible(data) {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            13,
                            scope.identifier().clone(),
                            parent_descriptor.identifier().clone(),
                            self.name.clone(),
                            value.clone(),
                            parameter.datatype().clone(),
                            data.datatype().clone(),
                            self.design_reference.clone(),
                        ));
                    }
                }
            }
            Value::Variable(name) => {
                self.value = Some(value.clone());

                if let Some(scope_variable) = scope.descriptor().parameters().get(name) {
                    if let Some(parameter) = parameter {
                        if *parameter.variability() == Variability::Const
                            && *scope_variable.variability() != Variability::Const
                        {
                            result
                                .errors_mut()
                                .push(LogicError::const_required_var_provided(
                                    60,
                                    scope.identifier().clone(),
                                    parent_descriptor.identifier().clone(),
                                    self.name.clone(),
                                    name.to_string(),
                                    self.design_reference.clone(),
                                ));
                        }

                        if scope_variable.datatype() != parameter.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                14,
                                scope.identifier().clone(),
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
                        scope.identifier().clone(),
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
                                scope.identifier().clone(),
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
                                scope.identifier().clone(),
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
                            scope.identifier().clone(),
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
                                            scope.identifier().clone(),
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
        let rc_scope = self.scope.upgrade().unwrap();
        let scope = rc_scope.read().unwrap();

        if descriptor.parameters().len() != parameters.len() {
            result
                .errors_mut()
                .push(LogicError::unmatching_number_of_parameters(
                    64,
                    scope.identifier().clone(),
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
                            scope.identifier().clone(),
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
                    if let Some(scope_variable) = scope.descriptor().parameters().get(name) {
                        if *scope_variable.variability() != Variability::Const {
                            result
                                .success_mut()
                                .map(|variability| *variability = Variability::Var);
                        }

                        if scope_variable.datatype() != param_descriptor.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                17,
                                scope.identifier().clone(),
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
                            scope.identifier().clone(),
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
                                scope.identifier().clone(),
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
                                scope.identifier().clone(),
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

        let rc_scope = self.scope.upgrade().unwrap();
        let scope = rc_scope.read().unwrap();
        let parent_descriptor = self.parent_descriptor.upgrade().unwrap();
        let parameter = parent_descriptor.parameters().get(&self.name);

        if self.value.is_none() {
            result.errors_mut().push(LogicError::no_value(
                27,
                scope.identifier().clone(),
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
                            scope.identifier().clone(),
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
                    if let Some(scope_variable) = scope.descriptor().parameters().get(name) {
                        if *parameter.variability() == Variability::Const
                            && *scope_variable.variability() != Variability::Const
                        {
                            result
                                .errors_mut()
                                .push(LogicError::const_required_var_provided(
                                    196,
                                    scope.identifier().clone(),
                                    parent_descriptor.identifier().clone(),
                                    self.name.clone(),
                                    name.to_string(),
                                    self.design_reference.clone(),
                                ));
                        }

                        if scope_variable.datatype() != parameter.datatype() {
                            result.errors_mut().push(LogicError::unmatching_datatype(
                                197,
                                scope.identifier().clone(),
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
                            scope.identifier().clone(),
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
                                scope.identifier().clone(),
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
                                scope.identifier().clone(),
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
                                scope.identifier().clone(),
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
                                            scope.identifier().clone(),
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
                        self.scope
                            .upgrade()
                            .unwrap()
                            .read()
                            .unwrap()
                            .identifier()
                            .clone(),
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
                scope.identifier().clone(),
                parent_descriptor.identifier().clone(),
                self.name.clone(),
                self.design_reference.clone(),
            ));
        }

        result
    }
}
