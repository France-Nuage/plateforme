use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, punctuated::Punctuated, spanned::Spanned, token::Comma};

use crate::extract_field_converter::extract_field_converter;

/// Build sqlx macro parameters from a set of fields
pub fn build_sqlx_macro_parameters(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    // Compute the parameters for the sqlx macro
    fields
        .iter()
        .filter_map(|field| {
            // Map the ident into a String, which allows for conversion handling
            field
                .ident
                .as_ref()
                .map(|ident| match extract_field_converter(field) {
                    // If there is a literal converter, apply it
                    Some(literal) => {
                        let type_ident = Ident::new(&literal.value(), field.span());
                        quote! { #type_ident::from(self.#ident) }
                    }
                    None => quote! { self.#ident },
                })
        })
        .collect()
}
