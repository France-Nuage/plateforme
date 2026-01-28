use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Meta, punctuated::Punctuated, token::Comma};

use crate::build_sqlx_macro_parameters::build_sqlx_macro_parameters;

/// Check if a field has the `#[fabrique(soft_delete)]` attribute
fn is_soft_delete_field(field: &Field) -> bool {
    for attr in &field.attrs {
        if attr.path().is_ident("fabrique") {
            if let Meta::List(meta_list) = &attr.meta {
                if let Ok(nested_metas) = meta_list.parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated) {
                    for meta in nested_metas {
                        if let Meta::Path(path) = meta {
                            if path.is_ident("soft_delete") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Build the create method for the repository.
pub fn build_method_create(table_name: &str, fields: &Punctuated<Field, Comma>) -> TokenStream {
    // Filter out soft_delete fields
    let mut non_soft_delete_fields = Punctuated::<Field, Comma>::new();
    for field in fields.iter() {
        if !is_soft_delete_field(field) {
            non_soft_delete_fields.push(field.clone());
        }
    }

    // Compute the sql column names for the query (excluding soft_delete fields)
    let column_names = non_soft_delete_fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    // Compute the sql parameters for the query
    let sql_parameters = non_soft_delete_fields
        .iter()
        .enumerate()
        .map(|(index, _)| format!("${}", index + 1))
        .collect::<Vec<String>>()
        .join(", ");

    // Compute the actual SQL query
    let query = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
        table_name, column_names, sql_parameters, column_names
    );

    // Compute the parameters for the sqlx macro (only for non-soft_delete fields)
    let parameters = build_sqlx_macro_parameters(&non_soft_delete_fields);

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

    #[test]
    fn test_soft_delete_field_is_excluded() {
        // Arrange an input with a soft_delete field
        let input: DeriveInput = parse_quote! {
            struct Instance {
                #[repository(primary)]
                id: String,
                name: String,
                #[fabrique(soft_delete)]
                deleted_at: Option<chrono::DateTime<chrono::Utc>>,
            }
        };
        let fields = extract_fields(&input).unwrap();

        // Act the call to the function
        let stream = build_method_create("instances", fields);

        // Assert the stream produces the expected tokens (soft_delete field excluded)
        let expected = quote! {
            /// Create a new record in the database.
            async fn create(self, executor: &Self::Connection) -> Result<Self, Self::Error> {
                sqlx::query_as!(
                    Self,
                    "INSERT INTO instances (id, name) VALUES ($1, $2) RETURNING id, name",
                    self.id,
                    self.name
                ).fetch_one(executor).await
            }
        };
        assert_eq!(stream.to_string(), expected.to_string());
    }
}
