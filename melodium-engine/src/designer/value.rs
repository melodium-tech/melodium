use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Context, DataType, DescribedType, Function, Generic, Identifier, Parameterized, Variability,
};
use melodium_common::executive::Value as ExecutiveValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{LogicError, LogicResult};

use super::{FunctionInstanciation, Reference};

#[derive(Clone, Debug)]
pub enum Value {
    Raw(ExecutiveValue),
    Array(Vec<Value>),
    Variable(String),
    Context(Arc<dyn Context>, String),
    Function(
        Arc<dyn Function>,
        HashMap<String, DescribedType>,
        Vec<Value>,
    ),
}

impl Value {
    pub fn make_use(&self, identifier: &Identifier) -> bool {
        match self {
            Value::Raw(_) => false,
            Value::Array(array) => array.iter().any(|val| val.make_use(identifier)),
            Value::Variable(_) => false,
            Value::Context(context, _) => context.identifier() == identifier,
            Value::Function(function, described_types, values) => {
                function.identifier() == identifier
                    || described_types.iter().any(|(_, dt)| {
                        dt.final_type()
                            .data()
                            .map(|data| data.identifier() == identifier)
                            .unwrap_or(false)
                    })
                    || values.iter().any(|value| value.make_use(identifier))
            }
        }
    }

    pub fn uses(&self) -> Vec<Identifier> {
        match self {
            Value::Raw(_) | Value::Variable(_) => vec![],
            Value::Array(array) => array.iter().fold(Vec::new(), |mut ids, val| {
                let identifiers: Vec<_> = val
                    .uses()
                    .into_iter()
                    .filter(|id| !ids.contains(id))
                    .collect();
                ids.extend(identifiers);
                ids
            }),
            Value::Context(context, _) => vec![context.identifier().clone()],
            Value::Function(function, described_types, values) => {
                let mut uses = vec![function.identifier().clone()];
                uses.extend(described_types.iter().filter_map(|(_, dt)| {
                    dt.final_type().data().map(|data| data.identifier().clone())
                }));
                uses.extend(values.iter().flat_map(|value| value.uses()));
                uses
            }
        }
    }

    pub fn check(
        &self,
        described_type: &DescribedType,
        scope_descriptor: &Arc<dyn Parameterized>,
        scope_generics: &Arc<RwLock<HashMap<String, DescribedType>>>,
        called_id: &Identifier,
        parent_generics: &Arc<RwLock<HashMap<String, DescribedType>>>,
        parameter_name: &str,
        parameter_variability: Variability,
        design_reference: &Option<Arc<dyn Reference>>,
    ) -> LogicResult<Variability> {
        match self {
            Value::Raw(data) => {
                if described_type.is_datatype(&data.datatype(), &parent_generics.read().unwrap()) {
                    LogicResult::new_success(Variability::Const)
                } else {
                    LogicResult::new_failure(LogicError::unmatching_datatype(
                        195,
                        scope_descriptor.identifier().clone(),
                        called_id.clone(),
                        parameter_name.to_string(),
                        self.clone(),
                        described_type.clone(),
                        data.datatype().into(),
                        design_reference.clone(),
                    ))
                }
            }
            Value::Array(array) => {
                if let Some(DataType::Vec(inner_type)) =
                    described_type.to_datatype(&parent_generics.read().unwrap())
                {
                    let mut result = LogicResult::new_success(());
                    let mut variability = Variability::Const;
                    for val in array {
                        if let Some(var) = result.merge_degrade_failure(val.check(
                            &DescribedType::from(&*inner_type),
                            scope_descriptor,
                            scope_generics,
                            called_id,
                            parent_generics,
                            parameter_name,
                            parameter_variability,
                            design_reference,
                        )) {
                            if var == Variability::Var {
                                variability = Variability::Var;
                            }
                        }
                    }
                    result.and(LogicResult::new_success(variability))
                } else {
                    LogicResult::new_failure(LogicError::unmatching_datatype(
                        226,
                        scope_descriptor.identifier().clone(),
                        called_id.clone(),
                        parameter_name.to_string(),
                        self.clone(),
                        described_type.clone(),
                        DescribedType::Vec(Box::new(DescribedType::Generic(Box::new(
                            Generic::new("_".to_string(), Vec::new()),
                        )))),
                        design_reference.clone(),
                    ))
                }
            }
            Value::Variable(name) => {
                if let Some(scope_variable) = scope_descriptor.parameters().get(name) {
                    let mut result = LogicResult::new_success(());
                    if parameter_variability == Variability::Const
                        && *scope_variable.variability() != Variability::Const
                    {
                        result
                            .errors_mut()
                            .push(LogicError::const_required_var_provided(
                                196,
                                scope_descriptor.identifier().clone(),
                                called_id.clone(),
                                parameter_name.to_string(),
                                name.to_string(),
                                design_reference.clone(),
                            ));
                    }

                    if !described_type.is_compatible(
                        &parent_generics.read().unwrap(),
                        scope_variable.described_type(),
                        &scope_generics.read().unwrap(),
                    ) {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            197,
                            scope_descriptor.identifier().clone(),
                            called_id.clone(),
                            parameter_name.to_string(),
                            self.clone(),
                            described_type.clone(),
                            scope_variable.described_type().clone(),
                            design_reference.clone(),
                        ));
                    }

                    result.and(LogicResult::new_success(*scope_variable.variability()))
                } else {
                    LogicResult::new_failure(LogicError::unexisting_variable(
                        198,
                        scope_descriptor.identifier().clone(),
                        parameter_name.to_string(),
                        name.to_string(),
                        design_reference.clone(),
                    ))
                }
            }
            Value::Context(context, name) => {
                let mut result = LogicResult::new_success(());
                if parameter_variability == Variability::Const {
                    result
                        .errors_mut()
                        .push(LogicError::const_required_context_provided(
                            199,
                            scope_descriptor.identifier().clone(),
                            called_id.clone(),
                            parameter_name.to_string(),
                            context.identifier().clone(),
                            name.to_string(),
                            design_reference.clone(),
                        ));
                }

                if let Some(context_variable_datatype) = context.values().get(name) {
                    if !described_type
                        .is_datatype(context_variable_datatype, &parent_generics.read().unwrap())
                    {
                        result.errors_mut().push(LogicError::unmatching_datatype(
                            200,
                            scope_descriptor.identifier().clone(),
                            called_id.clone(),
                            parameter_name.to_string(),
                            self.clone(),
                            described_type.clone(),
                            context_variable_datatype.into(),
                            design_reference.clone(),
                        ));
                    }
                } else {
                    result
                        .errors_mut()
                        .push(LogicError::unexisting_context_variable(
                            201,
                            scope_descriptor.identifier().clone(),
                            parameter_name.to_string(),
                            context.identifier().clone(),
                            name.clone(),
                            design_reference.clone(),
                        ));
                }

                result.and(LogicResult::new_success(Variability::Var))
            }
            Value::Function(descriptor, generics, parameters) => {
                let function_instanciation = FunctionInstanciation::new(
                    descriptor,
                    &scope_descriptor,
                    &scope_generics,
                    scope_descriptor.identifier().clone(),
                    Arc::new(RwLock::new(generics.clone())),
                    design_reference.clone(),
                );

                let mut result = LogicResult::new_success(());
                let mut variability = Variability::Var;

                if let Some((sub_variability, sub_return_type)) = result
                    .merge_degrade_failure(function_instanciation.check_function_return(parameters))
                {
                    if !described_type.is_compatible(
                        &scope_generics.read().unwrap(),
                        &sub_return_type,
                        &generics,
                    ) {
                        result = result.and_degrade_failure(LogicResult::new_failure(
                            LogicError::unmatching_datatype(
                                217,
                                scope_descriptor.identifier().clone(),
                                descriptor.identifier().clone(),
                                parameter_name.to_string(),
                                self.clone(),
                                described_type.clone(),
                                sub_return_type,
                                design_reference.clone(),
                            ),
                        ));
                    }

                    if parameter_variability == Variability::Const
                        && sub_variability != Variability::Const
                    {
                        result = result.and_degrade_failure(LogicResult::new_failure(
                            LogicError::const_required_function_returns_var(
                                202,
                                scope_descriptor.identifier().clone(),
                                called_id.clone(),
                                parameter_name.to_string(),
                                descriptor.identifier().clone(),
                                design_reference.clone(),
                            ),
                        ));
                    }

                    variability = sub_variability;
                }
                result.and(LogicResult::new_success(variability))
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::Raw(data) => write!(f, "{}", data),
            Value::Array(array) => write!(
                f,
                "[{}]",
                array
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Variable(name) => write!(f, "{}", name),
            Value::Context(desc, entry) => write!(f, "{}[{}]", desc.name(), entry),
            Value::Function(desc, described_types, params) => write!(
                f,
                "{}{}({})",
                desc.identifier().name(),
                if desc.generics().is_empty() {
                    "".to_string()
                } else {
                    format!(
                        "<{}>",
                        desc.generics()
                            .iter()
                            .map(|gen| if let Some(val) = described_types.get(&gen.name) {
                                val.to_string()
                            } else {
                                "_".to_string()
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                },
                params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
