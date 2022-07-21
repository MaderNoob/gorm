use convert_case::{Case, Casing};
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
        ident: table_struct_ident,
        generics,
        data,
        table_name: optional_table_name,
    } = input;

    let table_name =
        optional_table_name.unwrap_or_else(|| table_struct_ident.to_string().to_case(Case::Snake));

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

    // make sure there is a field named id
    if fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap().to_string() == "id")
        .is_none()
    {
        return quote_spanned! {
            derive_input.span() => compile_error!("table struct must have a field named \"id\"");
        }
        .into();
    };

    let field_names = fields.iter().map(|field| &field.ident);
    let table_field_structs = fields
        .iter()
        .map(|field| field.generate_table_field_struct());
    let fields_type = generate_fields_cons_list_type(&fields);

    let table_name_ident = Ident::new(&table_name, table_struct_ident.span());

    let column_structs = fields.iter().map(|field| {
        // it is safe to unwrap here since only named fields are allowed
        generate_column_struct(
            field.ident.as_ref().unwrap(),
            &field.ty,
            &table_struct_ident,
        )
    });

    let table_marker = generate_table_marker(&table_struct_ident);

    let foreign_key_impls = fields.iter().filter_map(|field| {
        let foreign_key_to_table_name = field.foreign_key.as_ref()?;

        Some(generate_foreign_key_impl(
            &table_struct_ident,
            &table_name_ident,
            &foreign_key_to_table_name,
            field.ident.as_ref().unwrap(),
        ))
    });

    return quote! {
        #[automatically_derived]
        impl ::gorm::table::Table for #table_struct_ident {
            type Fields = #fields_type;
            const FIELDS: &'static [::gorm::table::TableField] = &[
                #( #table_field_structs ),*
            ];
            const TABLE_NAME: &'static ::std::primitive::str = #table_name;
            type IdColumn = #table_name_ident::id;
        }

        #[automatically_derived]
        impl ::gorm::from_query_result::FromQueryResult for #table_struct_ident
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

        #(
            #foreign_key_impls
         )*

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
    table_name: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(table))]
struct TableInputField {
    ident: Option<Ident>,
    ty: Type,
    foreign_key: Option<String>,
}
impl TableInputField {
    fn generate_table_field_struct(&self) -> proc_macro2::TokenStream {
        // it is safe to unwrap here since only named fields are allowed.
        let name = self.ident.as_ref().unwrap();
        let is_primary_key = name.to_string() == "id";
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
        let foreign_key_to_table_name = match &self.foreign_key {
            Some(foreign_key_table_struct_name) => {
                let foreign_key_table_struct_ident =
                    Ident::new(&foreign_key_table_struct_name, self.ident.span());
                quote! {
                    Some(<#foreign_key_table_struct_ident as ::gorm::Table>::TABLE_NAME)
                }
            }
            None => quote! { None },
        };

        quote! {
            ::gorm::table::TableField {
                name: stringify!(#name),
                is_primary_key: #is_primary_key,
                foreign_key_to_table_name: #foreign_key_to_table_name,
                sql_type_name: #sql_type_name,
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

fn generate_column_struct(
    column_name_ident: &Ident,
    column_type: &Type,
    table_struct_ident: &Ident,
) -> proc_macro2::TokenStream {
    let column_name = column_name_ident.to_string();
    quote! {
        pub struct #column_name_ident;

        #[automatically_derived]
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

        #[automatically_derived]
        impl ::gorm::table::TableMarker for table {
            type Table = super::#table_struct_ident;
        }
    }
}

fn generate_foreign_key_impl(
    table_struct_ident: &Ident,
    table_name_ident: &Ident,
    other_table_name: &str,
    foreign_key_column_ident: &Ident,
) -> proc_macro2::TokenStream {
    let other_table_ident = Ident::new(other_table_name, foreign_key_column_ident.span());

    quote! {
        #[automatically_derived]
        impl ::gorm::table::HasForeignKey<#other_table_ident> for #table_struct_ident {
            type ForeignKeyColumn = #table_name_ident::#foreign_key_column_ident;
        }
    }
}

/*
 *
/// Indicates that some table has a foreign key to some other table
pub trait HasForeignKey<T: Table>: Table {
    type ForeignKeyColumn: Column;
}
*/
