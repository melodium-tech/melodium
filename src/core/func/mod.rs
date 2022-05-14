
use crate::core::prelude::*;

pub fn register(c: &mut CollectionPool) {
    c.functions.insert(&(add_function() as Arc<dyn FunctionDescriptor>));
}

fn add_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::I32(params[0].clone().i32() + params[1].clone().i32())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("func";"|add_i32"),
        parameters![
            parameter!("a", Scalar, I32, None),
            parameter!("b", Scalar, I32, None)
        ],
        datatype!(Scalar, I32),
        add
    )
}
