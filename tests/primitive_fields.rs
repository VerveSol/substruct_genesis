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
fn test_primitive_struct_apply_to() {
    let source = PrimitiveStruct {
        string_field: "old_string".to_string(),
        int_field: 10,
        float_field: 1.0,
        bool_field: false,
        option_string: Some("old_option".to_string()),
        option_int: Some(50),
    };

    let update = PrimitiveStructSubstruct::new(
        Some("new_string".to_string()),
        None, // no change
        Some(2.0),
        None,            // no change
        Some(None),      // explicitly set to None
        Some(Some(100)), // explicitly set to Some(100)
    );

    let result = update.apply_to(&source);

    assert_eq!(result.string_field, "new_string".to_string());
    assert_eq!(result.int_field, 10); // unchanged
    assert_eq!(result.float_field, 2.0);
    assert_eq!(result.bool_field, false); // unchanged
    assert_eq!(result.option_string, None); // explicitly set to None
    assert_eq!(result.option_int, Some(100)); // explicitly set to Some(100)
}

#[test]
fn test_primitive_struct_would_change() {
    let source = PrimitiveStruct {
        string_field: "old_string".to_string(),
        int_field: 10,
        float_field: 1.0,
        bool_field: false,
        option_string: Some("old_option".to_string()),
        option_int: Some(50),
    };

    // Test Some(None) - changing from Some to None
    let update1 = PrimitiveStructSubstruct::new(
        None,
        None,
        None,
        None,
        Some(None), // explicitly set to None
        None,
    );
    assert!(update1.would_change(&source)); // option_string changes from Some to None

    // Test Some(Some(value)) - changing the inner value
    let update2 = PrimitiveStructSubstruct::new(
        None,
        None,
        None,
        None,
        None,
        Some(Some(100)), // explicitly set to Some(100)
    );
    assert!(update2.would_change(&source)); // option_int changes from Some(50) to Some(100)

    // Test no change
    let update3 = PrimitiveStructSubstruct::new(
        None,
        None,
        None,
        None,
        Some(Some("old_option".to_string())), // same value
        Some(Some(50)),                       // same value
    );
    assert!(!update3.would_change(&source)); // no changes
}

#[test]
fn test_primitive_struct_empty_update() {
    let source = PrimitiveStruct {
        string_field: "test".to_string(),
        int_field: 42,
        float_field: 3.14,
        bool_field: true,
        option_string: Some("option".to_string()),
        option_int: Some(100),
    };

    let empty_update = PrimitiveStructSubstruct::default();

    assert!(empty_update.is_empty());
    assert!(!empty_update.would_change(&source));

    let result = empty_update.apply_to(&source);
    assert_eq!(result, source); // no changes applied
}
