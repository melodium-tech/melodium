
fn main() {

    let mut std_content = String::from("{");

    for entry in glob::glob("std/**/*.mel").unwrap() {
        match entry {
            Ok(relative_path) => {

                let absolute_path;
                match relative_path.canonicalize() {
                    Ok(ap) => absolute_path = ap,
                    Err(e) => {
                        panic!("{}", e)
                    },
                };

                std_content.push_str(&format!(r#"content.insert("{}", include_str!("{}"));"#, relative_path.to_str().unwrap(), absolute_path.to_str().unwrap()));
            }
            Err(e) => {
                panic!("{}", e)
            }
        }
    }

    std_content.push('}');

    std::fs::write(std::path::Path::new(std::env::var("OUT_DIR").as_ref().unwrap()).join("stdlib.rs"), std_content).unwrap();
}

