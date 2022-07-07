
use crate::core::prelude::*;

treatment!(size,
    core_identifier!("flow","vector","void";"Size"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("value",Vector,Void,Stream)
    ],
    outputs![
        output!("size",Scalar,U64,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("size");

        while let Ok(vectors) = input.recv_vec_void().await {

            let mut sizes = Vec::with_capacity(vectors.len());

            for v in vectors {
                sizes.push(v.len() as u64);
            }

            ok_or_break!(output.send_multiple_u64(sizes).await);
        }
    
        ResultStatus::Ok
    }
);

pub fn register(mut c: &mut CollectionPool) {

    size::register(&mut c);
}
