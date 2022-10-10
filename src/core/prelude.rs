
pub use crate::executive::result_status::ResultStatus;
pub use crate::executive::future::TrackFuture;
pub use crate::executive::model::{Model, ModelId, ModelHost, HostedModel};
pub use crate::executive::value::Value;
pub use crate::executive::transmitter::*;
pub use crate::executive::treatment::*;
pub use crate::executive::world::{World, TrackId};
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
pub use downcast_rs::DowncastSync;
pub use async_std::prelude::*;
pub use crate::logic::descriptor::CoreSourceDescriptor;
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
        
            pub fn desc() -> std::sync::Arc<CoreTreatmentDescriptor> {
        
                lazy_static! {
                    static ref DESCRIPTOR: std::sync::Arc<CoreTreatmentDescriptor> = 
                        CoreTreatmentDescriptor::new(
                            $identifier,
                            $models,
                            $sources,
                            $parameters,
                            $inputs,
                            $outputs,
                            treatment,
                        );
                }
            
                std::sync::Arc::clone(&DESCRIPTOR)
            }

            pub fn register(c: &mut CollectionPool) {

                c.treatments.insert(&(desc() as std::sync::Arc<dyn TreatmentDescriptor>));
            }
            
            async fn execute($host: &TreatmentHost) -> ResultStatus {
            
                $treatment
            }
            
            fn prepare(host: std::sync::Arc<TreatmentHost>) -> Vec<TrackFuture> {
                
                let future = Box::new(Box::pin(
                    async move {

                        let result = execute(&host).await;
                        host.close_all().await;

                        result
                    }
                ));
            
                vec![future]
            }
            
            fn treatment(_: std::sync::Arc<World>, track_id: TrackId) -> std::sync::Arc<dyn Treatment> {
            
                let treatment = TreatmentHost::new(desc(), track_id, prepare);
            
                treatment
            }
        }
    };
}
pub(crate) use treatment;

macro_rules! source {
    ($mod:ident,$identifier:expr,$models:expr,$sources:expr,$outputs:expr) => {
        pub mod $mod {

            use crate::core::prelude::*;
        
            pub fn desc() -> std::sync::Arc<CoreSourceDescriptor> {
        
                lazy_static! {
                    static ref DESCRIPTOR: std::sync::Arc<CoreSourceDescriptor> = 
                    CoreSourceDescriptor::new(
                            $identifier,
                            $models,
                            $sources,
                            $outputs,
                        );
                }
            
                std::sync::Arc::clone(&DESCRIPTOR)
            }

            pub fn register(c: &mut CollectionPool) {

                c.treatments.insert(&(desc() as std::sync::Arc<dyn TreatmentDescriptor>));
            }
        }
    };
}
pub(crate) use source;

macro_rules! model {
    ($rust_identifier:ident,$identifier:expr,$parameters:expr,$sources:expr) => {
        pub mod model_host {

            use crate::core::prelude::*;

            fn new(world: std::sync::Arc<World>) -> std::sync::Arc<dyn Model> {
                ModelHost::new(descriptor(), world, super::$rust_identifier::new)
            }

            pub fn descriptor() -> std::sync::Arc<CoreModelDescriptor> {
                lazy_static! {
                    static ref DESCRIPTOR: std::sync::Arc<CoreModelDescriptor> =
                        CoreModelDescriptor::new(
                            $identifier,
                            $parameters,
                            $sources,
                            new
                        );
                }
                
                std::sync::Arc::clone(&DESCRIPTOR)
            }

            pub fn register(c: &mut CollectionPool) {

                c.models.insert(&(descriptor() as std::sync::Arc<dyn ModelDescriptor>));
            }
        }
    };
}
pub(crate) use model;

macro_rules! model_desc {
    ($rust_identifier:ident,$identifier:expr,$parameters:expr,$sources:expr) => {
        {
            lazy_static! {
                static ref DESCRIPTOR: std::sync::Arc<CoreModelDescriptor> =
                    CoreModelDescriptor::new(
                        $identifier,
                        $parameters,
                        $sources,
                        $rust_identifier::new
                    );
            }
            
            std::sync::Arc::clone(&DESCRIPTOR)
        }
    };
}
pub(crate) use model_desc;

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
