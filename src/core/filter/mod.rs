
use crate::core::prelude::*;

macro_rules! impl_Filter {
    ($mel_name:expr, $mel_type_up:ident, $mel_type_name:expr, $func:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($func,
            core_identifier!("filter","scalar",$mel_type_name;$mel_name),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type_up,Stream),
                input!("decision",Scalar,Bool,Stream)
            ],
            outputs![
                output!("accepted",Scalar,$mel_type_up,Stream),
                output!("rejected",Scalar,$mel_type_up,Stream)
            ],
            host {
                let input_value = host.get_input("value");
                let input_decision = host.get_input("decision");

                let output_accepted = host.get_output("accepted");
                let output_rejected = host.get_output("rejected");

                let mut accepted_op = true, rejected_op = true;
            
                while let (Ok(value), Ok(decision)) = futures::join!(input_value.$recv_func(), input_decision.recv_one_bool()) {

                    if decision {
                        if let Err(_) = output_accepted.$send_func(value) {
                            // If we cannot send anymore on accepted, we note it,
                            // and check if rejected is still valid, else just terminate.
                            accepted_op = false;
                            if !rejected_op {
                                break;
                            }
                        }
                    }
                    else {
                        if let Err(_) = output_rejected.$send_func(value) {
                            // If we cannot send anymore on rejected, we note it,
                            // and check if accepted is still valid, else just terminate.
                            rejected_op = false;
                            if !accepted_op {
                                break;
                            }
                        }
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}
