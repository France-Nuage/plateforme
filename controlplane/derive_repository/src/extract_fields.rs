use syn::{Data, DeriveInput, Field, Fields, Token, punctuated::Punctuated, spanned::Spanned};

use crate::error::Error;

/// Extract the fields from a given input, failing if it is not a named struct representation.
pub fn extract_fields(input: &DeriveInput) -> Result<&Punctuated<Field, Token![,]>, Error> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(&fields.named),
            Fields::Unnamed(fields) => Err(Error::DerivedFromTupleStruct(fields.span())),
            Fields::Unit => Err(Error::DerivedFromUnitStruct(input.span())),
        },
        Data::Enum(_) => Err(Error::DerivedFromEnum(input.span())),
        Data::Union(_) => Err(Error::DerivedFromUnion(input.span())),
    }
}

#[cfg(test)]
mod tests {
    use super::extract_fields;
    use crate::error::Error;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_extracting_fields_from_a_struct_works() {
        // Arrange an enum input type
        let input: DeriveInput = parse_quote! { struct Category {} };

        // Act the field extraction
        let result = extract_fields(&input);

        // Assert the result is an expected error
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_extracting_fields_from_an_enum_fails_predictably() {
        // Arrange an enum input type
        let input: DeriveInput = parse_quote! { enum Category { Weapon, Armor } };

        // Act the field extraction
        let result = extract_fields(&input);

        // Assert the result is an expected error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::DerivedFromEnum(_)));
    }

    #[test]
    fn test_extracting_fields_from_a_tuple_struct_fails_predictably() {
        // Arrange an enum input type
        let input: DeriveInput = parse_quote! { struct Location(i32, i32); };

        // Act the field extraction
        let result = extract_fields(&input);

        // Assert the result is an expected error
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::DerivedFromTupleStruct(_)
        ));
    }

    #[test]
    fn test_extracting_fields_from_an_union_fails_predictably() {
        // Arrange an enum input type
        let input: DeriveInput =
            parse_quote! { union Damage { melee_damage: u32, range_damage: u32, } };

        // Act the field extraction
        let result = extract_fields(&input);

        // Assert the result is an expected error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::DerivedFromUnion(_)));
    }

    #[test]
    fn test_extracting_fields_from_a_unit_struct_fails_predictably() {
        // Arrange an enum input type
        let input: DeriveInput = parse_quote! { struct Dummy; };

        // Act the field extraction
        let result = extract_fields(&input);

        // Assert the result is an expected error
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::DerivedFromUnitStruct(_)
        ));
    }
}
