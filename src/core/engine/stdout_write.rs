
use crate::core::prelude::*;

treatment!(stdout_write_treatment,
    core_identifier!("engine";"Write"),
    r#"Write process standard output.

    Send the received text to stdout."#.to_string(),
    models![],
    treatment_sources![],
    parameters![],
    inputs![
        input!("text",Scalar,String,Stream)
    ],
    outputs![],
    host {

        let input = host.get_input("text");
    
        'main: while let Ok(text) = input.recv_string().await {

            let mut stdout = async_std::io::stdout();

            for part in text {
                ok_or_break!('main, stdout.write_all(part.as_bytes()).await);
            }

            ok_or_break!(stdout.flush().await);
        }

        ResultStatus::Ok
    }
);
