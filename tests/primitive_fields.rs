use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

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
    option_string: Option<String>,
    #[substruct_field(primitive)]
    option_int: Option<i32>,
}

#[test]
fn test_primitive_struct_creation() {
    // The update struct has Option-wrapped types
    let update = PrimitiveStructSubstruct::new(
        Some("new_string".to_string()),
        Some(42),
        Some(3.14),
        Some(true),
        Some(Some("new_option".to_string())),
        Some(Some(100)),
    );

    // Verify the update struct has the correct types
    assert_eq!(update.string_field, Some("new_string".to_string()));
    assert_eq!(update.int_field, Some(42));
    assert_eq!(update.float_field, Some(3.14));
    assert_eq!(update.bool_field, Some(true));
    assert_eq!(update.option_string, Some(Some("new_option".to_string())));
    assert_eq!(update.option_int, Some(Some(100)));
}

#[test]
fn test_primitive_struct_empty_update() {
    let empty_update = PrimitiveStructSubstruct::default();

    assert!(empty_update.is_empty());
}
