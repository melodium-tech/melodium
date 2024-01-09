#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use convert_case::{Case, Casing};
use core::{borrow::Borrow, convert::TryFrom, iter::FromIterator, slice::Iter};
use litrs::StringLit;
use proc_macro::TokenStream;
use proc_macro2::{token_stream::IntoIter as IntoIterTokenStream, TokenTree};
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse, parse_file, Fields, FnArg, GenericArgument, Item, ItemFn, ItemStruct, Pat,
    PathArguments, ReturnType, Type,
};

fn into_mel_type(ty: &Type) -> Vec<String> {
    match ty {
        Type::Path(path) => {
            let ty = path.path.segments.first().expect("Type expected");

            let text_ty = ty.ident.to_string();
            let mut desc = Vec::new();
            desc.push(text_ty.clone());
            match text_ty.as_str() {
                "Vec" | "Option" => {
                    if let PathArguments::AngleBracketed(ab) = &ty.arguments {
                        if let GenericArgument::Type(ty) = ab.args.first().expect("Type expected") {
                            desc.append(&mut into_mel_type(ty));
                        } else {
                            panic!("Type expected");
                        }
                    } else {
                        panic!("Type expected");
                    }
                }
                _ => {}
            }

            desc
        }
        _ => {
            panic!("Type expected");
        }
    }
}

fn into_mel_datatype(ty: &Vec<String>) -> String {
    fn write_datatype(iter: &mut Iter<String>) -> String {
        let mut desc = String::new();
        if let Some(ty) = iter.next() {
            desc.push_str("melodium_core::common::descriptor::DataType::");
            desc.push_str(&ty.to_case(Case::UpperCamel));

            let next = write_datatype(iter);
            if !next.is_empty() {
                desc.push_str("(Box::new(");
                desc.push_str(&next);
                desc.push_str("))");
            }
        }
        desc
    }

    write_datatype(&mut ty.iter())
}

fn into_mel_described_type(ty: &Vec<String>) -> String {
    fn write_described_type(iter: &mut Iter<String>) -> String {
        let mut desc = String::new();
        if let Some(ty) = iter.next() {
            match ty.as_str() {
                "byte" | "bool" | "void" | "char" | "string" | "f32" | "f64" | "u8" | "u16"
                | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" => {
                    desc.push_str("melodium_core::common::descriptor::DescribedType::");
                    desc.push_str(&ty.to_case(Case::UpperCamel));
                }
                "Vec" | "Option" => {
                    desc.push_str("melodium_core::common::descriptor::DescribedType::");
                    desc.push_str(ty.as_str());
                    desc.push_str("(Box::new(");
                    desc.push_str(&write_described_type(iter));
                    desc.push_str("))");
                }
                generic => {
                    desc.push_str(r#"melodium_core::common::descriptor::DescribedType::Generic(Box::new(melodium_core::common::descriptor::Generic::new(""#);
                    desc.push_str(generic);
                    desc.push_str(r#"".to_string(), Vec::new())))"#);
                }
            }
        }
        desc
    }

    write_described_type(&mut ty.iter())
}

fn into_rust_type(ty: &Vec<String>) -> String {
    fn add_type(iter: &mut Iter<String>) -> String {
        let mut desc = String::new();
        if let Some(ty) = iter.next() {
            desc.push_str(ty);

            let next = add_type(iter);
            if !next.is_empty() {
                desc.push('<');
                desc.push_str(&next);
                desc.push('>');
            }
        }
        desc
    }

    add_type(&mut ty.iter())
}

fn into_rust_value(ty: &Vec<String>, lit: &str) -> String {
    fn add_value(iter: &mut Iter<String>, lit: &str) -> String {
        let mut desc = String::new();
        if let Some(ty) = iter.next() {
            match ty.as_str() {
                "Vec" => {
                    desc.push_str("melodium_core::common::executive::Value::Vec(vec![");
                    let next = add_value(iter, lit);
                    if !next.is_empty() {
                        desc.push_str(&next);
                    }
                    desc.push_str("])");
                }
                "Option" => {
                    let next = add_value(iter, lit);
                    if !next.is_empty() {
                        desc.push_str(
                            "melodium_core::common::executive::Value::Option(Some(Box::new(",
                        );
                        desc.push_str(&next);
                        desc.push_str(")))");
                    } else {
                        desc.push_str(
                            "melodium_core::common::executive::Value::Option(Box::new(None))",
                        );
                    }
                }
                mel_ty => {
                    desc.push_str("melodium_core::common::executive::Value::");
                    desc.push_str(&mel_ty.to_case(Case::UpperCamel));
                    desc.push('(');
                    match mel_ty {
                        "byte" => {
                            desc.push_str(lit);
                            desc.push_str("u8");
                        }
                        "f32" => {
                            desc.push_str(lit);
                            desc.push_str("f32");
                        }
                        "f64" => {
                            desc.push_str(lit);
                            desc.push_str("f64");
                        }
                        "string" => {
                            desc.push_str(lit);
                            desc.push_str(".to_string()");
                        }
                        _ => desc.push_str(lit),
                    }
                    desc.push(')');
                }
            }
        }
        desc
    }

    add_value(&mut ty.iter(), lit)
}

fn into_mel_value_call(ty: &Vec<String>) -> String {
    format!(
        "melodium_core::common::executive::GetData::<{}>::try_data",
        into_rust_type(ty)
    )
}

fn config_default(ts: &mut IntoIterTokenStream) -> (String, String) {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        (name.to_string(), config_value(ts))
    } else {
        panic!("Name identity expected")
    }
}

fn config_param(
    ts: &mut IntoIterTokenStream,
) -> (String, Vec<String>, Option<String>, HashMap<String, String>) {
    let mut next = ts.next();
    let attributes;
    if let Some(TokenTree::Group(attrs)) = next {
        attributes = config_attributes(&mut attrs.stream().into_iter());
        next = ts.next();
    } else {
        attributes = HashMap::new();
    }

    if let Some(TokenTree::Ident(name)) = next {
        (
            name.to_string(),
            config_ty(ts),
            config_optionnal_value(ts),
            attributes,
        )
    } else {
        panic!(
            "Name identity expected, found: {}",
            next.unwrap().to_string()
        )
    }
}

fn config_full_source(
    ts: &mut IntoIterTokenStream,
) -> (
    String,
    Vec<String>,
    Vec<(String, String, Vec<String>, HashMap<String, String>)>,
    HashMap<String, String>,
) {
    let mut next = ts.next();
    let attributes;
    if let Some(TokenTree::Group(attrs)) = next {
        attributes = config_attributes(&mut attrs.stream().into_iter());
        next = ts.next();
    } else {
        attributes = HashMap::new();
    }
    if let Some(TokenTree::Ident(name)) = next {
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

        (name.to_string(), contextes, outputs, attributes)
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
        } else if ident.to_string() == "true" || ident.to_string() == "false" {
            Some(ident.to_string())
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

fn config_ty(mut ts: &mut IntoIterTokenStream) -> Vec<String> {
    let mut list = Vec::new();
    if let Some(TokenTree::Ident(ty)) = ts.next() {
        let ty = ty.to_string();
        list.push(ty.clone());
        match ty.as_str() {
            "Vec" | "Option" => {
                ts.next(); // <
                list.append(&mut config_ty(&mut ts));
                ts.next(); // >
            }
            _ => {}
        }
    } else {
        panic!("Type identity expected")
    }
    list
}

fn config_io(
    ts: &mut IntoIterTokenStream,
) -> Option<(String, String, Vec<String>, HashMap<String, String>)> {
    let mut next = ts.next();
    let attributes;
    if let Some(TokenTree::Group(attrs)) = next {
        attributes = config_attributes(&mut attrs.stream().into_iter());
        next = ts.next();
    } else {
        attributes = HashMap::new();
    }
    if let Some(TokenTree::Ident(name)) = next {
        if let Some(TokenTree::Ident(flow)) = ts.next() {
            ts.next(); // <

            let mel_ty = config_ty(ts);

            ts.next(); // >
            Some((name.to_string(), flow.to_string(), mel_ty, attributes))
        } else {
            panic!("Flow identity expected")
        }
    } else {
        None
    }
}

fn config_attributes(ts: &mut IntoIterTokenStream) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    while let Some(next) = ts.next() {
        if let TokenTree::Ident(name) = next {
            ts.next(); // (

            let mut attribute = String::new();
            while let Some(next) = ts.next() {
                if let TokenTree::Punct(punct) = &next {
                    if punct.as_char() == ')' {
                        break;
                    }
                }
                attribute.push_str(&next.to_string());
            }
            attributes.insert(name.to_string(), attribute);
        }
    }

    attributes
}

fn config_attribute(ts: &mut IntoIterTokenStream) -> (String, String) {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        ts.next(); // (

        let mut attribute = String::new();
        while let Some(next) = ts.next() {
            if let TokenTree::Punct(punct) = &next {
                if punct.as_char() == ')' {
                    break;
                }
            }
            attribute.push_str(&next.to_string());
        }
        (name.to_string(), attribute)
    } else {
        panic!("Name identity expected")
    }
}

fn config_generic(ts: &mut IntoIterTokenStream) -> (String, Vec<String>) {
    if let Some(TokenTree::Ident(name)) = ts.next() {
        let name = name.to_string();

        if let Some(TokenTree::Group(group)) = ts.next() {
            (
                name,
                group
                    .stream()
                    .into_iter()
                    .map(|tt| {
                        if let TokenTree::Ident(trait_name) = tt {
                            trait_name.to_string()
                        } else {
                            panic!("Expecting trait name")
                        }
                    })
                    .collect(),
            )
        } else {
            panic!("Trait list expected")
        }
    } else {
        panic!("Name identity expected")
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
        .iter()
        .filter_map(|(k, v)| {
            if let (Some(mel_pkg), Some(version)) = (k.strip_suffix("-mel"), v.as_table().and_then(|t| t.get("version").and_then(|v| v.as_str()))) {
                Some(format!(r#"melodium_core::common::descriptor::PackageRequirement{{package:"{mel_pkg}".to_string(),version_requirement:melodium_core::common::descriptor::VersionReq::parse("{version}").unwrap()}}"#))
            } else { None }
        })
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
    let mut attributes = HashMap::new();
    let mut generics = Vec::new();

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
                    let (name, flow, ty, attributes) =
                        config_io(&mut iter_attr).expect("Name identity expected");
                    inputs.insert(name, (flow, ty, attributes));
                }
                "output" => {
                    let (name, flow, ty, attributes) =
                        config_io(&mut iter_attr).expect("Name identity expected");
                    outputs.insert(name, (flow, ty, attributes));
                }
                "attribute" => {
                    let (name, value) = config_attribute(&mut iter_attr);
                    attributes.insert(name, value);
                }
                "generic" => {
                    let name = config_generic(&mut iter_attr);
                    generics.push(name);
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

                let attributes = t
                    .attrs
                    .iter()
                    .filter_map(|attr| {
                        if let Some(name) = attr.path.get_ident() {
                            if name.to_string() == "mel" {
                                Some(config_attributes(&mut attr.tokens.clone().into_iter()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .fold(HashMap::new(), |mut acc, attrs| {
                        for attr in attrs {
                            acc.insert(attr.0, attr.1);
                        }
                        acc
                    });

                let ty = into_mel_type(t.ty.borrow());

                params.insert(name, (ty, attributes));
            }
            _ => eprintln!("Only Mélodium types are admissible arguments"),
        }
    }

    let description;
    {
        let documentation = documentation.join("\n");
        let attributes: proc_macro2::TokenStream = attributes
            .iter()
            .map(|(name, value)| {
                format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)
            })
            .collect::<Vec<_>>()
            .join(";")
            .parse()
            .unwrap();
        let generics: proc_macro2::TokenStream = generics
            .iter()
            .map(|(name, traits)| format!(r#"melodium_core::common::descriptor::Generic::new("{name}".to_string(), vec![{}])"#, traits.iter().map(|tr| format!("melodium_core::common::descriptor::DataTrait::{tr}")).collect::<Vec<_>>().join(", ")))
            .collect::<Vec<_>>()
            .join(",")
            .parse()
            .unwrap();
        let parameters: proc_macro2::TokenStream = params.iter().map(|(name, (ty, attributes))| {
            let described_type = into_mel_described_type(ty);
            let default = defaults.get(name).map(|lit| format!("Some({val})", val = into_rust_value(ty, lit))).unwrap_or_else(|| String::from("None"));
            let attributes = attributes.iter().map(|(name, value)| format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)).collect::<Vec<_>>().join(";");
            format!(
                r#"melodium_core::common::descriptor::Parameter::new("{name}", melodium_core::common::descriptor::Variability::Var, {described_type}, {default}, {{let mut attrs = melodium_core::common::descriptor::Attributes::new();{attributes};attrs}})"#
            )
        }).collect::<Vec<_>>().join(",").parse().unwrap();
        let inputs: proc_macro2::TokenStream = inputs.iter().map(|(name, (flow, ty, attributes))| {
            let described_type = into_mel_described_type(ty);
            let attributes = attributes.iter().map(|(name, value)| format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)).collect::<Vec<_>>().join(";");
            format!(r#"melodium_core::common::descriptor::Input::new("{name}", {described_type}, melodium_core::common::descriptor::Flow::{flow}, {{let mut attrs = melodium_core::common::descriptor::Attributes::new();{attributes};attrs}})"#)
        }).collect::<Vec<_>>().join(",").parse().unwrap();
        let outputs: proc_macro2::TokenStream = outputs.iter().map(|(name, (flow, ty, attributes))| {
            let described_type = into_mel_described_type(ty);
            let attributes = attributes.iter().map(|(name, value)| format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)).collect::<Vec<_>>().join(";");
            format!(r#"melodium_core::common::descriptor::Output::new("{name}", {described_type}, melodium_core::common::descriptor::Flow::{flow}, {{let mut attrs = melodium_core::common::descriptor::Attributes::new();{attributes};attrs}})"#)
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
                        {
                            let mut attrs = melodium_core::common::descriptor::Attributes::new();
                            #attributes;
                            attrs
                        },
                        vec![#generics],
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
            .map(|(name, (ty, _))| {
                if generics.iter().any(|(gen, _)| gen == ty.last().unwrap()) {
                    format!(r#"r#{name}: std::sync::Mutex<Option<melodium_core::common::executive::Value>>,"#)
                } else {
                    let rust_type = into_rust_type(ty);
                    format!(r#"r#{name}: std::sync::Mutex<Option<{rust_type}>>,"#)
                }
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
            .map(|(name, (ty, _))| {
                let default = defaults
                    .get(name)
                    .map(|lit| {
                        if generics.iter().any(|(gen, _)| gen == ty.last().unwrap()) {
                            format!("Some({val})", val = into_rust_value(ty, lit))
                        } else {
                            format!(
                                "Some({call}({val}).unwrap())",
                                call = into_mel_value_call(ty),
                                val = into_rust_value(ty, lit)
                            )
                        }
                    })
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
            .map(|(name, (ty, _))| {
                if generics.iter().any(|(gen, _)| gen == ty.last().unwrap()) {
                    format!(r#""{name}" => *self.r#{name}.lock().unwrap() = Some(value),"#)
                } else {
                    let call = into_mel_value_call(ty);
                    format!(r#""{name}" => *self.r#{name}.lock().unwrap() = Some({call}(value).unwrap()),"#)
                }
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
    let mut attributes = HashMap::new();

    let mut iter_attr = Into::<proc_macro2::TokenStream>::into(attr).into_iter();
    while let Some(tt) = iter_attr.next() {
        if let TokenTree::Ident(id) = tt {
            let qualif = id.to_string();
            match qualif.as_str() {
                "param" => {
                    let (param, ty, default_val, attributes) = config_param(&mut iter_attr);
                    params.insert(param, (ty, default_val, attributes));
                }
                "source" => {
                    let (name, contexts, outputs, attributes) = config_full_source(&mut iter_attr);
                    sources.insert(name, (contexts, outputs, attributes));
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
                "attribute" => {
                    let (name, value) = config_attribute(&mut iter_attr);
                    attributes.insert(name, value);
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
        let attributes: proc_macro2::TokenStream = attributes
            .iter()
            .map(|(name, value)| {
                format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)
            })
            .collect::<Vec<_>>()
            .join(";")
            .parse()
            .unwrap();
        let parameters: proc_macro2::TokenStream = params.iter().map(|(name, (ty, default, attributes))| {
            let described_type = into_mel_described_type(ty);
            let default = default.as_ref().map(|lit| format!("Some({val})", val = into_rust_value(ty, lit))).unwrap_or_else(|| String::from("None"));
            let attributes = attributes.iter().map(|(name, value)| format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)).collect::<Vec<_>>().join(";");
            format!(
                r#"melodium_core::common::descriptor::Parameter::new("{name}", melodium_core::common::descriptor::Variability::Const, {described_type}, {default}, {{let mut attrs = melodium_core::common::descriptor::Attributes::new();{attributes};attrs}})"#
            )
        }).collect::<Vec<_>>().join(",").parse().unwrap();
        let sources: proc_macro2::TokenStream = sources
            .iter()
            .map(|(name, (contextes, _, _))| {
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
                    {
                        let mut attrs = melodium_core::common::descriptor::Attributes::new();
                        #attributes;
                        attrs
                    },
                    vec![#parameters],
                    vec![#sources],
                    AdHocModel::new,
                );
        };
    }

    let mut sources_description = proc_macro2::TokenStream::new();
    {
        let fancy_model_name = name.to_case(Case::Snake);
        for (source_name, (_, outputs, attributes)) in &sources {
            let attributes = attributes
                .iter()
                .map(|(name, value)| {
                    format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)
                })
                .collect::<Vec<_>>()
                .join(";");
            let outputs: proc_macro2::TokenStream = outputs.iter().map(|(name, flow, ty, attributes)| {
                let described_type = into_mel_described_type(ty);
                let attributes = attributes.iter().map(|(name, value)| format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)).collect::<Vec<_>>().join(";");
                format!(r#"melodium_core::common::descriptor::Output::new("{name}", {described_type}, melodium_core::common::descriptor::Flow::{flow}, {{let mut attrs = melodium_core::common::descriptor::Attributes::new();{attributes};attrs}})"#)
            }).collect::<Vec<_>>().join(",").parse().unwrap();

            sources_description = quote! {
                #sources_description
                melodium_core::descriptor::Source::new(
                    melodium_core::descriptor::module_path_to_identifier(module_path!(), #source_name),
                    "".to_string(),
                    {
                        let mut attrs = melodium_core::common::descriptor::Attributes::new();
                        #attributes;
                        attrs
                    },
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
            .map(|(name, (ty, _, _))| {
                let rust_type = into_rust_type(ty);
                let call = into_mel_value_call(ty);
                format!(
                    r#"
                pub fn get_{name}(&self) -> {rust_type} {{
                    {call}(self.parameter("{name}").unwrap()).unwrap()
                }}
            "#
                )
            })
            .collect::<Vec<_>>()
            .join("")
            .parse()
            .unwrap();

        helper_implementation = parameters;

        for (source_name, (contextes, _, _)) in sources {
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
                assign_contextes = format!("{assign_contextes} std::sync::Arc::new({fancy_name}),");
            }

            let param_contextes: proc_macro2::TokenStream = param_contextes.parse().unwrap();
            let assign_contextes: proc_macro2::TokenStream = assign_contextes.parse().unwrap();
            let fn_name: proc_macro2::TokenStream = format!("new_{source_name}").parse().unwrap();

            helper_implementation = quote! {
                #helper_implementation
                pub async fn #fn_name(&self,
                        parent_track: Option<melodium_core::common::executive::TrackId>,
                        #param_contextes
                        callback: Option<Box<dyn FnOnce(Box<melodium_core::common::executive::Outputs>) -> Vec<melodium_core::common::executive::TrackFuture> + Send>>
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

    /*let parameters_instanciation: proc_macro2::TokenStream = params.iter().map(|(name, (ty, default))| {
        let datatype = into_mel_datatype(ty);
        let default = default.as_ref().map(|lit| format!("Some(melodium_core::common::executive::Value::{ty}({val}))", val = into_rust_value(ty, lit))).unwrap_or_else(|| String::from("None"));
        format!(
            r#"melodium_core::common::descriptor::Parameter::new("{name}", melodium_core::common::descriptor::Variability::Const, {datatype}, {default})"#
        )
    }).collect::<Vec<_>>().join(",").parse().unwrap();*/

    let parameters_initialization: proc_macro2::TokenStream = params
        .iter()
        .filter_map(|(name, (ty, default, _))| {
            if let Some(default) = default {
                Some(format!(
                    r#"("{name}".to_string(), {val})"#,
                    val = into_rust_value(ty, default)
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(",")
        .parse()
        .unwrap();

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
                        params: std::sync::Mutex::new(vec![#parameters_initialization].into_iter().collect()),
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
pub fn mel_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    let context: ItemStruct = parse(item).unwrap();
    let mut attributes = HashMap::new();

    let mut iter_attr = Into::<proc_macro2::TokenStream>::into(attr).into_iter();
    while let Some(tt) = iter_attr.next() {
        if let TokenTree::Ident(id) = tt {
            let qualif = id.to_string();
            match qualif.as_str() {
                "attribute" => {
                    let (name, value) = config_attribute(&mut iter_attr);
                    attributes.insert(name, value);
                }
                _ => panic!("Unrecognized configuration"),
            }
        }
    }

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
        let attributes: proc_macro2::TokenStream = attributes
            .iter()
            .map(|(name, value)| {
                format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)
            })
            .collect::<Vec<_>>()
            .join(";")
            .parse()
            .unwrap();

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
                    {
                        let mut attrs = melodium_core::common::descriptor::Attributes::new();
                        #attributes;
                        attrs
                    },
                )
        };
    }

    let implementation;
    {
        let get: proc_macro2::TokenStream = fields.iter().map(|(name, _)| {
            format!(
                r#""{name}" => melodium_core::common::executive::Value::from(self.{name}.clone())"#
            )
        }).collect::<Vec<_>>().join(",").parse().unwrap();

        let set: proc_macro2::TokenStream = fields
            .iter()
            .map(|(name, ty)| {
                let call = into_mel_value_call(ty);
                format!(r#""{name}" => {{self.{name} = {call}(value).unwrap();}}"#)
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
pub fn mel_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function: ItemFn = parse(item).unwrap();
    let mut attributes = HashMap::new();
    let mut generics = Vec::new();

    let mut iter_attr = Into::<proc_macro2::TokenStream>::into(attr).into_iter();
    while let Some(tt) = iter_attr.next() {
        if let TokenTree::Ident(id) = tt {
            let qualif = id.to_string();
            match qualif.as_str() {
                "attribute" => {
                    let (name, value) = config_attribute(&mut iter_attr);
                    attributes.insert(name, value);
                }
                "generic" => {
                    let name = config_generic(&mut iter_attr);
                    generics.push(name);
                }
                _ => panic!("Unrecognized configuration"),
            }
        }
    }

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
    for arg in &mut function.sig.inputs {
        match arg {
            FnArg::Typed(t) => {
                let name = if let Pat::Ident(ident) = t.pat.borrow() {
                    ident.ident.to_string()
                } else {
                    eprintln!("Argument name expected");
                    break;
                };

                let attributes = t
                    .attrs
                    .iter()
                    .filter_map(|attr| {
                        if let Some(name) = attr.path.get_ident() {
                            if name.to_string() == "mel" {
                                Some(config_attributes(&mut attr.tokens.clone().into_iter()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .fold(HashMap::new(), |mut acc, attrs| {
                        for attr in attrs {
                            acc.insert(attr.0, attr.1);
                        }
                        acc
                    });
                t.attrs.retain(|attr| {
                    attr.path
                        .get_ident()
                        .map(|name| name.to_string() != "mel")
                        .unwrap_or(true)
                });

                let ty = into_mel_type(t.ty.borrow());

                args.push((name, (ty, attributes)));
            }
            _ => eprintln!("Only Mélodium types are admissible arguments"),
        }
    }

    let typedefs: proc_macro2::TokenStream = generics
        .iter()
        .map(|(name, _)| format!(r#"type {name} = melodium_core::common::executive::Value"#))
        .collect::<Vec<_>>()
        .join(";")
        .parse()
        .unwrap();
    let (return_type, is_return_type_generic) =
        if let ReturnType::Type(_, rt) = &function.sig.output {
            match rt.borrow() {
                Type::Path(path) => {
                    let ty = path.path.segments.first().expect("Type expected");
                    let ty = ty.ident.to_string();
                    if !generics.iter().any(|(gen, _)| gen == &ty) {
                        (into_mel_type(rt), false)
                    } else {
                        (vec![ty], true)
                    }
                }
                _ => panic!("Type expected"),
            }
        } else {
            panic!("Return type expected");
        };
    let params_call = args
        .iter()
        .enumerate()
        .map(|(i, (_, (ty, _)))| {
            if generics
                .iter()
                .find(|(gen, _)| gen == ty.last().unwrap())
                .is_some()
            {
                format!("params[{i}].clone()")
            } else {
                format!("{}(params[{i}].clone()).unwrap()", into_mel_value_call(ty))
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    let mel_call = if is_return_type_generic {
        format!("{name}({params_call})",)
    } else {
        format!("melodium_core::common::executive::Value::from({name}({params_call}))")
    };

    let attributes: proc_macro2::TokenStream = attributes
        .iter()
        .map(|(name, value)| {
            format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)
        })
        .collect::<Vec<_>>()
        .join(";")
        .parse()
        .unwrap();
    let generics: proc_macro2::TokenStream = generics
        .iter()
        .map(|(name, traits)| format!(r#"melodium_core::common::descriptor::Generic::new("{name}".to_string(), vec![{}])"#, traits.iter().map(|tr| format!("melodium_core::common::descriptor::DataTrait::{tr}")).collect::<Vec<_>>().join(", ")))
        .collect::<Vec<_>>()
        .join(",")
        .parse()
        .unwrap();
    let parameters = args.iter().map(|(name, (ty, attributes))| {
        let name = name.to_case(Case::Snake);
        let described_type = into_mel_described_type(ty);
        let attributes = attributes.iter().map(|(name, value)| format!(r#"attrs.insert("{name}".to_string(), "{value}".to_string())"#)).collect::<Vec<_>>().join(";");
        format!(
            r#"melodium_core::common::descriptor::Parameter::new("{name}", melodium_core::common::descriptor::Variability::Var, {described_type}, None, {{let mut attrs = melodium_core::common::descriptor::Attributes::new();{attributes};attrs}})"#
        )
    }).collect::<Vec<_>>().join(",");

    let element_name = format!("|{}", name.from_case(Case::Snake).to_case(Case::Snake));
    let module_name: proc_macro2::TokenStream = format!("__mel_function_{name}").parse().unwrap();
    let documentation = documentation.join("\n");
    let parameters: proc_macro2::TokenStream = parameters.parse().unwrap();
    let return_type: proc_macro2::TokenStream =
        into_mel_described_type(&return_type).parse().unwrap();
    let mel_call: proc_macro2::TokenStream = mel_call.parse().unwrap();

    let expanded = quote! {
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
                        {
                            let mut attrs = melodium_core::common::descriptor::Attributes::new();
                            #attributes;
                            attrs
                        },
                        vec![#generics],
                        vec![#parameters],
                        #return_type,
                        mel_function
                    );
                    *desc = Some(std::sync::Arc::clone(&new));
                    new
                }
            }

            fn mel_function(params: Vec<melodium_core::common::executive::Value>) -> melodium_core::common::executive::Value {

                #typedefs;

                #function

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
