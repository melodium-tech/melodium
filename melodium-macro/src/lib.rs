
use proc_macro::TokenStream;
use std::borrow::Borrow;
use syn::{parse, ItemFn, FnArg, Pat, Type, parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_attribute]
pub fn mel_function(_attr: TokenStream, item: TokenStream) -> TokenStream {

    /*let input = parse_macro_input!(item as DeriveInput);
    let doc = input.attrs.iter().for_each(|a| println!("{a:?}"));*/

    let function: ItemFn = parse(item).unwrap();
    function.attrs.iter().for_each(|a| println!("{a:?}"));

    let name = function.sig.ident.to_string();
    println!("Name: {name}");
    let mut args = Vec::new();
    for arg in function.sig.inputs {
        match arg {
            FnArg::Typed(t) => {
                let name = if let Pat::Ident(ident) = t.pat.borrow() {
                    ident.ident.to_string()
                } else { eprintln!("Argument name expected"); break };

                let ty = if let Type::Path(path) = t.ty.borrow() {
                    path.path.segments.first().expect("Type expected").ident.to_string()
                } else { eprintln!("Argument type expected"); break };

                args.push((name, ty));
            },
            _ => eprintln!("Only Mélodium types are admissible arguments"),
        }
    }

    let parameters = args.iter().map(|(name, ty)| format!(
        r#"melodium_common::descriptor::Parameter::new("{name}", melodium_common::descriptor::Variability::Var, melodium_common::descriptor::DataType::new(melodium_common::descriptor::DataType::Structure::Scalar, melodium_common::descriptor::DataType::Type::{ty}), None)"#
    )).collect::<Vec<_>>().join(",");

    //println!("Signature: {:?}", function.sig);
    println!("Block: {:?}", function.block);

    let expanded = quote! {
        pub mod #name {

            pub fn descriptor() -> melodium_core::descriptor::FunctionDescriptor {
                melodium_core::descriptor::FunctionDescriptor::new(
                    melodium_common::descriptor::Identifier::new(vec!["example".to_string()], "#name"),
                    "documentation".to_string(),
                    #parameters,
                    melodium_common::descriptor::DataType::#return_type,
                    mel_function
                )
            }

            pub fn function(#rust_params) -> #rust_return_type {
                #statements
            }

            fn mel_function(params: Vec<melodium_common::executive::Value>) -> melodium_common::executive::Value {
                //Trucs
            }
        }
    };

    TokenStream::from(expanded)
}
