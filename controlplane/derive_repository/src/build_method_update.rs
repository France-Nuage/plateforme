use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, punctuated::Punctuated, token::Comma};

use crate::build_sqlx_macro_parameters::build_sqlx_macro_parameters;

/// Build the update method for the repository.
pub fn build_method_update(
    table_name: &str,
    fields: &Punctuated<Field, Comma>,
    primary_idents: Vec<&Ident>,
) -> TokenStream {
    // Compute sql the column names from the query
    let column_names = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    // Compute the sql parameter for the sql query
    let sql_parameters = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .filter(|ident| !primary_idents.contains(ident))
        .enumerate()
        .map(|(index, ident)| format!("{} = ${}", ident, index + 2))
        .collect::<Vec<String>>()
        .join(", ");

    // Compute the actual SQL query
    let query = format!(
        "UPDATE {} SET {} WHERE id = $1 RETURNING {}",
        table_name, sql_parameters, column_names
    );

    // Compute the parameters for the sqlx macro
    let parameters = build_sqlx_macro_parameters(fields);

    quote! {
        /// Update an existing record in the database.
        async fn update(self, executor: &Self::Connection) -> Result<Self, Self::Error> {
            sqlx::query_as!(
                Self,
                #query,
                #(#parameters), *
            ).fetch_one(executor).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{extract_fields::extract_fields, extract_primary_idents::extract_primary_idents};
    use quote::quote;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_build_method_update_works() {
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

        let primary_idents = extract_primary_idents(fields);

        // Act the call to the function
        let stream = build_method_update("missiles", fields, primary_idents);

        // Assert the stream produces the expected tokens
        let expected = quote! {
            /// Update an existing record in the database.
            async fn update(self, executor: &Self::Connection) -> Result<Self, Self::Error> {
                sqlx::query_as!(
                    Self,
                    "UPDATE missiles SET blast_radius = $2, damage = $3 WHERE id = $1 RETURNING id, blast_radius, damage",
                    self.id,
                    self.blast_radius,
                    self.damage
                ).fetch_one(executor).await
            }
        };
        assert_eq!(stream.to_string(), expected.to_string());
    }
}
