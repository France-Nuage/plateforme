use proc_macro2::TokenTree;
use syn::{Field, Ident, punctuated::Punctuated, token::Comma};

/// Extract the primary attribute identity from a set of identity fields
pub fn extract_primary_idents(idents: &Punctuated<Field, Comma>) -> Vec<&Ident> {
    idents
        .into_iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| {
                if attr.path().is_ident("repository")
                    && let syn::Meta::List(list) = &attr.meta
                {
                    return list.tokens.clone().into_iter().any(|token| {
                        if let TokenTree::Ident(ident) = token {
                            ident == "primary"
                        } else {
                            false
                        }
                    });
                }
                false
            })
        })
        .filter_map(|field| field.ident.as_ref())
        .collect()
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use crate::extract_fields::extract_fields;

    use super::extract_primary_idents;

    #[test]
    fn test_extract_identity_fields_works() {
        // Arrange a set of fields matching a model input
        let input: DeriveInput = parse_quote! {
            struct Category {
                #[repository(primary)]
                id: String,
                name: String,
            }
        };
        let fields = extract_fields(&input).unwrap();

        // Act the primary idents extraction

        let idents = extract_primary_idents(fields);

        // Assert a single "id" ident is extracted
        assert!(idents.len() == 1);
        assert_eq!(idents[0], "id");
    }

    #[test]
    fn test_extract_identity_fields_works_with_composite_primary_keys() {
        // Arrange a set of fields matching a model input
        let input: DeriveInput = parse_quote! {
            struct ReusableMissileLaunch {
                #[repository(primary)]
                missile_id: String,

                #[repository(primary)]
                target_id: String,
            }
        };
        let fields = extract_fields(&input).unwrap();

        // Act the primary idents extraction
        let idents = extract_primary_idents(fields);

        // Assert the 2 primary idents are extracted
        assert!(idents.len() == 2);
        assert_eq!(idents[0], "missile_id");
        assert_eq!(idents[1], "target_id");
    }

    #[test]
    fn test_extract_identity_fields_works_with_no_primary_key() {
        // Arrange a set of fields matching a model input
        let input: DeriveInput = parse_quote! {
            struct Missile {
                damage: u8,
            }
        };
        let fields = extract_fields(&input).unwrap();

        // Act the primary idents extraction
        let idents = extract_primary_idents(fields);

        // Assert no ident is extracted
        assert!(idents.is_empty());
    }
}
