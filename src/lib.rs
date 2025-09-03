use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Meta, Type, TypePath};

use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Path, Token};

#[proc_macro_error]
#[proc_macro_derive(SubstructBuilder, attributes(substruct_field, substruct_builder))]
pub fn derive_updatable_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract the custom name from the struct-level attribute, or use default
    let update_struct_name = extract_struct_name(&input.attrs)
        .unwrap_or_else(|| format_ident!("{}Substruct", struct_name));

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => abort!(
            input.ident,
            "SubstructBuilder can only be derived for structs"
        ),
    };

    let mut trait_idents = extract_trait_idents(&input.attrs);

    let mut updatable_fields = Vec::new();
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut apply_lines = Vec::new();
    let mut change_lines = Vec::new();
    let mut contains_f64 = false;
    let mut contains_string = false;
    let mut json_field_names = Vec::new();
    let mut unwrapped_field_names = Vec::new();
    let mut wrapped_field_names = Vec::new();

    for field in fields.iter() {
        let span = field.span();
        if let Some(ident) = &field.ident {
            match get_redis_updatable_kind(&field.attrs) {
                FieldKind::Skip => continue,

                FieldKind::Primitive { wrap } => {
                    let ty = &field.ty;

                    // Detect f64 and String for trait filtering
                    if let Type::Path(TypePath { path, .. }) = ty {
                        if let Some(segment) = path.segments.last() {
                            if segment.ident == "f64" {
                                contains_f64 = true;
                            }
                            if segment.ident == "String" {
                                contains_string = true;
                            }
                        }
                    }

                    // Unwrap Option<T> if present
                    let (inner_ty, is_option) = match ty {
                        Type::Path(TypePath { path, .. }) => {
                            if let Some(segment) = path.segments.last() {
                                if segment.ident == "Option" {
                                    if let syn::PathArguments::AngleBracketed(args) =
                                        &segment.arguments
                                    {
                                        if let Some(syn::GenericArgument::Type(inner)) =
                                            args.args.first()
                                        {
                                            (inner.clone(), true)
                                        } else {
                                            abort!(span, "Unsupported Option type format");
                                        }
                                    } else {
                                        abort!(
                                            span,
                                            "Expected angle bracketed arguments in Option"
                                        );
                                    }
                                } else {
                                    (ty.clone(), false)
                                }
                            } else {
                                (ty.clone(), false)
                            }
                        }
                        _ => (ty.clone(), false),
                    };

                    let update_ty = if !wrap {
                        // No wrapping - use the type directly, regardless of whether it's Option<T>
                        quote_spanned! {span=> #ty }
                    } else if is_option {
                        // Wrap Option<T> in Option<Option<T>>
                        quote_spanned! {span=> Option<Option<#inner_ty>> }
                    } else {
                        // Wrap T in Option<T>
                        quote_spanned! {span=> Option<#inner_ty> }
                    };

                    updatable_fields.push(quote_spanned! {span=>
                        pub #ident: #update_ty
                    });

                    field_names.push(ident);
                    field_types.push(update_ty.clone());

                    if !wrap {
                        unwrapped_field_names.push(ident);
                    } else {
                        wrapped_field_names.push(ident);
                    }

                    let apply_line = if !wrap {
                        // No wrapping - use the value directly
                        quote_spanned! {span=>
                            #ident: self.#ident.clone()
                        }
                    } else if is_option {
                        // Handle Option<Option<T>>
                        quote_spanned! {span=>
                            #ident: match &self.#ident {
                                Some(Some(value)) => Some(value.clone()),
                                Some(None) => None,
                                None => source.#ident.clone(),
                            }
                        }
                    } else {
                        // Handle Option<T>
                        quote_spanned! {span=>
                            #ident: self.#ident.clone().unwrap_or_else(|| source.#ident.clone())
                        }
                    };

                    let change_line = if !wrap {
                        // No wrapping - compare directly
                        quote_spanned! {span=>
                            if self.#ident != source.#ident {
                                return true;
                            }
                        }
                    } else if is_option {
                        // Handle Option<Option<T>>
                        quote_spanned! {span=>
                            if let Some(value) = &self.#ident {
                                if value != &source.#ident {
                                    return true;
                                }
                            }
                        }
                    } else {
                        // Handle Option<T>
                        quote_spanned! {span=>
                            if let Some(value) = &self.#ident {
                                if value != &source.#ident {
                                    return true;
                                }
                            }
                        }
                    };

                    apply_lines.push(apply_line);
                    change_lines.push(change_line);
                }

                FieldKind::Nested { nested_type } => {
                    let ty = &field.ty;
                    let update_type = if let Some(nested_name) = nested_type {
                        // Use the specified nested type name
                        format_ident!("{}", nested_name)
                    } else {
                        // Fall back to original logic: append "Substruct" to the field type
                        match ty {
                            Type::Path(TypePath { path, .. }) => {
                                let last = path.segments.last().unwrap_or_else(|| {
                                    abort!(span, "Expected type segment in path")
                                });
                                format_ident!("{}Substruct", last.ident)
                            }
                            _ => abort!(span, "Nested updatable fields must be a named type"),
                        }
                    };

                    field_names.push(ident);
                    field_types.push(quote_spanned! {span=> Option<#update_type> });

                    updatable_fields.push(quote_spanned! {span=>
                        pub #ident: Option<#update_type>
                    });

                    apply_lines.push(quote_spanned! {span=>
                        #ident: self.#ident
                            .as_ref()
                            .map(|v| v.apply_to(&source.#ident))
                            .unwrap_or_else(|| source.#ident.clone())
                    });

                    change_lines.push(quote_spanned! {span=>
                        if let Some(value) = &self.#ident {
                            if value.would_change(&source.#ident) {
                                return true;
                            }
                        }
                    });
                }

                FieldKind::Json => {
                    let ty = &field.ty;

                    // Check if the type is already serde_json::Value — that's misuse
                    if let Type::Path(TypePath { path, .. }) = ty {
                        if let Some(segment) = path.segments.last() {
                            if segment.ident == "Value" {
                                abort!(
                                    span,
                                    "`serde_json::Value` should be annotated with #[substruct_field(primitive)], not #[substruct_field(json)]"
                                );
                            }
                        }
                    }

                    json_field_names.push(ident);

                    let json_ty = quote_spanned! {span=> Option<serde_json::Value>};
                    // Add to field_names and field_types so they can be used in the new method
                    field_names.push(ident);
                    field_types.push(json_ty.clone());

                    updatable_fields.push(quote_spanned! {span=>
                        pub #ident: #json_ty
                    });

                    apply_lines.push(quote_spanned! {span=>
                        #ident: self.#ident
                            .as_ref()
                            .map(|v| serde_json::from_value(v.clone()).expect("Failed to deserialize JSON"))
                            .unwrap_or_else(|| source.#ident.clone())
                    });

                    change_lines.push(quote_spanned! {span=>
                        if let Some(value) = &self.#ident {
                            let original = serde_json::to_value(&source.#ident).expect("Failed to serialize to JSON");
                            if *value != original {
                                return true;
                            }
                        }
                    });
                }

                FieldKind::None => {}
            }
        }
    }

    // Ensure required traits are present
    let required_traits = ["Clone", "Debug", "Serialize", "Deserialize"];

    for required in required_traits {
        if !trait_idents.iter().any(|ident| ident == required) {
            trait_idents.push(format_ident!("{}", required));
        }
    }

    if contains_f64 {
        trait_idents.retain(|ident| ident != "Eq");
    }

    if contains_string {
        trait_idents.retain(|ident| ident != "PartialEq");
        trait_idents.retain(|ident| ident != "Eq");
    }

    // Build the derive clause
    let derive_clause = quote! {
        #[derive(#(#trait_idents),*)]
    };

    let has_json_fields = !json_field_names.is_empty();

    let handled_fields: HashSet<_> = json_field_names.iter().collect();

    let defaulted_fields: Vec<_> = field_names
        .iter()
        .filter(|ident| !handled_fields.contains(ident))
        .collect();

    let from_ref_impl = if has_json_fields {
        quote! {
            impl From<&#struct_name> for #update_struct_name {
                fn from(source: &#struct_name) -> Self {
                    Self {
                        #(#defaulted_fields: Default::default(),)*
                        #(#json_field_names: Some(serde_json::to_value(&source.#json_field_names)
                            .expect("Failed to serialize field to JSON")),)*
                    }
                }
            }
        }
    } else {
        quote! {
            impl From<&#struct_name> for #update_struct_name {
                fn from(source: &#struct_name) -> Self {
                    Self {
                        #(#defaulted_fields: Default::default(),)*
                    }
                }
            }
        }
    };

    let output = if updatable_fields.is_empty() {
        quote! {
            #derive_clause
            pub struct #update_struct_name;

            impl #update_struct_name {
                pub fn new() -> Self {
                    Self
                }
            }

            impl Default for #update_struct_name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    } else {
        quote! {
            #derive_clause
            pub struct #update_struct_name {
                #(#updatable_fields,)*
            }

            impl #update_struct_name {
                pub fn new(#(#field_names: #field_types),*) -> Self {
                    Self {
                        #(#field_names,)*
                    }
                }

                pub fn from_source(source: &#struct_name) -> Self {
                    Self::from(source)
                }

                pub fn apply_to(&self, source: &#struct_name) -> #struct_name {
                    #struct_name {
                        #(#apply_lines,)*
                    }
                }

                pub fn would_change(&self, source: &#struct_name) -> bool {
                    #(#change_lines)*
                    false
                }

                pub fn is_empty(&self) -> bool {
                    #(if let Some(_) = &self.#wrapped_field_names { return false; })*
                    #(if self.#unwrapped_field_names != Default::default() { return false; })*
                    true
                }
            }

            impl Default for #update_struct_name {
                fn default() -> Self {
                    Self {
                        #(#field_names: Default::default(),)*
                    }
                }
            }

            impl From<#struct_name> for #update_struct_name {
                fn from(source: #struct_name) -> Self {
                    Self {
                        #(#field_names: Default::default(),)*
                    }
                }
            }

            #from_ref_impl
        }
    };

    TokenStream::from(output)
}

enum FieldKind {
    Primitive { wrap: bool },
    Nested { nested_type: Option<String> },
    Json,
    Skip,
    None,
}

fn get_redis_updatable_kind(attrs: &[Attribute]) -> FieldKind {
    for attr in attrs {
        if attr.path().is_ident("substruct_field") {
            let Ok(meta_list) =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                return FieldKind::Primitive { wrap: true };
            };

            let mut wrap = true; // Default to wrapping
            let mut field_type = None;
            let mut nested_type = None;

            for meta in meta_list {
                match meta {
                    Meta::Path(path) => {
                        if path.is_ident("primitive") {
                            field_type = Some("primitive");
                        } else if path.is_ident("nested") {
                            field_type = Some("nested");
                        } else if path.is_ident("json") {
                            field_type = Some("json");
                        } else if path.is_ident("skip") {
                            field_type = Some("skip");
                        }
                    }
                    Meta::NameValue(name_value) => {
                        if name_value.path.is_ident("wrap") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Bool(lit_bool),
                                ..
                            }) = &name_value.value
                            {
                                wrap = lit_bool.value;
                            }
                        } else if name_value.path.is_ident("nested_type") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = &name_value.value
                            {
                                nested_type = Some(lit_str.value());
                            }
                        }
                    }
                    _ => {}
                }
            }

            match field_type {
                Some("primitive") => return FieldKind::Primitive { wrap },
                Some("nested") => return FieldKind::Nested { nested_type },
                Some("json") => return FieldKind::Json,
                Some("skip") => return FieldKind::Skip,
                _ => return FieldKind::Skip,
            }
        }
    }
    FieldKind::None
}

fn extract_struct_name(attrs: &[Attribute]) -> Option<syn::Ident> {
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

fn extract_trait_idents(attrs: &[Attribute]) -> Vec<syn::Ident> {
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
