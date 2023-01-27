// lib.rs
extern crate proc_macro2;
use std::collections::HashSet;

use proc_macro2::{TokenStream, Ident, Span};
use quote::{quote, ToTokens};
use syn::{DeriveInput, parse_macro_input, FieldsNamed, FieldsUnnamed, DataEnum, DataUnion, Attribute, Variant, Fields, spanned::Spanned};




// // using proc_macro_attribute to declare an attribute like procedural macro
// #[proc_macro_attribute]
// // _metadata is argument provided to macro call and _input is code to which attribute like macro attaches
// pub fn my_custom_attribute(_metadata: TokenStream, _input: TokenStream) -> TokenStream {
//     // returing a simple TokenStream for Struct
//     TokenStream::from(quote!{struct H{}})
// }

#[proc_macro_derive(ToStruct, attributes(FsmTrait))]
pub fn to_struct (input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed: Result<syn::ItemEnum, syn::Error> = syn::parse(input.into());

    let parsed = match parsed {
        Ok(parsed) => parsed,
        Err(e) => return e.to_compile_error().into(),
    };

    

    let mut str = "".to_owned();
    str.push_str(&parsed.to_token_stream().to_string());
    quote!(
        fn my_test() {
            #str
        }
    ).into()
}

#[proc_macro_attribute]
pub fn fsm_trait (attribute: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    
    let parsed: Result<syn::ItemEnum, syn::Error> = syn::parse(input);

    let parsed = match parsed {
        Ok(parsed) => parsed,
        Err(e) => return e.to_compile_error().into(),
    };

    let ident = &parsed.ident;
    let mut enum_def = parsed.to_token_stream();
    let attribute_2:proc_macro2::TokenStream = attribute.clone().into();
    let mut fn_create = proc_macro2::TokenStream::new();

    let mut fn_create_body = proc_macro2::TokenStream::new();
    for v in &parsed.variants{
        quote!(
            #ident::#v =>  {
                let result: Box<dyn Stateful<State, Context, Event>> = Box::new(#v{});
                return result;
            }
        ).to_tokens(&mut fn_create_body);     
    }


    quote!(impl FsmEnum<#attribute_2> for #ident{
        fn create(enum_value: &#ident) ->Box<dyn Stateful<#attribute_2>> {
            match enum_value {
                #fn_create_body
            }            
        }        
    }).to_tokens(&mut fn_create);
    for v in &parsed.variants {    
        quote! {
                struct #v{}                
            }.to_tokens(&mut enum_def);
            
    }
    fn_create.to_tokens(&mut enum_def);
    enum_def.into()
    
}


