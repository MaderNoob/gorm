use std::collections::HashSet;

use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::As,
    DeriveInput, Expr, Token, Type,
};

#[proc_macro]
pub fn create_field_name_cons_list(item: TokenStream) -> TokenStream {
    generate_field_name_cons_list_type(&item.to_string()).into()
}

struct RawSelectedValue {
    selected_expr: Expr,
    select_as: Option<Ident>,
}
impl Parse for RawSelectedValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let selected_expr: Expr = input.parse()?;

        // if the cast is used without parentheses, extract the `as <name>` from the
        // expression.
        if let Expr::Cast(expr_cast) = selected_expr {
            let as_tokenstream: TokenStream = expr_cast.ty.into_token_stream().into();
            let as_ident: Ident = syn::parse(as_tokenstream)?;
            return Ok(Self {
                selected_expr: *expr_cast.expr,
                select_as: Some(as_ident),
            });
        }

        let lookahead = input.lookahead1();
        let select_as = if lookahead.peek(Token![as]) {
            input.parse::<As>()?;
            Some(input.parse::<Ident>()?)
        } else {
            None
        };

        Ok(Self {
            selected_expr,
            select_as,
        })
    }
}

struct RawSelectValuesInput {
    values: Punctuated<RawSelectedValue, Token![,]>,
}

impl Parse for RawSelectValuesInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let values = Punctuated::<RawSelectedValue, Token![,]>::parse_terminated(input)?;
        Ok(Self { values })
    }
}

struct SelectValuesInput {
    values: Vec<SelectedValue>,
}

struct SelectedValue {
    selected_expr: Expr,
    select_as: Ident,

    /// should an explicit `as <name>` be added to this value when formatting it
    /// to sql. This will be true for complicated expressions, but false for
    /// example for `person::name`.
    should_use_explicit_as_in_sql: bool,
}

#[proc_macro]
pub fn select_values(input_tokens: TokenStream) -> TokenStream {
    let raw_select_values_input = parse_macro_input!(input_tokens as RawSelectValuesInput);
    let mut values: Vec<SelectedValue> = Vec::with_capacity(raw_select_values_input.values.len());

    // make sure that there are no duplicate names
    let mut names_already_in_use = HashSet::new();

    for raw_value in raw_select_values_input.values {
        let value = match raw_value.select_as {
            Some(select_as) => SelectedValue {
                selected_expr: raw_value.selected_expr,
                select_as,
                should_use_explicit_as_in_sql: true,
            },
            None => {
                // if the expression doesn't have an `as` clause, it must be a path as in
                // `person::name` so that we can get its name from the last path segment.
                match &raw_value.selected_expr {
                    Expr::Path(expr_path) => {
                        let last_segment = expr_path.path.segments.last().unwrap();
                        let select_as_ident = last_segment.ident.clone();
                        SelectedValue {
                            selected_expr: raw_value.selected_expr,
                            select_as: select_as_ident,
                            should_use_explicit_as_in_sql: false,
                        }
                    }
                    _ => {
                        return quote_spanned! {
                            raw_value.selected_expr.span() => compile_error!("selecting a value without explicitly specifying its name can only be used with paths, like in `person::name`. please add `as <name>` to explicitly specify the name of this selected expression"),
                        }.into()
                    }
                }
            },
        };

        // if the name is already used, return an error
        if !names_already_in_use.insert(value.select_as.to_string()) {
            return quote_spanned!{
                value.selected_expr.span() => compile_error!("can't select 2 values with the same name")
            }.into();
        }

        values.push(value);
    }

    let select_values_input = SelectValuesInput { values };

    // the definition of the generics, for example: `E0,E1,E2`
    let struct_expr_generics_definition = (0..select_values_input.values.len()).map(|i| {
        let generic_name = Ident::new(&format!("E{}", i), proc_macro2::Span::call_site());
        quote! {
            #generic_name: ::gorm::sql::SqlExpression<S>
        }
    });

    let struct_generics_definition = quote! {
        S: ::gorm::sql::SelectableTables,
        #(#struct_expr_generics_definition),*
    };
    let struct_generics_definition_clone = struct_generics_definition.clone();

    let struct_tuple_fields_definition = (0..select_values_input.values.len())
        .map(|i| Ident::new(&format!("E{}", i), proc_macro2::Span::call_site()));

    let use_expr_generics_in_impl = (0..select_values_input.values.len())
        .map(|i| Ident::new(&format!("E{}", i), proc_macro2::Span::call_site()));

    let fields_cons_list = create_selected_values_fields_cons_list(&select_values_input);

    let write_selected_expressions_sql_string =
        select_values_input
            .values
            .iter()
            .enumerate()
            .map(|(i, selected_value)| {
                let write_as_sql_string = if selected_value.should_use_explicit_as_in_sql {
                    let select_as = selected_value.select_as.to_string();
                    let as_sql_string = format!(" as {}", select_as);

                    quote! {
                        write!(f, #as_sql_string)?;
                    }
                } else {
                    quote! {}
                };

                // as long as it's not the last item, write the comma
                let write_comma_string = if i + 1 < select_values_input.values.len() {
                    quote! {
                        write!(f, ",")?;
                    }
                } else {
                    quote! {}
                };

                let field_index = syn::Index::from(i);

                quote! {
                    self.#field_index.write_sql_string(f, parameter_binder)?;
                    #write_as_sql_string
                    #write_comma_string
                }
            });

    let create_instance_tuple_values = select_values_input
        .values
        .iter()
        .map(|selected_value| &selected_value.selected_expr);

    quote! {
        {
            struct CustomSelectedValues<#struct_generics_definition>(
                #(#struct_tuple_fields_definition),* ,
                ::std::marker::PhantomData<S>,
            );

            #[automatically_derived]
            impl<#struct_generics_definition_clone>
                ::gorm::sql::SelectedValues<S> for CustomSelectedValues<S, #(#use_expr_generics_in_impl),*>
            {
                type Fields = #fields_cons_list;

                fn write_sql_string<'s, 'a>(
                    &'s self,
                    f: &mut ::std::string::String,
                    parameter_binder: &mut ::gorm::sql::ParameterBinder<'a>,
                ) -> ::std::fmt::Result
                where 's: 'a
                {
                    use ::std::fmt::Write;
                    #(
                        #write_selected_expressions_sql_string
                     )*
                    Ok(())
                }
            }
            CustomSelectedValues(#(#create_instance_tuple_values),* , ::std::marker::PhantomData)
        }
    }.into()
}

fn create_selected_values_fields_cons_list(
    select_values_input: &SelectValuesInput,
) -> proc_macro2::TokenStream {
    // start with the inner most type and wrap it each time with each field.
    let mut cur = quote! { ::gorm::TypedConsListNil };

    for (i, selected_value) in select_values_input.values.iter().enumerate().rev() {
        // safe to unwrap here because only structs with named fields are allowed.
        let field_name = selected_value.select_as.to_string();
        let field_name_type = generate_field_name_cons_list_type(&field_name);

        let selected_expr_generic_ident =
            Ident::new(&format!("E{}", i), proc_macro2::Span::call_site());
        let field_type = quote! {
            #selected_expr_generic_ident::RustType
        };

        cur = quote! {
            ::gorm::sql::FieldsConsListCons<
                #field_name_type,
                #field_type,
                #cur
            >
        };
    }

    cur
}

#[proc_macro_derive(FromQueryResult)]
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

    let field_names = fields.iter().map(|field| &field.ident);
    let table_field_structs = fields
        .iter()
        .map(|field| field.generate_table_field_struct());
    let fields_type =
        generate_fields_cons_list_type(fields.iter().map(|field| (&field.ident, &field.ty)));

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
            foreign_key_to_table_name,
            field.ident.as_ref().unwrap(),
        ))
    });

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
        mod #table_name_ident {
            #(
                #column_structs
             )*

            #table_marker
        }
    }
    .into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct FromQueryResultInput {
    ident: Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), FromQueryResultInputField>,
}

#[derive(Debug, FromField)]
struct FromQueryResultInputField {
    ident: Option<Ident>,
    ty: Type,
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
        let is_primary_key = name == "id";
        let ty = &self.ty;
        let sql_type_name = if is_primary_key {
            quote! {
                <<#ty as ::gorm::sql::IntoSqlSerialType>::SqlSerialType as ::gorm::sql::SqlSerialType>::SQL_NAME
            }
        } else {
            quote! {
                <<#ty as ::gorm::sql::IntoSqlType>::SqlType as ::gorm::sql::SqlType>::SQL_NAME
            }
        };
        let foreign_key_to_table_name = match &self.foreign_key {
            Some(foreign_key_table_struct_name) => {
                let foreign_key_table_struct_ident =
                    Ident::new(foreign_key_table_struct_name, self.ident.span());
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
            }
        }
    }
}

fn generate_field_name_cons_list_type(field_name: &str) -> proc_macro2::TokenStream {
    // start with the inner most type and wrap it each time with each character.
    let mut cur = quote! { ::gorm::TypedConsListNil };

    for chr in field_name.chars().rev() {
        cur = quote! {
            ::gorm::sql::FieldNameCharsConsListCons<#chr, #cur>
        };
    }

    cur
}

fn generate_fields_cons_list_type<'a>(
    fields: impl Iterator<Item = (&'a Option<Ident>, &'a Type)> + DoubleEndedIterator,
) -> proc_macro2::TokenStream {
    // start with the inner most type and wrap it each time with each field.
    let mut cur = quote! { ::gorm::TypedConsListNil };

    for field in fields.rev() {
        // safe to unwrap here because only structs with named fields are allowed.
        let field_name = field.0.as_ref().unwrap().to_string();
        let field_name_type = generate_field_name_cons_list_type(&field_name);
        let field_type = &field.1;
        cur = quote! {
            ::gorm::sql::FieldsConsListCons<
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
        impl ::gorm::sql::Column for #column_name_ident {
            const COLUMN_NAME:&'static str = #column_name;
            type Table = super::#table_struct_ident;
            type SqlType = <#column_type as ::gorm::sql::IntoSqlType>::SqlType;
            type RustType = #column_type;
        }
    }
}

fn generate_table_marker(table_struct_ident: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub struct table;

        #[automatically_derived]
        impl ::gorm::sql::TableMarker for table {
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
        impl ::gorm::sql::HasForeignKey<#other_table_ident> for #table_struct_ident {
            type ForeignKeyColumn = #table_name_ident::#foreign_key_column_ident;
        }
    }
}
