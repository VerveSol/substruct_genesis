use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct BasicStruct {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    age: u32,
    #[substruct_field(primitive)]
    active: bool,
}

#[test]
fn test_basic_struct_derivation() {
    let update = BasicStructSubstruct::new(Some("John".to_string()), Some(25), Some(true));

    assert_eq!(update.name, Some("John".to_string()));
    assert_eq!(update.age, Some(25));
    assert_eq!(update.active, Some(true));
}

#[test]
fn test_basic_struct_default() {
    let update = BasicStructSubstruct::default();

    assert_eq!(update.age, None);
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

    assert_eq!(update.age, None);
    assert_eq!(update.active, None);
}

#[test]
fn test_basic_struct_apply_to() {
    let source = BasicStruct {
        name: "Alice".to_string(),
        age: 30,
        active: false,
    };

    let update = BasicStructSubstruct::new(None, Some(35), None);

    let result = update.apply_to(&source);

    assert_eq!(result.age, 35);
    assert_eq!(result.active, false); // unchanged
}

#[test]
fn test_basic_struct_would_change() {
    let source = BasicStruct {
        name: "Alice".to_string(),
        age: 30,
        active: false,
    };

    let update = BasicStructSubstruct::new(
        None,     // no change
        Some(35), // different value
        None,     // no change
    );

    assert!(update.would_change(&source)); // age changed
}

#[test]
fn test_basic_struct_would_not_change() {
    let source = BasicStruct {
        name: "Alice".to_string(),
        age: 30,
        active: false,
    };

    let update = BasicStructSubstruct::new(
        Some("Alice".to_string()), // same value
        Some(30),                  // same value
        Some(false),               // same value
    );

    assert!(!update.would_change(&source)); // no changes
}

#[test]
fn test_basic_struct_is_empty() {
    let empty_update = BasicStructSubstruct::default();
    assert!(empty_update.is_empty());

    let partial_update = BasicStructSubstruct::new(None, None, None);
    assert!(partial_update.is_empty());

    let full_update = BasicStructSubstruct::new(Some("Test".to_string()), Some(25), Some(true));
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

    assert_eq!(update.age, None);
    assert_eq!(update.active, None);
}
