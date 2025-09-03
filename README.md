# Substruct Genesis

A Rust procedural macro that automatically generates independent substruct builders for your data structures, making it easy to create partial update structures with clear semantics.

## 📚 Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Usage](#usage)
  - [Basic Example](#basic-example)
  - [Field Types](#field-types)
  - [Generated Methods](#generated-methods)
- [Examples](#examples)
- [Important Notes](#important-notes)
- [Requirements](#requirements)
- [Error Handling](#error-handling)
- [Performance](#performance)
- [Testing](#testing)
  - [Test Files Overview](#test-files-overview)
  - [Detailed Test Breakdown](#detailed-test-breakdown)
  - [Running Tests](#running-tests)
  - [Test Architecture](#test-architecture)
- [Future Considerations](#future-considerations)

## Overview

The `SubstructBuilder` macro generates a substruct for your original struct, containing only the fields you explicitly mark for updates. The generated substruct is completely independent of the original struct, providing a clean separation of concerns for building update operations.

## Key Features

- **Independent Substructs**: Generated substructs are completely independent of the original struct
- **Selective Field Inclusion**: Only fields with `#[substruct_field]` attributes are included
- **Primitive Fields**: Automatically wrapped in `Option<T>` for nullable updates (configurable with `wrap` attribute)
- **Option Fields**: Use `Option<Option<T>>` for clear update semantics
- **JSON Fields**: Handle complex types as `Option<serde_json::Value>`
- **Nested Types**: Support nested substruct builders with custom naming
- **Custom Struct Names**: Configure the generated substruct name at the struct level
- **No Dependencies**: Substructs don't reference or depend on the original struct

## Usage

### Basic Example

```rust
use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct User {
    #[substruct_field(primitive)]
    name: String,
    // age field has no attribute, so it's completely excluded from the substruct
    age: u32,
    #[substruct_field(primitive)]
    active: bool,
}

// The macro generates UserSubstruct with ONLY:
// - name: Option<String>
// - active: Option<bool>
// Note: age field is completely absent
```

### Field Types

#### Primitive Fields (`#[substruct_field(primitive)]`)
- **Update type**: `Option<T>` (default) or `T` (when `wrap = false`)
- **Semantics**: 
  - `Some(value)` = set to value (when wrapped)
  - `None` = no change (when wrapped)
  - `value` = set to value (when not wrapped)

```rust
#[derive(SubstructBuilder)]
struct User {
    #[substruct_field(primitive)]                    // wrapped (default)
    name: String,                                    // -> Option<String>
    
    #[substruct_field(primitive, wrap = false)]      // not wrapped
    id: u32,                                         // -> u32
}
```

#### Option Fields (`#[substruct_field(primitive)]` on `Option<T>`)
- **Update type**: `Option<Option<T>>`
- **Semantics**:
  - `None` = no change
  - `Some(None)` = set to None
  - `Some(Some(value))` = set to Some(value)

```rust
#[derive(SubstructBuilder)]
struct Config {
    #[substruct_field(primitive)]
    theme: Option<String>,
}

let update = ConfigSubstruct::new(
    Some(Some("dark".to_string())),  // set to Some("dark")
    // or Some(None) to set to None
    // or None for no change
);
```

#### JSON Fields (`#[substruct_field(json)]`)
- **Update type**: `Option<serde_json::Value>`
- **Semantics**: 
  - `Some(value)` = set to deserialized value
  - `None` = no change

```rust
#[derive(SubstructBuilder)]
struct Settings {
    #[substruct_field(json)]
    preferences: UserPreferences,
}

let update = SettingsSubstruct::new(
    Some(serde_json::to_value(&new_prefs).unwrap())
);
```

#### Nested Types (`#[substruct_field(nested)]`)
- **Update type**: `Option<TypeSubstruct>`
- **Semantics**: Recursive updates for nested structs

```rust
#[derive(SubstructBuilder)]
struct Profile {
    #[substruct_field(nested)]
    address: Address,
}

// Generates ProfileSubstruct with:
// - address: Option<AddressSubstruct>
```

#### Custom Nested Type Names
You can specify custom names for nested types:

```rust
#[derive(SubstructBuilder)]
struct Profile {
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
}

// Generates ProfileSubstruct with:
// - address: Option<AddressBuilder>
```

#### Struct-Level Naming
Customize the entire substruct name:

```rust
#[derive(SubstructBuilder)]
#[substruct_builder(name = "UserBuilder")]
struct User {
    #[substruct_field(primitive)]
    name: String,
}

// Generates UserBuilder instead of UserSubstruct
```

### Generated Methods

#### `new(...)`
Constructor that takes all updatable fields as parameters.

#### `from_source(source: &T) -> Self`
Creates a substruct from an existing instance (all fields set to no-change).

#### `is_empty(&self) -> bool`
Returns `true` if no fields would be changed by this update.

#### `Default::default()`
Creates a substruct where all fields indicate "no change".

#### `From<T>` and `From<&T>`
Implementations for creating substructs from owned and borrowed instances.

## Examples

### Complete Example

```rust
use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct UserProfile {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    age: u32,
    #[substruct_field(primitive)]
    email: Option<String>,
    #[substruct_field(json)]
    preferences: UserPreferences,
    id: String,  // completely excluded from substruct
}

// Create an update
let update = UserProfileSubstruct::new(
    Some("John Doe".to_string()),     // change name
    None,                             // don't change age
    Some(None),                       // set email to None
    Some(serde_json::to_value(&new_prefs).unwrap()), // change preferences
);

// The substruct is completely independent
// No apply_to or would_change methods exist
```

### Nested Structs with Custom Names

```rust
#[derive(SubstructBuilder)]
#[substruct_builder(name = "AddressBuilder")]
struct Address {
    #[substruct_field(primitive)]
    street: String,
    #[substruct_field(primitive)]
    city: String,
}

#[derive(SubstructBuilder)]
struct Profile {
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
}

let address_update = AddressBuilder::new(
    Some("123 New St".to_string()),
    Some("New City".to_string()),
);

let profile_update = ProfileSubstruct::new(Some(address_update));
```

### Unwrapped Fields

```rust
#[derive(SubstructBuilder)]
struct Config {
    #[substruct_field(primitive, wrap = false)]
    version: u32,  // Always required, not wrapped in Option
    #[substruct_field(primitive)]
    theme: String, // Optional, wrapped in Option<String>
}

let update = ConfigSubstruct::new(
    2,                              // version is required
    Some("dark".to_string()),      // theme is optional
);
```

## Important Notes

- **Field Exclusion**: Fields without `#[substruct_field]` attributes are completely excluded from the generated substruct
- **Independent Substructs**: The generated substruct has no dependencies on the original struct
- **No Update Methods**: The substruct is designed for building update data, not applying it to the original struct
- **Clean Separation**: Perfect for API endpoints, configuration updates, and other scenarios where you want to separate update data from the target structure

## Requirements

- Rust 1.56+
- `serde` for serialization support
- Fields must implement `Clone` and `PartialEq`

## Error Handling

The macro provides clear error messages for:
- Invalid attribute syntax
- Unsupported field types
- Misuse of `serde_json::Value` with `#[substruct_field(json)]`
- Invalid nested type specifications

## Performance

- No runtime overhead for field access
- Minimal memory allocation during creation
- Smart trait derivation (avoids `Eq` for `f64`, `PartialEq` for `String`)
- Clean, independent data structures

## Testing

The project includes a comprehensive test suite that validates all macro functionality and edge cases.

### Test Files Overview

| Test File | Tests | Status | Purpose |
|-----------|-------|--------|---------|
| `basic_functionality.rs` | 5 | ✅ All Passing | Core macro functionality and field exclusion |
| `custom_types.rs` | 4 | ✅ All Passing | Nested structs and complex field types |
| `primitive_fields.rs` | 2 | ✅ All Passing | Primitive field handling and wrapping |
| `json_fields.rs` | 3 | ✅ All Passing | JSON field serialization support |
| `nested_types.rs` | 2 | ✅ All Passing | Nested struct generation |
| `edge_cases.rs` | 2 | ✅ All Passing | Edge case handling |
| `debug_wrap.rs` | 1 | ✅ All Passing | Debug attribute and wrapping |
| `simple_nested.rs` | 1 | ✅ All Passing | Simple nested struct scenarios |
| `struct_level_names.rs` | 1 | ✅ All Passing | Custom struct naming |
| `wrap_attribute.rs` | 1 | ✅ All Passing | Field wrapping configuration |

**Total: 22 tests, all passing** ✅

### Detailed Test Breakdown

#### 1. `basic_functionality.rs` - Core Functionality Tests

Tests the fundamental behavior of the macro with a simple struct containing both marked and unmarked fields.

**Test Cases:**
- **`test_basic_struct_derivation`**: Validates substruct creation with only marked fields
- **`test_basic_struct_default`**: Tests default substruct creation
- **`test_basic_struct_from_source`**: Tests creating substruct from source struct
- **`test_basic_struct_is_empty`**: Validates empty state detection
- **`test_basic_struct_from_owned`**: Tests owned source conversion

**Key Validation:**
- Fields without `#[substruct_field]` are completely excluded
- Only `name` and `active` fields appear in the substruct
- `age` field is absent from all substruct operations

#### 2. `custom_types.rs` - Complex Type Handling

Tests nested structs, custom types, and complex field relationships.

**Test Cases:**
- **`test_custom_type_derivation`**: Basic nested substruct creation
- **`test_nested_custom_types`**: Deep nesting with company → person → address
- **`test_custom_type_none_update`**: Partial field updates with None values
- **`test_custom_type_empty_update`**: Default substruct creation and validation

**Key Validation:**
- Nested substructs are generated correctly
- Complex field hierarchies work properly
- Field values are set and retrieved correctly
- Default states are handled properly

#### 3. `primitive_fields.rs` - Primitive Field Tests

Tests primitive field handling, wrapping, and unwrapping behavior.

**Test Cases:**
- **`test_primitive_struct_creation`**: Basic primitive field substruct creation
- **`test_primitive_struct_empty_update`**: Empty substruct validation

**Key Validation:**
- Primitive fields are wrapped in `Option<T>` by default
- Unwrapped fields work correctly
- Default values are properly set

#### 4. `json_fields.rs` - JSON Field Support

Tests JSON field serialization and handling.

**Test Cases:**
- **`test_json_field`**: Basic JSON field functionality
- **`test_primitive_field`**: Mixed JSON and primitive fields
- **`test_json_struct_has_preferences_field`**: JSON field in nested context

**Key Validation:**
- JSON fields are properly typed as `Option<serde_json::Value>`
- Mixed field types work together
- JSON fields integrate with other field types

#### 5. `nested_types.rs` - Nested Structure Tests

Tests nested struct generation and field handling.

**Test Cases:**
- **`test_nested_struct_derivation`**: Basic nested struct creation
- **`test_nested_struct_from_source`**: Source conversion for nested types

**Key Validation:**
- Nested structs are generated with correct field types
- Source conversion works for nested structures
- Field relationships are maintained

#### 6. `edge_cases.rs` - Edge Case Handling

Tests edge cases and boundary conditions.

**Test Cases:**
- **`test_empty_struct`**: Struct with no fields
- **`test_single_field_struct`**: Minimal struct with one field

**Key Validation:**
- Empty structs are handled gracefully
- Single field structs work correctly
- Edge cases don't cause compilation errors

#### 7. `debug_wrap.rs` - Debug and Wrapping

Tests debug attribute and field wrapping behavior.

**Test Cases:**
- **`test_debug_wrap`**: Debug attribute with wrapped fields

**Key Validation:**
- Debug attributes work with wrapped fields
- Field wrapping doesn't interfere with debug output

#### 8. `simple_nested.rs` - Simple Nesting

Tests basic nested struct scenarios.

**Test Cases:**
- **`test_simple_nested`**: Simple nested struct creation

**Key Validation:**
- Basic nesting works correctly
- Field types are properly inferred

#### 9. `struct_level_names.rs` - Custom Naming

Tests custom struct naming functionality.

**Test Cases:**
- **`test_struct_level_names`**: Custom substruct naming

**Key Validation:**
- Custom names are applied correctly
- Generated structs use specified names

#### 10. `wrap_attribute.rs` - Wrapping Configuration

Tests field wrapping attribute behavior.

**Test Cases:**
- **`test_wrap_attribute_parsing`**: Wrap attribute parsing

**Key Validation:**
- Wrap attributes are parsed correctly
- Field wrapping behavior is configurable

### Running Tests

#### Individual Test Files
```bash
cargo test --test basic_functionality
cargo test --test custom_types
cargo test --test primitive_fields
# ... etc
```

#### All Tests
```bash
cargo test
```

#### Test with Output
```bash
cargo test -- --nocapture
```

### Test Architecture

**Test Structure:**
Each test file follows a consistent pattern:
1. **Struct Definition**: Defines test structs with various field types and attributes
2. **Substruct Generation**: Tests that substructs are generated correctly
3. **Field Validation**: Validates that fields have correct types and values
4. **Method Testing**: Tests generated methods like `new()`, `default()`, `from_source()`
5. **Edge Case Handling**: Tests boundary conditions and error cases

**Key Testing Principles:**
- **Independence**: Tests don't depend on external state
- **Completeness**: Each test validates a specific aspect of functionality
- **Clarity**: Test names and assertions clearly indicate what's being tested
- **Coverage**: Tests cover all major macro features and edge cases

## Future Considerations

### Potential Additions
- **Performance Tests**: Benchmark macro compilation and runtime performance
- **Error Case Tests**: Test macro error messages and edge case handling
- **Integration Tests**: Test with real-world use cases and frameworks
- **Documentation Tests**: Ensure examples in documentation compile and work

### Test Maintenance
- Tests are updated whenever macro behavior changes
- New features require corresponding test coverage
- Edge cases are added as they're discovered
- Test names and structure follow consistent patterns

---

## Conclusion

The Substruct Genesis macro provides a clean, efficient way to generate independent substruct builders for your Rust data structures. With comprehensive test coverage and clear documentation, the project ensures reliability and ease of use.

**Key Benefits:**
1. **Field exclusion works correctly** - unmarked fields are completely absent
2. **Substructs are independent** - no dependencies on original structs
3. **All field types are supported** - primitive, JSON, nested, and custom types
4. **Configuration options work** - wrapping, naming, and attribute parsing
5. **Edge cases are handled** - empty structs, single fields, complex nesting

The test suite serves as both validation of current functionality and documentation of expected behavior, ensuring the macro remains reliable and well-tested as it evolves.


