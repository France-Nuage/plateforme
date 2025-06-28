use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, punctuated::Punctuated, token::Comma};

/// Build the list method for the repository.
pub fn build_method_list(table_name: &str, fields: &Punctuated<Field, Comma>) -> TokenStream {
    // Compute the sql column names for the query
    let column_names = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    // Compute the actual SQL query
    let query = format!("SELECT {} FROM {}", column_names, table_name);
    quote! {
        /// List all existing records in the database.
        async fn list(executor: &Self::Connection) -> Result<Vec<Self>, Self::Error> {
            sqlx::query_as!(Self, #query).fetch_all(executor).await
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use crate::extract_fields::extract_fields;

    use super::*;

    #[test]
    fn test_build_method_list_works() {
        // Arrange a set of fields
        let input: DeriveInput = parse_quote! {
            struct Missile {
                #[repository(primary)]
                id: String,
                blast_radius: u8,
                damage: u8,
            }
        };
        let fields = extract_fields(&input).unwrap();

        // Act the call to the function
        let stream = build_method_list("missiles", fields);

        // Assert the stream produces the expected tokens
        let expected = quote! {
            /// List all existing records in the database.
            async fn list(executor: &Self::Connection) -> Result<Vec<Self>, Self::Error> {
                sqlx::query_as!(
                    Self,
                    "SELECT id, blast_radius, damage FROM missiles"
                ).fetch_all(executor).await
            }
        };
        assert_eq!(stream.to_string(), expected.to_string());
    }
}
