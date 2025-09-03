use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct BasicStruct {
    #[substruct_field(primitive)]
    name: String,
    // Deliberately no attribute here to test the fix
    age: u32,
    #[substruct_field(primitive)]
    active: bool,
}

#[test]
fn test_basic_struct_derivation() {
    // Note: we only have name and active as fields in the substruct (age has no attribute)
    let update = BasicStructSubstruct::new(Some("John".to_string()), Some(true));

    assert_eq!(update.name, Some("John".to_string()));
    assert_eq!(update.active, Some(true));
}

#[test]
fn test_basic_struct_default() {
    let update = BasicStructSubstruct::default();

    // No age field in the substruct
    assert_eq!(update.active, None);
}

#[test]
fn test_basic_struct_from_source() {
    let source = BasicStruct {
        name: "Alice".to_string(),
        age: 30,
        active: false,
    };

    let update = BasicStructSubstruct::from_source(&source);

    // No age field in the substruct
    assert_eq!(update.active, None);
}

#[test]
fn test_basic_struct_is_empty() {
    let empty_update = BasicStructSubstruct::default();
    assert!(empty_update.is_empty());

    let partial_update = BasicStructSubstruct::new(None, None);
    assert!(partial_update.is_empty());

    let full_update = BasicStructSubstruct::new(Some("Test".to_string()), Some(true));
    assert!(!full_update.is_empty());
}

#[test]
fn test_basic_struct_from_owned() {
    let source = BasicStruct {
        name: "Alice".to_string(),
        age: 30,
        active: false,
    };

    let update = BasicStructSubstruct::from(source.clone());

    // No age field in the substruct
    assert_eq!(update.active, None);
}
