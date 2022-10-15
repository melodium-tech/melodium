
use std::sync::Arc;
use crate::core::prelude::*;


treatment!(and,
    core_identifier!("logic","byte";"And"),
    r#"Makes "and" ⋀ binary operation on `byte`."#.to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("a",Scalar,Byte,Stream),
        input!("b",Scalar,Byte,Stream)
    ],
    outputs![
        output!("result",Scalar,Byte,Stream)
    ],
    host {
        let input_a = host.get_input("a");
        let input_b = host.get_input("b");
        let result = host.get_output("result");
    
        while let (Ok(a), Ok(b)) = futures::join!(input_a.recv_one_byte(), input_b.recv_one_byte()) {

            ok_or_break!(result.send_byte(a & b).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(or,
    core_identifier!("logic","byte";"Or"),
    r#"Makes "or" ⋁ binary operation on `byte`."#.to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("a",Scalar,Byte,Stream),
        input!("b",Scalar,Byte,Stream)
    ],
    outputs![
        output!("result",Scalar,Byte,Stream)
    ],
    host {
        let input_a = host.get_input("a");
        let input_b = host.get_input("b");
        let result = host.get_output("result");
    
        while let (Ok(a), Ok(b)) = futures::join!(input_a.recv_one_byte(), input_b.recv_one_byte()) {

            ok_or_break!(result.send_byte(a | b).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(xor,
    core_identifier!("logic","byte";"Xor"),
    r#"Makes "xor" ⊕ binary operation on `byte`."#.to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("a",Scalar,Byte,Stream),
        input!("b",Scalar,Byte,Stream)
    ],
    outputs![
        output!("result",Scalar,Byte,Stream)
    ],
    host {
        let input_a = host.get_input("a");
        let input_b = host.get_input("b");
        let result = host.get_output("result");
    
        while let (Ok(a), Ok(b)) = futures::join!(input_a.recv_one_byte(), input_b.recv_one_byte()) {

            ok_or_break!(result.send_byte(a ^ b).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(not,
    core_identifier!("logic","byte";"Not"),
    r#"Makes "not" ¬ binary operation on `byte`."#.to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("value",Scalar,Byte,Stream)
    ],
    outputs![
        output!("value",Scalar,Byte,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("value");
    
        while let Ok(values) = input.recv_byte().await {

            ok_or_break!(output.send_multiple_byte(values.iter().map(|b| !b).collect()).await);
        }
    
        ResultStatus::Ok
    }
);

fn and_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::Byte(params[0].clone().byte() & params[1].clone().byte())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("logic","byte";"|and"),
        r#"_And_ ⋀ binary operation"#.to_string(),
        parameters![
            parameter!("a", Scalar, Byte, None),
            parameter!("b", Scalar, Byte, None)
        ],
        datatype!(Scalar, Byte),
        add
    )
}

fn or_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::Byte(params[0].clone().byte() | params[1].clone().byte())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("logic","byte";"|or"),
        r#"_Or_ ⋁ binary operation"#.to_string(),
        parameters![
            parameter!("a", Scalar, Byte, None),
            parameter!("b", Scalar, Byte, None)
        ],
        datatype!(Scalar, Byte),
        add
    )
}

fn xor_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::Byte(params[0].clone().byte() ^ params[1].clone().byte())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("logic","byte";"|xor"),
        r#"_Xor_ ⊕ binary operation"#.to_string(),
        parameters![
            parameter!("a", Scalar, Byte, None),
            parameter!("b", Scalar, Byte, None)
        ],
        datatype!(Scalar, Byte),
        add
    )
}

fn not_function() -> Arc<CoreFunctionDescriptor> {

    fn add(params: Vec<Value>) -> Value {
        Value::Byte(!params[0].clone().byte())
    }

    CoreFunctionDescriptor::new(
        core_identifier!("logic","byte";"|not"),
        r#"_Not_ ¬ binary operation"#.to_string(),
        parameters![
            parameter!("v", Scalar, Byte, None)
        ],
        datatype!(Scalar, Byte),
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
