
//! Provides collection of contexts types available.

use std::collections::HashMap;
use std::sync::Arc;
use super::descriptor::ContextDescriptor;
use super::descriptor::{DataTypeDescriptor as Data,
                        DataTypeStructureDescriptor as Structure,
                        DataTypeTypeDescriptor as Type};

pub struct Contexts {
    contexts: HashMap<String, Arc<ContextDescriptor>>
}

impl Contexts {

    fn init() -> Self {
        let mut contexts = Self {
            contexts: HashMap::new()
        };

        contexts.insert(ContextDescriptor::new("Track", vec![
            ("number", Data::new(Structure::Scalar, Type::Integer)),
            ("time",  Data::new(Structure::Scalar, Type::Real)),
        ]));

        contexts.insert(ContextDescriptor::new("File", vec![
            ("path", Data::new(Structure::Scalar, Type::String)),
            ("directory",  Data::new(Structure::Scalar, Type::String)),
            ("name",  Data::new(Structure::Scalar, Type::String)),
            ("stem",  Data::new(Structure::Scalar, Type::String)),
            ("extension",  Data::new(Structure::Scalar, Type::String)),
        ]));

        contexts.insert(ContextDescriptor::new("Signal", vec![
            ("sampleRate", Data::new(Structure::Scalar, Type::Integer)),
        ]));

        // Add there existing contexts.

        contexts
    }
    
    fn singleton() -> &'static Self {
        lazy_static! {
            static ref SINGLETON: Contexts = Contexts::init();
        }
        &SINGLETON
    }

    fn insert(&mut self, context: ContextDescriptor) {
        self.contexts.insert(context.name().to_string(), Arc::new(context));
    }

    pub fn get(name: &str) -> Option<&'static Arc<ContextDescriptor>> {

        Self::singleton().contexts.get(name)
    }
}
