use serde::{Deserialize, Serialize};
use serde_json::Value;
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UserPreferences {
    theme: String,
    notifications: bool,
    language: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct PrimitiveStruct {
    #[substruct_field(primitive)]
    data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct JsonStruct {
    #[substruct_field(json)]
    preferences: UserPreferences,
}

#[test]
fn test_primitive_field() {
    let data = serde_json::json!({"test": "value"});

    let update = PrimitiveStructSubstruct::new(Some(data.clone()));

    assert_eq!(update.data, Some(data));
}

#[test]
fn test_json_field() {
    let preferences = UserPreferences {
        theme: "dark".to_string(),
        notifications: true,
        language: "en".to_string(),
    };

    let update = JsonStructSubstruct::new(Some(serde_json::to_value(&preferences).unwrap()));

    assert_eq!(
        update.preferences,
        Some(serde_json::to_value(&preferences).unwrap())
    );
}

#[test]
fn test_json_struct_has_preferences_field() {
    // This test verifies that the macro generates the preferences field
    let update = JsonStructSubstruct {
        preferences: Some(serde_json::json!({"theme": "dark"})),
    };

    assert!(update.preferences.is_some());

    // Test that new() method works with preferences parameter
    let update2 = JsonStructSubstruct::new(Some(serde_json::json!({"theme": "light"})));
    assert!(update2.preferences.is_some());
}
