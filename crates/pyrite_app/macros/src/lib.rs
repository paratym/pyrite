extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::{parse_macro_input, DeriveInput, LitInt, Result};

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    impl_derive_resource(&ast)
}

fn impl_derive_resource(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let gen = quote! {
        impl pyrite::app::resource::Resource for #name {}
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
