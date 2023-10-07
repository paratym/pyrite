extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    parse_macro_input,
    token::Comma,
    DeriveInput,
    LitInt,
    Result,
};

fn get_calling_crate() -> String {
    return std::env::var("CARGO_PKG_NAME").unwrap();
}

fn app_mod_path() -> proc_macro2::TokenStream {
    if get_calling_crate().starts_with("pyrite_") {
        return quote! { pyrite_app };
    }
    return quote! { pyrite::app };
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    impl_derive_resource(&ast)
}

fn impl_derive_resource(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let app_mod_path = app_mod_path();

    let gen = quote! {
        impl #app_mod_path::resource::Resource for #name {}
    };

    gen.into()
}

struct GenerateSystemHandlersInput {
    macro_impl: Ident,
    count: usize,
}

impl Parse for GenerateSystemHandlersInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_impl = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let count = input.parse::<LitInt>()?.base10_parse()?;

        Ok(Self { macro_impl, count })
    }
}

#[proc_macro]
pub fn generate_system_function_handlers(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GenerateSystemHandlersInput);

    impl_generate_system_function_handlers(&input)
}

fn impl_generate_system_function_handlers(input: &GenerateSystemHandlersInput) -> TokenStream {
    let macro_impl = &input.macro_impl;
    let count = input.count;

    let mut generated = vec![quote! { #macro_impl!(); }];

    let mut generics = Vec::new();
    for i in 0..count {
        let name = Ident::new(&format!("P{}", i), proc_macro2::Span::call_site());
        generics.push(quote! { #name });

        generated.push(quote! { #macro_impl!(#(#generics),*); });
    }

    let gen = quote! {
        #(#generated)*
    };
    gen.into()
}
