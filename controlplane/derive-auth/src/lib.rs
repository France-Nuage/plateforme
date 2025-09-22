use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Authorize)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let stream = make_derive(ast);
    TokenStream::from(stream)
}

fn make_derive(input: DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let resource_name = name.to_string().to_lowercase();

    quote! {
        impl auth::Authorize for #name {
            type Id = uuid::Uuid;

            fn any_resource() -> (&'static str, &'static str) {
                (#resource_name, "*")
            }

            fn resource(&self) -> (&'static str, &Self::Id) {
                (#resource_name, &self.id)
            }

            fn resource_name() -> &'static str {
                #resource_name
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
            impl auth::Authorize for Anvil {
                type Id = uuid::Uuid;

                fn any_resource() -> (&'static str, &'static str) {
                    ("anvil", "*")
                }

                fn resource(&self) -> (&'static str, &Self::Id) {
                    ("anvil", &self.id)
                }

                fn resource_name() -> &'static str {
                    "anvil"
                }
            }
        };
        assert_eq!(output.to_string(), expected.to_string());
    }
}
