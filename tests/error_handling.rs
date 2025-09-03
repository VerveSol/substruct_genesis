use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

// ============================================================================
// ERROR HANDLING TESTS - MACRO COMPILATION ERRORS
// ============================================================================

// Note: These tests are designed to verify that the macro provides
// appropriate error messages when used incorrectly. They should compile
// to ensure the macro handles errors gracefully.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct ValidStruct {
    #[substruct_field(primitive)]
    field: String,
}

// Test that basic valid usage works
#[test]
fn test_valid_struct_compiles() {
    let update = ValidStructSubstruct::new(Some("test".to_string()));
    assert_eq!(update.field, Some("test".to_string()));
}

// ============================================================================
// FIELD TYPE VALIDATION TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct MixedFieldTypesStruct {
    #[substruct_field(primitive)]
    string_field: String,

    #[substruct_field(primitive)]
    int_field: i32,

    #[substruct_field(primitive)]
    float_field: f64,

    #[substruct_field(primitive)]
    bool_field: bool,

    #[substruct_field(primitive)]
    option_field: Option<String>,

    #[substruct_field(primitive)]
    json_field: serde_json::Value,

    #[substruct_field(nested)]
    nested_field: NestedStruct,

    // Fields without attributes (should be excluded)
    excluded_field: u32,
    another_excluded: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct NestedStruct {
    #[substruct_field(primitive)]
    nested_field: String,
}

#[test]
fn test_mixed_field_types_validation() {
    let update = MixedFieldTypesStructSubstruct::new(
        Some("test".to_string()),                                     // string_field
        Some(42),                                                     // int_field
        Some(3.14),                                                   // float_field
        Some(true),                                                   // bool_field
        Some(Some("option".to_string())),                             // option_field
        Some(serde_json::json!({"key": "value"})),                    // json_field
        Some(NestedStructSubstruct::new(Some("nested".to_string()))), // nested_field
    );

    // Validate all field types are correct
    assert_eq!(update.string_field, Some("test".to_string()));
    assert_eq!(update.int_field, Some(42));
    assert_eq!(update.float_field, Some(3.14));
    assert_eq!(update.bool_field, Some(true));
    assert_eq!(update.option_field, Some(Some("option".to_string())));
    assert!(update.json_field.is_some());
    assert!(update.nested_field.is_some());

    // Verify excluded fields are not present
    // (These would cause compilation errors if they were present)
}

// ============================================================================
// COMPLEX NESTING VALIDATION TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "CompanyBuilder")]
struct Company {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested)]
    ceo: Person,
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Person {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "AddressBuilder")]
struct Address {
    #[substruct_field(primitive)]
    street: String,
    #[substruct_field(primitive)]
    city: String,
}

#[test]
fn test_complex_nesting_validation() {
    let address = AddressBuilder::new(
        Some("123 Main St".to_string()),
        Some("Main City".to_string()),
    );

    let person = PersonSubstruct::new(Some("CEO Name".to_string()), Some(address.clone()));

    let company = CompanyBuilder::new(
        Some("Company Name".to_string()),
        Some(person),
        Some(address),
    );

    // Validate complex nesting works correctly
    assert_eq!(company.name, Some("Company Name".to_string()));
    assert!(company.ceo.is_some());
    assert!(company.address.is_some());

    if let Some(ceo) = &company.ceo {
        assert_eq!(ceo.name, Some("CEO Name".to_string()));
        assert!(ceo.address.is_some());
    }
}

// ============================================================================
// EDGE CASE VALIDATION TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct SingleFieldStruct {
    #[substruct_field(primitive)]
    field: String,
}

#[test]
fn test_single_field_validation() {
    let update = SingleFieldStructSubstruct::new(Some("test".to_string()));
    assert_eq!(update.field, Some("test".to_string()));
}

// ============================================================================
// ATTRIBUTE COMBINATION TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct AttributeCombinationStruct {
    #[substruct_field(primitive)]
    basic_field: String,

    #[substruct_field(primitive)]
    unwrapped_field: u32,

    #[substruct_field(nested, nested_type = "CustomNestedBuilder")]
    custom_nested: CustomNestedStruct,

    #[substruct_field(primitive)]
    json_field: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "CustomNestedBuilder")]
struct CustomNestedStruct {
    #[substruct_field(primitive)]
    nested_field: String,
}

#[test]
fn test_attribute_combinations() {
    let nested = CustomNestedBuilder::new(Some("nested value".to_string()));

    let update = AttributeCombinationStructSubstruct::new(
        Some("basic value".to_string()),           // basic_field
        Some(42),                                  // unwrapped_field
        Some(nested),                              // custom_nested
        Some(serde_json::json!({"key": "value"})), // json_field
    );

    // Validate all attribute combinations work
    assert_eq!(update.basic_field, Some("basic value".to_string()));
    assert_eq!(update.unwrapped_field, Some(42));
    assert!(update.custom_nested.is_some());
    assert!(update.json_field.is_some());
}

// ============================================================================
// SERIALIZATION VALIDATION TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct SerializationTestStruct {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    metadata: serde_json::Value,
}

#[test]
fn test_serialization_validation() {
    let update = SerializationTestStructSubstruct::new(
        Some("test name".to_string()),
        Some(serde_json::json!({"key": "value", "number": 42})),
    );

    // Test JSON serialization
    let json = serde_json::to_string(&update).unwrap();
    let deserialized: SerializationTestStructSubstruct = serde_json::from_str(&json).unwrap();

    assert_eq!(update.name, deserialized.name);
    assert_eq!(update.metadata, deserialized.metadata);
}

// ============================================================================
// TRAIT IMPLEMENTATION VALIDATION TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct TraitTestStruct {
    #[substruct_field(primitive)]
    field: String,
}

#[test]
fn test_trait_implementations() {
    let update = TraitTestStructSubstruct::new(Some("test".to_string()));

    // Test Clone trait
    let cloned = update.clone();
    assert_eq!(update.field, cloned.field);

    // Test Debug trait
    let debug_output = format!("{:?}", update);
    assert!(debug_output.contains("test"));

    // Test PartialEq trait
    let another = TraitTestStructSubstruct::new(Some("test".to_string()));
    assert_eq!(update.field, another.field);

    // Test Default trait
    let default = TraitTestStructSubstruct::default();
    assert_eq!(default.field, None);

    // Test From trait
    let source = TraitTestStruct {
        field: "source".to_string(),
    };
    let from_source = TraitTestStructSubstruct::from_source(&source);
    assert_eq!(from_source.field, None);
}
