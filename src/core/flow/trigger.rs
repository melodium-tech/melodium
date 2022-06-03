
use crate::core::prelude::*;

treatment!(trigger,
    core_identifier!("flow";"Trigger"),
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

