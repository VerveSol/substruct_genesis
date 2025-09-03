use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

// ============================================================================
// COMPLEX CUSTOM TYPES TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Person {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    age: u32,
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
    #[substruct_field(primitive)]
    zip: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Company {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested)]
    ceo: Person,
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
    #[substruct_field(primitive)]
    employee_count: u32,
}

#[test]
fn test_custom_type_derivation() {
    let address_update = AddressBuilder::new(
        Some("123 Main St".to_string()),
        Some("Main City".to_string()),
        Some("12345".to_string()),
    );

    let person_update =
        PersonSubstruct::new(Some("John".to_string()), Some(30), Some(address_update));

    assert_eq!(person_update.name, Some("John".to_string()));
    assert_eq!(person_update.age, Some(30));
    assert!(person_update.address.is_some());

    if let Some(address) = &person_update.address {
        assert_eq!(address.street, Some("123 Main St".to_string()));
        assert_eq!(address.city, Some("Main City".to_string()));
        assert_eq!(address.zip, Some("12345".to_string()));
    }
}

#[test]
fn test_nested_custom_types() {
    // Test deep nesting with company → person → address
    let address_update = AddressBuilder::new(
        Some("456 New St".to_string()),
        Some("New City".to_string()),
        None,
    );

    let person_update = PersonSubstruct::new(
        Some("New CEO".to_string()),
        Some(50),
        Some(address_update.clone()),
    );

    let company_update = CompanySubstruct::new(
        Some("New Corp".to_string()),
        Some(person_update),
        Some(address_update),
        Some(200),
    );

    // Validate company substruct fields
    assert_eq!(company_update.name, Some("New Corp".to_string()));
    assert_eq!(company_update.employee_count, Some(200));
    assert!(company_update.ceo.is_some());
    assert!(company_update.address.is_some());

    // Validate person substruct fields
    if let Some(ceo) = &company_update.ceo {
        assert_eq!(ceo.name, Some("New CEO".to_string()));
        assert_eq!(ceo.age, Some(50));
        assert!(ceo.address.is_some());
    }

    // Validate address substruct fields
    if let Some(address) = &company_update.address {
        assert_eq!(address.street, Some("456 New St".to_string()));
        assert_eq!(address.city, Some("New City".to_string()));
        assert_eq!(address.zip, None);
    }
}

#[test]
fn test_custom_type_none_update() {
    // Test substruct with some fields set to None
    let person_update = PersonSubstruct::new(
        Some("Bob".to_string()),
        None,
        None, // no address update
    );

    // Validate the fields
    assert_eq!(person_update.name, Some("Bob".to_string()));
    assert_eq!(person_update.age, None);
    assert!(person_update.address.is_none());
}

#[test]
fn test_custom_type_empty_update() {
    // Test default substruct creation
    let empty_update = PersonSubstruct::default();

    // Validate that it's empty
    assert!(empty_update.is_empty());

    // Validate all fields are None
    assert_eq!(empty_update.name, None);
    assert_eq!(empty_update.age, None);
    assert!(empty_update.address.is_none());
}

// ============================================================================
// EDGE CASES TESTS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct EmptyStruct {}

#[test]
fn test_empty_struct() {
    let update = EmptyStructSubstruct::default();

    // Should compile and work with empty structs
    // Empty structs have no fields, so no is_empty method is generated
    let _: EmptyStructSubstruct = update;
}

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
