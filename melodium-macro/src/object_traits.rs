
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn object_traits<T: AsRef<str> + PartialEq<str>>(name: &str, traits: &Vec<T>) -> TokenStream {

    let to_i8 = to_i8(name, traits.iter().any(|t| t == "ToI8"));

    /*let mut impls = Vec::new();
    impls.push();

    let impls = impls.into_iter().map(|t| t.to_tokens()).collect::<Vec<_>>();*/

    quote! {

        impl melodium_core::common::executive::DataTrait for #name {
            #to_i8
        }

    }
}

fn to_i8(name: &str, implemented: bool) -> TokenStream {
    if implemented {
        quote! {
            fn to_i8(&self) -> i8 {
                melodium_core::executive::ToI8::to_i8(self)
            }
        }
    } else {
        quote! {
            fn to_i8(&self) -> i8 {
                panic!("ToI8 not implemented for {}", name)
            }
        }
    }
}
