use super::{
    BuildId, CheckBuildResult, CheckEnvironment, CheckStep, ContextualEnvironment, DynamicBuildResult, GenesisEnvironment, HostTreatment, StaticBuildResult
};
use crate::{design::Value, error::LogicResult};
use core::fmt::Debug;
use melodium_common::{
    descriptor::{DescribedType, Treatment},
    executive::Value as ExecutiveValue,
};
use std::{collections::HashMap, sync::Arc};

pub trait Builder: Debug + Send + Sync {
    fn static_build(
        &self,
        host_treatment: HostTreatment,
        host_build: Option<BuildId>,
        label: String,
        environment: &GenesisEnvironment,
    ) -> LogicResult<StaticBuildResult>;

    fn dynamic_build(
        &self,
        build: BuildId,
        environment: &ContextualEnvironment,
    ) -> Option<DynamicBuildResult>;
    fn give_next(
        &self,
        within_build: BuildId,
        for_label: String,
        environment: &ContextualEnvironment,
    ) -> Option<DynamicBuildResult>;

    fn check_dynamic_build(
        &self,
        build: BuildId,
        environment: CheckEnvironment,
        previous_steps: Vec<CheckStep>,
    ) -> Option<CheckBuildResult>;
    fn check_give_next(
        &self,
        within_build: BuildId,
        for_label: String,
        environment: CheckEnvironment,
        previous_steps: Vec<CheckStep>,
    ) -> Option<CheckBuildResult>;
}

/*
Notes: should builder be transformed to:
```
pub trait Builder : Debug + Send + Sync {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, ExecutiveError>;

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;
    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult>;
}
```
*/

pub(crate) fn get_value(
    value: &Value,
    genesis_environment: &GenesisEnvironment,
    contextual_environment: Option<&ContextualEnvironment>,
) -> Option<ExecutiveValue> {
    match value {
        Value::Raw(data) => Some(data.clone()),
        Value::Array(array) => {
            let mut vector = Vec::with_capacity(array.len());

            for val in array {
                if let Some(val) = get_value(val, genesis_environment, contextual_environment) {
                    vector.push(val);
                } else {
                    return None;
                }
            }

            Some(ExecutiveValue::Vec(vector))
        }
        Value::Variable(name) => {
            if let Some(data) = contextual_environment
                .map(|ce| ce.get_variable(&name))
                .flatten()
            {
                Some(data.clone())
            } else {
                genesis_environment.get_variable(&name).cloned()
            }
        }
        Value::Context(context, entry) => contextual_environment
            .map(|ce| ce.get_context(context.name()).map(|c| c.get_value(entry)))
            .flatten(),
        Value::Function(descriptor, generics, params) => {
            let generics = generics
                .iter()
                .map(|(name, generic)| {
                    (
                        name.clone(),
                        if generic.contains_generic() {
                            match generic {
                                DescribedType::Generic(generic) => genesis_environment
                                    .get_generic(&generic.name)
                                    .unwrap()
                                    .clone(),
                                _ => panic!("Impossible generic recoverage"),
                            }
                        } else {
                            generic.to_datatype(&HashMap::new()).unwrap()
                        },
                    )
                })
                .collect();
            let mut executive_values = Vec::with_capacity(descriptor.parameters().len());
            for parameter in params {
                if let Some(value) =
                    get_value(parameter, genesis_environment, contextual_environment)
                {
                    executive_values.push(value);
                } else {
                    return None;
                }
            }

            Some(descriptor.function()(generics, executive_values))
        }
    }
}
