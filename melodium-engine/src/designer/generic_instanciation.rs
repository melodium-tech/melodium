use super::Reference;
use crate::{LogicError, LogicResult};
use melodium_common::descriptor::{Collection, DescribedType, Entry, Identifier};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::RwLockReadGuard;

pub trait GenericInstanciation: Send + Sync + Debug {
    fn set_generic(&mut self, generic_name: String, r#type: DescribedType) -> LogicResult<()>;
    fn generics(&self) -> RwLockReadGuard<HashMap<String, DescribedType>>;
}

pub(crate) fn import_design_described_type(
    described_type: &mut DescribedType,
    collection: &Arc<Collection>,
    replace: &HashMap<Identifier, Identifier>,
    scope_id: &Identifier,
    design_reference: &Option<Arc<dyn Reference>>,
) -> LogicResult<()> {
    if let Some(former_data) = described_type.final_type_mut().data_mut() {
        if let Some(Entry::Data(new_data)) = collection.get(
            &replace
                .get(former_data.identifier())
                .unwrap_or_else(|| former_data.identifier())
                .into(),
        ) {
            *former_data = Arc::clone(new_data);
            LogicResult::new_success(())
        } else {
            LogicResult::new_failure(LogicError::unexisting_data(
                227,
                scope_id.clone(),
                former_data.identifier().into(),
                design_reference.clone(),
            ))
        }
    } else {
        LogicResult::new_success(())
    }
}
