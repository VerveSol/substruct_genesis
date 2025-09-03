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
fn test_custom_type_apply_to() {
    let source_address = Address {
        street: "123 Old St".to_string(),
        city: "Old City".to_string(),
        zip: "54321".to_string(),
    };

    let source_person = Person {
        name: "Alice".to_string(),
        age: 25,
        address: source_address.clone(),
    };

    let address_update = AddressSubstruct::new(
        Some("456 New St".to_string()),
        None, // no change
        Some("67890".to_string()),
    );

    let person_update = PersonSubstruct::new(
        Some("Bob".to_string()),
        None, // no change
        Some(address_update),
    );

    let result = person_update.apply_to(&source_person);

    assert_eq!(result.name, "Bob".to_string());
    assert_eq!(result.age, 25); // unchanged
    assert_eq!(result.address.street, "456 New St".to_string());
    assert_eq!(result.address.city, "Old City".to_string()); // unchanged
    assert_eq!(result.address.zip, "67890".to_string());
}

#[test]
fn test_custom_type_would_change() {
    let source_address = Address {
        street: "123 Old St".to_string(),
        city: "Old City".to_string(),
        zip: "54321".to_string(),
    };

    let source_person = Person {
        name: "Alice".to_string(),
        age: 25,
        address: source_address.clone(),
    };

    // Test nested custom type change
    let address_update = AddressSubstruct::new(Some("456 New St".to_string()), None, None);

    let person_update = PersonSubstruct::new(None, None, Some(address_update));

    assert!(person_update.would_change(&source_person)); // address changed

    // Test no change
    let no_change_address = AddressSubstruct::new(
        Some("123 Old St".to_string()), // same value
        None,
        None,
    );

    let no_change_person = PersonSubstruct::new(None, None, Some(no_change_address));

    assert!(!no_change_person.would_change(&source_person)); // no changes
}

#[test]
fn test_nested_custom_types() {
    let source_address = Address {
        street: "123 Main St".to_string(),
        city: "Main City".to_string(),
        zip: "12345".to_string(),
    };

    let source_person = Person {
        name: "CEO".to_string(),
        age: 45,
        address: source_address.clone(),
    };

    let source_company = Company {
        name: "Test Corp".to_string(),
        ceo: source_person.clone(),
        address: source_address.clone(),
        employee_count: 100,
    };

    // Create updates for nested structures
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
        Some(person_update),
        Some(address_update),
        Some(200),
    );

    let result = company_update.apply_to(&source_company);

    assert_eq!(result.name, "New Corp".to_string());
    assert_eq!(result.ceo.name, "New CEO".to_string());
    assert_eq!(result.ceo.age, 50);
    assert_eq!(result.ceo.address.street, "456 New St".to_string());
    assert_eq!(result.ceo.address.city, "New City".to_string());
    assert_eq!(result.ceo.address.zip, "12345".to_string()); // unchanged
    assert_eq!(result.address.street, "456 New St".to_string());
    assert_eq!(result.address.city, "New City".to_string());
    assert_eq!(result.address.zip, "12345".to_string()); // unchanged
    assert_eq!(result.employee_count, 200);
}

#[test]
fn test_custom_type_none_update() {
    let source_address = Address {
        street: "123 St".to_string(),
        city: "City".to_string(),
        zip: "12345".to_string(),
    };

    let source_person = Person {
        name: "Alice".to_string(),
        age: 25,
        address: source_address.clone(),
    };

    let person_update = PersonSubstruct::new(
        Some("Bob".to_string()),
        None,
        None, // no address update
    );

    let result = person_update.apply_to(&source_person);

    assert_eq!(result.name, "Bob".to_string());
    assert_eq!(result.age, 25); // unchanged
    assert_eq!(result.address, source_address); // unchanged
}

#[test]
fn test_custom_type_empty_update() {
    let source_address = Address {
        street: "123 Test St".to_string(),
        city: "Test City".to_string(),
        zip: "12345".to_string(),
    };

    let source_person = Person {
        name: "Test Person".to_string(),
        age: 30,
        address: source_address.clone(),
    };

    let empty_update = PersonSubstruct::default();

    assert!(empty_update.is_empty());
    assert!(!empty_update.would_change(&source_person));

    let result = empty_update.apply_to(&source_person);
    assert_eq!(result, source_person); // no changes applied
}
