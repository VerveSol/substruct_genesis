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

#[test]
fn test_basic_struct_field_count() {
    let empty_update = BasicStructSubstruct::default();
    assert_eq!(empty_update.field_count(), 0);

    let partial_update = BasicStructSubstruct::new(Some("Test".to_string()), None);
    assert_eq!(partial_update.field_count(), 1);

    let full_update = BasicStructSubstruct::new(Some("Test".to_string()), Some(true));
    assert_eq!(full_update.field_count(), 2);
}

#[test]
fn test_basic_struct_clear() {
    let mut update = BasicStructSubstruct::new(Some("Test".to_string()), Some(true));
    assert_eq!(update.field_count(), 2);

    update.clear();
    assert_eq!(update.field_count(), 0);
    assert!(update.is_empty());
}

#[test]
fn test_basic_struct_apply_to() {
    let mut user = BasicStruct {
        name: "Alice".to_string(),
        age: 25,
        active: false,
    };

    let update = BasicStructSubstruct::new(Some("Bob".to_string()), Some(true));

    update.apply_to(&mut user);

    assert_eq!(user.name, "Bob");
    assert_eq!(user.active, true);
}

#[test]
fn test_basic_struct_would_change() {
    let user = BasicStruct {
        name: "Alice".to_string(),
        age: 25,
        active: false,
    };

    let update = BasicStructSubstruct::new(Some("Bob".to_string()), Some(true));

    assert!(update.would_change(&user));

    let no_change_update = BasicStructSubstruct::new(Some("Alice".to_string()), Some(false));

    assert!(!no_change_update.would_change(&user));
}

#[test]
fn test_basic_struct_merge() {
    let update1 = BasicStructSubstruct::new(Some("Alice".to_string()), None);

    let update2 = BasicStructSubstruct::new(None, Some(true));

    let merged = update1.merge(update2);

    assert_eq!(merged.name, Some("Alice".to_string()));
    assert_eq!(merged.active, Some(true));
}

#[test]
fn test_basic_struct_has_field() {
    let update = BasicStructSubstruct::new(Some("Alice".to_string()), None);

    assert!(update.has_field("name"));
    assert!(!update.has_field("active"));
}
