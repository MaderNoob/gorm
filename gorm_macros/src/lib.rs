mod from_query_result;
mod migration;
mod select_values;
mod selected_value_to_order_by;
mod table;
mod util;

use proc_macro::TokenStream;
use util::generate_field_name_cons_list_type;

#[proc_macro]
pub fn select_values(input_tokens: TokenStream) -> TokenStream {
    select_values::select_values(input_tokens)
}

#[proc_macro]
pub fn migration(input_tokens: TokenStream) -> TokenStream {
    migration::migration(input_tokens)
}

#[proc_macro_derive(FromQueryResult)]
pub fn from_query_result(input_tokens: TokenStream) -> TokenStream {
    from_query_result::from_query_result(input_tokens)
}

#[proc_macro]
pub fn create_field_name_cons_list(item: TokenStream) -> TokenStream {
    generate_field_name_cons_list_type(&item.to_string()).into()
}

#[proc_macro_derive(Table, attributes(table))]
pub fn table(input_tokens: TokenStream) -> TokenStream {
    table::table(input_tokens)
}

#[proc_macro]
pub fn selected_value_to_order_by(input_tokens: TokenStream) -> TokenStream {
    selected_value_to_order_by::selected_value_to_order_by(input_tokens)
}
