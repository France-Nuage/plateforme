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
            fn resource(&self) -> (String, String) {
                (#resource_name.to_string(), self.id.to_string())
            }

            fn some(id: impl ToString) -> Box<dyn frn_core::authorization::Resource + Send + Sync>
            where
                Self: Sized,
            {
                Box::new(#companion_struct_ident {
                    identifier: id.to_string(),
                })
            }
        }

        pub struct #companion_struct_ident {
            identifier: String,
        }

        impl frn_core::authorization::Resource for #companion_struct_ident {
            fn resource(&self) -> (String, String) {
                (#resource_name.to_string(), self.identifier.clone())
            }

            fn some(id: impl ToString) -> Box<dyn frn_core::authorization::Resource + Send + Sync>
            where
                Self: Sized,
            {
                Box::new(Self {
                    identifier: id.to_string(),
                })
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
                fn resource(&self) -> (String, String) {
                    ("anvil".to_string(), self.id.to_string())
                }

                fn some(id: impl ToString) -> Box<dyn frn_core::authorization::Resource + Send + Sync>
                where
                    Self: Sized,
                {
                    Box::new(AnvilResource {
                        identifier: id.to_string(),
                    })
                }
            }

            pub struct AnvilResource {
                identifier: String,
            }

            impl frn_core::authorization::Resource for AnvilResource {
                fn resource(&self) -> (String, String) {
                    ("anvil".to_string(), self.identifier.clone())
                }

                fn some(id: impl ToString) -> Box<dyn frn_core::authorization::Resource + Send + Sync>
                where
                    Self: Sized,
                {
                    Box::new(Self {
                        identifier: id.to_string(),
                    })
                }
            }
        };
        assert_eq!(output.to_string(), expected.to_string());
    }
}
