
use crate::core::prelude::*;

treatment!(size,
    core_identifier!("flow","vector","void";"Size"),
    indoc!(r#"Gives number of elements present in each vector passing through input stream.

    For each vector one `size` value is sent, giving the number of elements contained within matching vector.
    
    ```mermaid
    graph LR
        T("Size()")
        V["…［🟦 🟦］［🟦］［🟦 🟦 🟦］…"] -->|value| T
        
        T -->|size| P["… 2️⃣ 1️⃣ 3️⃣ …"]
    
        style V fill:#ffff,stroke:#ffff
        style P fill:#ffff,stroke:#ffff
    ```"#).to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("pattern",Vector,Void,Stream)
    ],
    outputs![
        output!("size",Scalar,U64,Stream)
    ],
    host {
        let input = host.get_input("pattern");
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
