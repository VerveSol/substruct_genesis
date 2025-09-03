use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

// ============================================================================
// INTEGRATION TESTS - MULTIPLE FEATURES WORKING TOGETHER
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
#[substruct_builder(name = "UserProfileBuilder")]
struct UserProfile {
    #[substruct_field(primitive)]
    username: String,
    #[substruct_field(primitive)]
    email: Option<String>,
    #[substruct_field(json)]
    preferences: UserPreferences,
    #[substruct_field(nested)]
    address: UserAddress,
    #[substruct_field(primitive)]
    version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct UserPreferences {
    #[substruct_field(primitive)]
    theme: String,
    #[substruct_field(primitive)]
    language: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct UserAddress {
    #[substruct_field(primitive)]
    street: String,
    #[substruct_field(primitive)]
    city: String,
    #[substruct_field(primitive)]
    country: String,
}

#[test]
fn test_complex_integration() {
    // Test that all features work together: custom naming, mixed field types, nesting, and wrapping

    // Create nested updates
    let preferences = UserPreferences {
        theme: "dark".to_string(),
        language: "en".to_string(),
    };

    let address_update = UserAddressSubstruct::new(
        Some("123 Main St".to_string()),
        Some("New York".to_string()),
        Some("USA".to_string()),
    );

    let profile_update = UserProfileBuilder::new(
        Some("john_doe".to_string()),
        Some(Some("john@example.com".to_string())),
        Some(serde_json::to_value(&preferences).unwrap()),
        Some(address_update),
        Some(2), // version (wrapped)
    );

    // Validate all features work together
    assert_eq!(profile_update.username, Some("john_doe".to_string()));
    assert_eq!(
        profile_update.email,
        Some(Some("john@example.com".to_string()))
    );
    assert!(profile_update.preferences.is_some());
    assert!(profile_update.address.is_some());
    assert_eq!(profile_update.version, Some(2));

    // Test nested structures
    if let Some(address) = &profile_update.address {
        assert_eq!(address.street, Some("123 Main St".to_string()));
        assert_eq!(address.city, Some("New York".to_string()));
        assert_eq!(address.country, Some("USA".to_string()));
    }

    // Test JSON field
    if let Some(prefs) = &profile_update.preferences {
        let _: serde_json::Value = prefs.clone();
    }
}

#[test]
fn test_mixed_field_operations() {
    // Test various operations on the integrated struct

    // Test default creation
    let default_profile = UserProfileBuilder::default();
    assert!(default_profile.is_empty());

    // Test partial updates
    let partial_update = UserProfileBuilder::new(
        Some("jane_doe".to_string()),
        None,    // no email change
        None,    // no preferences change
        None,    // no address change
        Some(3), // version change
    );

    assert_eq!(partial_update.username, Some("jane_doe".to_string()));
    assert_eq!(partial_update.email, None);
    assert_eq!(partial_update.preferences, None);
    assert!(partial_update.address.is_none());
    assert_eq!(partial_update.version, Some(3));

    // Test from_source
    let source_profile = UserProfile {
        username: "source_user".to_string(),
        email: Some("source@example.com".to_string()),
        preferences: UserPreferences {
            theme: "light".to_string(),
            language: "fr".to_string(),
        },
        address: UserAddress {
            street: "Source St".to_string(),
            city: "Source City".to_string(),
            country: "Source Country".to_string(),
        },
        version: 1,
    };

    let from_source = UserProfileBuilder::from_source(&source_profile);
    assert_eq!(from_source.username, None);
    assert_eq!(from_source.email, None);
    assert!(from_source.preferences.is_some());
    assert!(from_source.address.is_none());
    assert_eq!(from_source.version, None); // wrapped field gets default
}
