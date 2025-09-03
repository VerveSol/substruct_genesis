use serde::{Deserialize, Serialize};
use substruct_genesis::SubstructBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SubstructBuilder)]
struct DebugStruct {
    #[substruct_field(primitive, wrap = false)]
    unwrapped: u32,
}

#[test]
fn test_debug_wrap() {
    // This should compile if the macro generates the correct types
    let _update = DebugStructSubstruct::new(42);
    println!("DebugStructSubstruct compiled successfully!");
}
