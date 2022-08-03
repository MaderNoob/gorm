use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, Token};

pub fn migration(input_tokens: TokenStream) -> TokenStream {
    let migration_input = parse_macro_input!(input_tokens as MigrationInput);
    let MigrationInput {
        struct_name,
        table_names,
    } = migration_input;

    let create_tables = table_names.iter().map(|table_name| {
        quote! {
            #table_name::table
                .create()
                .execute(executor)
                .await?;
        }
    });

    // note that we must reverse the order here because we must first remove the tables that depend
    // on others, unlike when creating the tables where we must create the tables that depend on
    // others last.
    let drop_tables = table_names.iter().rev().map(|table_name| {
        quote! {
            #table_name::table
                .drop()
                .if_exists()
                .execute(executor)
                .await?;
        }
    });

    quote!{
        #[::gorm::async_trait]
        impl ::gorm::sql::Migration for #struct_name{
            async fn up<E: ::gorm::execution::SqlStatementExecutor + ::std::marker::Send + std::marker::Sync>(executor: &E) -> ::gorm::Result<()>{
                #(#create_tables)*
                Ok(())
            }

            async fn down<E: ::gorm::execution::SqlStatementExecutor + ::std::marker::Send + std::marker::Sync>(executor: &E) -> ::gorm::Result<()>{
                #(#drop_tables)*
                Ok(())
            }
        }
    }.into()
}

struct MigrationInput {
    struct_name: proc_macro2::Ident,
    table_names: TableNames,
}

type TableNames = Punctuated<proc_macro2::Ident, Token![,]>;

impl Parse for MigrationInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let struct_name: proc_macro2::Ident = input.parse()?;
        let _arrow: Token![=>] = input.parse()?;
        let table_names: TableNames = TableNames::parse_terminated(input)?;
        Ok(Self {
            struct_name,
            table_names,
        })
    }
}
