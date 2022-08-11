use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, Expr, Path, Token};

pub fn update_set(input_tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_tokens as UpdateSetInput);

    if input.assignments.is_empty() {
        return quote! {
            compile_error!("the update set can't be empty");
        }
        .into();
    }

    let expr_generic_name_idents: Vec<_> = (0..input.assignments.len())
        .map(|i| {
            let generic_name_string = format!("E{}", i);
            proc_macro2::Ident::new(&generic_name_string, proc_macro2::Span::call_site())
        })
        .collect();
    let expr_generic_name_idents_ref = expr_generic_name_idents.as_slice();

    let expr_generics_definition = quote! {
        #(#expr_generic_name_idents_ref: ::gorm::sql::SqlExpression<T>),*
    };
    let expr_generics_definition_ref = &expr_generics_definition;

    let column_generic_name_idents: Vec<_> = (0..input.assignments.len())
        .map(|i| {
            let generic_name_string = format!("C{}", i);
            proc_macro2::Ident::new(&generic_name_string, proc_macro2::Span::call_site())
        })
        .collect();
    let column_generic_name_idents_ref = column_generic_name_idents.as_slice();

    let column_generics_definition = quote! {
        #(#column_generic_name_idents_ref: ::gorm::sql::Column<Table = T>),*
    };
    let column_generics_definition_ref = &column_generics_definition;

    let expr_field_name_idents: Vec<_> = (0..input.assignments.len())
        .map(|i| {
            let generic_name_string = format!("e{}", i);
            proc_macro2::Ident::new(&generic_name_string, proc_macro2::Span::call_site())
        })
        .collect();
    let expr_field_name_idents_ref = expr_field_name_idents.as_slice();

    let column_field_name_idents: Vec<_> = (0..input.assignments.len())
        .map(|i| {
            let generic_name_string = format!("_c{}", i);
            proc_macro2::Ident::new(&generic_name_string, proc_macro2::Span::call_site())
        })
        .collect();
    let column_field_name_idents_ref = column_field_name_idents.as_slice();

    let struct_fields_definition = quote! {
        #(#column_field_name_idents_ref: #column_generic_name_idents_ref),* ,
        #(#expr_field_name_idents_ref: #expr_generic_name_idents_ref),*
    };

    let initialize_struct_fields = {
        let columns = input
            .assignments
            .iter()
            .map(|assignment| &assignment.column_to_set);
        let exprs = input
            .assignments
            .iter()
            .map(|assignment| &assignment.new_value);
        quote! {
            #(#column_field_name_idents_ref: #columns),* ,
            #(#expr_field_name_idents_ref: #exprs),*
        }
    };

    let write_each_assignment = column_generic_name_idents_ref.iter().zip(expr_field_name_idents_ref).enumerate().map(|(i,(column_generic_name_ident, expr_field_name_ident))| {
        let is_last_item = i + 1 == input.assignments.len();
        let write_comma = if is_last_item {
            quote! {}
        } else {
            quote! {
                ::std::write!(f, ",")?;
            }
        };
        quote! {
            ::std::write!(f, "{} = ", <#column_generic_name_ident as ::gorm::sql::Column>::COLUMN_NAME)?;
            ::gorm::sql::SqlExpression::write_sql_string(&self.#expr_field_name_ident, f, parameter_binder)?;
            #write_comma
        }
    });

    quote! {
        {
            struct CustomUpdateSet<
                #column_generics_definition_ref,
                #expr_generics_definition_ref,
                T: ::gorm::sql::Table
            >{
                #struct_fields_definition
            }

            #[automatically_derived]
            impl<
                #column_generics_definition_ref,
                #expr_generics_definition_ref,
                T: ::gorm::sql::Table
            > ::gorm::sql::UpdateSet for CustomUpdateSet<
                #(#column_generic_name_idents_ref),* ,
                #(#expr_generic_name_idents_ref),* ,
                T
            > {
                type UpdateTable = T;

                fn write_sql_string<'s, 'a>(
                    &'s self,
                    f: &mut ::std::string::String,
                    parameter_binder: &mut ::gorm::sql::ParameterBinder<'a>,
                ) -> ::std::fmt::Result
                where
                    's: 'a
                {
                    use ::std::fmt::Write;

                    #(#write_each_assignment)*

                    Ok(())
                }
            }

            CustomUpdateSet { #initialize_struct_fields }
        }
    }
    .into()
}

// pub trait UpdateSet {
//     /// The table which this update set operates on.
//     type UpdateTable: Table;
//
//     /// Writes the update set as a comma seperated list of assignments to
// columns of the table.     fn write_sql_string<'s, 'a>(
//         &'s self,
//         f: &mut String,
//         parameter_binder: &mut ParameterBinder<'a>,
//     ) -> std::fmt::Result
//     where
//         's: 'a;
// }

/// A single assignment in the update set macro input
struct UpdateSetInputAssignment {
    column_to_set: Path,
    new_value: Expr,
}

impl Parse for UpdateSetInputAssignment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let column_to_set = input.parse()?;
        let _equals_sign: Token![=] = input.parse()?;
        let new_value = input.parse()?;
        Ok(Self {
            column_to_set,
            new_value,
        })
    }
}

/// An update set parsed macro input
struct UpdateSetInput {
    assignments: Punctuated<UpdateSetInputAssignment, Token![,]>,
}

impl Parse for UpdateSetInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            assignments: Punctuated::parse_terminated(input)?,
        })
    }
}
