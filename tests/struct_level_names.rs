use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

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
#[substruct_builder(name = "CityBuilder")]
struct City {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    state: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "PersonBuilder")]
struct Person {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    age: u32,
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
    #[substruct_field(nested, nested_type = "CityBuilder")]
    city: City,
}

#[test]
fn test_struct_level_names() {
    // Test that struct-level names work correctly

    // Address should generate AddressBuilder
    let address_update = AddressBuilder::new(
        Some("123 New St".to_string()),
        Some("New City".to_string()),
        Some("12345".to_string()),
    );

    // City should generate CityBuilder
    let city_update = CityBuilder::new(Some("New York".to_string()), Some("NY".to_string()));

    // Person should generate PersonBuilder
    let person_update = PersonBuilder::new(
        Some("John".to_string()),
        Some(30),
        Some(address_update),
        Some(city_update),
    );

    // Verify the types are correct
    assert_eq!(person_update.name, Some("John".to_string()));
    assert_eq!(person_update.age, Some(30));
    assert!(person_update.address.is_some());
    assert!(person_update.city.is_some());

    let source = PersonBuilder::default();
    assert!(source.name.is_none());
    assert!(source.age.is_none());
    assert!(source.address.is_none());
    assert!(source.city.is_none());

    // Test creating actual Person struct instances
    let address = Address {
        street: "123 Old St".to_string(),
        city: "Old City".to_string(),
        zip: "54321".to_string(),
    };

    let city = City {
        name: "Old Town".to_string(),
        state: "OT".to_string(),
    };

    let person = Person {
        name: "Alice".to_string(),
        age: 25,
        address,
        city,
    };

    // Verify the Person struct was created correctly
    assert_eq!(person.name, "Alice");
    assert_eq!(person.age, 25);
    assert_eq!(person.address.street, "123 Old St");
    assert_eq!(person.city.name, "Old Town");
}
