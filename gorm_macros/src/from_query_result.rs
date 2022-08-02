use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{quote_spanned, quote};
use syn::{parse_macro_input, DeriveInput, Type, spanned::Spanned};

use crate::util::generate_fields_cons_list_type;

pub fn from_query_result(input_tokens: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let input = FromQueryResultInput::from_derive_input(&derive_input).unwrap();
    let FromQueryResultInput {
        ident: struct_ident,
        generics,
        data,
    } = input;

    if !generics.params.is_empty() {
        return quote_spanned! {
            generics.span() => compile_error!("generics are not supported for parsing query results")
        }
        .into();
    }

    let fields = data.take_struct().unwrap();
    if !fields.style.is_struct() {
        return quote_spanned! {
            derive_input.span() => compile_error!("only named structs are supported for parsing query results")
        }
        .into();
    }

    let field_names = fields.iter().map(|field| &field.ident);
    let fields_type =
        generate_fields_cons_list_type(fields.iter().map(|field| (&field.ident, &field.ty)));

    quote!{
        #[automatically_derived]
        impl ::gorm::FromQueryResult for #struct_ident
        {
            type Fields = #fields_type;

            fn from_row(row: ::gorm::tokio_postgres::row::Row) -> ::gorm::Result<Self>{
                Ok(
                    Self{
                        #(
                            #field_names: row.try_get(stringify!(#field_names)).map_err(::gorm::Error::FailedToGetColumn)?
                         ),*
                    }
                )
            }
        }
    }.into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct FromQueryResultInput {
    ident: proc_macro2::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), FromQueryResultInputField>,
}

#[derive(Debug, FromField)]
struct FromQueryResultInputField {
    ident: Option<proc_macro2::Ident>,
    ty: Type,
}

