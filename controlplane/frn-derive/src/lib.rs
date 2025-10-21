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
    let resource_name = struct_ident.to_string().to_lowercase();

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
        impl frn_core::authorization::Resource for #struct_ident {
            type Id = #id_type;
            const NAME: &'static str = #resource_name;

            fn id(&self) -> &Self::Id {
                &self.id
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
                type Id = String;
                const NAME: &'static str = "anvil";

                fn id(&self) -> &Self::Id {
                    &self.id
                }
            }
        };
        assert_eq!(output.to_string(), expected.to_string());
    }
}
