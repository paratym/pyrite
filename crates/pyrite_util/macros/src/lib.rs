extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::{self},
    Attribute, DataStruct, Fields, Result, Token, Visibility, WhereClause,
};

fn get_calling_crate() -> String {
    return std::env::var("CARGO_PKG_NAME").unwrap();
}

fn util_mod_path() -> proc_macro2::TokenStream {
    if get_calling_crate().starts_with("pyrite_") {
        return quote! { pyrite_util };
    }
    return quote! { pyrite::util };
}

struct DependableStruct {
    attrs: Vec<Attribute>,
    name: Ident,
    visibility: syn::Visibility,
    data: DataStruct,
}

impl Parse for DependableStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let outer_attrs = input.call(Attribute::parse_outer)?;
        let attrs = vec![outer_attrs].concat();

        let visibility = input.parse::<Visibility>()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            let struct_token = input.parse::<Token![struct]>()?;
            let name = input.parse::<Ident>()?;
            let (_where_clause, fields, semi) = data_struct(input)?;
            Ok(DependableStruct {
                name,
                visibility,
                attrs,
                data: DataStruct {
                    struct_token,
                    fields,
                    semi_token: semi,
                },
            })
        } else {
            Err(lookahead.error())
        }
    }
}

// Copied from syn::derive
fn data_struct(input: ParseStream) -> Result<(Option<WhereClause>, Fields, Option<Token![;]>)> {
    let mut lookahead = input.lookahead1();
    let mut where_clause = None;
    if lookahead.peek(Token![where]) {
        where_clause = Some(input.parse()?);
        lookahead = input.lookahead1();
    }

    if where_clause.is_none() && lookahead.peek(token::Paren) {
        let fields = input.parse()?;

        lookahead = input.lookahead1();
        if lookahead.peek(Token![where]) {
            where_clause = Some(input.parse()?);
            lookahead = input.lookahead1();
        }

        if lookahead.peek(Token![;]) {
            let semi = input.parse()?;
            Ok((where_clause, Fields::Unnamed(fields), Some(semi)))
        } else {
            Err(lookahead.error())
        }
    } else if lookahead.peek(token::Brace) {
        let fields = input.parse()?;
        Ok((where_clause, Fields::Named(fields), None))
    } else if lookahead.peek(Token![;]) {
        let semi = input.parse()?;
        Ok((where_clause, Fields::Unit, Some(semi)))
    } else {
        Err(lookahead.error())
    }
}

#[proc_macro_attribute]
pub fn dependable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DependableStruct);

    let util_mod_path = util_mod_path();

    let name = &ast.name;
    let visibility = ast.visibility;
    let inner_name = syn::Ident::new(&format!("{}Inner", ast.name), ast.name.span());
    let attrs = ast.attrs;
    let fields = ast.data.fields;

    let struct_definitions = quote! {
        #(#attrs)*
        #visibility struct #name {
            inner: std::sync::Arc<#inner_name>,
        }

        #visibility struct #inner_name #fields
    };

    let impl_definitions = quote! {
        impl #util_mod_path::Dependable for #name {
            type Dep = #inner_name;

            fn create_dep(&self) -> std::sync::Arc<Self::Dep> {
                self.inner.clone()
            }
        }

        impl std::ops::Deref for #name {
            type Target = #inner_name;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };

    let dep_type_name = syn::Ident::new(&format!("{}Dep", ast.name), ast.name.span());
    let ref_type_name = syn::Ident::new(&format!("{}Ref", ast.name), ast.name.span());
    let impl_types = quote! {
        #visibility type #dep_type_name = std::sync::Arc<#inner_name>;
        #visibility type #ref_type_name<'a> = &'a #inner_name;
    };

    let gen = quote! {
        #struct_definitions

        #impl_definitions

        #impl_types
    };

    gen.into()
}
