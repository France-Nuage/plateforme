use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Resource)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let stream = make_derive(ast);
    TokenStream::from(stream)
}

fn make_derive(input: DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &input.ident;
    let resource_name = struct_ident.to_string().to_snake_case();

    // Generate the companion struct name: [StructName]Resource
    let companion_ident =
        syn::Ident::new(&format!("{}Resource", struct_ident), struct_ident.span());

    // Extract the type of the `id` field
    let id_type = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .find(|f| f.ident.as_ref().map(|i| i == "id").unwrap_or(false))
                .map(|f| &f.ty)
                .expect("Resource derive requires an 'id' field"),
            _ => panic!("Resource derive only supports structs with named fields"),
        },
        _ => panic!("Resource derive only supports structs"),
    };

    quote! {
        pub struct #companion_ident {
            id: #id_type,
        }

        impl frn_core::authorization::Resource for #companion_ident {
            type Id = #id_type;
            const RESOURCE_NAME: &'static str = #resource_name;

            #[allow(refining_impl_trait)]
            fn some(id: Self::Id) -> #companion_ident {
                #companion_ident { id }
            }

            fn id(&self) -> &Self::Id {
                &self.id
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
                &self.id
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
    fn test_derive() {
        // Arrange the test
        let input: DeriveInput = parse_quote! {
            struct Anvil {
                id: String,
            }
        };

        // Act the call to the derive function
        let output = make_derive(input);

        let expected = quote! {
            pub struct AnvilResource {
                id: String,
            }

            impl frn_core::authorization::Resource for AnvilResource {
                type Id = String;
                const RESOURCE_NAME: &'static str = "anvil";

                #[allow(refining_impl_trait)]
                fn some(id: Self::Id) -> AnvilResource {
                    AnvilResource { id }
                }

                fn id(&self) -> &Self::Id {
                    &self.id
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
}
