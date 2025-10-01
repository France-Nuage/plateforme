use heck::ToSnakeCase;
use pluralizer::pluralize;
use syn::{Attribute, DeriveInput, Lit, Meta, Token, punctuated::Punctuated};

/// Extract the table name from the input
pub fn extract_table_name(input: &DeriveInput) -> String {
    if let Some(name) = extract_table_attribute(&input.attrs) {
        return name;
    }

    let cased = input.ident.to_string().to_snake_case();

    pluralize(&cased, 2, false)
}

/// Extract table name from attributes like #[table(name = "custom_table")]
fn extract_table_attribute(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("table")
            && let Meta::List(meta_list) = &attr.meta
        {
            // Parse the content inside the parentheses
            let parsed: Result<Punctuated<Meta, Token![,]>, _> =
                meta_list.parse_args_with(Punctuated::parse_terminated);

            if let Ok(nested_metas) = parsed {
                for meta in nested_metas {
                    if let Meta::NameValue(name_value) = meta
                        && name_value.path.is_ident("name")
                        && let syn::Expr::Lit(expr_lit) = &name_value.value
                        && let Lit::Str(lit_str) = &expr_lit.lit
                    {
                        return Some(lit_str.value());
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use super::extract_table_name;

    #[test]
    fn test_extract_table_name_works() {
        // Arrange a model input
        let input: DeriveInput = parse_quote! { struct Anvil {} };

        // Act the call to extract_table_name function
        let table_name = extract_table_name(&input);

        // Assert the table name is "missiles"
        assert_eq!(table_name, "anvils");
    }

    #[test]
    fn test_extract_custom_table_name_works() {
        // Arrange a model input
        let input: DeriveInput = parse_quote! {
            #[table(name = "anvil_queue")]
            struct Anvil {}
        };

        // Act the call to extract_table_name function
        let table_name = extract_table_name(&input);

        // Assert the table name is "missiles"
        assert_eq!(table_name, "anvil_queue");
    }
}
