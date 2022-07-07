
use crate::core::prelude::*;

treatment!(count_scalar,
    core_identifier!("flow","scalar","void";"Count"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("iter",Scalar,Void,Stream)
    ],
    outputs![
        output!("count",Scalar,U128,Stream)
    ],
    host {
        let input = host.get_input("iter");
        let output = host.get_output("count");

        let mut count = 1u128;

        while let Ok(iter) = input.recv_void().await {

            let mut vec = Vec::with_capacity(iter.len());

            for _ in 0..iter.len() {
                vec.push(count);
                count += 1;
            }

            ok_or_break!(output.send_multiple_u128(vec).await);
        }
    
        ResultStatus::Ok
    }
);

treatment!(count_vector,
    core_identifier!("flow","vector","void";"Count"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("iter",Vector,Void,Stream)
    ],
    outputs![
        output!("count",Scalar,U128,Stream)
    ],
    host {
        let input = host.get_input("iter");
        let output = host.get_output("count");

        let mut count = 1u128;

        while let Ok(iter) = input.recv_vec_void().await {

            let mut vec = Vec::with_capacity(iter.len());

            for _ in 0..iter.len() {
                vec.push(count);
                count += 1;
            }

            ok_or_break!(output.send_multiple_u128(vec).await);
        }
    
        ResultStatus::Ok
    }
);

pub fn register(mut c: &mut CollectionPool) {

    count_scalar::register(&mut c);
    count_vector::register(&mut c);
}
