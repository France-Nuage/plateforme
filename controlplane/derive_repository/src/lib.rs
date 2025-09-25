use proc_macro::TokenStream;

mod build_method_create;
mod build_method_list;
mod build_method_update;
mod build_sqlx_macro_parameters;
mod builder;
mod error;
mod extract_field_converter;
mod extract_fields;
mod extract_primary_idents;
mod extract_table_name;
use builder::RepositoryBuilder;
use syn::parse_macro_input;

#[proc_macro_derive(Repository, attributes(repository, table))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    RepositoryBuilder::new(ast)
        .build()
        .unwrap_or_else(|err| Into::<syn::Error>::into(err).to_compile_error())
        .into()
}
