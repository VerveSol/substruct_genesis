use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

// ============================================================================
// DEBUG AND WRAPPING TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct DebugWrapStruct {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    value: i32,
}

#[test]
fn test_debug_wrap() {
    let update = DebugWrapStructSubstruct::new(Some("test".to_string()), Some(42));

    // Test that debug formatting works with wrapped fields
    let debug_output = format!("{:?}", update);
    assert!(debug_output.contains("test"));
    assert!(debug_output.contains("42"));
}

// ============================================================================
// SIMPLE NESTING TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct SimpleNestedStruct {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested)]
    details: SimpleDetails,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct SimpleDetails {
    #[substruct_field(primitive)]
    description: String,
}

#[test]
fn test_simple_nested() {
    let details_update = SimpleDetailsSubstruct::new(Some("A simple description".to_string()));

    let nested_update =
        SimpleNestedStructSubstruct::new(Some("Test Struct".to_string()), Some(details_update));

    assert_eq!(nested_update.name, Some("Test Struct".to_string()));
    assert!(nested_update.details.is_some());

    if let Some(details) = &nested_update.details {
        assert_eq!(
            details.description,
            Some("A simple description".to_string())
        );
    }
}

// ============================================================================
// STRUCT LEVEL NAMING TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "CustomNameBuilder")]
struct CustomNamedStruct {
    #[substruct_field(primitive)]
    field: String,
}

#[test]
fn test_struct_level_names() {
    let update = CustomNameBuilder::new(Some("test".to_string()));

    assert_eq!(update.field, Some("test".to_string()));

    // Verify the custom name was used
    let _: CustomNameBuilder = update;
}

// ============================================================================
// WRAP ATTRIBUTE TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct WrapAttributeStruct {
    #[substruct_field(primitive, wrap = false)]
    required_field: u32,
    #[substruct_field(primitive)]
    optional_field: String,
}

#[test]
fn test_wrap_attribute_parsing() {
    let update = WrapAttributeStructSubstruct::new(
        42,                           // required field (not wrapped)
        Some("optional".to_string()), // optional field (wrapped)
    );

    assert_eq!(update.required_field, 42);
    assert_eq!(update.optional_field, Some("optional".to_string()));
}
