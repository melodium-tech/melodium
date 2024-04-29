use super::{GenericInstanciation, Reference, Value};
use crate::{LogicError, LogicResult};
use melodium_common::descriptor::{
    DataTrait, DescribedType, Function as FunctionDescriptor, Identifier, Parameterized,
    Variability,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, Weak},
};

#[derive(Debug)]
pub struct FunctionInstanciation {
    descriptor: Weak<dyn FunctionDescriptor>,
    scope_descriptor: Weak<dyn Parameterized>,
    scope_generics: Arc<RwLock<HashMap<String, DescribedType>>>,
    scope_id: Identifier,
    generics: Arc<RwLock<HashMap<String, DescribedType>>>,
    design_reference: Option<Arc<dyn Reference>>,
}

impl FunctionInstanciation {
    pub fn new(
        descriptor: &Arc<dyn FunctionDescriptor>,
        scope_descriptor: &Arc<dyn Parameterized>,
        scope_generics: &Arc<RwLock<HashMap<String, DescribedType>>>,
        scope_id: Identifier,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            descriptor: Arc::downgrade(descriptor),
            scope_descriptor: Arc::downgrade(scope_descriptor),
            scope_generics: Arc::clone(scope_generics),
            scope_id,
            generics: Arc::new(RwLock::new(HashMap::with_capacity(
                descriptor.generics().len(),
            ))),
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

        let mut result = LogicResult::new_success(());
        let mut variability = Variability::Const;

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
            if let Some(var) = result.merge_degrade_failure(parameters[i].check(
                param_descriptor.described_type(),
                &self.scope_descriptor.upgrade().unwrap(),
                &self.scope_generics,
                descriptor.identifier(),
                &self.generics,
                param_descriptor.name(),
                *param_descriptor.variability(),
                &self.design_reference,
            )) {
                if var == Variability::Var {
                    variability = Variability::Var;
                }
            }
        }

        result.and_then(|_| {
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

    fn set_generic(&mut self, generic_name: String, r#type: DescribedType) -> LogicResult<()> {
        let descriptor = self.descriptor();
        if let Some(generic) = descriptor
            .generics()
            .iter()
            .find(|gen| gen.name == generic_name)
        {
            if let Some(r#type) = r#type.as_defined(&self.scope_generics.read().unwrap()) {
                let unimplemented: Vec<DataTrait> = generic
                    .traits
                    .iter()
                    .filter(|tr| !r#type.implements(tr))
                    .map(|dt| *dt)
                    .collect();
                if unimplemented.is_empty() {
                    self.generics.write().unwrap().insert(generic_name, r#type);
                    LogicResult::new_success(())
                } else {
                    LogicResult::new_failure(LogicError::unsatisfied_traits(
                        222,
                        self.scope_id.clone(),
                        descriptor.identifier().clone(),
                        r#type,
                        unimplemented,
                        self.design_reference.clone(),
                    ))
                }
            } else {
                LogicResult::new_failure(LogicError::unexisting_generic(
                    229,
                    self.scope_id.clone(),
                    descriptor.identifier().clone(),
                    generic_name,
                    r#type,
                    self.design_reference.clone(),
                ))
            }
        } else {
            LogicResult::new_failure(LogicError::unexisting_generic(
                218,
                self.scope_id.clone(),
                descriptor.identifier().clone(),
                generic_name,
                r#type,
                self.design_reference.clone(),
            ))
        }
    }
}
