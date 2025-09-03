use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct WrapTestStruct {
    #[substruct_field(primitive)]
    wrapped_field: String,

    #[substruct_field(primitive, wrap = true)]
    explicitly_wrapped_field: u32,

    #[substruct_field(primitive, wrap = false)]
    unwrapped_field: u32,
}

#[test]
fn test_wrap_attribute_parsing() {
    // Test that the wrap attribute is parsed correctly
    let update = WrapTestStructSubstruct::new(Some("test".to_string()), Some(42), 100);

    assert_eq!(update.wrapped_field, Some("test".to_string()));
    assert_eq!(update.explicitly_wrapped_field, Some(42));
    assert_eq!(update.unwrapped_field, 100);

    // Test that wrapped fields are wrapped in Option<T>, unwrapped fields are not
    let source = WrapTestStructSubstruct::default();
    assert_eq!(source.wrapped_field, None);
    assert_eq!(source.explicitly_wrapped_field, None);
    assert_eq!(source.unwrapped_field, 0); // u32::default()
}
