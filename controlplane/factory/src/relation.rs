//! # Relation Module
//!
//! This module handles relation field processing for the Factory derive macro.
//! Relations allow factory fields to reference other factory types, enabling
//! complex object composition patterns.

use syn::Field;

/// Information about a relation field and its generated factory field.
#[derive(Debug, Clone)]
pub struct Relation {
    /// The original relation field ident.
    pub field: Field,
    /// The generated factory field identifier (e.g., "category_factory")
    pub factory_field_ident: syn::Ident,
    /// The factory type specified in the attribute (e.g., "CategoryFactory")
    pub factory_type: String,
    /// The relation name.
    pub relation_name: String,
}

impl Relation {
    /// Create a new Relation from a field and its relation data.
    pub fn new(field: &Field, factory_type: String) -> Self {
        let field_name = format!("{}", field.ident.as_ref().unwrap());
        let relation_name = field_name
            .strip_suffix("_id")
            .unwrap_or(&field_name)
            .to_string();

        let factory_field_ident = syn::Ident::new(
            &format!("{}_factory", relation_name),
            proc_macro2::Span::call_site(),
        );

        Self {
            field: field.clone(),
            relation_name,
            factory_field_ident,
            factory_type,
        }
    }

    /// Get the factory type as a syn::Ident.
    pub fn factory_type_ident(&self) -> syn::Ident {
        syn::parse_str(&self.factory_type).unwrap_or_else(|_| {
            syn::Ident::new("InvalidFactoryType", proc_macro2::Span::call_site())
        })
    }
}
