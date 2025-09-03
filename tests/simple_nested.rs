use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Address {
    #[substruct_field(primitive)]
    street: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct Person {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested)]
    address: Address,
}

#[test]
fn test_simple_nested() {
    // This should compile if nested fields work
    let _person = PersonSubstruct::new(
        Some("John".to_string()),
        Some(AddressSubstruct::new(Some("123 St".to_string()))),
    );
    println!("Simple nested test compiled successfully!");
}
