use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Meta, parse_macro_input};

#[proc_macro_derive(OperationError, attributes(operation_error))]
pub fn derive_operation_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(name, "OperationError can only be derived for enums")
            .to_compile_error()
            .into();
    };

    let mut consume_retry_arms = Vec::new();
    let mut is_violated_invariant_arms = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;

        let mut is_transient = false;
        let mut is_invariant = false;

        for attr in &variant.attrs {
            if attr.path().is_ident("operation_error")
                && let Meta::List(meta_list) = &attr.meta
            {
                let tokens = &meta_list.tokens;
                let tokens_str = tokens.to_string();

                if tokens_str == "transient" {
                    is_transient = true;
                } else if tokens_str == "invariant" {
                    is_invariant = true;
                }
            }
        }

        let pattern = match &variant.fields {
            Fields::Named(_) => quote! { Self::#variant_name { .. } },
            Fields::Unnamed(_) => quote! { Self::#variant_name(..) },
            Fields::Unit => quote! { Self::#variant_name },
        };

        if is_transient {
            consume_retry_arms.push(quote! {
                #pattern => false
            });
        }

        if is_invariant {
            is_violated_invariant_arms.push(quote! {
                #pattern => true
            });
        }
    }

    let consume_retry_impl = if consume_retry_arms.is_empty() {
        quote! { true }
    } else {
        quote! {
            match self {
                #(#consume_retry_arms,)*
                _ => true,
            }
        }
    };

    let is_violated_invariant_impl = if is_violated_invariant_arms.is_empty() {
        quote! { false }
    } else {
        quote! {
            match self {
                #(#is_violated_invariant_arms,)*
                _ => false,
            }
        }
    };

    let expanded = quote! {
        impl crate::operations::OperationError for #name {
            fn consume_retry(&self) -> bool {
                #consume_retry_impl
            }

            fn is_violated_invariant(&self) -> bool {
                #is_violated_invariant_impl
            }
        }
    };

    TokenStream::from(expanded)
}
