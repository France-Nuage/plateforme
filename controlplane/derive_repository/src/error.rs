use proc_macro2::Span;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Macro cannot be derived from an enum.")]
    DerivedFromEnum(Span),

    #[error("Macro cannot be derived from a tuple struct.")]
    DerivedFromTupleStruct(Span),

    #[error("Macro cannot be derived from an union.")]
    DerivedFromUnion(Span),

    #[error("Macro cannot be derived from a unit struct.")]
    DerivedFromUnitStruct(Span),
}

impl From<Error> for syn::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::DerivedFromEnum(span) => syn::Error::new(span, value.to_string()),
            Error::DerivedFromTupleStruct(span) => syn::Error::new(span, value.to_string()),
            Error::DerivedFromUnion(span) => syn::Error::new(span, value.to_string()),
            Error::DerivedFromUnitStruct(span) => syn::Error::new(span, value.to_string()),
        }
    }
}
