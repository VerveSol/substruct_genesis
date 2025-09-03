# Update Macro

A Rust procedural macro that automatically generates substruct builders for your data structures, making it easy to implement partial updates with clear semantics.

## Overview

The `SubstructBuilder` macro generates a substruct for your original struct, along with methods to apply updates, detect changes, and handle different field types appropriately. This is particularly useful for building update operations where you want to specify only the fields that should change.

## Features

- **Primitive Fields**: Automatically wrapped in `Option<T>` for nullable updates (configurable with `wrap` attribute)
- **Option Fields**: Use `Option<Option<T>>` for clear update semantics
- **JSON Fields**: Handle complex types as `Option<serde_json::Value>`
- **Nested Types**: Support nested substruct builders with custom naming
- **Custom Struct Names**: Configure the generated substruct name at the struct level
- **Change Detection**: Built-in methods to detect if updates would change the source
- **Flexible Wrapping**: Control whether fields are wrapped in `Option<T>` or not

## Usage

### Basic Example

```rust
use serde::{Deserialize, Serialize};
use update_macro::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct User {
    #[substruct_field(primitive)]
    name: String,
    #[substruct_field(primitive)]
    age: u32,
    #[substruct_field(primitive)]
    active: bool,
}

// The macro generates UserSubstruct with:
// - name: Option<String>
// - age: Option<u32>  
// - active: Option<bool>
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

#### `apply_to(&self, source: &T) -> T`
Applies the updates to the source struct, returning a new instance.

#### `would_change(&self, source: &T) -> bool`
Returns `true` if applying this update would change the source struct.

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
use update_macro::SubstructBuilder;

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
    id: String,  // always copied
}

// Create an update
let update = UserProfileSubstruct::new(
    Some("John Doe".to_string()),     // change name
    None,                             // don't change age
    Some(None),                       // set email to None
    Some(serde_json::to_value(&new_prefs).unwrap()), // change preferences
);

// Apply the update
let updated_profile = update.apply_to(&original_profile);

// Check if it would change anything
if update.would_change(&original_profile) {
    println!("Profile would be updated");
}
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
- Efficient change detection
- Minimal memory allocation during updates
- Smart trait derivation (avoids `Eq` for `f64`, `PartialEq` for `String`)


