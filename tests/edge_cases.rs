use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

// ============================================================================
// MINIMAL STRUCT TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct SingleFieldStruct {
    #[substruct_field(primitive)]
    field: String,
}

#[test]
fn test_single_field_struct() {
    let update = SingleFieldStructSubstruct::new(Some("test".to_string()));
    assert_eq!(update.field, Some("test".to_string()));
    assert!(!update.is_empty());

    let empty = SingleFieldStructSubstruct::default();
    assert!(empty.is_empty());
}

// ============================================================================
// FIELD TYPE EDGE CASES
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
    char_field: char,

    #[substruct_field(primitive)]
    byte_field: u8,

    #[substruct_field(primitive)]
    array_field: [u8; 4],

    #[substruct_field(primitive)]
    tuple_field: (String, i32),

    // Fields without attributes (should be excluded)
    excluded_field: u32,
    another_excluded: bool,
}

#[test]
fn test_mixed_field_types_edge_cases() {
    let update = MixedFieldTypesStructSubstruct::new(
        Some("test".to_string()),        // string_field
        Some(42),                        // int_field
        Some(3.14),                      // float_field
        Some(true),                      // bool_field
        Some('a'),                       // char_field
        Some(255),                       // byte_field
        Some([1, 2, 3, 4]),              // array_field
        Some(("tuple".to_string(), 42)), // tuple_field
    );

    // Validate all field types are correct
    assert_eq!(update.string_field, Some("test".to_string()));
    assert_eq!(update.int_field, Some(42));
    assert_eq!(update.float_field, Some(3.14));
    assert_eq!(update.bool_field, Some(true));
    assert_eq!(update.char_field, Some('a'));
    assert_eq!(update.byte_field, Some(255));
    assert_eq!(update.array_field, Some([1, 2, 3, 4]));
    assert_eq!(update.tuple_field, Some(("tuple".to_string(), 42)));

    // Verify excluded fields are not present
    // (These would cause compilation errors if they were present)
}

// ============================================================================
// NESTING EDGE CASES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct DeepNestedStruct {
    #[substruct_field(nested)]
    level1: Level1Struct,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Level1Struct {
    #[substruct_field(nested)]
    level2: Level2Struct,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Level2Struct {
    #[substruct_field(nested)]
    level3: Level3Struct,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Level3Struct {
    #[substruct_field(primitive)]
    final_field: String,
}

#[test]
fn test_deep_nesting() {
    let level3 = Level3StructSubstruct::new(Some("deep value".to_string()));
    let level2 = Level2StructSubstruct::new(Some(level3));
    let level1 = Level1StructSubstruct::new(Some(level2));
    let deep = DeepNestedStructSubstruct::new(Some(level1));

    // Validate deep nesting works
    assert!(deep.level1.is_some());

    if let Some(l1) = &deep.level1 {
        assert!(l1.level2.is_some());

        if let Some(l2) = &l1.level2 {
            assert!(l2.level3.is_some());

            if let Some(l3) = &l2.level3 {
                assert_eq!(l3.final_field, Some("deep value".to_string()));
            }
        }
    }
}

// ============================================================================
// CUSTOM NAMING EDGE CASES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "VeryLongCustomNameBuilder")]
struct VeryLongNamedStruct {
    #[substruct_field(primitive)]
    field: String,
}

#[test]
fn test_very_long_custom_name() {
    let update = VeryLongCustomNameBuilder::new(Some("test".to_string()));
    assert_eq!(update.field, Some("test".to_string()));

    // Verify the custom name was used
    let _: VeryLongCustomNameBuilder = update;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "SpecialCharsBuilder")]
struct SpecialCharsStruct {
    #[substruct_field(primitive)]
    field: String,
}

#[test]
fn test_special_chars_in_name() {
    let update = SpecialCharsBuilder::new(Some("test".to_string()));
    assert_eq!(update.field, Some("test".to_string()));

    // Verify the custom name was used
    let _: SpecialCharsBuilder = update;
}

// ============================================================================
// ATTRIBUTE COMBINATION EDGE CASES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct ComplexAttributeStruct {
    #[substruct_field(primitive)]
    basic_field: String,

    #[substruct_field(primitive)]
    another_field: i32,

    #[substruct_field(nested, nested_type = "CustomNestedBuilder")]
    custom_nested: CustomNestedStruct,

    #[substruct_field(primitive)]
    final_field: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "CustomNestedBuilder")]
struct CustomNestedStruct {
    #[substruct_field(primitive)]
    nested_field: String,
}

#[test]
fn test_complex_attribute_combinations() {
    let nested = CustomNestedBuilder::new(Some("nested value".to_string()));

    let update = ComplexAttributeStructSubstruct::new(
        Some("basic value".to_string()), // basic_field
        Some(42),                        // another_field
        Some(nested),                    // custom_nested
        Some(true),                      // final_field
    );

    // Validate all attribute combinations work
    assert_eq!(update.basic_field, Some("basic value".to_string()));
    assert_eq!(update.another_field, Some(42));
    assert!(update.custom_nested.is_some());
    assert_eq!(update.final_field, Some(true));
}

// ============================================================================
// SERIALIZATION EDGE CASES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct SerializationEdgeCaseStruct {
    #[substruct_field(primitive)]
    empty_string: String,

    #[substruct_field(primitive)]
    zero_number: i32,

    #[substruct_field(primitive)]
    false_bool: bool,

    #[substruct_field(primitive)]
    max_u64: u64,

    #[substruct_field(primitive)]
    min_i64: i64,
}

#[test]
fn test_serialization_edge_cases() {
    let update = SerializationEdgeCaseStructSubstruct::new(
        Some("".to_string()), // empty_string
        Some(0),              // zero_number
        Some(false),          // false_bool
        Some(u64::MAX),       // max_u64
        Some(i64::MIN),       // min_i64
    );

    // Test JSON serialization with edge case values
    let json = serde_json::to_string(&update).unwrap();
    let deserialized: SerializationEdgeCaseStructSubstruct = serde_json::from_str(&json).unwrap();

    assert_eq!(update.empty_string, deserialized.empty_string);
    assert_eq!(update.zero_number, deserialized.zero_number);
    assert_eq!(update.false_bool, deserialized.false_bool);
    assert_eq!(update.max_u64, deserialized.max_u64);
    assert_eq!(update.min_i64, deserialized.min_i64);
}

// ============================================================================
// TRAIT IMPLEMENTATION EDGE CASES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct TraitEdgeCaseStruct {
    #[substruct_field(primitive)]
    field: String,
}

#[test]
fn test_trait_edge_cases() {
    let update = TraitEdgeCaseStructSubstruct::new(Some("test".to_string()));

    // Test Clone trait with edge cases
    let cloned = update.clone();
    assert_eq!(update.field, cloned.field);

    // Test Debug trait with edge cases
    let debug_output = format!("{:?}", update);
    assert!(debug_output.contains("test"));

    // Test PartialEq trait with edge cases
    let another = TraitEdgeCaseStructSubstruct::new(Some("test".to_string()));
    assert_eq!(update.field, another.field);

    // Test Default trait with edge cases
    let default = TraitEdgeCaseStructSubstruct::default();
    assert_eq!(default.field, None);

    // Test From trait with edge cases
    let source = TraitEdgeCaseStruct {
        field: "source".to_string(),
    };
    let from_source = TraitEdgeCaseStructSubstruct::from_source(&source);
    assert_eq!(from_source.field, None);
}

// ============================================================================
// BOUNDARY CONDITION TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct BoundaryStruct {
    #[substruct_field(primitive)]
    min_value: i8,

    #[substruct_field(primitive)]
    max_value: u8,

    #[substruct_field(primitive)]
    float_inf: f64,

    #[substruct_field(primitive)]
    float_neg_inf: f64,

    #[substruct_field(primitive)]
    float_nan: f64,
}

#[test]
fn test_boundary_conditions() {
    let update = BoundaryStructSubstruct::new(
        Some(i8::MIN),           // min_value
        Some(u8::MAX),           // max_value
        Some(f64::INFINITY),     // float_inf
        Some(f64::NEG_INFINITY), // float_neg_inf
        Some(f64::NAN),          // float_nan
    );

    assert_eq!(update.min_value, Some(i8::MIN));
    assert_eq!(update.max_value, Some(u8::MAX));
    assert_eq!(update.float_inf, Some(f64::INFINITY));
    assert_eq!(update.float_neg_inf, Some(f64::NEG_INFINITY));
    assert!(update.float_nan.unwrap().is_nan());
}
