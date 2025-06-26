//! # Factory Derive Macro
//!
//! This crate provides a derive macro for generating builder patterns with support for
//! relation factories, enabling complex object creation workflows.
//!
//! ## Basic Usage
//!
//! ```
//! use database::Persistable;
//! use derive_factory::Factory;
//!
//! #[derive(Default, Factory)]
//! struct Missile {
//!     max_range: u32,
//!     owner: String,
//!     target: String,
//! }
//!
//! impl database::Persistable for Missile {
//!     type Connection = ();
//!     type Error = ();
//!     
//!     async fn create(self, _pool: &Self::Connection) -> Result<Self, Self::Error> {
//!         Ok(self)
//!     }
//!
//!     async fn update(self, _pool: &Self::Connection) -> Result<Self, Self::Error> {
//!         Ok(self)
//!     }
//! }
//!
//!
//! # tokio_test::block_on( async {
//! let missile = Missile::factory()
//!     .owner("Wile E. Coyote".to_owned())
//!     .target("Road Runner".to_owned())
//!     .max_range(1000)
//!     .create(&())
//!     .await
//!     .unwrap();
//! # })
//! ```
//!
//! ## Relation Factories
//!
//! Use `#[factory(relation = "FactoryType")]` to create dependent objects:
//!
//! ```
//! use database::Persistable;
//! use derive_factory::Factory;
//!
//! #[derive(Default, Factory)]
//! struct Category {
//!     id: String,
//!     name: String,
//! }
//!
//! impl database::Persistable for Category {
//!     type Connection = ();
//!     type Error = ();
//!     
//!     async fn create(self, _pool: &Self::Connection) -> Result<Self, Self::Error> {
//!         Ok(self)
//!     }
//!
//!     async fn update(self, _pool: &Self::Connection) -> Result<Self, Self::Error> {
//!         Ok(self)
//!     }
//! }
//!
//! #[derive(Default, Factory)]
//! struct Missile {
//!     name: String,
//!     #[factory(relation = "CategoryFactory")]
//!     category_id: String,
//! }
//!
//! impl database::Persistable for Missile {
//!     type Connection = ();
//!     type Error = ();
//!     
//!     async fn create(self, _pool: &Self::Connection) -> Result<Self, Self::Error> {
//!         Ok(self)
//!     }
//!
//!     async fn update(self, _pool: &Self::Connection) -> Result<Self, Self::Error> {
//!         Ok(self)
//!     }
//! }
//!
//! # tokio_test::block_on( async {
//! let missile = Missile::factory()
//!     .name("ACME Rocket".to_string())
//!     .for_category_with(|factory| {
//!         factory
//!             .name("Explosive".to_string())
//!             .id("cat-001".to_string())
//!     })
//!     .create(&())
//!     .await
//!     .unwrap();
//! # })
//! ```
extern crate database;
extern crate proc_macro;
mod relation;
use proc_macro::TokenStream;
use quote::quote;
use relation::Relation;
use syn::{DeriveInput, Field, Token, parse_macro_input, punctuated::Punctuated};

/// Derives a builder factory pattern for structs with support for relation factories.
///
/// # Generated Code
///
/// This macro generates:
/// - A `{StructName}Factory` struct with all fields wrapped in `Option<T>`
/// - A `factory()` constructor method on the original struct
/// - Builder methods for each field that consume and return `Self`
/// - Additional factory fields for relations (strips `_id` suffix from field names)
/// - A `create()` method that consumes the factory and returns `Result<StructName, Box<dyn Error>>`
///
/// # Attributes
///
/// `#[factory(relation = "FactoryType")]` - Marks a field as having a relation factory.
/// Field names ending with `_id` will have that suffix stripped for the factory field name.
///
/// # Requirements
///
/// - Must be a struct with named fields (not tuple or unit structs)
/// - All field types must implement `Default`
/// - Relation factory types must be in scope and implement the factory pattern
///
/// # Performance
///
/// - Factory creation is zero-cost (all fields initialize to `None`)
/// - Builder methods use move semantics for efficient chaining
/// - The `create()` method consumes the factory to prevent accidental reuse
#[proc_macro_derive(Factory, attributes(factory))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let factory_name = format!("{}Factory", name);
    let factory_ident = syn::Ident::new(&factory_name, name.span());

    // Extract and validate fields
    let fields = match extract_fields(&ast) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error().into(),
    };

    // Extract relation fields
    let relation_fields = extract_relation_fields(fields);

    // Generate different parts using helper functions
    let factory_fields = generate_factory_fields(fields, relation_fields.iter());
    let factory_methods = generate_factory_methods(fields);
    let factory_empty = generate_factory_empty(fields, relation_fields.iter());
    let create_fields = generate_create_fields(fields);
    let relation_factory_methods = generate_relation_factory_methods(relation_fields.iter());
    let default_relation_factory_methods =
        generate_default_relation_factory_methods(relation_fields.iter());
    let relation_creation = generate_relation_creation(relation_fields.iter());

    let expanded = quote! {
        pub struct #factory_ident
        {
            #(#factory_fields,)*
        }

        impl #name {
            pub fn factory() -> #factory_ident {
                #factory_ident::new()
            }
        }

        impl #factory_ident
        where
            #name: Default + database::Persistable,
        {
            pub fn new() -> Self {
                Self {
                    #(#factory_empty,)*
                }
            }

            #(#factory_methods)*
            #(#relation_factory_methods)*
            #(#default_relation_factory_methods)*

            pub async fn create(mut self, connection: &<#name as database::Persistable>::Connection) -> Result<#name, <#name as database::Persistable>::Error>
            {
                #(#relation_creation)*

                let model = #name {
                    #(#create_fields,)*
                };

                model.create(connection).await
            }
        }
    };

    expanded.into()
}

/// Extract and validate fields from the struct.
fn extract_fields(ast: &DeriveInput) -> Result<&Punctuated<Field, Token![,]>, syn::Error> {
    match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => Ok(named),
        syn::Data::Struct(_) => Err(syn::Error::new_spanned(
            ast,
            "Factory can only be derived for structs with named fields",
        )),
        _ => Err(syn::Error::new_spanned(
            ast,
            "Factory can only be derived for structs",
        )),
    }
}

/// Extract relation fields from struct fields.
fn extract_relation_fields(fields: &Punctuated<Field, Token![,]>) -> Vec<Relation> {
    fields
        .iter()
        .filter_map(|field| {
            for attr in &field.attrs {
                if attr.path().is_ident("factory") {
                    // Combine the nested if let statements
                    if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args::<syn::Meta>() {
                        if nv.path.is_ident("relation") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = nv.value
                            {
                                return Some(Relation::new(field, lit_str.value()));
                            }
                        }
                    }
                }
            }
            None
        })
        .collect()
}

/// Generate factory struct fields (both regular and relation factory fields).
fn generate_factory_fields<'a>(
    fields: &'a Punctuated<Field, Token![,]>,
    relation_fields: impl Iterator<Item = &'a Relation> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    let regular_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: std::option::Option<#ty> }
    });

    let relation_factory_fields = relation_fields.map(|relation| {
        let factory_field_name = &relation.factory_field_ident;
        match relation.factory_type_ident() {
            Ok(factory_type_ident) => quote! {
                #factory_field_name: std::option::Option<Box<dyn FnOnce(#factory_type_ident) -> #factory_type_ident + Send>>
            },
            Err(error) => error.to_compile_error(),
        }
    });

    regular_fields.chain(relation_factory_fields)
}

/// Generate builder methods for regular fields.
fn generate_factory_methods(
    fields: &Punctuated<Field, Token![,]>,
) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(mut self, #name: #ty) -> Self {
                self.#name = Some(#name);
                self
            }
        }
    })
}

/// Generate factory initialization (all fields set to None).
fn generate_factory_empty<'a>(
    fields: &'a Punctuated<Field, Token![,]>,
    relations: impl Iterator<Item = &'a Relation> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    let regular_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: None }
    });

    let factory_fields = relations.map(|relation| {
        let factory_field_name = &relation.factory_field_ident;
        quote! {
            #factory_field_name: None
        }
    });

    regular_fields.chain(factory_fields)
}

/// Generate field assignments for the create method.
fn generate_create_fields(
    fields: &Punctuated<Field, Token![,]>,
) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: self.#name.unwrap_or_else(|| <#ty as Default>::default())
        }
    })
}

/// Generate relation factory methods.
fn generate_relation_factory_methods<'a>(
    relations: impl Iterator<Item = &'a Relation> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    relations.map(|relation| {
        let factory_field_name = &relation.factory_field_ident;
        match relation.factory_type_ident() {
            Ok(factory_type_ident) => {
                let method_name = syn::Ident::new(
                    &format!("for_{}_with", relation.relation_name),
                    proc_macro2::Span::call_site(),
                );
                quote! {
                    pub fn #method_name<F>(mut self, configure: F) -> Self
                    where
                        F: FnOnce(#factory_type_ident) -> #factory_type_ident + Send + 'static,
                    {
                        self.#factory_field_name = Some(Box::new(configure));
                        self
                    }
                }
            }
            Err(error) => error.to_compile_error(),
        }
    })
}

/// Generate relation factory methods.
fn generate_default_relation_factory_methods<'a>(
    relations: impl Iterator<Item = &'a Relation> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    relations.map(|relation| {
        let factory_field_name = &relation.factory_field_ident;
        let method_name = syn::Ident::new(
            &format!("for_default_{}", relation.relation_name),
            proc_macro2::Span::call_site(),
        );
        quote! {
            pub fn #method_name(mut self) -> Self
            {
                self.#factory_field_name = Some(Box::new(|factory| factory));
                self
            }
        }
    })
}

/// Generate relation creation logic for the create method.
fn generate_relation_creation<'a>(
    relations: impl Iterator<Item = &'a Relation> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    relations.map(|relation| {
        let factory_field_name = &relation.factory_field_ident;
        match relation.factory_type_ident() {
            Ok(factory_type_ident) => {
                let original_field_name = &relation.field.ident;
                quote! {
                    if let Some(factory_fn) = self.#factory_field_name {
                        let factory = #factory_type_ident::new();
                        let factory = factory_fn(factory);
                        let model = factory.create(connection).await?;
                        self.#original_field_name = Some(model.id);
                    }
                }
            }
            Err(error) => error.to_compile_error(),
        }
    })
}
