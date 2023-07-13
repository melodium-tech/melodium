#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use convert_case::{Case, Casing};
use core::{borrow::Borrow, convert::TryFrom, iter::FromIterator};
use litrs::StringLit;
use proc_macro::TokenStream;
use proc_macro2::{token_stream::IntoIter as IntoIterTokenStream, TokenTree};
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse, parse_file, Fields, FnArg, GenericArgument, Item, ItemFn, ItemStruct, Pat,
    PathArguments, ReturnType, Type,
};

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

fn into_rust_type(ty: &str) -> String {
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
        "VecU8" => "Vec<u8>",
        "VecU16" => "Vec<u16>",
        "VecU32" => "Vec<u32>",
        "VecU64" => "Vec<u64>",
        "VecU128" => "Vec<u128>",
        "VecI8" => "Vec<i8>",
        "VecI16" => "Vec<i16>",
        "VecI32" => "Vec<i32>",
        "VecI64" => "Vec<i64>",
        "VecI128" => "Vec<i128>",
        "VecF32" => "Vec<f32>",
        "VecF64" => "Vec<f64>",
        "VecBool" => "Vec<bool>",
        "VecByte" => "Vec<byte>",
        "VecChar" => "Vec<char>",
        "VecString" => "Vec<string>",
        "VecVoid" => "Vec<void>",
        _ => panic!("Given type cannot be made into rust one"),
    }
    .to_string()
}

fn into_rust_value(ty: &str, lit: &str) -> String {
    match ty {
        "Byte" => format!("{lit}u8"),
        "F32" => format!("{lit}f32"),
        "F64" => format!("{lit}f64"),
        "String" => format!("{lit}.to_string()"),
        _ => lit.to_string(),
    }
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

fn config_default(ts: &mut IntoIterTokenStream) -> (String, String) {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        (name.to_string(), config_value(ts))
    } else {
        panic!("Name identity expected")
    }
}

fn config_param(ts: &mut IntoIterTokenStream) -> (String, String, Option<String>) {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        (name.to_string(), config_ty(ts), config_optionnal_value(ts))
    } else {
        panic!("Name identity expected")
    }
}

fn config_full_source(
    ts: &mut IntoIterTokenStream,
) -> (String, Vec<String>, Vec<(String, String, String)>) {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        let mut contextes = Vec::new();
        if let Some(TokenTree::Group(group)) = ts.next() {
            for tt in group.stream() {
                if let TokenTree::Ident(c) = tt {
                    contextes.push(format!("__mel_context_{c}"))
                } else if let TokenTree::Group(g) = tt {
                    contextes.push(token_stream_to_context_address(g.stream()))
                } else {
                    panic!("Context identity expected")
                }
            }
        } else {
            panic!("Context list expected")
        }

        let mut outputs = Vec::new();
        if let Some(TokenTree::Group(group)) = ts.next() {
            let mut ts = group.stream().into_iter();
            loop {
                if let Some(output) = config_io(&mut ts) {
                    outputs.push(output);
                } else {
                    break;
                }
            }
        } else {
            panic!("Outputs list expected")
        }

        (name.to_string(), contextes, outputs)
    } else {
        panic!("Name identity expected")
    }
}

fn token_stream_to_context_address(ts: proc_macro2::TokenStream) -> String {
    let full_address = ts.to_string().replace(" ", "");
    let mut steps = full_address
        .split("::")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    if let Some(last) = steps.pop() {
        steps.push(format!("__mel_context_{last}"));
    } else {
        panic!("Wrong context address")
    }
    steps.join("::")
}

fn config_value(ts: &mut IntoIterTokenStream) -> String {
    let next = ts.next();
    if let Some(TokenTree::Literal(default)) = next {
        default.to_string()
    } else if let Some(TokenTree::Ident(ident)) = next {
        ident.to_string()
    } else if let Some(TokenTree::Punct(punct)) = next {
        match punct.as_char() {
            '-' => {
                if let Some(TokenTree::Literal(default)) = ts.next() {
                    format!("-{}", default.to_string())
                } else {
                    panic!("Default value expected")
                }
            }
            _ => panic!("Unexpected punctuation"),
        }
    } else {
        panic!("Default value expected")
    }
}

fn config_optionnal_value(ts: &mut IntoIterTokenStream) -> Option<String> {
    let next = ts.next();
    if let Some(TokenTree::Literal(default)) = next {
        Some(default.to_string())
    } else if let Some(TokenTree::Punct(punct)) = next {
        match punct.as_char() {
            '-' => {
                if let Some(TokenTree::Literal(default)) = ts.next() {
                    Some(format!("-{}", default.to_string()))
                } else {
                    panic!("Default value expected")
                }
            }
            _ => panic!("Unexpected punctuation"),
        }
    } else if let Some(TokenTree::Ident(ident)) = next {
        if ident.to_string() == "none" {
            None
        } else {
            panic!("Unrecognized default value")
        }
    } else {
        panic!("Default value expected")
    }
}

fn config_model(ts: &mut IntoIterTokenStream) -> (String, String) {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        let next = ts.next();
        if let Some(TokenTree::Ident(ty)) = next {
            (name.to_string(), format!("__mel_model_{ty}"))
        } else if let Some(TokenTree::Group(group)) = next {
            (
                name.to_string(),
                token_stream_to_model_address(group.stream()),
            )
        } else {
            panic!("Type identity expected")
        }
    } else {
        panic!("Name identity expected")
    }
}

fn config_source(ts: &mut IntoIterTokenStream) -> (String, String, String) {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        let next = ts.next();
        if let Some(TokenTree::Ident(ty)) = next {
            if let Some(TokenTree::Ident(source)) = ts.next() {
                (
                    name.to_string(),
                    format!("__mel_model_{ty}"),
                    source.to_string(),
                )
            } else {
                panic!("Source identity expected")
            }
        } else if let Some(TokenTree::Group(group)) = next {
            if let Some(TokenTree::Ident(source)) = ts.next() {
                (
                    name.to_string(),
                    token_stream_to_model_address(group.stream()),
                    source.to_string(),
                )
            } else {
                panic!("Source identity expected")
            }
        } else {
            panic!("Type identity expected")
        }
    } else {
        panic!("Name identity expected")
    }
}

fn token_stream_to_model_address(ts: proc_macro2::TokenStream) -> String {
    let full_address = ts.to_string().replace(" ", "");
    let mut steps = full_address
        .split("::")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    if let Some(last) = steps.pop() {
        steps.push(format!("__mel_model_{last}"));
    } else {
        panic!("Wrong model address")
    }
    steps.join("::")
}

fn config_ty(ts: &mut IntoIterTokenStream) -> String {
    let mel_ty;
    if let Some(TokenTree::Ident(ty_env)) = ts.next() {
        let ty_env = ty_env.to_string();
        let env;
        let ty;
        if ty_env == "Vec" {
            env = "Vec";
            ts.next(); // <
            if let Some(TokenTree::Ident(mandatory_ty)) = ts.next() {
                ty = mandatory_ty.to_string().to_case(Case::UpperCamel);
            } else {
                panic!("Type identity expected")
            }
            ts.next(); // >
        } else {
            env = "";
            ty = ty_env.to_case(Case::UpperCamel);
        }
        mel_ty = format!("{env}{ty}");
    } else {
        panic!("Type identity expected")
    }

    mel_ty
}

fn config_io(ts: &mut IntoIterTokenStream) -> Option<(String, String, String)> {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        if let Some(TokenTree::Ident(flow)) = ts.next() {
            ts.next(); // <

            let mel_ty = config_ty(ts);

            ts.next(); // >
            Some((name.to_string(), flow.to_string(), mel_ty))
        } else {
            panic!("Flow identity expected")
        }
    } else {
        None
    }
}

#[proc_macro]
pub fn mel_package(_: TokenStream) -> TokenStream {
    let mut functions = Vec::new();
    let mut contexts = Vec::new();
    let mut models = Vec::new();
    let mut sources = Vec::new();
    let mut treatments = Vec::new();

    let mut root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    root.push_str("/src/");
    root = root.replace("/", &std::path::MAIN_SEPARATOR.to_string());
    for entry in glob::glob(&format!("{root}**/*.rs")).unwrap() {
        match &entry {
            Ok(path) => {
                if let Ok(content) = parse_file(&std::fs::read_to_string(path).unwrap()) {
                    for item in &content.items {
                        let name;
                        let mut is_mel_function = false;
                        let mut is_mel_treatment = false;
                        let mut is_mel_model = false;
                        let mut is_mel_context = false;
                        match item {
                            Item::Fn(item_fn) => {
                                name = item_fn.sig.ident.to_string();

                                item_fn.attrs.iter().for_each(|attr| {
                                    match attr
                                        .path
                                        .segments
                                        .first()
                                        .unwrap()
                                        .ident
                                        .to_string()
                                        .as_str()
                                    {
                                        "mel_function" => is_mel_function = true,
                                        "mel_treatment" => is_mel_treatment = true,
                                        _ => {}
                                    }
                                });
                            }
                            Item::Struct(item_struct) => {
                                name = item_struct.ident.to_string();

                                item_struct.attrs.iter().for_each(|attr| {
                                    match attr
                                        .path
                                        .segments
                                        .first()
                                        .unwrap()
                                        .ident
                                        .to_string()
                                        .as_str()
                                    {
                                        "mel_model" => is_mel_model = true,
                                        "mel_context" => is_mel_context = true,
                                        _ => {}
                                    }
                                });
                            }
                            _ => continue,
                        }

                        let mut call = path
                            .to_str()
                            .unwrap()
                            .strip_prefix(&root)
                            .unwrap()
                            .strip_suffix(".rs")
                            .unwrap()
                            .replace(std::path::MAIN_SEPARATOR, "::");

                        if call == "lib" {
                            call = "".to_string();
                        } else {
                            call = format!("::{call}");
                        }

                        if call.ends_with("::mod") {
                            call = call.strip_suffix("::mod").unwrap().to_string();
                        }

                        if is_mel_function {
                            call.push_str(&format!("::__mel_function_{name}::descriptor()"));
                            functions.push(call);
                        } else if is_mel_treatment {
                            call.push_str(&format!("::__mel_treatment_{name}::descriptor()"));
                            treatments.push(call);
                        } else if is_mel_model {
                            let mut model_call = call.clone();
                            model_call.push_str(&format!("::__mel_model_{name}::descriptor()"));
                            models.push(model_call);
                            let mut sources_call = call.clone();
                            sources_call.push_str(&format!("::__mel_model_{name}::sources()"));
                            sources.push(sources_call);
                        } else if is_mel_context {
                            call.push_str(&format!("::__mel_context_{name}::descriptor()"));
                            contexts.push(call);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let functions = functions
        .iter()
        .map(|elmt| {
            format!(
                "collection.insert(melodium_core::common::descriptor::Entry::Function(crate{elmt}));"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let contexts = contexts
        .iter()
        .map(|elmt| {
            format!("collection.insert(melodium_core::common::descriptor::Entry::Context(crate{elmt}));")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let models = models
        .iter()
        .map(|elmt| {
            format!(
                "collection.insert(melodium_core::common::descriptor::Entry::Model(crate{elmt}));"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let sources = sources
    .iter()
    .map(|elmt| format!(
        "crate{elmt}.into_iter().for_each(|s| collection.insert(melodium_core::common::descriptor::Entry::Treatment(s)));"
    ))
    .collect::<Vec<_>>()
    .join("\n");

    let treatments = treatments
        .iter()
        .map(|elmt| {
            format!(
                "collection.insert(melodium_core::common::descriptor::Entry::Treatment(crate{elmt}));"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let collection: proc_macro2::TokenStream = format!(
        r"
            let mut collection = melodium_core::common::descriptor::Collection::new();
            {functions}
            {contexts}
            {models}
            {sources}
            {treatments}
            collection
        "
    )
    .parse()
    .unwrap();

    let cargo_toml: toml::Table = toml::from_str(
        &std::fs::read_to_string(&format!(
            "{}/Cargo.toml",
            std::env::var("CARGO_MANIFEST_DIR").unwrap()
        ))
        .unwrap(),
    )
    .unwrap();
    let name = cargo_toml
        .get("package")
        .and_then(|v| {
            if let toml::Value::Table(pkg) = v {
                Some(pkg)
            } else {
                None
            }
        })
        .unwrap()
        .get("name")
        .and_then(|v| {
            if let toml::Value::String(s) = v {
                Some(s)
            } else {
                None
            }
        })
        .unwrap()
        .strip_suffix("-mel")
        .unwrap()
        .to_string();

    let requirements: proc_macro2::TokenStream = cargo_toml
        .get("dependencies")
        .and_then(|v| {
            if let toml::Value::Table(deps) = v {
                Some(deps)
            } else {
                None
            }
        })
        .unwrap()
        .keys()
        .filter_map(|k| k.strip_suffix("-mel").map(|k| format!(r#"melodium_core::common::descriptor::PackageRequirement{{package:"{k}".to_string(),version_requirement:melodium_core::common::descriptor::VersionReq::STAR}}"#)))
        .collect::<Vec<_>>()
        .join(",")
        .parse()
        .unwrap();

    let mut embedded = Vec::new();
    let mut root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    root.push_str("/mel/");
    root = root.replace("/", &std::path::MAIN_SEPARATOR.to_string());
    for entry in glob::glob(&format!("{root}**/*.mel")).unwrap() {
        match &entry {
            Ok(path) => {
                let plat_name = path.to_string_lossy().to_string();
                let mel_name = plat_name
                    .strip_prefix(&root)
                    .unwrap()
                    .replace(std::path::MAIN_SEPARATOR, "/");
                embedded.push((plat_name, mel_name))
            }
            _ => {}
        }
    }

    let embedded: proc_macro2::TokenStream = embedded
        .into_iter()
        .map(|(path, filename)| {
            format!(r#"embedded.insert("{name}/{filename}", &include_bytes!(r"{path}")[..])"#)
        })
        .collect::<Vec<_>>()
        .join(";")
        .parse()
        .unwrap();

    let expanded = quote! {

        #[no_mangle]
        #[cfg(feature = "plugin")]
        pub extern "C" fn melodium_package() -> *const melodium_core::common::descriptor::Package {
            std::sync::Arc::into_raw(__mel_package::package())
        }

        pub mod __mel_package {

            static NAME: &str = #name;
            static VERSION: melodium_core::Lazy<melodium_core::common::descriptor::Version> = melodium_core::Lazy::new(|| melodium_core::common::descriptor::Version::parse(env!("CARGO_PKG_VERSION")).unwrap());
            static REQUIREMENTS: melodium_core::Lazy<Vec<melodium_core::common::descriptor::PackageRequirement>> = melodium_core::Lazy::new(|| { vec![#requirements] });
            static EMBEDDED: melodium_core::Lazy<std::collections::HashMap<&'static str, &'static [u8]>> = melodium_core::Lazy::new(|| { let mut embedded = std::collections::HashMap::new(); #embedded; embedded });

            pub fn package() -> std::sync::Arc<dyn melodium_core::common::descriptor::Package> {
                std::sync::Arc::new(MelPackage::new())
            }

            #[derive(Debug)]
            pub struct MelPackage {}

            impl MelPackage {
                pub fn new() -> Self {
                    Self {}
                }

                pub fn collection(&self) -> melodium_core::common::descriptor::Collection {
                    #collection
                }
            }

            impl melodium_core::common::descriptor::Package for MelPackage {
                fn name(&self) -> &str {
                    NAME
                }

                fn version(&self) -> &melodium_core::common::descriptor::Version {
                    &VERSION
                }

                fn requirements(&self) -> &Vec<melodium_core::common::descriptor::PackageRequirement> {
                    &REQUIREMENTS
                }

                fn collection(&self, _: &dyn melodium_core::common::descriptor::Loader) -> melodium_core::common::descriptor::LoadingResult<melodium_core::common::descriptor::Collection> {
                    melodium_core::common::descriptor::LoadingResult::new_success(MelPackage::collection(&self))
                }

                fn embedded(&self) -> &std::collections::HashMap<&'static str, &'static [u8]> {
                    &EMBEDDED
                }
            }
        }

    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn mel_treatment(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut defaults = HashMap::new();
    let mut models = HashMap::new();
    let mut inputs = HashMap::new();
    let mut outputs = HashMap::new();

    let mut iter_attr = Into::<proc_macro2::TokenStream>::into(attr).into_iter();
    while let Some(tt) = iter_attr.next() {
        if let TokenTree::Ident(id) = tt {
            let qualif = id.to_string();
            match qualif.as_str() {
                "default" => {
                    let (param, default_val) = config_default(&mut iter_attr);
                    defaults.insert(param, default_val);
                }
                "model" => {
                    let (name, ident) = config_model(&mut iter_attr);
                    models.insert(name, (ident, None));
                }
                "source" => {
                    let (name, ident, source) = config_source(&mut iter_attr);
                    models.insert(name, (ident, Some(source)));
                }
                "input" => {
                    let (name, flow, ty) =
                        config_io(&mut iter_attr).expect("Name identity expected");
                    inputs.insert(name, (flow, ty));
                }
                "output" => {
                    let (name, flow, ty) =
                        config_io(&mut iter_attr).expect("Name identity expected");
                    outputs.insert(name, (flow, ty));
                }
                _ => panic!("Unrecognized configuration"),
            }
        }
    }

    let treatment: ItemFn = parse(item).unwrap();
    if treatment.sig.asyncness.is_none() {
        panic!("Treatments must be async");
    }
    let mut documentation = Vec::new();
    for attr in treatment.attrs.clone() {
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

    let name = treatment.sig.ident.to_string();
    let mut params = HashMap::new();
    for arg in &treatment.sig.inputs {
        match arg {
            FnArg::Typed(t) => {
                let name = if let Pat::Ident(ident) = t.pat.borrow() {
                    ident.ident.to_string()
                } else {
                    eprintln!("Argument name expected");
                    break;
                };

                let ty = into_mel_type(t.ty.borrow());

                params.insert(name, ty);
            }
            _ => eprintln!("Only Mélodium types are admissible arguments"),
        }
    }

    let description;
    {
        let documentation = documentation.join("\n");
        let parameters: proc_macro2::TokenStream = params.iter().map(|(name, ty)| {
            let datatype = into_mel_datatype(ty);
            let default = defaults.get(name).map(|lit| format!("Some(melodium_core::common::executive::Value::{ty}({val}))", val = into_rust_value(ty, lit))).unwrap_or_else(|| String::from("None"));
            format!(
                r#"melodium_core::common::descriptor::Parameter::new("{name}", melodium_core::common::descriptor::Variability::Var, {datatype}, {default})"#
            )
        }).collect::<Vec<_>>().join(",").parse().unwrap();
        let inputs: proc_macro2::TokenStream = inputs.iter().map(|(name, (flow, ty))| {
            let datatype = into_mel_datatype(ty);
            format!(r#"melodium_core::common::descriptor::Input::new("{name}", {datatype}, melodium_core::common::descriptor::Flow::{flow})"#)
        }).collect::<Vec<_>>().join(",").parse().unwrap();
        let outputs: proc_macro2::TokenStream = outputs.iter().map(|(name, (flow, ty))| {
            let datatype = into_mel_datatype(ty);
            format!(r#"melodium_core::common::descriptor::Output::new("{name}", {datatype}, melodium_core::common::descriptor::Flow::{flow})"#)
        }).collect::<Vec<_>>().join(",").parse().unwrap();
        let sources: proc_macro2::TokenStream = models
            .iter()
            .filter(|(_, (_, source))| source.is_some())
            .map(|(name, (_, source))| {
                format!(
                    r#"("{name}".to_string(), vec!["{}".to_string()])"#,
                    source.as_ref().unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
            .parse()
            .unwrap();
        let models: proc_macro2::TokenStream = models
            .iter()
            .map(|(name, (ident, _))| format!(r#"("{name}".to_string(), {ident}::descriptor())"#))
            .collect::<Vec<_>>()
            .join(",")
            .parse()
            .unwrap();

        let element_name = name.to_case(Case::Camel);

        description = quote! {

            static DESCRIPTOR: std::sync::Mutex<Option<std::sync::Arc<melodium_core::descriptor::Treatment>>> = std::sync::Mutex::new(None);

            pub fn identifier() -> melodium_core::common::descriptor::Identifier {
                melodium_core::descriptor::module_path_to_identifier(module_path!(), #element_name)
            }

            pub fn descriptor() -> std::sync::Arc<melodium_core::descriptor::Treatment> {
                let mut desc = DESCRIPTOR.lock().unwrap();
                if let Some(desc) = &*desc {
                    std::sync::Arc::clone(&desc)
                }
                else {

                    let new = melodium_core::descriptor::Treatment::new(
                        identifier(),
                        #documentation.to_string(),
                        vec![#models],
                        vec![#sources],
                        vec![#parameters],
                        vec![#inputs],
                        vec![#outputs],
                        AdHocTreatment::new
                    );

                    *desc = Some(std::sync::Arc::clone(&new));

                    new
                }
            }
        };
    }

    let declaration;
    {
        let parameters: proc_macro2::TokenStream = params
            .iter()
            .map(|(name, ty)| {
                let rust_type = into_rust_type(ty);
                format!(r#"r#{name}: std::sync::Mutex<Option<{rust_type}>>,"#)
            })
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();
        let inputs: proc_macro2::TokenStream = inputs.iter().map(|(name, _)| {
            format!(r#"r#{name}: std::sync::Mutex<Option<Box<dyn melodium_core::common::executive::Input>>>,"#)
        }).collect::<Vec<_>>().join("").parse().unwrap();
        let outputs: proc_macro2::TokenStream = outputs.iter().map(|(name, _)| {
            format!(r#"r#{name}: std::sync::Mutex<Option<Box<dyn melodium_core::common::executive::Output>>>,"#)
        }).collect::<Vec<_>>().join("").parse().unwrap();
        let models: proc_macro2::TokenStream = models.iter().map(|(name, _)| {
            format!(r#"r#{name}: std::sync::Mutex<Option<std::sync::Arc<dyn melodium_core::common::executive::Model>>>,"#)
        }).collect::<Vec<_>>().join("").parse().unwrap();

        declaration = quote! {
            #[derive(Debug)]
            pub struct AdHocTreatment {
                #models
                #inputs
                #outputs
                #parameters
            }
        };
    }

    let self_implementation;
    {
        let parameters: proc_macro2::TokenStream = params
            .iter()
            .map(|(name, ty)| {
                let default = defaults
                    .get(name)
                    .map(|lit| format!("Some({val})", val = into_rust_value(ty, lit)))
                    .unwrap_or_else(|| String::from("None"));
                format!(r#"r#{name}: std::sync::Mutex::new({default}),"#)
            })
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();
        let inputs: proc_macro2::TokenStream = inputs
            .iter()
            .map(|(name, _)| format!(r#"r#{name}: std::sync::Mutex::new(None),"#))
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();
        let outputs: proc_macro2::TokenStream = outputs
            .iter()
            .map(|(name, _)| format!(r#"r#{name}: std::sync::Mutex::new(None),"#))
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();
        let models: proc_macro2::TokenStream = models
            .iter()
            .map(|(name, _)| format!(r#"r#{name}: std::sync::Mutex::new(None),"#))
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();

        self_implementation = quote! {
            impl AdHocTreatment {
                pub fn new() -> std::sync::Arc<dyn melodium_core::common::executive::Treatment> {
                    std::sync::Arc::new(Self {
                        #parameters
                        #inputs
                        #outputs
                        #models
                    })
                }
            }
        };
    }

    let trait_implementation;
    {
        let parameters: proc_macro2::TokenStream = params
            .iter()
            .map(|(name, ty)| {
                let call = into_mel_value_call(ty);
                format!(r#""{name}" => *self.r#{name}.lock().unwrap() = Some(value.{call}()),"#)
            })
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();
        let inputs: proc_macro2::TokenStream = inputs
            .iter()
            .map(|(name, _)| {
                format!(r#""{name}" => *self.r#{name}.lock().unwrap() = Some(transmitter),"#)
            })
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();
        let outputs: proc_macro2::TokenStream = outputs
            .iter()
            .map(|(name, _)| {
                format!(r#""{name}" => *self.r#{name}.lock().unwrap() = Some(transmitter),"#)
            })
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();
        let models: proc_macro2::TokenStream = models
            .iter()
            .map(|(name, _)| {
                format!(r#""{name}" => *self.r#{name}.lock().unwrap() = Some(model),"#)
            })
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();

        trait_implementation = quote! {
            fn descriptor(&self) -> std::sync::Arc<dyn melodium_core::common::descriptor::Treatment> {
                descriptor()
            }

            fn set_parameter(&self, param: &str, value: melodium_core::common::executive::Value) {
                match param {
                    #parameters
                    _ => {},
                }
            }

            fn set_model(&self, name: &str, model: std::sync::Arc<dyn melodium_core::common::executive::Model>) {
                match name {
                    #models
                    _ => {},
                }
            }

            fn assign_input(&self, input_name: &str, transmitter: Box<dyn melodium_core::common::executive::Input>) {
                match input_name {
                    #inputs
                    _ => {},
                }
            }

            fn assign_output(&self, output_name: &str, transmitter: Box<dyn melodium_core::common::executive::Output>) {
                match output_name {
                    #outputs
                    _ => {},
                }
            }
        };
    }

    let prepare_implementation;
    {
        let parameters: proc_macro2::TokenStream = params.iter().map(|(name, _)| {
            format!(r#"let {name} = std::mem::replace(&mut *self.r#{name}.lock().unwrap(), None).unwrap()"#)
        }).collect::<Vec<_>>().join(";").parse().unwrap();
        let pre_inputs: proc_macro2::TokenStream = inputs.iter().map(|(name, _)| {
            format!(r#"let {name} = std::mem::replace(&mut *self.r#{name}.lock().unwrap(), None).unwrap()"#)
        }).collect::<Vec<_>>().join(";").parse().unwrap();
        let post_inputs: proc_macro2::TokenStream = inputs
            .iter()
            .map(|(name, _)| format!(r#"{name}.close()"#))
            .collect::<Vec<_>>()
            .join(";")
            .parse()
            .unwrap();
        let pre_outputs: proc_macro2::TokenStream = outputs.iter().map(|(name, _)| {
            format!(r#"let {name} = std::mem::replace(&mut *self.r#{name}.lock().unwrap(), None).unwrap()"#)
        }).collect::<Vec<_>>().join(";").parse().unwrap();
        let post_outputs: proc_macro2::TokenStream = outputs
            .iter()
            .map(|(name, _)| format!(r#"{name}.close().await"#))
            .collect::<Vec<_>>()
            .join(";")
            .parse()
            .unwrap();
        let models: proc_macro2::TokenStream = models.iter().map(|(name, _)| {
            format!(r#"let {name} = std::mem::replace(&mut *self.r#{name}.lock().unwrap(), None).unwrap()"#)
        }).collect::<Vec<_>>().join(";").parse().unwrap();

        let body = treatment.block;

        prepare_implementation = quote! {
            fn prepare(&self) -> Vec<melodium_core::common::executive::TrackFuture> {

                #parameters;
                #models;
                #pre_inputs;
                #pre_outputs;

                vec![Box::new(Box::pin(async move {

                    #body

                    #post_inputs;
                    #post_outputs;

                    melodium_core::common::executive::ResultStatus::Ok
                }))]
            }
        };
    }

    let module_name: proc_macro2::TokenStream = format!("__mel_treatment_{name}").parse().unwrap();

    let expanded = quote! {
        pub mod #module_name {
            use super::*;

            #description

            #declaration

            #self_implementation

            impl melodium_core::common::executive::Treatment for AdHocTreatment {
                #trait_implementation
                #prepare_implementation
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn mel_model(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut params = HashMap::new();
    let mut sources = HashMap::new();
    let mut initialization = None;
    let mut continuous = Vec::new();
    let mut shutdown = None;

    let mut iter_attr = Into::<proc_macro2::TokenStream>::into(attr).into_iter();
    while let Some(tt) = iter_attr.next() {
        if let TokenTree::Ident(id) = tt {
            let qualif = id.to_string();
            match qualif.as_str() {
                "param" => {
                    let (param, ty, default_val) = config_param(&mut iter_attr);
                    params.insert(param, (ty, default_val));
                }
                "source" => {
                    let (name, contexts, outputs) = config_full_source(&mut iter_attr);
                    sources.insert(name, (contexts, outputs));
                }
                "initialize" => {
                    if let Some(TokenTree::Ident(name)) = iter_attr.next() {
                        initialization = Some(name.to_string());
                    } else {
                        panic!("Initialize function name expected")
                    }
                }
                "continuous" => {
                    if let Some(TokenTree::Group(group)) = iter_attr.next() {
                        for tt in group.stream() {
                            if let TokenTree::Ident(c) = tt {
                                continuous.push(c.to_string())
                            } else {
                                panic!("Function identity expected")
                            }
                        }
                    } else {
                        panic!("Continuous list expected")
                    }
                }
                "shutdown" => {
                    if let Some(TokenTree::Ident(name)) = iter_attr.next() {
                        shutdown = Some(name.to_string());
                    } else {
                        panic!("Shutdown function name expected")
                    }
                }
                _ => panic!("Unrecognized configuration"),
            }
        }
    }

    let model: ItemStruct = parse(item).unwrap();
    let mut documentation = Vec::new();
    for attr in model.attrs.clone() {
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
    let name = model.ident.to_string();

    let model_description;
    {
        let documentation = documentation.join("\n");
        let parameters: proc_macro2::TokenStream = params.iter().map(|(name, (ty, default))| {
            let datatype = into_mel_datatype(ty);
            let default = default.as_ref().map(|lit| format!("Some(melodium_core::common::executive::Value::{ty}({val}))", val = into_rust_value(ty, lit))).unwrap_or_else(|| String::from("None"));
            format!(
                r#"melodium_core::common::descriptor::Parameter::new("{name}", melodium_core::common::descriptor::Variability::Const, {datatype}, {default})"#
            )
        }).collect::<Vec<_>>().join(",").parse().unwrap();
        let sources: proc_macro2::TokenStream = sources
            .iter()
            .map(|(name, (contextes, _))| {
                let contextes = contextes
                    .iter()
                    .map(|name| format!(r#"{name}::descriptor()"#))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(r#"("{name}".to_string(), vec![{contextes}])"#)
            })
            .collect::<Vec<_>>()
            .join(",")
            .parse()
            .unwrap();

        model_description = quote! {
            let model = melodium_core::descriptor::Model::new(
                    melodium_core::descriptor::module_path_to_identifier(module_path!(), #name),
                    #documentation.to_string(),
                    vec![#parameters],
                    vec![#sources],
                    AdHocModel::new,
                );
        };
    }

    let mut sources_description = proc_macro2::TokenStream::new();
    {
        let fancy_model_name = name.to_case(Case::Snake);
        for (source_name, (_, outputs)) in &sources {
            let outputs: proc_macro2::TokenStream = outputs.iter().map(|(name, flow, ty)| {
                let datatype = into_mel_datatype(ty);
                format!(r#"melodium_core::common::descriptor::Output::new("{name}", {datatype}, melodium_core::common::descriptor::Flow::{flow})"#)
            }).collect::<Vec<_>>().join(",").parse().unwrap();

            sources_description = quote! {
                #sources_description
                melodium_core::descriptor::Source::new(
                    melodium_core::descriptor::module_path_to_identifier(module_path!(), #source_name),
                    "".to_string(),
                    vec![(#fancy_model_name.to_string(), std::sync::Arc::clone(&model) as std::sync::Arc<dyn melodium_core::common::descriptor::Model>)],
                    vec![(#fancy_model_name.to_string(), vec![#source_name.to_string()])],
                    vec![#outputs],
                ),
            };
        }
    }

    let mut helper_implementation: proc_macro2::TokenStream;
    {
        let parameters: proc_macro2::TokenStream = params
            .iter()
            .map(|(name, (ty, _))| {
                let rust_type = into_rust_type(ty);
                let call = into_mel_value_call(ty);
                format!(
                    r#"
                pub fn get_{name}(&self) -> {rust_type} {{
                    self.parameter("{name}").unwrap().{call}()
                }}
            "#
                )
            })
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();

        helper_implementation = parameters;

        for (source_name, (contextes, _)) in sources {
            let mut param_contextes = String::new();
            let mut assign_contextes = String::new();
            for context in &contextes {
                let mut path = context
                    .split("::")
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                let name = path.pop().unwrap().split("_").last().unwrap().to_string();
                let fancy_name = name.to_case(Case::Snake);
                path.push(name);

                param_contextes = format!("{param_contextes} {fancy_name}: {},", path.join("::"));
                assign_contextes = format!("{assign_contextes} Box::new({fancy_name}),");
            }

            let param_contextes: proc_macro2::TokenStream = param_contextes.parse().unwrap();
            let assign_contextes: proc_macro2::TokenStream = assign_contextes.parse().unwrap();
            let fn_name: proc_macro2::TokenStream = format!("new_{source_name}").parse().unwrap();

            helper_implementation = quote! {
                #helper_implementation
                pub async fn #fn_name(&self,
                        parent_track: Option<melodium_core::common::executive::TrackId>,
                        #param_contextes
                        callback: Option<Box<dyn FnOnce(std::collections::HashMap<String, Box<dyn melodium_core::common::executive::Output>>) -> Vec<melodium_core::common::executive::TrackFuture> + Send>>
                    ) {
                    self.world.create_track(
                        self.id().unwrap(),
                        #source_name,
                        vec![#assign_contextes],
                        parent_track,
                        callback,
                    ).await
                }
            };
        }
    }

    let element_name = name.to_case(Case::UpperCamel);
    let module_name: proc_macro2::TokenStream = format!("__mel_model_{name}").parse().unwrap();
    let model_name: proc_macro2::TokenStream = name.parse().unwrap();
    let adhoc_model_name: proc_macro2::TokenStream = format!("{name}Model").parse().unwrap();
    let initialize: proc_macro2::TokenStream = initialization
        .map(|s| format!("self.model.{s}()"))
        .unwrap_or_else(|| String::from("()"))
        .parse()
        .unwrap();
    let continuous: proc_macro2::TokenStream = continuous.iter().map(|c| format!("let auto_self = self.auto_reference.upgrade().unwrap(); self.world.add_continuous_task(Box::new(Box::pin(async move {{ auto_self.inner().{c}().await }})));")).collect::<Vec<_>>().join("").parse()
    .unwrap();
    let shutdown: proc_macro2::TokenStream = shutdown
        .map(|s| format!("self.model.{s}()"))
        .unwrap_or_else(|| String::from("()"))
        .parse()
        .unwrap();

    let expanded = quote! {
        pub mod #module_name {

            use super::*;

            static DESCRIPTOR: std::sync::Mutex<Option<std::sync::Arc<melodium_core::descriptor::Model>>> = std::sync::Mutex::new(None);
            static SOURCES: std::sync::Mutex<Option<Vec<std::sync::Arc<melodium_core::descriptor::Source>>>> = std::sync::Mutex::new(None);

            pub fn identifier() -> melodium_core::common::descriptor::Identifier {
                melodium_core::descriptor::module_path_to_identifier(module_path!(), #element_name)
            }

            pub fn descriptor() -> std::sync::Arc<melodium_core::descriptor::Model> {

                let mut desc = DESCRIPTOR.lock().unwrap();
                if let Some(desc) = &*desc {
                    std::sync::Arc::clone(&desc)
                }
                else {
                    #model_description

                    *desc = Some(std::sync::Arc::clone(&model));
                    model
                }
            }

            pub fn sources() -> Vec<std::sync::Arc<melodium_core::descriptor::Source>> {
                let mut desc = SOURCES.lock().unwrap();
                if let Some(sources) = &*desc {
                    sources.iter().map(|s| std::sync::Arc::clone(&s)).collect::<Vec<_>>()
                }
                else {
                    let model = descriptor();
                    let sources = vec![#sources_description];

                    *desc = Some(sources.iter().map(|s| std::sync::Arc::clone(&s)).collect::<Vec<_>>());
                    sources
                }
            }

            #[derive(Debug)]
            pub struct AdHocModel {
                id: std::sync::Mutex<Option<melodium_core::common::executive::ModelId>>,
                params: std::sync::Mutex<std::collections::HashMap<String, melodium_core::common::executive::Value>>,
                model: #model_name,
                world: std::sync::Arc<dyn melodium_core::common::executive::World>,
                auto_reference: std::sync::Weak<Self>,
            }

            impl AdHocModel {

                pub fn new(world: std::sync::Arc<dyn melodium_core::common::executive::World>) -> std::sync::Arc<dyn melodium_core::common::executive::Model> {
                    std::sync::Arc::new_cyclic(|me| Self {
                        id: std::sync::Mutex::new(None),
                        params: std::sync::Mutex::new(std::collections::HashMap::new()),
                        model: #model_name::new(me.clone()),
                        world,
                        auto_reference: me.clone(),
                    })
                }

                pub fn into(model: std::sync::Arc<dyn melodium_core::common::executive::Model>) -> std::sync::Arc<Self> {
                    model.downcast_arc::<Self>().unwrap()
                }

                pub fn inner(&self) -> &#model_name {
                    &self.model
                }

                pub fn id(&self) -> Option<melodium_core::common::executive::ModelId> {
                    *self.id.lock().unwrap()
                }

                pub fn set_id(&self, id: melodium_core::common::executive::ModelId) {
                    *self.id.lock().unwrap() = Some(id);
                }

                pub fn parameter(&self, name: &str) -> Option<melodium_core::common::executive::Value> {
                    self.params.lock().unwrap().get(name).cloned()
                }

                pub fn set_parameter(&self, param: &str, value: melodium_core::common::executive::Value) {
                    self.params.lock().unwrap().insert(param.to_string(), value);
                }

                #helper_implementation
            }

            impl melodium_core::common::executive::Model for AdHocModel {
                fn descriptor(&self) -> std::sync::Arc<dyn melodium_core::common::descriptor::Model> {
                    descriptor()
                }

                fn id(&self) -> Option<melodium_core::common::executive::ModelId> {
                    Self::id(self)
                }

                fn set_id(&self, id: melodium_core::common::executive::ModelId) {
                    Self::set_id(self, id)
                }

                fn set_parameter(&self, param: &str, value: melodium_core::common::executive::Value) {
                    Self::set_parameter(self, param, value)
                }

                fn initialize(&self) {
                    #initialize;
                    #continuous
                }

                fn shutdown(&self) {
                    #shutdown
                }
            }
        }
        use #module_name::AdHocModel as #adhoc_model_name;

        #model
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn mel_context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let context: ItemStruct = parse(item).unwrap();

    let mut documentation = Vec::new();
    for attr in context.attrs.clone() {
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

    let name = context.ident.to_string();
    let mut fields = HashMap::new();
    if let Fields::Named(fields_named) = &context.fields {
        for field in &fields_named.named {
            if let Some(field_ident) = &field.ident {
                let ty = into_mel_type(&field.ty);
                fields.insert(field_ident.to_string(), ty);
            } else {
                panic!("Field identity expected")
            }
        }
    } else {
        panic!("Named field expected")
    }

    let description;
    {
        let documentation = documentation.join("\n");
        let fields: proc_macro2::TokenStream = fields
            .iter()
            .map(|(name, ty)| {
                let datatype = into_mel_datatype(ty);
                format!(r#"("{name}", {datatype})"#)
            })
            .collect::<Vec<_>>()
            .join(",")
            .parse()
            .unwrap();

        description = quote! {
            melodium_core::descriptor::Context::new(
                    identifier(),
                    vec![#fields],
                    #documentation.to_string(),
                )
        };
    }

    let implementation;
    {
        let get: proc_macro2::TokenStream = fields.iter().map(|(name, ty)| {
            format!(
                r#""{name}" => melodium_core::common::executive::Value::{ty}(self.{name}.clone())"#
            )
        }).collect::<Vec<_>>().join(",").parse().unwrap();

        let set: proc_macro2::TokenStream = fields
            .iter()
            .map(|(name, ty)| {
                let call = into_mel_value_call(ty);
                format!(r#""{name}" => {{self.{name} = value.{call}();}}"#)
            })
            .collect::<Vec<_>>()
            .join(",")
            .parse()
            .unwrap();

        implementation = quote! {
            fn set_value(&mut self, name: &str, value: melodium_core::common::executive::Value) {
                match name {
                    #set,
                    _ => {}
                }
            }

            fn get_value(&self, name: &str) -> melodium_core::common::executive::Value {
                match name {
                    #get,
                    _ => panic!("Unexisting field")
                }
            }
        };
    }

    let element_name = format!("@{}", name.to_case(Case::UpperCamel));
    let name: proc_macro2::TokenStream = name.parse().unwrap();
    let module_name: proc_macro2::TokenStream = format!("__mel_context_{name}").parse().unwrap();
    let expanded = quote! {
        pub mod #module_name {
            use super::*;

            static DESCRIPTOR: std::sync::Mutex<Option<std::sync::Arc<melodium_core::descriptor::Context>>> = std::sync::Mutex::new(None);

            pub fn identifier() -> melodium_core::common::descriptor::Identifier {
                melodium_core::descriptor::module_path_to_identifier(module_path!(), #element_name)
            }

            pub fn descriptor() -> std::sync::Arc<melodium_core::descriptor::Context> {
                let mut desc = DESCRIPTOR.lock().unwrap();
                if let Some(desc) = &*desc {
                    std::sync::Arc::clone(&desc)
                }
                else {
                    let new = #description;
                    *desc = Some(std::sync::Arc::clone(&new));
                    new
                }
            }
        }

        #[derive(Debug)]
        #context

        impl melodium_core::common::executive::Context for #name {
            fn descriptor(&self) -> std::sync::Arc<dyn melodium_core::common::descriptor::Context> {
                #module_name::descriptor()
            }

            #implementation
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn mel_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function: ItemFn = parse(item).unwrap();
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
        let name = name.to_case(Case::Snake);
        let datatype = into_mel_datatype(ty);
        format!(
            r#"melodium_core::common::descriptor::Parameter::new("{name}", melodium_core::common::descriptor::Variability::Var, {datatype}, None)"#
        )
    }).collect::<Vec<_>>().join(",");
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

    let element_name = format!("|{}", name.from_case(Case::Snake).to_case(Case::Snake));
    let module_name: proc_macro2::TokenStream = format!("__mel_function_{name}").parse().unwrap();
    let documentation = documentation.join("\n");
    let parameters: proc_macro2::TokenStream = parameters.parse().unwrap();
    let return_type: proc_macro2::TokenStream = into_mel_datatype(&return_type).parse().unwrap();
    let mel_call: proc_macro2::TokenStream = mel_call.parse().unwrap();

    let expanded = quote! {
        #function

        pub mod #module_name {

            use super::*;

            static DESCRIPTOR: std::sync::Mutex<Option<std::sync::Arc<melodium_core::descriptor::Function>>> = std::sync::Mutex::new(None);

            pub fn identifier() -> melodium_core::common::descriptor::Identifier {
                melodium_core::descriptor::module_path_to_identifier(module_path!(), #element_name)
            }

            pub fn descriptor() -> std::sync::Arc<melodium_core::descriptor::Function> {
                let mut desc = DESCRIPTOR.lock().unwrap();
                if let Some(desc) = &*desc {
                    std::sync::Arc::clone(&desc)
                }
                else {
                    let new = melodium_core::descriptor::Function::new(
                        identifier(),
                        #documentation.to_string(),
                        vec![#parameters],
                        #return_type,
                        mel_function
                    );
                    *desc = Some(std::sync::Arc::clone(&new));
                    new
                }
            }

            fn mel_function(params: Vec<melodium_core::common::executive::Value>) -> melodium_core::common::executive::Value {
                #mel_call
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn check(item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = proc_macro2::TokenStream::from(item);

    let mut iter = item.clone().into_iter();
    if let Some(TokenTree::Punct(punct)) = iter.next() {
        if punct.as_char() == '\'' {
            let label = if let Some(TokenTree::Ident(label)) = iter.next() {
                label
            } else {
                panic!("Label expected")
            };

            let label: proc_macro2::TokenStream =
                format!("'{}", label.to_string()).parse().unwrap();

            // Discarding ','
            let _ = iter.next();

            let expr = proc_macro2::TokenStream::from_iter(iter);

            let expanded = quote! {
                if let Err(_) = {#expr} {
                    break #label;
                }
            };

            return TokenStream::from(expanded);
        }
    }

    let expanded = quote! {
        if let Err(_) = {#item} {
            break;
        }
    };

    TokenStream::from(expanded)
}
