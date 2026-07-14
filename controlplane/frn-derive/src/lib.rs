use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Resource, attributes(resource))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let stream = make_derive(ast);
    TokenStream::from(stream)
}

fn find_resource_id_field(fields: &syn::FieldsNamed) -> (&syn::Ident, &syn::Type) {
    for field in &fields.named {
        for attr in &field.attrs {
            if attr.path().is_ident("resource") {
                let has_id = attr
                    .parse_args::<syn::Ident>()
                    .map(|ident| ident == "id")
                    .unwrap_or(false);
                if has_id {
                    let ident = field.ident.as_ref().expect("named field must have ident");
                    return (ident, &field.ty);
                }
            }
        }
    }

    let field = fields
        .named
        .iter()
        .find(|f| f.ident.as_ref().map(|i| i == "id").unwrap_or(false))
        .expect("Resource derive requires a field named 'id' or annotated with #[resource(id)]");
    (field.ident.as_ref().unwrap(), &field.ty)
}

fn make_derive(input: DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &input.ident;
    let resource_name = struct_ident.to_string().to_snake_case();

    let companion_ident =
        syn::Ident::new(&format!("{}Resource", struct_ident), struct_ident.span());

    let (id_field_ident, id_type) = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => find_resource_id_field(fields),
            _ => panic!("Resource derive only supports structs with named fields"),
        },
        _ => panic!("Resource derive only supports structs"),
    };

    let companion_field = syn::Ident::new("_id", struct_ident.span());

    quote! {
        pub struct #companion_ident {
            #companion_field: #id_type,
        }

        impl frn_core::authorization::Resource for #companion_ident {
            type Id = #id_type;
            const RESOURCE_NAME: &'static str = #resource_name;

            #[allow(refining_impl_trait)]
            fn some(id: Self::Id) -> #companion_ident {
                #companion_ident { #companion_field: id }
            }

            fn id(&self) -> &Self::Id {
                &self.#companion_field
            }

            fn name(&self) -> &'static str {
                Self::RESOURCE_NAME
            }
        }

        impl frn_core::authorization::Resource for #struct_ident {
            type Id = #id_type;
            const RESOURCE_NAME: &'static str = #resource_name;

            #[allow(refining_impl_trait)]
            fn some(id: Self::Id) -> #companion_ident {
                #companion_ident::some(id)
            }

            fn id(&self) -> &Self::Id {
                &self.#id_field_ident
            }

            fn name(&self) -> &'static str {
                Self::RESOURCE_NAME
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_derive_with_id_field() {
        let input: DeriveInput = parse_quote! {
            struct Anvil {
                id: String,
            }
        };

        let output = make_derive(input);

        let expected = quote! {
            pub struct AnvilResource {
                _id: String,
            }

            impl frn_core::authorization::Resource for AnvilResource {
                type Id = String;
                const RESOURCE_NAME: &'static str = "anvil";

                #[allow(refining_impl_trait)]
                fn some(id: Self::Id) -> AnvilResource {
                    AnvilResource { _id: id }
                }

                fn id(&self) -> &Self::Id {
                    &self._id
                }

                fn name(&self) -> &'static str {
                    Self::RESOURCE_NAME
                }
            }

            impl frn_core::authorization::Resource for Anvil {
                type Id = String;
                const RESOURCE_NAME: &'static str = "anvil";

                #[allow(refining_impl_trait)]
                fn some(id: Self::Id) -> AnvilResource {
                    AnvilResource::some(id)
                }

                fn id(&self) -> &Self::Id {
                    &self.id
                }

                fn name(&self) -> &'static str {
                    Self::RESOURCE_NAME
                }
            }
        };
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_derive_with_resource_id_attribute() {
        let input: DeriveInput = parse_quote! {
            struct Forge {
                #[resource(id)]
                slug: String,
                name: String,
            }
        };

        let output = make_derive(input);

        let expected = quote! {
            pub struct ForgeResource {
                _id: String,
            }

            impl frn_core::authorization::Resource for ForgeResource {
                type Id = String;
                const RESOURCE_NAME: &'static str = "forge";

                #[allow(refining_impl_trait)]
                fn some(id: Self::Id) -> ForgeResource {
                    ForgeResource { _id: id }
                }

                fn id(&self) -> &Self::Id {
                    &self._id
                }

                fn name(&self) -> &'static str {
                    Self::RESOURCE_NAME
                }
            }

            impl frn_core::authorization::Resource for Forge {
                type Id = String;
                const RESOURCE_NAME: &'static str = "forge";

                #[allow(refining_impl_trait)]
                fn some(id: Self::Id) -> ForgeResource {
                    ForgeResource::some(id)
                }

                fn id(&self) -> &Self::Id {
                    &self.slug
                }

                fn name(&self) -> &'static str {
                    Self::RESOURCE_NAME
                }
            }
        };
        assert_eq!(output.to_string(), expected.to_string());
    }
}
