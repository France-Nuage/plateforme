use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, parse_macro_input};

#[proc_macro_derive(Resource)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let stream = make_derive(ast);
    TokenStream::from(stream)
}

fn make_derive(input: DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &input.ident;
    let companion_struct_ident =
        Ident::new(&format!("{}Resource", &struct_ident), struct_ident.span());
    let resource_name = struct_ident.to_string().to_lowercase();

    quote! {
        impl frn_core::authorization::Resource for #struct_ident {
            type Id = uuid::Uuid;
            const NAME: &'static str = #resource_name;

            fn resource_identifier(&self) -> (&'static str, &Self::Id) {
                (&Self::NAME, &self.id)
            }

            fn any() -> impl frn_core::authorization::Resource<Id = String> {
                #companion_struct_ident::any()
            }
        }

        pub struct #companion_struct_ident {
            identifier: String,
        }

         impl #companion_struct_ident {
             pub fn from(identifier: String) -> Self {
                 Self {
                     identifier,
                 }
             }
         }

         impl frn_core::authorization::Resource for #companion_struct_ident {
             type Id = String;
             const NAME: &'static str = #resource_name;

             fn resource_identifier(&self) -> (&'static str, &Self::Id) {
                 (&#struct_ident::NAME, &self.identifier)
             }

             fn any() -> impl frn_core::authorization::Resource<Id = String> {
                 Self {
                     identifier: "*".to_owned(),
                 }
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
            impl frn_core::authorization::Resource for Anvil {
                type Id = uuid::Uuid;
                const NAME: &'static str = "anvil";

                fn resource_identifier(&self) -> (&'static str, &Self::Id) {
                    (&Self::NAME, &self.id)
                }

                fn any() -> impl frn_core::authorization::Resource<Id = String> {
                    AnvilResource::any()
                }
            }

            pub struct AnvilResource {
                identifier: String,
            }

            impl AnvilResource {
                pub fn from(identifier: String) -> Self {
                    Self {
                        identifier,
                    }
                }
            }

            impl frn_core::authorization::Resource for AnvilResource {
                type Id = String;
                const NAME: &'static str = "anvil";

                fn resource_identifier(&self) -> (&'static str, &Self::Id) {
                    (&Anvil::NAME, &self.identifier)
                }

                fn any() -> impl frn_core::authorization::Resource<Id = String> {
                    Self {
                        identifier: "*".to_owned(),
                    }
                }
            }
        };
        assert_eq!(output.to_string(), expected.to_string());
    }
}
