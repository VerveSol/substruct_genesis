# Substruct Genesis

A Rust procedural macro that automatically generates independent substruct builders for your data structures, making it easy to create partial update structures with clear semantics.

## 📚 Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Code Architecture](#-code-architecture)
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

Built with a clean, modular architecture, the macro separates processing logic from code generation, making it maintainable, extensible, and easy to understand.

## Key Features

- **Independent Substructs**: Generated substructs are completely independent of the original struct
- **Selective Field Inclusion**: Only fields with `#[substruct_field]` attributes are included
- **Primitive Fields**: Automatically wrapped in `Option<T>` for nullable updates (configurable with `option` attribute)
- **Option Fields**: Use `Option<Option<T>>` for clear update semantics
- **JSON Fields**: Handle complex types as `Option<serde_json::Value>`
- **Nested Types**: Support nested substruct builders with custom naming
- **Custom Struct Names**: Configure the generated substruct name at the struct level
- **Advanced Nested Naming**: Custom naming for nested types in complex hierarchies
- **Utility Methods**: Built-in methods for field counting, clearing, state management, and update operations
- **No Dependencies**: Substructs don't reference or depend on the original struct

## 🏗️ Code Architecture

The macro is built with a clean, modular architecture that separates concerns for maintainability and extensibility:

### Module Structure

```
src/
├── lib.rs                    # Main macro entry point and orchestration
├── generator.rs              # Code generation and output formatting
└── processor/                # Processing logic organized in subfolder
    ├── mod.rs               # Module declarations and exports
    ├── attributes.rs        # Attribute parsing utilities
    └── fields.rs            # Field processing and analysis
```

### Key Components

- **`lib.rs`** - The main procedural macro entry point that orchestrates the entire process
- **`generator.rs`** - Handles all code generation logic, including trait derivation, struct definitions, and implementation blocks
- **`processor/attributes.rs`** - Parses and extracts information from struct-level attributes like `substruct_builder` and `derive`
- **`processor/fields.rs`** - Processes individual fields, determines their types, and handles the complex logic for different field kinds (primitive, nested, JSON)

### Design Principles

- **Separation of Concerns**: Each module has a single, focused responsibility
- **Modularity**: Easy to extend with new field types or generation features
- **Maintainability**: Clear module boundaries make the codebase easy to understand and modify
- **Testability**: Individual components can be unit tested in isolation
- **Performance**: Efficient processing with minimal overhead

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
- **Update type**: `Option<T>` (default) or `T` (when `option = false`)
- **Semantics**: 
  - `Some(value)` = set to value (when wrapped)
  - `None` = no change (when wrapped)
  - `value` = set to value (when not wrapped)

```rust
#[derive(SubstructBuilder)]
struct User {
    #[substruct_field(primitive)]                    // option wrapped (default)
    name: String,                                    // -> Option<String>
    
    #[substruct_field(primitive, option = false)]      // not option wrapped
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

**Advanced Usage in Complex Hierarchies:**
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
struct Person {
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
}

// Both Person and Company can use AddressBuilder
// for consistent naming across the hierarchy
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

#### `field_count(&self) -> usize`
Returns the number of fields that have values set (non-default fields).

```rust
let update = UserSubstruct::new(Some("John".to_string()), None, Some(true));
assert_eq!(update.field_count(), 2); // name and active are set
```

#### `clear(&mut self)`
Resets all fields to their default values (None for Option fields, default for unwrapped fields).

```rust
let mut update = UserSubstruct::new(Some("John".to_string()), Some(true));
assert_eq!(update.field_count(), 2);

update.clear();
assert_eq!(update.field_count(), 0);
assert!(update.is_empty());
```

#### `apply_to(&self, target: &mut StructName)`
Applies the updates to a target struct instance. Works with all field types including nested fields (recursive application).

```rust
let mut user = User::new("Alice".to_string(), false, 25);
let update = UserSubstruct::new(Some("Bob".to_string()), Some(true));

update.apply_to(&mut user);
// user.name is now "Bob", user.active is now true
// user.age remains unchanged (25)
```

**Nested Field Support:**
```rust
#[derive(SubstructBuilder)]
struct Person {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested)]
    address: Address,
}

#[derive(SubstructBuilder)]
struct Address {
    #[substruct_field(primitive)]
    street: String,
    #[substruct_field(primitive)]
    city: String,
}

let mut person = Person {
    name: "Alice".to_string(),
    address: Address {
        street: "Old Street".to_string(),
        city: "Old City".to_string(),
    },
};

let address_update = AddressSubstruct::new(
    Some("123 New St".to_string()),
    Some("New City".to_string()),
);
let update = PersonSubstruct::new(
    Some("Bob".to_string()),
    Some(address_update),
);

update.apply_to(&mut person);
// person.name is now "Bob"
// person.address.street is now "123 New St"
// person.address.city is now "New City"
```

#### `would_change(&self, target: &StructName) -> bool`
Checks if applying this update would modify the target struct. Works with all field types including nested fields (recursive checking).

```rust
let user = User::new("Alice".to_string(), false, 25);
let update = UserSubstruct::new(Some("Bob".to_string()), Some(true));

assert!(update.would_change(&user)); // Would change name and active

let no_change = UserSubstruct::new(Some("Alice".to_string()), Some(false));
assert!(!no_change.would_change(&user)); // Would not change anything
```

**Nested Field Support:**
```rust
let person = Person {
    name: "Alice".to_string(),
    address: Address {
        street: "Old Street".to_string(),
        city: "Old City".to_string(),
    },
};

// Update that would change the target
let address_update = AddressSubstruct::new(
    Some("123 New St".to_string()),
    Some("New City".to_string()),
);
let update = PersonSubstruct::new(
    Some("Bob".to_string()),
    Some(address_update),
);

assert!(update.would_change(&person)); // Would change both name and address

// Update that wouldn't change the target
let no_change_address = AddressSubstruct::new(
    Some("Old Street".to_string()),
    Some("Old City".to_string()),
);
let no_change_update = PersonSubstruct::new(
    Some("Alice".to_string()),
    Some(no_change_address),
);

assert!(!no_change_update.would_change(&person)); // Would not change anything
```

#### `merge(self, other: Self) -> Self`
Combines two substructs, with the `other` substruct taking precedence for conflicting fields.

```rust
let update1 = UserSubstruct::new(Some("Alice".to_string()), None);
let update2 = UserSubstruct::new(None, Some(true));

let merged = update1.merge(update2);
// merged has name: Some("Alice") and active: Some(true)
```

#### `has_field(&self, field_name: &str) -> bool`
Checks if a specific field has a value set (non-default value).

```rust
let update = UserSubstruct::new(Some("Alice".to_string()), None);

assert!(update.has_field("name"));    // name is set
assert!(!update.has_field("active")); // active is not set
assert!(!update.has_field("age"));    // age field doesn't exist in substruct
```

#### `into_partial(self) -> HashMap<String, String>`
Converts the substruct into a flexible HashMap representation with string values for easy comparison and inspection.

```rust
let update = UserSubstruct::new(Some("Alice".to_string()), Some(true));

let partial = update.into_partial();

// Check that fields are present
assert!(partial.contains_key("name"));
assert!(partial.contains_key("active"));

// Compare actual values (as string representations)
assert_eq!(partial.get("name"), Some(&"\"Alice\"".to_string()));
assert_eq!(partial.get("active"), Some(&"true".to_string()));

// Fields that aren't set are not included
assert!(!partial.contains_key("age")); // age field doesn't exist in substruct
```

**Nested Field Support:**
```rust
#[derive(SubstructBuilder)]
struct Person {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested)]
    address: Address,
}

#[derive(SubstructBuilder)]
struct Address {
    #[substruct_field(primitive)]
    street: String,
    #[substruct_field(primitive)]
    city: String,
}

let address_update = AddressSubstruct::new(
    Some("123 New St".to_string()),
    Some("New City".to_string()),
);
let update = PersonSubstruct::new(
    Some("Bob".to_string()),
    Some(address_update),
);

let partial = update.into_partial();

// Top-level field
assert_eq!(partial.get("name"), Some(&"\"Bob\"".to_string()));

// Nested field (recursively converted)
assert!(partial.contains_key("address"));
let address_str = partial.get("address").unwrap();
assert!(address_str.contains("street"));
assert!(address_str.contains("city"));
```

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
// Now includes utility methods for update operations
```

### Utility Methods Example

```rust
use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct UserSettings {
    #[substruct_field(primitive)]
    theme: String,
    #[substruct_field(primitive)]
    notifications: bool,
    #[substruct_field(primitive, option = false)]
    version: u32,
}

// Create a partial update
let mut update = UserSettingsSubstruct::new(
    Some("dark".to_string()),
    Some(true),
    2,
);

// Check how many fields are being updated
assert_eq!(update.field_count(), 3);

// Check if the update is empty
assert!(!update.is_empty());

// Clear all fields to reset
update.clear();
assert_eq!(update.field_count(), 0);
assert!(update.is_empty());

// Create a minimal update
let minimal_update = UserSettingsSubstruct::new(
    Some("light".to_string()),
    None,  // no change to notifications
    0,     // version is unwrapped, so 0 means "set to 0"
);
assert_eq!(minimal_update.field_count(), 2);

// Check if specific fields are set
assert!(minimal_update.has_field("theme"));
assert!(!minimal_update.has_field("notifications"));
assert!(minimal_update.has_field("version"));

// Apply updates to a target struct
let mut settings = UserSettings::new("dark".to_string(), false, 1);
minimal_update.apply_to(&mut settings);
// settings.theme is now "light", settings.version is now 0
// settings.notifications remains false (unchanged)

// Check if updates would change a target
let other_settings = UserSettings::new("light".to_string(), false, 0);
assert!(!minimal_update.would_change(&other_settings)); // No changes needed

// Merge two updates
let theme_update = UserSettingsSubstruct::new(Some("dark".to_string()), None, 0);
let notification_update = UserSettingsSubstruct::new(None, Some(true), 0);
let combined = theme_update.merge(notification_update);
// combined has theme: Some("dark") and notifications: Some(true)

// Convert to partial representation for flexible handling
let partial = combined.into_partial();
assert_eq!(partial.get("theme"), Some(&"\"dark\"".to_string()));
assert_eq!(partial.get("notifications"), Some(&"true".to_string()));
assert_eq!(partial.get("version"), Some(&"0".to_string()));
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

**Advanced Nested Naming Example:**
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
struct Person {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
}

#[derive(SubstructBuilder)]
struct Company {
    #[substruct_field(nested)]
    ceo: Person,
    #[substruct_field(nested, nested_type = "AddressBuilder")]
    address: Address,
}

// Creates a deep hierarchy with custom naming
let company_update = CompanySubstruct::new(
    Some(person_update),
    Some(address_update),
);
```

### Unwrapped Fields

```rust
#[derive(SubstructBuilder)]
struct Config {
    #[substruct_field(primitive, option = false)]
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
- **Update Methods**: The substruct includes utility methods for applying updates and checking changes
- **Clean Separation**: Perfect for API endpoints, configuration updates, and other scenarios where you want to separate update data from the target structure
- **Validation**: The macro now requires at least one field to be tagged with `#[substruct_field]` - empty structs or structs with no tagged fields will cause compilation errors

## Requirements

- **Rust 1.85.0+** (Rust Edition 2024)
- `serde` for serialization support
- Fields must implement `Clone` and `PartialEq`

## Error Handling

The macro provides clear error messages for:
- Invalid attribute syntax
- Unsupported field types
- Misuse of `serde_json::Value` with `#[substruct_field(json)]`
- Invalid nested type specifications
- **No tagged fields**: Compilation error when no fields are tagged with `#[substruct_field]`

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
| `basic_functionality.rs` | 12 | ✅ All Passing | Core macro functionality, field exclusion, and utility methods |
| `field_types.rs` | 10 | ✅ All Passing | Primitive, JSON, and nested field handling |
| `configuration.rs` | 4 | ✅ All Passing | Attributes, wrapping, naming, and debug |
| `complex_scenarios.rs` | 5 | ✅ All Passing | Complex nested types and edge cases |
| `integration.rs` | 2 | ✅ All Passing | Multiple features working together |
| `error_handling.rs` | 7 | ✅ All Passing | Macro validation and error handling |
| `real_world.rs` | 9 | ✅ All Passing | API, database, and e-commerce patterns |
| `edge_cases.rs` | 9 | ✅ All Passing | Boundary conditions and edge cases |

**Total: 58 tests, all passing** ✅

### Detailed Test Breakdown

#### 1. `basic_functionality.rs` - Core Functionality Tests

Tests the fundamental behavior of the macro with a simple struct containing both marked and unmarked fields.

**Test Cases:**
- **`test_basic_struct_derivation`**: Validates substruct creation with only marked fields
- **`test_basic_struct_default`**: Tests default substruct creation
- **`test_basic_struct_from_source`**: Tests creating substruct from source struct
- **`test_basic_struct_is_empty`**: Validates empty state detection
- **`test_basic_struct_from_owned`**: Tests owned source conversion
- **`test_basic_struct_field_count`**: Tests field counting functionality
- **`test_basic_struct_clear`**: Tests field clearing functionality
- **`test_basic_struct_apply_to`**: Tests applying updates to target struct
- **`test_basic_struct_would_change`**: Tests change detection functionality
- **`test_basic_struct_merge`**: Tests merging two substructs
- **`test_basic_struct_has_field`**: Tests field presence checking
- **`test_basic_struct_into_partial`**: Tests conversion to flexible HashMap representation

**Key Validation:**
- Fields without `#[substruct_field]` are completely excluded
- Only `name` and `active` fields appear in the substruct
- `age` field is absent from all substruct operations
- All utility methods work correctly with the generated substruct
- Update operations only affect marked fields

#### 2. `field_types.rs` - Field Type Handling

Comprehensive tests for all field types: primitive, JSON, and nested.

**Test Cases:**
- **Primitive Fields**: Basic primitive field substruct creation and validation
- **JSON Fields**: JSON field serialization, mixed field types, and nested context
- **Nested Types**: Basic nested struct creation and source conversion
- **Nested Field Operations**: Recursive `apply_to()`, `would_change()`, and `into_partial()` functionality for nested structs

**Key Validation:**
- Primitive fields are wrapped in `Option<T>` by default
- JSON fields are properly typed as `Option<serde_json::Value>`
- Nested structs are generated with correct field types
- Mixed field types work together seamlessly
- Nested field operations work recursively for any depth of nesting
- `into_partial()` method provides flexible HashMap representation with comparable string values

#### 3. `configuration.rs` - Configuration and Attributes

Tests all configuration options: attributes, wrapping, naming, and debug.

**Test Cases:**
- **Debug and Wrapping**: Debug attribute with wrapped fields
- **Simple Nesting**: Basic nested struct scenarios
- **Custom Naming**: Struct-level naming functionality
- **Wrap Attributes**: Field wrapping configuration and parsing

**Key Validation:**
- Debug attributes work with wrapped fields
- Custom names are applied correctly
- Wrap attributes are parsed correctly
- Field wrapping behavior is configurable

#### 4. `complex_scenarios.rs` - Complex Type Scenarios

Tests complex nested types, custom types, and edge cases with advanced naming features.

**Test Cases:**
- **Complex Custom Types**: Deep nesting with company → person → address using custom nested type names
- **Custom Nested Naming**: `AddressBuilder` custom naming for nested structs
- **Edge Cases**: Empty structs, single field structs, and boundary conditions
- **Advanced Scenarios**: Partial updates, None values, and empty states

**Key Validation:**
- Complex field hierarchies work properly with custom naming
- Custom nested type names are applied correctly (`AddressBuilder` instead of `AddressSubstruct`)
- Edge cases are handled gracefully
- Empty structs don't cause compilation errors
- Advanced scenarios work as expected

#### 5. `integration.rs` - Feature Integration

Tests how multiple features work together in complex scenarios.

**Test Cases:**
- **Complex Integration**: Custom naming, mixed field types, nesting, and wrapping
- **Mixed Operations**: Default creation, partial updates, and source conversion

**Key Validation:**
- All features work together seamlessly
- Complex integrations handle multiple field types
- Mixed operations work correctly
- Source conversion handles all field types properly

#### 6. `error_handling.rs` - Macro Validation and Error Handling

Comprehensive tests for macro compilation, validation, and edge case handling.

**Test Cases:**
- **Field Type Validation**: Mixed field types, complex nesting, and attribute combinations
- **Trait Implementation**: Clone, Debug, PartialEq, Default, and From traits
- **Serialization**: JSON serialization and deserialization validation
- **Edge Cases**: Single fields and complex scenarios

**Key Validation:**
- Macro handles all field types correctly
- Generated structs implement required traits
- Serialization works with edge case values
- Complex scenarios work properly

#### 7. `real_world.rs` - Real-World Use Case Patterns

Tests common patterns used in production applications.

**Test Cases:**
- **API Update Patterns**: User profile updates and nested structure modifications
- **Database Patterns**: Record updates, soft deletes, and version management
- **Configuration Management**: App settings and partial configuration updates
- **E-commerce Patterns**: Product updates, inventory management, and category changes
- **Workflow Patterns**: Step status updates and process management

**Key Validation:**
- Common update patterns work correctly
- Partial updates handle field exclusion properly
- Nested updates maintain data integrity
- Real-world scenarios are handled gracefully

#### 8. `edge_cases.rs` - Boundary Conditions and Edge Cases

Tests extreme scenarios and boundary conditions to ensure robustness.

**Test Cases:**
- **Minimal Structs**: Single fields and complex field types
- **Field Type Edge Cases**: Arrays, tuples, chars, bytes, and complex types
- **Deep Nesting**: Multi-level nested structures (4+ levels deep)
- **Custom Naming**: Very long names and special characters
- **Serialization Edge Cases**: Empty strings, zero values, infinity, and NaN
- **Boundary Conditions**: Min/max values and extreme numeric ranges

**Key Validation:**
- Extreme scenarios are handled gracefully
- Deep nesting works correctly
- Custom naming handles edge cases
- Serialization works with boundary values
- All field types are supported properly

### Running Tests

#### Individual Test Files
```bash
cargo test --test basic_functionality
cargo test --test field_types
cargo test --test configuration
cargo test --test complex_scenarios
cargo test --test integration
cargo test --test error_handling
cargo test --test real_world
cargo test --test edge_cases
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
- **Coverage**: Tests cover all major macro features and edge case\

### Architecture Benefits
- **Easy Extension**: The modular design allows for simple addition of new features
- **Maintainable Code**: Clear separation of concerns makes the codebase easy to maintain
- **Testable Components**: Individual modules can be unit tested in isolation
- **Scalable Design**: The processor module can be extended with new processing logic

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

The test suite serves as both validation of current functionality and documentation of expected behavior, ensuring the macro remains reliable and well-tested as it evolves. With 58 comprehensive tests covering all aspects of the macro's functionality, the project maintains high quality and reliability standards.


