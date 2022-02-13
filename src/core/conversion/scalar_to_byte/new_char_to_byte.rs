
/*mod char_to_byte {

    use super::super::super::super::prelude::*;

    fn desc() -> Arc<CoreTreatmentDescriptor> {

        lazy_static! {
            static ref DESCRIPTOR: Arc<CoreTreatmentDescriptor> = {
    
                let rc_descriptor = CoreTreatmentDescriptor::new(
                    core_identifier!("conversion";"NewCharToByte"),
                    models![],
                    treatment_sources![],
                    vec![],
                    vec![
                        input!("value",Scalar,Char,Stream)
                    ],
                    vec![
                        output!("data",Scalar,Byte,Stream)
                    ],
                    treatment,
                );
    
                rc_descriptor
            };
        }
    
        Arc::clone(&DESCRIPTOR)
    }
    
    async fn execute(host: &TreatmentHost) -> ResultStatus {
    
        let input = host.get_input("value");
        let output = host.get_output("data");
    
        while let Ok(chars) = input.recv_char().await {
    
            for ch in chars {
                output.send_multiple_byte(ch.to_string().as_bytes().to_vec()).await;
            }
        }
    
        ResultStatus::Ok
    }
    
    fn prepare(host: &TreatmentHost) -> Vec<TrackFuture> {
        
        let future = Box::new(Box::pin(async move { execute(host).await }));
    
        vec![future]
    }
    
    fn treatment(_: Arc<World>) -> Arc<dyn Treatment> {
    
        let treatment = TreatmentHost::new(desc(), prepare);
    
        Arc::new(treatment)
    }
}*/

use crate::core::prelude::*;

treatment!(char_to_byte,
    core_identifier!("conversion";"NewCharToByte"),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("value",Scalar,Char,Stream)
    ],
    outputs![
        output!("data",Scalar,Byte,Stream)
    ],
    host {
        let input = host.get_input("value");
        let output = host.get_output("data");
    
        while let Ok(chars) = input.recv_char().await {
    
            for ch in chars {
                output.send_multiple_byte(ch.to_string().as_bytes().to_vec()).await;
            }
        }
    
        ResultStatus::Ok
    }
);

