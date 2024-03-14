use super::{Reference, Scope, Value};
use crate::design::Parameter as ParameterDesign;
use crate::error::{LogicError, LogicResult};
use melodium_common::descriptor::{Collection, DescribedType, Entry, Identifier, Parameterized};
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
        fn import_value(
            value: &Value,
            design: &ParameterDesign,
            collection: &Arc<Collection>,
            replace: &HashMap<Identifier, Identifier>,
            scope_id: &Identifier,
            design_reference: &Option<Arc<dyn Reference>>,
        ) -> Result<Value, LogicError> {
            match value {
                Value::Raw(executive_value) => Ok(Value::Raw(executive_value.clone())),
                Value::Array(array) => {
                    let mut new_array = Vec::with_capacity(array.len());
                    for val in array {
                        match import_value(
                            val,
                            design,
                            collection,
                            replace,
                            scope_id,
                            design_reference,
                        ) {
                            Ok(val) => new_array.push(val),
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(Value::Array(new_array))
                }
                Value::Variable(variable) => Ok(Value::Variable(variable.clone())),
                Value::Context(former_context, entry) => {
                    if let Some(Entry::Context(new_context)) = collection.get(
                        &replace
                            .get(former_context.identifier())
                            .unwrap_or_else(|| former_context.identifier())
                            .into(),
                    ) {
                        Ok(Value::Context(new_context.clone(), entry.clone()))
                    } else {
                        Err(LogicError::unexisting_context(
                            204,
                            scope_id.clone(),
                            former_context.identifier().into(),
                            design_reference.clone(),
                        ))
                    }
                }
                Value::Function(former_function, generics, values) => {
                    if let Some(Entry::Function(new_function)) = collection.get(
                        &replace
                            .get(former_function.identifier())
                            .unwrap_or_else(|| former_function.identifier())
                            .into(),
                    ) {
                        Ok(Value::Function(
                            new_function.clone(),
                            generics.clone(),
                            values.clone(),
                        ))
                    } else {
                        Err(LogicError::unexisting_function(
                            205,
                            scope_id.clone(),
                            former_function.identifier().into(),
                            design_reference.clone(),
                        ))
                    }
                }
            }
        }

        let value = import_value(
            &design.value,
            design,
            collection,
            replace,
            &self.scope_id,
            &self.design_reference,
        );

        match value {
            Ok(value) => self.set_value(value),
            Err(err) => LogicResult::new_failure(err),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_value(&mut self, value: Value) -> LogicResult<()> {
        let parent_descriptor = self.parent_descriptor.upgrade().unwrap();
        let parameter = parent_descriptor.parameters().get(&self.name);
        if let Some(parameter) = parameter {
            let result = value.check(
                parameter.described_type(),
                &self.scope_descriptor.upgrade().unwrap(),
                &self.scope_generics,
                self.parent_descriptor.upgrade().unwrap().identifier(),
                &self.parent_generics,
                parameter.name(),
                *parameter.variability(),
                &self.design_reference,
            );

            if result.is_success() {
                self.value = Some(value.clone());
            }
            result.and(LogicResult::new_success(()))
        } else {
            LogicResult::new_success(())
        }
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
            if let Some(value) = self.value.as_ref() {
                result = result
                    .and_degrade_failure(value.check(
                        parameter.described_type(),
                        &self.scope_descriptor.upgrade().unwrap(),
                        &self.scope_generics,
                        self.parent_descriptor.upgrade().unwrap().identifier(),
                        &self.parent_generics,
                        parameter.name(),
                        *parameter.variability(),
                        &self.design_reference,
                    ))
                    .and(LogicResult::new_success(()));
            } else {
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
