
pub use crate::executive::result_status::ResultStatus;
pub use crate::executive::future::TrackFuture;
pub use std::collections::HashMap;
pub use crate::executive::model::{Model, ModelId};
pub use crate::executive::value::Value;
pub use crate::executive::transmitter::*;
pub use crate::executive::treatment::*;
pub use crate::executive::world::World;
pub use crate::executive::context::Context;
pub use crate::executive::input::Input;
pub use crate::executive::output::Output;
pub use crate::logic::descriptor::*;
pub use crate::logic::error::LogicError;
pub use crate::logic::builder::*;
pub use crate::logic::contexts::Contexts;
pub(crate) use crate::logic::descriptor::identifier::core_identifier;
pub(crate) use crate::logic::descriptor::datatype::datatype;
pub(crate) use crate::logic::descriptor::input::input;
pub(crate) use crate::logic::descriptor::output::output;
pub(crate) use crate::logic::descriptor::parameter::parameter;
pub(crate) use crate::logic::descriptor::core_treatment::{models, treatment_sources};
pub(crate) use crate::logic::descriptor::core_model::{model_sources};
pub use std::sync::{Arc, Weak, RwLock};
pub use downcast_rs::DowncastSync;
pub use async_std::prelude::*;
pub use crate::logic::descriptor::CoreTreatmentDescriptor;
pub use crate::logic::collection_pool::CollectionPool;

macro_rules! parameters {
    ($( $x:expr ),*) => {
        vec![
                $($x,)*
            ]
    };
}
pub(crate) use parameters;

macro_rules! inputs {
    ($( $x:expr ),*) => {
        vec![
                $($x,)*
            ]
    };
}
pub(crate) use inputs;

macro_rules! outputs {
    ($( $x:expr ),*) => {
        vec![
                $($x,)*
            ]
    };
}
pub(crate) use outputs;

macro_rules! treatment {
    ($mod:ident,$identifier:expr,$models:expr,$sources:expr,$parameters:expr,$inputs:expr,$outputs:expr,$host:ident $treatment:expr) => {
        pub mod $mod {

            use crate::core::prelude::*;
        
            pub fn desc() -> Arc<CoreTreatmentDescriptor> {
        
                lazy_static! {
                    static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {
            
                        let rc_descriptor = CoreTreatmentDescriptor::new(
                            $identifier,
                            $models,
                            $sources,
                            $parameters,
                            $inputs,
                            $outputs,
                            treatment,
                        );
            
                        rc_descriptor
                    };
                }
            
                Arc::clone(&DESCRIPTOR)
            }

            pub fn register(c: &mut CollectionPool) {

                c.treatments.insert(&(desc() as Arc<dyn TreatmentDescriptor>));
            }
            
            async fn execute($host: &TreatmentHost) -> ResultStatus {
            
                $treatment
            }
            
            fn prepare(host: Arc<TreatmentHost>) -> Vec<TrackFuture> {
                
                let future = Box::new(Box::pin(
                    async move {

                        let result = execute(&host).await;
                        host.close_all().await;

                        result
                    }
                ));
            
                vec![future]
            }
            
            fn treatment(_: Arc<World>) -> Arc<dyn Treatment> {
            
                let treatment = TreatmentHost::new(desc(), prepare);
            
                treatment
            }
        }
    };
}
pub(crate) use treatment;

macro_rules! model_desc {
    ($rust_identifier:ident,$identifier:expr,$parameters:expr,$sources:expr) => {
        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreModelDescriptor> = {
                
                let builder = CoreModelBuilder::new($rust_identifier::new);

                let descriptor = CoreModelDescriptor::new(
                    $identifier,
                    $parameters,
                    $sources,
                    Box::new(builder)
                );

                let rc_descriptor = Arc::new(descriptor);
                rc_descriptor.set_autoref(&rc_descriptor);

                rc_descriptor
            };
        }
        
        Arc::clone(&DESCRIPTOR)
    };
}
pub(crate) use model_desc;

macro_rules! model_trait {
    ($identifier:ident,$initialize:ident,$shutdown:ident) => {
        impl Model for $identifier {
    
            fn descriptor(&self) -> Arc<CoreModelDescriptor> {
                Self::descriptor()
            }
        
            fn id(&self) -> Option<ModelId> {
                self.helper.id()
            }
        
            fn set_id(&self, id: ModelId) {
                self.helper.set_id(id);
            }
        
            fn set_parameter(&self, param: &str, value: &Value) {
                self.helper.set_parameter(param, value);
            }
        
            fn initialize(&self) {
                self.$initialize()
            }
        
            fn shutdown(&self) {
                self.$shutdown()
            }
        }
    };
    ($identifier:ident,$initialize:ident) => {
        impl Model for $identifier {
    
            fn descriptor(&self) -> Arc<CoreModelDescriptor> {
                Self::descriptor()
            }
        
            fn id(&self) -> Option<ModelId> {
                self.helper.id()
            }
        
            fn set_id(&self, id: ModelId) {
                self.helper.set_id(id);
            }
        
            fn set_parameter(&self, param: &str, value: &Value) {
                self.helper.set_parameter(param, value);
            }
        
            fn initialize(&self) {
                self.$initialize()
            }
        
            fn shutdown(&self) {
            }
        }
    };
    ($identifier:ident) => {
        impl Model for $identifier {
    
            fn descriptor(&self) -> Arc<CoreModelDescriptor> {
                Self::descriptor()
            }
        
            fn id(&self) -> Option<ModelId> {
                self.helper.id()
            }
        
            fn set_id(&self, id: ModelId) {
                self.helper.set_id(id);
            }
        
            fn set_parameter(&self, param: &str, value: &Value) {
                self.helper.set_parameter(param, value);
            }
        
            fn initialize(&self) {
            }
        
            fn shutdown(&self) {
            }
        }
    };
}
pub(crate) use model_trait;

macro_rules! ok_or_break {
    ($l:tt, $x:expr) => {
        if let Err(_) = $x {
            break $l;
        }
    };
    
    ($x:expr) => {
        if let Err(_) = $x {
            break;
        }
    };
}
pub(crate) use ok_or_break;
