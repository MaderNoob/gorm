use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_roids::DeriveInputStructExt;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Colon2, DeriveInput,
    Fields, Path, PathSegment, Type, TypePath, Visibility,
};

#[proc_macro_derive(Table)]
pub fn table(input_tokens: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let input = TableInput::from_derive_input(&derive_input).unwrap();
    let TableInput {
        ident,
        generics,
        data,
    } = input;

    if !generics.params.is_empty() {
        return quote_spanned! {
            generics.span() => compile_error!("generics are not supported on tables")
        }
        .into();
    }

    let fields = data.take_struct().unwrap();
    if !fields.style.is_struct() {
        return quote_spanned! {
            derive_input.span() => compile_error!("only named struct are supported for tables")
        }
        .into();
    }

    let field_infos = fields.iter().map(|field| field.field_info());

    return quote! {
        impl ::gorm::Table for #ident {
            fn fields() -> &'static [::gorm::FieldInfo] {
                &[
                    #(#field_infos),*
                ]
            }
        }
    }.into();
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(table), supports(struct_named))]
struct TableInput {
    ident: Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), TableInputField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(table))]
struct TableInputField {
    ident: Option<Ident>,
    ty: Type,
}
impl TableInputField {
    fn field_info(&self) -> proc_macro2::TokenStream {
        let name = self.ident.as_ref().unwrap();
        let ty = &self.ty;
        quote! {
            ::gorm::FieldInfo{
                name: stringify!(#name),
                ty: <#ty as ::gorm::IntoSqlType>::SQL_TYPE,
            }
        }
    }
}
