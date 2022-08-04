use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::As, Expr,
    Token,
};

use crate::util::generate_field_name_cons_list_type;

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
        let generic_name =
            proc_macro2::Ident::new(&format!("E{}", i), proc_macro2::Span::call_site());
        quote! {
            #generic_name: ::gorm::sql::SqlExpression<S>
        }
    });

    let struct_generics_definition = quote! {
        S: ::gorm::sql::SelectableTables,
        #(#struct_expr_generics_definition),*
    };
    let struct_generics_definition_ref = &struct_generics_definition;

    let struct_tuple_fields_definition = (0..select_values_input.values.len())
        .map(|i| proc_macro2::Ident::new(&format!("E{}", i), proc_macro2::Span::call_site()));

    let use_expr_generics_in_impl = (0..select_values_input.values.len())
        .map(|i| proc_macro2::Ident::new(&format!("E{}", i), proc_macro2::Span::call_site()));

    let is_aggregate = (0..select_values_input.values.len()).map(|i| {
        let expr_generic_ident =
            proc_macro2::Ident::new(&format!("E{}", i), proc_macro2::Span::call_site());
        let is_last_item = i + 1 == select_values_input.values.len();
        if is_last_item {
            quote! {
                #expr_generic_ident::IS_AGGREGATE
            }
        } else {
            quote! {
                #expr_generic_ident::IS_AGGREGATE ||
            }
        }
    });

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

    let use_expr_generics_in_impl_clone = use_expr_generics_in_impl.clone();
    let impl_contains_field_names = select_values_input.values.iter().map(|selected_value|{
        let name_string = selected_value.select_as.to_string();
        let field_name_type = generate_field_name_cons_list_type(&name_string);
        let use_expr_generics_in_impl = use_expr_generics_in_impl_clone.clone();
        quote!{
            #[automatically_derived]
            impl<#struct_generics_definition_ref>
                ::gorm::sql::SelectedValuesContainsFieldWithName<#field_name_type> for CustomSelectedValues<S, #(#use_expr_generics_in_impl),*>
            {
            }
        }
    });

    quote! {
        {
            struct CustomSelectedValues<#struct_generics_definition_ref>(
                #(#struct_tuple_fields_definition),* ,
                ::std::marker::PhantomData<S>,
            );

            #[automatically_derived]
            impl<#struct_generics_definition_ref>
                ::gorm::sql::SelectedValues<S> for CustomSelectedValues<S, #(#use_expr_generics_in_impl),*>
            {
                type Fields = #fields_cons_list;

                const IS_AGGREGATE:bool = #(#is_aggregate)*;

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

            #(
                #impl_contains_field_names
            )*

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
            proc_macro2::Ident::new(&format!("E{}", i), proc_macro2::Span::call_site());
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

struct RawSelectedValue {
    selected_expr: Expr,
    select_as: Option<proc_macro2::Ident>,
}
impl Parse for RawSelectedValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let selected_expr: Expr = input.parse()?;

        // if the cast is used without parentheses, extract the `as <name>` from the
        // expression.
        if let Expr::Cast(expr_cast) = selected_expr {
            let as_tokenstream: TokenStream = expr_cast.ty.into_token_stream().into();
            let as_ident: proc_macro2::Ident = syn::parse(as_tokenstream)?;
            return Ok(Self {
                selected_expr: *expr_cast.expr,
                select_as: Some(as_ident),
            });
        }

        let lookahead = input.lookahead1();
        let select_as = if lookahead.peek(Token![as]) {
            input.parse::<As>()?;
            Some(input.parse::<proc_macro2::Ident>()?)
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
    select_as: proc_macro2::Ident,

    /// should an explicit `as <name>` be added to this value when formatting it
    /// to sql. This will be true for complicated expressions, but false for
    /// example for `person::name`.
    should_use_explicit_as_in_sql: bool,
}
