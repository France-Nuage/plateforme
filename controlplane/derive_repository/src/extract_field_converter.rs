/// Extract the primary attribute identity from a set of identity fields
pub fn extract_field_converter(field: &syn::Field) -> Option<syn::LitStr> {
    field.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("sqlx") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args::<syn::Meta>() {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(literal),
                    ..
                }) = nv.value
                {
                    Some(literal)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use crate::{extract_field_converter::extract_field_converter, extract_fields::extract_fields};

    #[test]
    fn test_extract_field_converter_works() {
        // Arrange a field with an sqlx::try_from
        let input: DeriveInput = parse_quote! {
            struct Missile {
                #[sqlx(try_from = "String")]
                launch_status: LaunchStatus
            }
        };
        let fields = extract_fields(&input).unwrap().clone();
        let field = fields.get(0).unwrap();

        // Act the call to the extract_field_converter
        let convert = extract_field_converter(field);

        // Assert the result is "String"
        assert!(convert.is_some());
        assert!(convert.unwrap().value() == "String")
    }

    #[test]
    fn test_extract_field_converter_returns_none_without_try_from() {
        // Arrange a field with an sqlx::try_from
        let input: DeriveInput = parse_quote! {
            struct Missile {
                launch_status: LaunchStatus
            }
        };
        let fields = extract_fields(&input).unwrap().clone();
        let field = fields.get(0).unwrap();

        // Act the call to the extract_field_converter
        let convert = extract_field_converter(field);

        // Assert the result is none
        assert!(convert.is_none());
    }
}
