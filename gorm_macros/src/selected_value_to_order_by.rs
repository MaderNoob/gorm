use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::util::generate_field_name_cons_list_type;

pub fn selected_value_to_order_by(input_tokens: TokenStream) -> TokenStream {
    let selected_value_ident = parse_macro_input!(input_tokens as proc_macro2::Ident);
    let selected_value_name_string = selected_value_ident.to_string();

    let field_name_type = generate_field_name_cons_list_type(&selected_value_name_string);

    quote! {
        {
            struct CustomSelectedValueToOrderBy;

            impl ::gorm::statements::SelectedValueToOrderBy for CustomSelectedValueToOrderBy {
                type Name = #field_name_type;
                const NAME_STR: &'static str = #selected_value_name_string;
            }

            CustomSelectedValueToOrderBy
        }
    }
    .into()
}
