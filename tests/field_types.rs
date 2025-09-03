use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

// ============================================================================
// PRIMITIVE FIELD TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct PrimitiveStruct {
    #[substruct_field(primitive)]
    string_field: String,
    #[substruct_field(primitive)]
    int_field: i32,
    #[substruct_field(primitive)]
    float_field: f64,
    #[substruct_field(primitive)]
    bool_field: bool,
    #[substruct_field(primitive)]
    unwrapped_field: u32,
    #[substruct_field(primitive)]
    option_int: Option<i32>,
}

#[test]
fn test_primitive_struct_creation() {
    let update = PrimitiveStructSubstruct::new(
        Some("test".to_string()),
        Some(42),
        Some(3.14),
        Some(true),
        Some(100), // wrapped field
        Some(Some(200)),
    );

    assert_eq!(update.string_field, Some("test".to_string()));
    assert_eq!(update.int_field, Some(42));
    assert_eq!(update.float_field, Some(3.14));
    assert_eq!(update.bool_field, Some(true));
    assert_eq!(update.unwrapped_field, Some(100));
    assert_eq!(update.option_int, Some(Some(200)));
}

#[test]
fn test_primitive_struct_empty_update() {
    let empty_update = PrimitiveStructSubstruct::default();

    assert!(empty_update.is_empty());
}

// ============================================================================
// JSON FIELD TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct JsonStruct {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(json)]
    preferences: UserPreferences,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UserPreferences {
    theme: String,
    language: String,
}

#[test]
fn test_json_field() {
    let preferences = UserPreferences {
        theme: "dark".to_string(),
        language: "en".to_string(),
    };

    let update = JsonStructSubstruct::new(
        Some("John".to_string()),
        Some(serde_json::to_value(&preferences).unwrap()),
    );

    assert_eq!(update.name, Some("John".to_string()));
    assert!(update.preferences.is_some());
}

#[test]
fn test_primitive_field() {
    let update = JsonStructSubstruct::new(Some("Alice".to_string()), None);

    assert_eq!(update.name, Some("Alice".to_string()));
    assert!(update.preferences.is_none());
}

#[test]
fn test_json_struct_has_preferences_field() {
    let update = JsonStructSubstruct::default();

    // Verify the field exists and is properly typed
    let _: Option<serde_json::Value> = update.preferences;
    assert!(update.preferences.is_none());
}

// ============================================================================
// NESTED TYPE TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct NestedStruct {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested)]
    address: Address,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Address {
    #[substruct_field(primitive)]
    street: String,
    #[substruct_field(primitive)]
    city: String,
}

#[test]
fn test_nested_struct_derivation() {
    let address_update = AddressSubstruct::new(
        Some("123 Main St".to_string()),
        Some("Main City".to_string()),
    );

    let nested_update = NestedStructSubstruct::new(Some("John".to_string()), Some(address_update));

    assert_eq!(nested_update.name, Some("John".to_string()));
    assert!(nested_update.address.is_some());

    if let Some(address) = &nested_update.address {
        assert_eq!(address.street, Some("123 Main St".to_string()));
        assert_eq!(address.city, Some("Main City".to_string()));
    }
}

#[test]
fn test_nested_struct_from_source() {
    let source_address = Address {
        street: "123 St".to_string(),
        city: "City".to_string(),
    };

    let source_nested = NestedStruct {
        name: "Alice".to_string(),
        address: source_address,
    };

    let update = NestedStructSubstruct::from_source(&source_nested);

    assert_eq!(update.name, None);
    assert!(update.address.is_none());
}
