use quote::format_ident;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, Meta, Path, Token};

/// Extract the custom struct name from the `substruct_builder` attribute
///
/// This function looks for the `name` parameter in the `substruct_builder` attribute
/// and returns the custom name if found, or None if not specified.
pub fn extract_struct_name(attrs: &[Attribute]) -> Option<syn::Ident> {
    for attr in attrs {
        if attr.path().is_ident("substruct_builder") {
            let Ok(meta_list) =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                continue;
            };

            for meta in meta_list {
                if let Meta::NameValue(name_value) = &meta {
                    if name_value.path.is_ident("name") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) = &name_value.value
                        {
                            return Some(format_ident!("{}", lit_str.value()));
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract trait identifiers from the `derive` attribute
///
/// This function parses the `derive` attribute and extracts all trait names
/// that should be derived for the generated struct.
pub fn extract_trait_idents(attrs: &[Attribute]) -> Vec<syn::Ident> {
    let mut trait_idents = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("derive") {
            let Ok(meta_list) = attr.parse_args_with(Punctuated::<Path, Comma>::parse_terminated)
            else {
                continue;
            };

            for path in meta_list {
                if let Some(ident) = path.get_ident() {
                    trait_idents.push(ident.clone());
                }
            }
        }
    }

    trait_idents
}
