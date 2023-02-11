use convert_case::{Case, Casing};
use core::{borrow::Borrow, convert::TryFrom};
use litrs::StringLit;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse, FnArg, GenericArgument, ItemFn, Pat, PathArguments, ReturnType, Type};

fn into_mel_type(ty: &Type) -> String {
    match ty {
        Type::Path(path) => {
            let ty = path.path.segments.first().expect("Type expected");

            if ty.ident.to_string() == "Vec" {
                if let PathArguments::AngleBracketed(ab) = &ty.arguments {
                    if let GenericArgument::Type(ty) = ab.args.first().expect("Type expected") {
                        let internal_type = into_mel_type(ty);
                        format!("Vec{internal_type}")
                    } else {
                        panic!("Type expected");
                    }
                } else {
                    panic!("Type expected");
                }
            } else {
                let internal_type = ty.ident.to_string();
                match internal_type.as_str() {
                    "byte" | "bool" | "void" | "char" | "string" | "f32" | "f64" | "u8" | "u16"
                    | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" => {
                        internal_type.to_case(Case::UpperCamel)
                    }
                    _ => panic!("Given type is not a Mélodium one"),
                }
            }
        }
        _ => {
            panic!("Type expected");
        }
    }
}

fn into_mel_datatype(ty: &str) -> String {
    match ty {
        "U8" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::U8)",
"U16" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::U16)",
"U32" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::U32)",
"U64" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::U64)",
"U128" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::U128)",
"I8" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::I8)",
"I16" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::I16)",
"I32" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::I32)",
"I64" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::I64)",
"I128" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::I128)",
"F32" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::F32)",
"F64" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::F64)",
"Bool" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::Bool)",
"Byte" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::Byte)",
"Char" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::Char)",
"String" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::String)",
"Void" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Scalar, melodium_core::common::descriptor::Type::Void)",
"VecU8" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::U8)",
"VecU16" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::U16)",
"VecU32" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::U32)",
"VecU64" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::U64)",
"VecU128" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::U128)",
"VecI8" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::I8)",
"VecI16" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::I16)",
"VecI32" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::I32)",
"VecI64" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::I64)",
"VecI128" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::I128)",
"VecF32" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::F32)",
"VecF64" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::F64)",
"VecBool" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::Bool)",
"VecByte" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::Byte)",
"VecChar" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::Char)",
"VecString" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::String)",
"VecVoid" => "melodium_core::common::descriptor::DataType::new(melodium_core::common::descriptor::Structure::Vector, melodium_core::common::descriptor::Type::Void)",
_ => panic!("Given type cannot be made into datatype"),
    }.to_string()
}

fn into_mel_value_call(ty: &str) -> String {
    match ty {
        "U8" => "u8",
        "U16" => "u16",
        "U32" => "u32",
        "U64" => "u64",
        "U128" => "u128",
        "I8" => "i8",
        "I16" => "i16",
        "I32" => "i32",
        "I64" => "i64",
        "I128" => "i128",
        "F32" => "f32",
        "F64" => "f64",
        "Bool" => "bool",
        "Byte" => "byte",
        "Char" => "char",
        "String" => "string",
        "Void" => "void",
        "VecU8" => "vec_u8",
        "VecU16" => "vec_u16",
        "VecU32" => "vec_u32",
        "VecU64" => "vec_u64",
        "VecU128" => "vec_u128",
        "VecI8" => "vec_i8",
        "VecI16" => "vec_i16",
        "VecI32" => "vec_i32",
        "VecI64" => "vec_i64",
        "VecI128" => "vec_i128",
        "VecF32" => "vec_f32",
        "VecF64" => "vec_f64",
        "VecBool" => "vec_bool",
        "VecByte" => "vec_byte",
        "VecChar" => "vec_char",
        "VecString" => "vec_string",
        "VecVoid" => "vec_void",
        _ => panic!("Given type cannot be made into value call"),
    }
    .to_string()
}

#[proc_macro_attribute]
pub fn mel_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function: ItemFn = parse(item).unwrap();
    //function.attrs.iter().for_each(|a| println!("{a:?}"));
    let mut documentation = Vec::new();
    for attr in function.attrs.clone() {
        if let Some(segment) = attr.path.segments.first() {
            if segment.ident.to_string() == "doc" {
                for tt in attr.tokens {
                    if let TokenTree::Literal(lit) = tt {
                        let doclit = StringLit::try_from(lit).unwrap();
                        documentation.push(doclit.value().to_string());
                    }
                }
            }
        }
    }

    let name = function.sig.ident.to_string();
    let mut args = Vec::new();
    for arg in &function.sig.inputs {
        match arg {
            FnArg::Typed(t) => {
                let name = if let Pat::Ident(ident) = t.pat.borrow() {
                    ident.ident.to_string()
                } else {
                    eprintln!("Argument name expected");
                    break;
                };

                let ty = into_mel_type(t.ty.borrow());

                args.push((name, ty));
            }
            _ => eprintln!("Only Mélodium types are admissible arguments"),
        }
    }

    let parameters = args.iter().map(|(name, ty)| {
    let datatype = into_mel_datatype(ty);
    format!(
        r#"melodium_core::common::descriptor::Parameter::new("{name}", melodium_core::common::descriptor::Variability::Var, {datatype}, None)"#
    )}).collect::<Vec<_>>().join(",");
    let return_type = if let ReturnType::Type(_, rt) = &function.sig.output {
        into_mel_type(rt)
    } else {
        panic!("Return type expected");
    };

    let mel_call = format!(
        "melodium_core::common::executive::Value::{return_type}({name}({}))",
        args.iter()
            .enumerate()
            .map(|(i, (_, ty))| format!("params[{i}].clone().{}()", into_mel_value_call(ty)))
            .collect::<Vec<_>>()
            .join(",")
    );

    let module_name: proc_macro2::TokenStream = format!("__mel_function_{name}").parse().unwrap();
    let documentation = documentation.join("\n");
    let parameters: proc_macro2::TokenStream = parameters.parse().unwrap();
    let return_type: proc_macro2::TokenStream = into_mel_datatype(&return_type).parse().unwrap();
    let mel_call: proc_macro2::TokenStream = mel_call.parse().unwrap();

    let expanded = quote! {
        #function

        pub mod #module_name {

            use super::*;

            pub fn descriptor() -> std::sync::Arc<melodium_core::descriptor::Function> {
                melodium_core::descriptor::Function::new(
                    melodium_core::descriptor::module_path_to_identifier(module_path!(), #name),
                    #documentation.to_string(),
                    vec![#parameters],
                    #return_type,
                    mel_function
                )
            }

            fn mel_function(params: Vec<melodium_core::common::executive::Value>) -> melodium_core::common::executive::Value {
                #mel_call
            }
        }
    };

    TokenStream::from(expanded)
}
