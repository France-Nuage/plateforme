use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::{
    build_method_create::build_method_create, build_method_update::build_method_update,
    error::Error, extract_fields::extract_fields, extract_primary_idents::extract_primary_idents,
    extract_table_name::extract_table_name,
};

pub struct RepositoryBuilder {
    pub input: DeriveInput,
}

impl RepositoryBuilder {
    // Build the stream, consuming the builder
    pub fn build(self) -> Result<TokenStream, Error> {
        let model_name = &self.input.ident;
        let fields = extract_fields(&self.input)?;

        let primary_idents = extract_primary_idents(fields);

        let table_name = extract_table_name(&self.input);
        let method_create = build_method_create(&table_name, fields);
        let method_update = build_method_update(&table_name, fields, primary_idents);

        Ok(quote! {
            impl database::Persistable for #model_name {
                type Connection = sqlx::PgPool;
                type Error = sqlx::Error;

                #method_create

                #method_update
            }
        })
    }

    /// Create a new RepositoryBuilder instance
    pub fn new(input: DeriveInput) -> Self {
        Self { input }
    }
}

#[cfg(test)]
mod tests {
    use super::RepositoryBuilder;
    use quote::quote;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_the_repository_builder_builds() {
        // Arrange an input stream
        let input: DeriveInput = parse_quote! {
            struct Missile {
                #[repository(primary)]
                id: String,
                blast_radius: u8,
                damage: u8,
            }
        };

        // Act a call to the builder
        let result = RepositoryBuilder::new(input).build();

        // Assert the result
        assert!(result.is_ok());
        let expected = quote! {
            impl database::Persistable for Missile {
                type Connection = sqlx::PgPool;
                type Error = sqlx::Error;

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
            }
        };
        assert_eq!(result.unwrap().to_string(), expected.to_string());
    }
}
