use proc_macro_error::abort;
use quote::{format_ident, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Field, Ident, Meta, Token, Type, TypePath};

/// Enum representing different types of field processing
#[derive(Debug, Clone)]
pub enum FieldKind {
    Primitive { wrap: bool },
    Nested { nested_type: Option<String> },
    Json,
    Skip,
    None,
}

/// Context for field processing that accumulates state during struct generation
pub struct FieldContext {
    pub updatable_fields: Vec<proc_macro2::TokenStream>,
    pub field_names: Vec<Ident>,
    pub field_types: Vec<proc_macro2::TokenStream>,
    pub apply_lines: Vec<proc_macro2::TokenStream>,
    pub change_lines: Vec<proc_macro2::TokenStream>,
    pub contains_f64: bool,
    pub contains_string: bool,
    pub json_field_names: Vec<Ident>,
    pub unwrapped_field_names: Vec<Ident>,
    pub wrapped_field_names: Vec<Ident>,
}

impl FieldContext {
    pub fn new() -> Self {
        Self {
            updatable_fields: Vec::new(),
            field_names: Vec::new(),
            field_types: Vec::new(),
            apply_lines: Vec::new(),
            change_lines: Vec::new(),
            contains_f64: false,
            contains_string: false,
            json_field_names: Vec::new(),
            unwrapped_field_names: Vec::new(),
            wrapped_field_names: Vec::new(),
        }
    }
}

/// Process a primitive field with optional wrapping
pub fn handle_primitive_field(
    field: &Field,
    ident: &Ident,
    wrap: bool,
    context: &mut FieldContext,
) {
    let span = field.span();
    let ty = &field.ty;

    // Detect f64 and String for trait filtering
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "f64" {
                context.contains_f64 = true;
            }
            if segment.ident == "String" {
                context.contains_string = true;
            }
        }
    }

    // Unwrap Option<T> if present
    let (inner_ty, is_option) = match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(segment) = path.segments.last() {
                if segment.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                            (inner.clone(), true)
                        } else {
                            abort!(span, "Unsupported Option type format");
                        }
                    } else {
                        abort!(span, "Expected angle bracketed arguments in Option");
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

    context.updatable_fields.push(quote_spanned! {span=>
        pub #ident: #update_ty
    });

    context.field_names.push(ident.clone());
    context.field_types.push(update_ty.clone());

    if !wrap {
        context.unwrapped_field_names.push(ident.clone());
    } else {
        context.wrapped_field_names.push(ident.clone());
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

    context.apply_lines.push(apply_line);
    context.change_lines.push(change_line);
}

/// Process a nested field with optional custom type name
pub fn handle_nested_field(
    field: &Field,
    ident: &Ident,
    nested_type: Option<String>,
    context: &mut FieldContext,
) {
    let span = field.span();
    let ty = &field.ty;

    let update_type = if let Some(nested_name) = nested_type {
        // Use the specified nested type name
        format_ident!("{}", nested_name)
    } else {
        // Fall back to original logic: append "Substruct" to the field type
        match ty {
            Type::Path(TypePath { path, .. }) => {
                let last = path
                    .segments
                    .last()
                    .unwrap_or_else(|| abort!(span, "Expected type segment in path"));
                format_ident!("{}Substruct", last.ident)
            }
            _ => abort!(span, "Nested updatable fields must be a named type"),
        }
    };

    context.field_names.push(ident.clone());
    context
        .field_types
        .push(quote_spanned! {span=> Option<#update_type> });

    context.updatable_fields.push(quote_spanned! {span=>
        pub #ident: Option<#update_type>
    });

    context.apply_lines.push(quote_spanned! {span=>
        #ident: self.#ident
            .as_ref()
            .map(|v| v.apply_to(&source.#ident))
            .unwrap_or_else(|| source.#ident.clone())
    });

    context.change_lines.push(quote_spanned! {span=>
        if let Some(value) = &self.#ident {
            if value.would_change(&source.#ident) {
                return true;
            }
        }
    });
}

/// Process a JSON field
pub fn handle_json_field(field: &Field, ident: &Ident, context: &mut FieldContext) {
    let span = field.span();
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

    context.json_field_names.push(ident.clone());

    let json_ty = quote_spanned! {span=> Option<serde_json::Value>};
    // Add to field_names and field_types so they can be used in the new method
    context.field_names.push(ident.clone());
    context.field_types.push(json_ty.clone());

    context.updatable_fields.push(quote_spanned! {span=>
        pub #ident: #json_ty
    });

    context.apply_lines.push(quote_spanned! {span=>
        #ident: self.#ident
            .as_ref()
            .map(|v| serde_json::from_value(v.clone()).expect("Failed to deserialize JSON"))
            .unwrap_or_else(|| source.#ident.clone())
    });

    context.change_lines.push(quote_spanned! {span=>
        if let Some(value) = &self.#ident {
            let original = serde_json::to_value(&source.#ident).expect("Failed to serialize to JSON");
            if *value != original {
                return true;
            }
        }
    });
}

/// Main field processing function that dispatches to appropriate handlers
pub fn process_field(field: &Field, field_kind: &FieldKind, context: &mut FieldContext) {
    let ident = match &field.ident {
        Some(ident) => ident,
        None => return, // Skip unnamed fields
    };

    match field_kind {
        FieldKind::Skip => return,
        FieldKind::Primitive { wrap } => {
            handle_primitive_field(field, ident, *wrap, context);
        }
        FieldKind::Nested { nested_type } => {
            handle_nested_field(field, ident, nested_type.clone(), context);
        }
        FieldKind::Json => {
            handle_json_field(field, ident, context);
        }
        FieldKind::None => return,
    }
}

/// Parse field attributes to determine the field kind
pub fn get_redis_updatable_kind(attrs: &[Attribute]) -> FieldKind {
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
