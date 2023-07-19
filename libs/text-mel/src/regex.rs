use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};
use regex::Regex;

#[mel_treatment(
    input text Stream<string>
    output matches Stream<bool>
    output error Block<string>
)]
pub async fn matches(regex: string) {
    match Regex::new(&regex) {
        Ok(regex) => {
            error.close().await;

            while let Ok(text) = text.recv_string().await {
                check!(matches.send_bool(text.into_iter().map(|txt| regex.is_match(&txt)).collect()).await);
            }
        },
        Err(err) => {
            let _ = error.send_one_string(err.to_string()).await;
        }
    }
}

#[mel_function]
pub fn matches(text: string, regex: string) -> bool {
    match Regex::new(&regex) {
        Ok(regex) => {
            regex.is_match(&text)
        },
        Err(_) => {
            false
        }
    }
}

#[mel_treatment(
    input text Stream<string>
    output matches Stream<bool>
    output found Stream<string>
    output error Block<string>
)]
pub async fn find(regex: string) {
    match Regex::new(&regex) {
        Ok(regex) => {
            error.close().await;

            while let Ok(text) = text.recv_string().await {

                let mut vec_matches = Vec::with_capacity(text.len());
                let mut vec_found = Vec::with_capacity(text.len());

                for text in text {
                    match regex.find(&text) {
                        Some(m) => {
                            vec_matches.push(true);
                            vec_found.push(m.as_str().to_string());
                        },
                        None => {
                            vec_matches.push(false);
                            vec_found.push(String::default());
                        }
                    }
                }

                if let (Err(_), Err(_)) = futures::join!(matches.send_bool(vec_matches), found.send_string(vec_found)) {
                    break;
                }
            }
        },
        Err(err) => {
            let _ = error.send_one_string(err.to_string()).await;
        }
    }
}

#[mel_function]
pub fn find(text: string, regex: string) -> string {
    match Regex::new(&regex) {
        Ok(regex) => {
            regex.find(&text).map(|m| m.as_str().to_string()).unwrap_or_default()
        },
        Err(_) => {
            String::default()
        }
    }
}

///////
/* 
#[mel_treatment(
    input text Stream<string>
    output matches Stream<bool>
    output groups Stream<Vec<string>>
    output group_names Stream<Vec<string>>
    output group_matches Stream<Vec<bool>>
    output error Block<string>
)]
pub async fn capture(regex: string) {
    match Regex::new(&regex) {
        Ok(regex) => {
            error.close().await;

            let names = regex.capture_names().map(|name| name.map(|n| n.to_string()).unwrap_or_default()).collect();

            while let Ok(text) = text.recv_string().await {

                let mut vec_matches = Vec::with_capacity(text.len());
                let mut vec_groups = Vec::with_capacity(text.len());
                let mut vec_group_names = vec![names; text.len()];
                let mut vec_group_matches = Vec::with_capacity(text.len());

                for text in text {
                    match regex.captures(&text) {
                        Some(captures) => {
                            vec_matches.push(true);
                            vec_found.push(m.as_str().to_string());
                        },
                        None => {
                            vec_matches.push(false);
                            vec_found.push(String::default());
                        }
                    }
                }

                if let (Err(_), Err(_)) = futures::join!(matches.send_bool(vec_matches), found.send_string(vec_found)) {
                    break;
                }
            }
        },
        Err(err) => {
            let _ = error.send_one_string(err.to_string()).await;
        }
    }
}

#[mel_function]
pub fn find(text: string, regex: string) -> string {
    match Regex::new(&regex) {
        Ok(regex) => {
            regex.find(&text).map(|m| m.as_str().to_string()).unwrap_or_default()
        },
        Err(_) => {
            String::default()
        }
    }
}


*/