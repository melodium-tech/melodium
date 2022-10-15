
use crate::core::prelude::*;

treatment!(trigger,
    core_identifier!("flow";"Trigger"),
    r#"Trigger on `void` stream start and finish.

    Send `start` when a first value is send through the stream.
    Send `finish` when stream is finally over.
    
    ```mermaid
    graph LR
        T(Trigger)
        B["🔴 … 🟦 🟦 🟦 🟦 🟦 🟦 … 🟢"] -->|value| T
        
        T -->|start| S["〈🟩〉"]
        T -->|finish| F["〈🟥〉"]
    
        style B fill:#ffff,stroke:#ffff
        style S fill:#ffff,stroke:#ffff
        style F fill:#ffff,stroke:#ffff
    ```
    
    ℹ️ If the stream never receive any data before being closed, only `finish` will be emitted."#.to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("iter",Scalar,Void,Stream)
    ],
    outputs![
        output!("start",Scalar,Void,Block),
        output!("finish",Scalar,Void,Block)
    ],
    host {
        let input = host.get_input("iter");
        let start = host.get_output("start");
        let finish = host.get_output("finish");

        if let Ok(_) = input.recv_void().await {
            let _ = start.send_void(()).await;
            start.close().await;
        }

        while let Ok(_) = input.recv_void().await {}

        let _ = finish.send_void(()).await;
    
        ResultStatus::Ok
    }
);

