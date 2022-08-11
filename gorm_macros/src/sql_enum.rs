use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

pub fn sql_enum(input_tokens: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);
    let data_enum = match &derive_input.data {
        Data::Enum(data_enum) => data_enum,
        _ => {
            return quote_spanned!{
                derive_input.span() => compile_error!("sql enum can only be implemented for enum types");
            }.into();
        },
    };

    if derive_input.generics.lt_token.is_some() {
        return quote_spanned! {
            derive_input.generics.span() => compile_error!("sql enums can't have generic parameters");
        }.into();
    }

    for variant in &data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return quote_spanned! {
                variant.fields.span() => compile_error!("sql enum variants can't contain data");
            }
            .into();
        }

        if variant.discriminant.is_some() {
            return quote_spanned!{
                variant.span() => compile_error!("sql enums don't support specifying custom values for enum variants");
            }.into();
        }
    }

    let variants_amount = data_enum.variants.len();
    if variants_amount == 0 {
        return quote_spanned! {
            derive_input.span() => compile_error!("sql enums can't be empty");
        }
        .into();
    }
    let highest_variant_value = variants_amount - 1;
    let integer_type = if let Ok(_) = i16::try_from(highest_variant_value) {
        quote! {i16}
    } else if let Ok(_) = i32::try_from(highest_variant_value) {
        quote! {i32}
    } else {
        quote! {i64}
    };

    let enum_ident = &derive_input.ident;

    let enum_name_string = enum_ident.to_string();

    let variant_list = data_enum.variants.iter().map(|variant| {
        let ident = &variant.ident;
        quote! {
            Self::#ident
        }
    });

    quote! {
        #[automatically_derived]
        impl ::gorm::sql::SqlEnum for #enum_ident{
            type IntegerType = #integer_type;

            const ENUM_NAME: &'static str = #enum_name_string;

            const VARIANTS_IN_ORDER: &'static [Self] = &[
                #(#variant_list),*
            ];

            fn to_integer(self) -> Self::IntegerType{
                self as #integer_type
            }
        }

        #[automatically_derived]
        impl ::gorm::sql::IntoSqlType for #enum_ident{
            type SqlType = <#integer_type as ::gorm::sql::IntoSqlType>::SqlType;
        }

        #[automatically_derived]
        impl<'a> ::gorm::tokio_postgres::types::FromSql<'a> for #enum_ident{
            fn from_sql(
                ty: &::gorm::tokio_postgres::types::Type,
                raw: &'a [u8]
            ) -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Sync + ::std::marker::Send + 'static>>{
                Ok(<Self as ::gorm::sql::SqlEnum>::from_integer(
                    <#integer_type as ::gorm::tokio_postgres::types::FromSql<'a>>::from_sql(ty,raw)?
                )?)
            }
            fn accepts(ty: &::gorm::tokio_postgres::types::Type) -> bool{
                <#integer_type as ::gorm::tokio_postgres::types::FromSql<'a>>::accepts(ty)
            }

            fn from_sql_null(
                ty: &::gorm::tokio_postgres::types::Type
            ) -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Sync + ::std::marker::Send + 'static>>{
                Ok(<Self as ::gorm::sql::SqlEnum>::from_integer(
                    <#integer_type as ::gorm::tokio_postgres::types::FromSql<'a>>::from_sql_null(ty)?
                )?)
            }
            fn from_sql_nullable(
                ty: &::gorm::tokio_postgres::types::Type,
                raw: ::std::option::Option<&'a [u8]>
            ) -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Sync + ::std::marker::Send + 'static>>{
                Ok(<Self as ::gorm::sql::SqlEnum>::from_integer(
                    <#integer_type as ::gorm::tokio_postgres::types::FromSql<'a>>::from_sql_nullable(ty, raw)?
                )?)
            }
        }

        #[automatically_derived]
        impl ::gorm::tokio_postgres::types::ToSql for #enum_ident{
            fn to_sql(
                &self,
                ty: &::gorm::tokio_postgres::types::Type,
                out: &mut ::gorm::bytes::BytesMut
            ) -> ::std::result::Result<::gorm::tokio_postgres::types::IsNull, ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Sync + ::std::marker::Send + 'static>>{
                <#integer_type as ::gorm::tokio_postgres::types::ToSql>::to_sql(&<Self as ::gorm::sql::SqlEnum>::to_integer(self.clone()), ty, out)
            }
            fn accepts(ty: &::gorm::tokio_postgres::types::Type) -> bool{
                <#integer_type as ::gorm::tokio_postgres::types::ToSql>::accepts(ty)
            }
            fn to_sql_checked(
                &self,
                ty: &::gorm::tokio_postgres::types::Type,
                out: &mut ::gorm::bytes::BytesMut
            ) -> ::std::result::Result<::gorm::tokio_postgres::types::IsNull, ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Sync + ::std::marker::Send + 'static>>{
                <#integer_type as ::gorm::tokio_postgres::types::ToSql>::to_sql_checked(&<Self as ::gorm::sql::SqlEnum>::to_integer(self.clone()), ty, out)
            }
        }

        #[automatically_derived]
        impl<S: ::gorm::sql::SelectableTables> ::gorm::sql::SqlExpression<S> for #enum_ident
        {
            type RustType = #integer_type;
            type SqlType = <#integer_type as ::gorm::sql::IntoSqlType>::SqlType;

            const IS_AGGREGATE: bool = false;

            fn write_sql_string<'s, 'a>(
                &'s self,
                f: &mut ::std::string::String,
                parameter_binder: &mut ::gorm::sql::ParameterBinder<'a>,
            ) -> ::std::fmt::Result
            where
                's: 'a,
            {
                use ::std::fmt::Write;

                ::std::write!(
                    f,
                    "{}",
                    parameter_binder.bind_parameter(self)
                )
            }
        }

    }
    .into()
}
