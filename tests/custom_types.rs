use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Address {
    #[substruct_field(primitive)]
    street: String,
    #[substruct_field(primitive)]
    city: String,
    #[substruct_field(primitive)]
    zip: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Person {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    age: u32,
    #[substruct_field(nested)]
    address: Address,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Company {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested)]
    ceo: Person,
    #[substruct_field(nested)]
    address: Address,
    #[substruct_field(primitive)]
    employee_count: u32,
}

#[test]
fn test_custom_type_derivation() {
    let address_update = AddressSubstruct::new(
        Some("123 New St".to_string()),
        Some("New City".to_string()),
        Some("12345".to_string()),
    );

    let person_update =
        PersonSubstruct::new(Some("John".to_string()), Some(30), Some(address_update));

    assert_eq!(person_update.name, Some("John".to_string()));
    assert_eq!(person_update.age, Some(30));
    assert!(person_update.address.is_some());
}
#[test]
fn test_nested_custom_types() {
    // Test nested substruct creation and field validation
    let address_update = AddressSubstruct::new(
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
        Some(person_update.clone()),
        Some(address_update.clone()),
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
