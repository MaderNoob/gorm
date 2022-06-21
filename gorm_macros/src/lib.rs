use darling::{FromDeriveInput, FromField};
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Type};

#[proc_macro_derive(Table, attributes(table))]
pub fn table(input_tokens: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let input = TableInput::from_derive_input(&derive_input).unwrap();
    let TableInput {
        ident,
        generics,
        data,
        table_name,
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

    let field_names = fields.iter().map(|field| &field.ident);
    let field_types = fields.iter().map(|field| &field.ty).unique();
    let table_field_structs = fields
        .iter()
        .map(|field| field.generate_table_field_struct());
    let fields_type = generate_fields_cons_list_type(&fields);

    let table_name_ident = Ident::new(&table_name, proc_macro2::Span::call_site());

    let column_structs = fields.iter().map(|field| {
        // it is safe to unwrap here since only named fields are allowed
        generate_column_struct(field.ident.as_ref().unwrap(), &field.ty, &ident)
    });

    let table_marker = generate_table_marker(&ident);

    return quote! {
        #[automatically_derived]
        impl ::gorm::table::Table for #ident {
            type Fields = #fields_type;
            const FIELDS: &'static [::gorm::table::TableField] = &[
                #( #table_field_structs ),*
            ];
            const TABLE_NAME: &'static ::std::primitive::str = #table_name;
        }

        #[automatically_derived]
        impl<'a, R: ::gorm::sqlx::Row> ::gorm::sqlx::FromRow<'a, R> for #ident
        where
            &'a ::std::primitive::str: ::gorm::sqlx::ColumnIndex<R>,
            #(
                #field_types: ::gorm::sqlx::decode::Decode<'a, R::Database>,
                #field_types: ::gorm::sqlx::types::Type<R::Database>
            ),*
        {
            fn from_row(row: &'a R) -> ::gorm::sqlx::Result<Self> {
                ::std::result::Result::Ok(#ident {
                    #(
                        #field_names: row.try_get(stringify!(#field_names))?
                     ),*
                })
            }
        }

        #[allow(non_camel_case_types)]
        mod #table_name_ident {
            #(
                #column_structs
             )*

            #table_marker
        }
    }
    .into();
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(table), supports(struct_named))]
struct TableInput {
    ident: Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), TableInputField>,
    table_name: String,
}

#[derive(Debug, FromField)]
#[darling(attributes(table))]
struct TableInputField {
    ident: Option<Ident>,
    ty: Type,
    primary_key: darling::util::Flag,
}
impl TableInputField {
    fn generate_table_field_struct(&self) -> proc_macro2::TokenStream {
        // it is safe to unwrap here since only named fields are allowed.
        let name = self.ident.as_ref().unwrap();
        let is_primary_key = self.primary_key.is_present();
        let ty = &self.ty;
        let sql_type_name = if is_primary_key {
            quote! {
                <<#ty as ::gorm::types::IntoSqlSerialType>::SqlSerialType as ::gorm::types::SqlSerialType>::SQL_NAME
            }
        } else {
            quote! {
                <<#ty as ::gorm::types::IntoSqlType>::SqlType as ::gorm::types::SqlType>::SQL_NAME
            }
        };
        quote! {
            ::gorm::table::TableField {
                name: stringify!(#name),
                is_primary_key: #is_primary_key,
                sql_type_name: #sql_type_name
            }
        }
    }
}

fn generate_field_name_cons_list_type(field_name: &str) -> proc_macro2::TokenStream {
    // start with the inner most type and wrap it each time with each character.
    let mut cur = quote! { ::gorm::fields_list::TypedConsListNil };

    for chr in field_name.chars().rev() {
        cur = quote! {
            ::gorm::fields_list::FieldNameCharsConsListCons<#chr, #cur>
        };
    }

    cur
}

fn generate_fields_cons_list_type(
    fields: &darling::ast::Fields<TableInputField>,
) -> proc_macro2::TokenStream {
    // start with the inner most type and wrap it each time with each field.
    let mut cur = quote! { ::gorm::fields_list::TypedConsListNil };

    for field in fields.iter().rev() {
        // safe to unwrap here because only structs with named fields are allowed.
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_name_type = generate_field_name_cons_list_type(&field_name);
        let field_type = &field.ty;
        cur = quote! {
            ::gorm::fields_list::FieldsConsListCons<
                #field_name_type,
                #field_type,
                #cur
            >
        };
    }

    cur
}

fn generate_column_struct(column_name_ident: &Ident, column_type: &Type, table_struct_ident: &Ident) -> proc_macro2::TokenStream {
    let column_name = column_name_ident.to_string();
    quote! {
        pub struct #column_name_ident;
        impl ::gorm::table::Column for #column_name_ident {
            const COLUMN_NAME:&'static str = #column_name;
            type Table = super::#table_struct_ident;
            type SqlType = <#column_type as ::gorm::types::IntoSqlType>::SqlType;
            type RustType = #column_type;
        }
    }
}

fn generate_table_marker(table_struct_ident: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub struct table;
        impl ::gorm::table::TableMarker for table {
            type Table = super::#table_struct_ident;
        }
    }
}
