use std::collections::{HashMap, HashSet};

use convert_case::{Case, Casing};
use darling::{ast::Fields, util::Flag, FromDeriveInput, FromField, FromMeta};
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Lifetime, Type, Visibility};

use crate::util::{generate_field_name_cons_list_type, generate_fields_cons_list_type};

pub fn table(input_tokens: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let parse_input_result = TableInput::from_derive_input(&derive_input);
    let input = match parse_input_result {
        Ok(input) => input,
        Err(err) => return err.write_errors().into(),
    };
    let TableInput {
        ident: table_struct_ident,
        generics,
        data,
        vis,
        table_name: optional_table_name,
        unique: unique_constraints,
    } = input;

    let table_name =
        optional_table_name.unwrap_or_else(|| table_struct_ident.to_string().to_case(Case::Snake));

    if !generics.params.is_empty() {
        return quote_spanned! {
            generics.span() => compile_error!("generics are not supported on tables");
        }
        .into();
    }

    let fields = data.take_struct().unwrap();
    if !fields.style.is_struct() {
        return quote_spanned! {
            derive_input.span() => compile_error!("only named structs are supported for tables");
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

    // make sure that the unique constraints are all referring to valid fields.
    let field_names_strings_set: HashSet<String> = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect();
    for unique_constraint in &unique_constraints {
        // if the unique constraint doesn't mention any field names
        if unique_constraint.fields.is_empty() {
            // return quote_spanned!{}
        }

        for (field_name, span) in &unique_constraint.fields {
            // if there is no such field
            if !field_names_strings_set.contains(field_name) {
                let span = span.clone();
                return quote_spanned! {
                    span => compile_error!("no such field");
                }
                .into();
            }
        }
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

    // if this table has only 1 foreign key column to some other table, we can
    // implement the `TableHasOneForeignKey` trait for it.
    struct ForeignKeyCounter {
        table_struct_ident: proc_macro2::Ident,
        field_ident: proc_macro2::Ident,
        amount: usize,
    }
    let mut foreign_keys_amount_for_each_table = HashMap::new();
    for field in fields.iter() {
        let to_table = match field.foreign_key.to_table.as_ref() {
            Some(v) => v,
            None => continue,
        };
        let table_name_string = to_table.to_string();
        foreign_keys_amount_for_each_table
            .entry(table_name_string)
            .or_insert_with(|| ForeignKeyCounter {
                table_struct_ident: to_table.clone(),
                field_ident: field.ident.as_ref().unwrap().clone(),
                amount: 0,
            })
            .amount += 1;
    }
    let table_foreign_key_impls = foreign_keys_amount_for_each_table
        .iter()
        .filter(|(_, counter)| counter.amount == 1)
        .map(|(_, counter)| {
            generate_table_has_one_foreign_key_impl(
                &table_struct_ident,
                &table_name_ident,
                &counter.table_struct_ident,
                &counter.field_ident,
            )
        });

    let column_foreign_key_impls = fields
        .iter()
        .filter_map(|field| field.generate_column_foreign_key_impl());

    let unique_constraint_structs = unique_constraints
        .iter()
        .map(|unique_constraint| unique_constraint.generate_table_unique_constraint_struct())
        .chain(
            fields
                .iter()
                .filter_map(|field| field.generate_table_unique_constraint_struct()),
        );

    let insertables = implement_insertables(&table_struct_ident, &fields);

    let all_fields_selected_struct =
        implement_all_fields_selected_struct(&fields_type, &table_struct_ident, &table_name);

    let unique_constraint_marker_structs = unique_constraints
        .iter()
        .map(|unique_constraint| {
            unique_constraint.generate_unique_constraint_marker_struct(&table_struct_ident)
        })
        .chain(fields.iter().filter_map(|field| {
            field.generate_unique_constraint_marker_struct(&table_struct_ident)
        }));

    quote! {
        #[automatically_derived]
        impl ::gorm::sql::Table for #table_struct_ident {
            type Fields = #fields_type;
            const FIELDS: &'static [::gorm::sql::TableField] = &[
                #( #table_field_structs ),*
            ];
            const UNIQUE_CONSTRAINTS: &'static [::gorm::sql::TableUniqueConstraint] = &[
                # ( #unique_constraint_structs ),*
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
            #table_foreign_key_impls
         )*

        #[allow(non_camel_case_types)]
        pub mod #table_name_ident {
            use super::*;

            #(
                #column_structs
            )*

            #(
                #column_foreign_key_impls
            )*

            #table_marker

            #insertables

            #all_fields_selected_struct

            pub mod unique_constraints {
                use super::super::*;

                pub struct id;

                #[automatically_derived]
                impl ::gorm::sql::UniqueConstraint for id {
                    type Table = super::super::#table_struct_ident;
                    const FIELDS_COMMA_SEPERATED:&'static str = "id";
                }

                #(
                    #unique_constraint_marker_structs
                )*
            }
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

    #[darling(multiple)]
    unique: Vec<UniqueConstraintFieldsList>,
}

#[derive(Debug, FromField)]
#[darling(attributes(table))]
struct TableInputField {
    ident: Option<proc_macro2::Ident>,
    ty: Type,
    foreign_key: ForeignKeySpecification,
    unique: Flag,
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

        let foreign_key_to_table_name = match &self.foreign_key.to_table {
            Some(foreign_key_table_struct_path) => {
                quote! {
                    Some(<#foreign_key_table_struct_path as ::gorm::Table>::TABLE_NAME)
                }
            },
            None => quote! { None },
        };

        let is_unique = self.unique.is_present();

        quote! {
            ::gorm::sql::TableField {
                name: stringify!(#name),
                is_primary_key: #is_primary_key,
                is_unique: #is_unique,
                foreign_key_to_table_name: #foreign_key_to_table_name,
                sql_type_name: #sql_type_name,
                is_null: #is_null,
            }
        }
    }

    fn generate_table_unique_constraint_struct(&self) -> Option<proc_macro2::TokenStream> {
        if !self.unique.is_present() {
            return None;
        }
        let field_name_string = self.ident.as_ref().unwrap().to_string();
        Some(quote! {
            ::gorm::sql::TableUniqueConstraint {
                fields: &[#field_name_string],
            }
        })
    }

    fn generate_unique_constraint_marker_struct(
        &self,
        table_struct_ident: &proc_macro2::Ident,
    ) -> Option<proc_macro2::TokenStream> {
        if !self.unique.is_present() {
            return None;
        }
        let field_name_ident = self.ident.as_ref().unwrap();
        let field_name_string = field_name_ident.to_string();
        Some(quote! {
            pub struct #field_name_ident;

            #[automatically_derived]
            impl ::gorm::sql::UniqueConstraint for #field_name_ident {
                type Table = super::super::#table_struct_ident;
                const FIELDS_COMMA_SEPERATED:&'static str = #field_name_string;
            }
        })
    }

    fn generate_column_foreign_key_impl(&self) -> Option<proc_macro2::TokenStream> {
        let to_table = self.foreign_key.to_table.as_ref()?;
        let field_name_ident = self.ident.as_ref().unwrap();
        Some(quote! {
            impl ::gorm::sql::ColumnIsForeignKey<#to_table> for #field_name_ident { }
        })
    }
}

#[derive(Debug)]
struct ForeignKeySpecification {
    to_table: Option<proc_macro2::Ident>,
}
impl FromMeta for ForeignKeySpecification {
    fn from_list(items: &[syn::NestedMeta]) -> darling::Result<Self> {
        if items.is_empty() {
            return Err(darling::Error::custom(
                "you must specify which table this foreign key constraint refers to",
            ));
        }
        if items.len() > 1 {
            return Err(darling::Error::custom(
                "can't have a single foreign key column to multiple tables",
            ));
        }

        let item = &items[0];

        let err = darling::Error::custom("expected a table struct name").with_span(item);

        let path = match item {
            syn::NestedMeta::Meta(meta) => match meta {
                syn::Meta::Path(path) => path,
                _ => return Err(err),
            },
            _ => return Err(err),
        };

        let ident = path.get_ident().ok_or(err)?;

        Ok(Self {
            to_table: Some(ident.clone()),
        })
    }

    fn from_none() -> Option<Self> {
        Some(Self { to_table: None })
    }
}

#[derive(Debug)]
struct UniqueConstraintFieldsList {
    fields: Vec<(String, proc_macro2::Span)>,
}
impl UniqueConstraintFieldsList {
    fn generate_table_unique_constraint_struct(&self) -> proc_macro2::TokenStream {
        let field_name_strings = self
            .fields
            .iter()
            .map(|(field_name, _span)| field_name.as_str());
        quote! {
            ::gorm::sql::TableUniqueConstraint {
                fields: &[#(#field_name_strings),*],
            }
        }
    }

    fn generate_unique_constraint_marker_struct(
        &self,
        table_struct_ident: &proc_macro2::Ident,
    ) -> proc_macro2::TokenStream {
        let struct_name_string = self
            .fields
            .iter()
            .map(|(field_name, _span)| field_name.as_str())
            .join("_");
        let struct_name_ident =
            proc_macro2::Ident::new(&struct_name_string, proc_macro2::Span::call_site());
        let fields_comma_seperated = self
            .fields
            .iter()
            .map(|(field_name, _span)| field_name.as_str())
            .join(",");
        quote! {
            pub struct #struct_name_ident;

            #[automatically_derived]
            impl ::gorm::sql::UniqueConstraint for #struct_name_ident {
                type Table = super::super::#table_struct_ident;
                const FIELDS_COMMA_SEPERATED:&'static str = #fields_comma_seperated;
            }
        }
    }
}
impl FromMeta for UniqueConstraintFieldsList {
    fn from_list(items: &[syn::NestedMeta]) -> darling::Result<Self> {
        if items.is_empty() {
            return Err(darling::Error::custom(
                "the unique constraint fields list can't be empty, please specify a list of fields to be included in this unique constraint",
            ));
        }
        let mut fields = Vec::with_capacity(items.len());
        for nested_meta in items {
            let field_name_string = quote! {#nested_meta}.to_string();

            // make sure the provided field name is a valid identifier
            if syn::parse_str::<proc_macro2::Ident>(&field_name_string).is_err() {
                return Err(darling::Error::custom("expected a field name"));
            }

            fields.push((field_name_string, nested_meta.span()))
        }
        Ok(Self { fields })
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

fn generate_table_has_one_foreign_key_impl(
    table_struct_ident: &proc_macro2::Ident,
    table_name_ident: &proc_macro2::Ident,
    other_table_path: &proc_macro2::Ident,
    foreign_key_column_ident: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    quote! {
        #[automatically_derived]
        impl ::gorm::sql::TableHasOneForeignKey<#other_table_path> for #table_struct_ident {
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

            #[automatically_derived]
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
