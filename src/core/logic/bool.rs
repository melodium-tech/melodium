
use std::sync::Arc;
use crate::core::prelude::*;

treatment!(and,
    core_identifier!("logic","bool";"And"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("a",Scalar,Bool,Stream),
        input!("b",Scalar,Bool,Stream)
    ],
    outputs![
        output!("result",Scalar,Bool,Stream)
    ],
    host {
        let input_a = host.get_input("a");
        let input_b = host.get_input("b");
        let result = host.get_output("result");
    
        while let (Ok(a), Ok(b)) = futures::join!(input_a.recv_one_bool(), input_b.recv_one_bool()) {

            ok_or_break!(result.send_bool(a && b).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(or,
    core_identifier!("logic","bool";"Or"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("a",Scalar,Bool,Stream),
        input!("b",Scalar,Bool,Stream)
    ],
    outputs![
        output!("result",Scalar,Bool,Stream)
    ],
    host {
        let input_a = host.get_input("a");
        let input_b = host.get_input("b");
        let result = host.get_output("result");
    
        while let (Ok(a), Ok(b)) = futures::join!(input_a.recv_one_bool(), input_b.recv_one_bool()) {

            ok_or_break!(result.send_bool(a || b).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(xor,
    core_identifier!("logic","bool";"Xor"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("a",Scalar,Bool,Stream),
        input!("b",Scalar,Bool,Stream)
    ],
    outputs![
        output!("result",Scalar,Bool,Stream)
    ],
    host {
        let input_a = host.get_input("a");
        let input_b = host.get_input("b");
        let result = host.get_output("result");
    
        while let (Ok(a), Ok(b)) = futures::join!(input_a.recv_one_bool(), input_b.recv_one_bool()) {

            ok_or_break!(result.send_bool(a ^ b).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(not,
    core_identifier!("logic","bool";"Not"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("value",Scalar,Bool,Stream)
    ],
    outputs![
        output!("value",Scalar,Bool,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("value");
    
        while let Ok(values) = input.recv_bool().await {

            ok_or_break!(output.send_multiple_bool(values.iter().map(|b| !b).collect()).await);
        }
    
        ResultStatus::Ok
    }
);

fn and_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::Bool(params[0].clone().bool() && params[1].clone().bool())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("logic","bool";"|and"),
        parameters![
            parameter!("a", Scalar, Bool, None),
            parameter!("b", Scalar, Bool, None)
        ],
        datatype!(Scalar, Bool),
        add
    )
}

fn or_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::Bool(params[0].clone().bool() || params[1].clone().bool())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("logic","bool";"|or"),
        parameters![
            parameter!("a", Scalar, Bool, None),
            parameter!("b", Scalar, Bool, None)
        ],
        datatype!(Scalar, Bool),
        add
    )
}

fn xor_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::Bool(params[0].clone().bool() ^ params[1].clone().bool())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("logic","bool";"|xor"),
        parameters![
            parameter!("a", Scalar, Bool, None),
            parameter!("b", Scalar, Bool, None)
        ],
        datatype!(Scalar, Bool),
        add
    )
}

fn not_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::Bool(!params[0].clone().bool())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("logic","bool";"|not"),
        parameters![
            parameter!("v", Scalar, Bool, None)
        ],
        datatype!(Scalar, Bool),
        add
    )
}

pub fn register(mut c: &mut CollectionPool) {

    and::register(&mut c);
    or::register(&mut c);
    xor::register(&mut c);
    not::register(&mut c);
    
    c.functions.insert(&(and_function() as Arc<dyn FunctionDescriptor>));
    c.functions.insert(&(or_function() as Arc<dyn FunctionDescriptor>));
    c.functions.insert(&(xor_function() as Arc<dyn FunctionDescriptor>));
    c.functions.insert(&(not_function() as Arc<dyn FunctionDescriptor>));
}
