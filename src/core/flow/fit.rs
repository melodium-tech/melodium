
use crate::core::prelude::*;

treatment!(fit,
    core_identifier!("flow","vector","void";"Fit"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("size",Scalar,U64,Stream)
    ],
    outputs![
        output!("pattern",Vector,Void,Stream)
    ],
    host {
        let input = host.get_input("size");
        let output = host.get_output("pattern");

        while let Ok(sizes) = input.recv_u64().await {

            let mut vectors = Vec::with_capacity(sizes.len());

            for s in sizes {
                vectors.push(vec![(); s as usize]);
            }

            ok_or_break!(output.send_multiple_vec_void(vectors).await);
        }
    
        ResultStatus::Ok
    }
);

pub fn register(mut c: &mut CollectionPool) {

    fit::register(&mut c);
}
