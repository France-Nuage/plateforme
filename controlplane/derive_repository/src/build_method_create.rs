use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, punctuated::Punctuated, token::Comma};

use crate::build_sqlx_macro_parameters::build_sqlx_macro_parameters;

/// Build the create method for the repository.
pub fn build_method_create(table_name: &str, fields: &Punctuated<Field, Comma>) -> TokenStream {
    // Compute the sql column names for the query
    let column_names = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    // Compute the sql parameters for the query
    let sql_parameters = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .enumerate()
        .map(|(index, _)| format!("${}", index + 1))
        .collect::<Vec<String>>()
        .join(", ");

    // Compute the actual SQL query
    let query = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
        table_name, column_names, sql_parameters, column_names
    );

    // Compute the parameters for the sqlx macro
    let parameters = build_sqlx_macro_parameters(fields);

    quote! {
        /// Create a new record in the database.
        async fn create(self, executor: &Self::Connection) -> Result<Self, Self::Error> {
            sqlx::query_as!(
                Self,
                #query,
                #(#parameters),*
            ).fetch_one(executor).await
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_method_create::build_method_create, extract_fields::extract_fields};
    use quote::quote;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_build_method_create_works() {
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
        let stream = build_method_create("missiles", fields);

        // Assert the stream produces the expected tokens
        let expected = quote! {
            /// Create a new record in the database.
            async fn create(self, executor: &Self::Connection) -> Result<Self, Self::Error> {
                sqlx::query_as!(
                    Self,
                    "INSERT INTO missiles (id, blast_radius, damage) VALUES ($1, $2, $3) RETURNING id, blast_radius, damage",
                    self.id,
                    self.blast_radius,
                    self.damage
                ).fetch_one(executor).await
            }
        };
        assert_eq!(stream.to_string(), expected.to_string());
    }

    #[test]
    fn test_handle_sqlx_try_from() {
        // Arrange an input with an sqlx::try_from
        let input: DeriveInput = parse_quote! {
            struct Missile {
                #[repository(primary)]
                id: String,
                #[sqlx(try_from = "String")]
                launch_status: LaunchStatus
            }
        };
        let fields = extract_fields(&input).unwrap();

        // Act the call to the function
        let stream = build_method_create("missiles", fields);

        // Assert the stream produces the expected tokens
        let expected = quote! {
            /// Create a new record in the database.
            async fn create(self, executor: &Self::Connection) -> Result<Self, Self::Error> {
                sqlx::query_as!(
                    Self,
                    "INSERT INTO missiles (id, launch_status) VALUES ($1, $2) RETURNING id, launch_status",
                    self.id,
                    String::from(self.launch_status)
                ).fetch_one(executor).await
            }
        };
        assert_eq!(stream.to_string(), expected.to_string());
    }
}
