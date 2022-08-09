use quote::quote;
use syn::Type;

pub fn generate_field_name_cons_list_type(field_name: &str) -> proc_macro2::TokenStream {
    // start with the inner most type and wrap it each time with each character.
    let mut cur = quote! { ::gorm::util::TypedConsListNil };

    for chr in field_name.chars().rev() {
        cur = quote! {
            ::gorm::sql::FieldNameCharsConsListCons<#chr, #cur>
        };
    }

    cur
}

pub fn generate_fields_cons_list_type<'a>(
    fields: impl Iterator<Item = (&'a Option<proc_macro2::Ident>, &'a Type)> + DoubleEndedIterator,
) -> proc_macro2::TokenStream {
    // start with the inner most type and wrap it each time with each field.
    let mut cur = quote! { ::gorm::util::TypedConsListNil };

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
