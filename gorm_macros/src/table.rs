use convert_case::{Case, Casing};
use darling::{ast::Fields, FromDeriveInput, FromField};
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Lifetime, Type, Visibility};

use crate::util::{generate_field_name_cons_list_type, generate_fields_cons_list_type};

pub fn table(input_tokens: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let input = TableInput::from_derive_input(&derive_input).unwrap();
    let TableInput {
        ident: table_struct_ident,
        generics,
        data,
        vis,
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
            derive_input.span() => compile_error!("only named structs are supported for tables")
        }
        .into();
    }

    // make sure there is a field named id
    if !fields
        .iter()
        .any(|field| field.ident.as_ref().unwrap() == "id")
    {
        return quote_spanned! {
            derive_input.span() => compile_error!("table struct must have a field named \"id\"");
        }
        .into();
    };

    if !matches!(vis, Visibility::Public(_)) {
        return quote_spanned! {
            vis.span() => compile_error!("table struct must be public");
        }
        .into();
    }

    let field_names = fields.iter().map(|field| &field.ident);
    let table_field_structs = fields
        .iter()
        .map(|field| field.generate_table_field_struct());
    let fields_type =
        generate_fields_cons_list_type(fields.iter().map(|field| (&field.ident, &field.ty)));

    let table_name_ident = proc_macro2::Ident::new(&table_name, table_struct_ident.span());

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
            foreign_key_to_table_name,
            field.ident.as_ref().unwrap(),
        ))
    });

    let insertables = implement_insertables(&table_struct_ident, &fields);

    let all_fields_selected_struct =
        implement_all_fields_selected_struct(&fields_type, &table_struct_ident, &table_name);

    quote! {
        #[automatically_derived]
        impl ::gorm::sql::Table for #table_struct_ident {
            type Fields = #fields_type;
            const FIELDS: &'static [::gorm::sql::TableField] = &[
                #( #table_field_structs ),*
            ];
            const TABLE_NAME: &'static ::std::primitive::str = #table_name;
            type IdColumn = #table_name_ident::id;
        }

        #[automatically_derived]
        impl ::gorm::FromQueryResult for #table_struct_ident
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
        pub mod #table_name_ident {
            #(
                #column_structs
             )*

            #table_marker

            #insertables

            #all_fields_selected_struct
        }
    }
    .into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(table), supports(struct_named))]
struct TableInput {
    ident: proc_macro2::Ident,
    generics: syn::Generics,
    vis: syn::Visibility,
    data: darling::ast::Data<(), TableInputField>,
    table_name: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(table))]
struct TableInputField {
    ident: Option<proc_macro2::Ident>,
    ty: Type,
    foreign_key: Option<String>,
}
impl TableInputField {
    fn generate_table_field_struct(&self) -> proc_macro2::TokenStream {
        // it is safe to unwrap here since only named fields are allowed.
        let name = self.ident.as_ref().unwrap();
        let is_primary_key = name == "id";
        let ty = &self.ty;
        let as_sql_type_trait = if is_primary_key {
            quote! {
                <<#ty as ::gorm::sql::IntoSqlSerialType>::SqlSerialType as ::gorm::sql::SqlSerialType>
            }
        } else {
            quote! {
                <<#ty as ::gorm::sql::IntoSqlType>::SqlType as ::gorm::sql::SqlType>
            }
        };
        let sql_type_name = quote! { #as_sql_type_trait::SQL_NAME };
        let is_null = quote! { <#as_sql_type_trait::IsNull as ::gorm::util::TypedBool>::VALUE };

        let foreign_key_to_table_name = match &self.foreign_key {
            Some(foreign_key_table_struct_name) => {
                let foreign_key_table_struct_ident =
                    proc_macro2::Ident::new(foreign_key_table_struct_name, self.ident.span());
                quote! {
                    Some(<#foreign_key_table_struct_ident as ::gorm::Table>::TABLE_NAME)
                }
            },
            None => quote! { None },
        };

        quote! {
            ::gorm::sql::TableField {
                name: stringify!(#name),
                is_primary_key: #is_primary_key,
                foreign_key_to_table_name: #foreign_key_to_table_name,
                sql_type_name: #sql_type_name,
                is_null: #is_null,
            }
        }
    }
}

fn generate_column_struct(
    column_name_ident: &proc_macro2::Ident,
    column_type: &Type,
    table_struct_ident: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let column_name = column_name_ident.to_string();
    let column_name_type = generate_field_name_cons_list_type(&column_name);
    quote! {
        pub struct #column_name_ident;

        #[automatically_derived]
        impl ::gorm::sql::Column for #column_name_ident {
            const COLUMN_NAME:&'static str = #column_name;
            type ColumnName = #column_name_type;
            type Table = super::#table_struct_ident;
            type SqlType = <#column_type as ::gorm::sql::IntoSqlType>::SqlType;
            type RustType = #column_type;
        }
    }
}

fn generate_table_marker(table_struct_ident: &proc_macro2::Ident) -> proc_macro2::TokenStream {
    quote! {
        pub struct table;

        #[automatically_derived]
        impl ::gorm::sql::TableMarker for table {
            type Table = super::#table_struct_ident;
        }
    }
}

fn generate_foreign_key_impl(
    table_struct_ident: &proc_macro2::Ident,
    table_name_ident: &proc_macro2::Ident,
    other_table_name: &str,
    foreign_key_column_ident: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let other_table_ident =
        proc_macro2::Ident::new(other_table_name, foreign_key_column_ident.span());

    quote! {
        #[automatically_derived]
        impl ::gorm::sql::HasForeignKey<#other_table_ident> for #table_struct_ident {
            type ForeignKeyColumn = #table_name_ident::#foreign_key_column_ident;
        }
    }
}

fn implement_insertables(
    table_struct_ident: &proc_macro2::Ident,
    fields: &Fields<TableInputField>,
) -> proc_macro2::TokenStream {
    let fields_other_than_id: Vec<_> = fields
        .iter()
        .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
        .filter(|(ident, _ty)| ident.to_string() != "id")
        .collect();

    let all_fields: Vec<_> = fields
        .iter()
        .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
        .collect();

    let insertable_without_id =
        implement_insertable_with_fields("new", table_struct_ident, &fields_other_than_id);
    let insertable_with_id =
        implement_insertable_with_fields("new_with_id", table_struct_ident, &all_fields);

    quote! {
        #insertable_without_id

        #insertable_with_id
    }
}

fn implement_insertable_with_fields(
    insertable_struct_name: &str,
    table_struct_ident: &proc_macro2::Ident,
    fields: &[(&proc_macro2::Ident, &Type)],
) -> proc_macro2::TokenStream {
    let insertable_struct_name_ident =
        proc_macro2::Ident::new(insertable_struct_name, proc_macro2::Span::call_site());

    // special case for when the insertable has no fields. this happens when
    // implementing the `new` struct for a table that only has an `id` field.
    if fields.is_empty() {
        return quote! {
            /// A struct which allows inserting new records into the table.
            pub struct #insertable_struct_name_ident;

            impl ::gorm::sql::Insertable for #insertable_struct_name_ident
            {
                type Table = super::#table_struct_ident;

                fn write_value_names(
                    &self,
                    f: &mut ::std::string::String,
                ) -> ::std::fmt::Result {
                    Ok(())
                }

                fn write_values<'s, 'a>(
                    &'s self,
                    f: &mut String,
                    parameter_binder: &mut ::gorm::sql::ParameterBinder<'a>,
                ) -> std::fmt::Result
                where
                    's: 'a
                {
                    Ok(())
                }
            }
        };
    }

    let new_fields = fields.iter().enumerate().map(|(i, (ident, _ty))| {
        let borrow_generic_ident =
            proc_macro2::Ident::new(&format!("Q{}", i), proc_macro2::Span::call_site());
        let lifetime = Lifetime::new(&format!("'{}", ident), proc_macro2::Span::call_site());
        quote! {
            pub #ident: &#lifetime #borrow_generic_ident
        }
    });

    let value_names = fields.iter().map(|(ident, _ty)| ident).join(",");

    let write_each_field_value = fields.iter().enumerate().map(|(i, (ident, _ty))| {
        let is_last_item = i + 1 == fields.len();
        let format_string = if is_last_item { "{}" } else { "{}," };

        quote! {
            ::std::write!(f, #format_string, parameter_binder.bind_parameter(&self.#ident))?;
        }
    });

    let lifetimes = {
        let lifetime_tokens_iter = fields.iter().map(|(ident, _ty)| {
            Lifetime::new(&format!("'{}", ident), proc_macro2::Span::call_site())
        });
        quote! {
            #(#lifetime_tokens_iter),*
        }
    };
    let lifetimes_ref = &lifetimes;

    let borrow_generics_definition = {
        let idents = (0..fields.len())
            .map(|i| proc_macro2::Ident::new(&format!("Q{}", i), proc_macro2::Span::call_site()));
        quote! {
            #(#idents),*
        }
    };
    let borrow_generics_definition_ref = &borrow_generics_definition;

    let where_clause_conditions = {
        let conditions = fields.iter().enumerate().map(|(i, (ident, ty))| {
            let generic_ident =
                proc_macro2::Ident::new(&format!("Q{}", i), proc_macro2::Span::call_site());
            let generic_ident_ref = &generic_ident;
            let lifetime_token = Lifetime::new(&format!("'{}", ident), proc_macro2::Span::call_site());
            quote! {
                #generic_ident_ref: ?::std::marker::Sized + ::std::marker::Send + ::std::marker::Sync,
                &#lifetime_token #generic_ident_ref: ::gorm::tokio_postgres::types::ToSql,
                #ty: ::std::borrow::Borrow<#generic_ident_ref>,
            }
        });
        quote! {
            #(#conditions)*
        }
    };
    let where_clause_conditions_ref = &where_clause_conditions;

    quote! {
        /// A struct which allows inserting new records into the table.
        pub struct #insertable_struct_name_ident<#lifetimes_ref , #borrow_generics_definition_ref>
        where
            #where_clause_conditions_ref
        {
            #(
                #new_fields
            ),*
        }

        impl<#lifetimes_ref , #borrow_generics_definition_ref> ::gorm::sql::Insertable for #insertable_struct_name_ident<#lifetimes_ref, #borrow_generics_definition_ref>
        where
            #where_clause_conditions_ref
        {
            type Table = super::#table_struct_ident;

            fn write_value_names(
                &self,
                f: &mut ::std::string::String,
            ) -> ::std::fmt::Result {
                use ::std::fmt::Write;

                ::std::write!(f, #value_names)
            }

            fn write_values<'s, 'a>(
                &'s self,
                f: &mut String,
                parameter_binder: &mut ::gorm::sql::ParameterBinder<'a>,
            ) -> std::fmt::Result
            where
                's: 'a
            {
                use ::std::fmt::Write;

                #(
                    #write_each_field_value
                )*

                Ok(())
            }
        }
    }
}

fn implement_all_fields_selected_struct(
    fields_cons_list_type: &proc_macro2::TokenStream,
    table_struct_ident: &proc_macro2::Ident,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let table_dot_asterisk = format!("\"{}\".*", table_name);

    quote! {
        pub struct all;

        impl<
            S: ::gorm::sql::SelectableTables
                + ::gorm::sql::SelectableTablesContains<super::#table_struct_ident>
        > ::gorm::sql::SelectedValues<S> for all {
            type Fields = #fields_cons_list_type;

            const IS_AGGREGATE: bool = false;

            fn write_sql_string<'s, 'a>(
                &'s self,
                f: &mut ::std::string::String,
                _parameter_binder: &mut ::gorm::sql::ParameterBinder<'a>,
            ) -> ::std::fmt::Result
            where
                's: 'a
            {
                use ::std::fmt::Write;

                ::std::write!(f, #table_dot_asterisk)
            }
        }
    }
}
