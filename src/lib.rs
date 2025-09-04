use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::format_ident;
use syn::{parse_macro_input, Data, DeriveInput};

mod generator;
mod processor;

use generator::generate_complete_output;
use processor::attributes::{extract_struct_name, extract_trait_idents};
use processor::fields::{get_redis_updatable_kind, process_field, FieldContext};

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

    let mut context = FieldContext::new();

    for field in fields.iter() {
        let field_kind = get_redis_updatable_kind(&field.attrs);
        process_field(field, &field_kind, &mut context);
    }

    // Generate the complete output using the generator module
    let output = generate_complete_output(
        struct_name,
        &update_struct_name,
        &mut trait_idents,
        &context,
    );

    TokenStream::from(output)
}
