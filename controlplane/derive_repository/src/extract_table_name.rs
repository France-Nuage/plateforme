use heck::ToSnakeCase;
use pluralizer::pluralize;
use syn::DeriveInput;

/// Extract the table name from the input
pub fn extract_table_name(input: &DeriveInput) -> String {
    let cased = input.ident.to_string().to_snake_case();

    pluralize(&cased, 2, false)
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use super::extract_table_name;

    #[test]
    fn test_extract_table_name_works() {
        // Arrange a model input
        let input: DeriveInput = parse_quote! { struct Missile {} };

        // Act the call to extract_table_name function
        let table_name = extract_table_name(&input);

        // Assert the table name is "missiles"
        assert_eq!(table_name, "missiles");
    }
}
