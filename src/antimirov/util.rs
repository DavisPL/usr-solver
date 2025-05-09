//!
//! A few utility functions on `char`
//!

use crate::types::expr::CharVar;

// Minimum and maximum char values
pub const CHAR_MIN: char = char::MIN;
pub const CHAR_MAX: char = char::MAX;

// PRECONDITION: assumes c+1 is a valid Unicode scalar value, if it exists
// We don't currently handle the general case soundly.
// Returns None if arithmetic is out of bounds or u32-to-char conversion fails.
pub fn char_plus_one(c: char) -> Option<char> {
    let c_val = c as u32;
    c_val.checked_add(1)?.try_into().ok()
}

// PRECONDITION: assumes c-1 is a valid Unicode scalar value, if it exists
// We don't currently handle the unicode case soundly.
// Returns None if arithmetic is out of bounds.
pub fn char_minus_one(c: char) -> Option<char> {
    let c_val: u32 = c as u32;
    c_val.checked_sub(1)?.try_into().ok()
}

use lazy_static::lazy_static;

use std::sync::Mutex;

lazy_static! {
    static ref FRESH_COUNTER: Mutex<usize> = Mutex::new(0);
}

// TODO: Create a global fresh var generator - using a counter with lazy_static
pub fn get_fresh_var() -> CharVar {
    // Lazy static to increment a counter, then use that to create a new var

    let mut counter_handle = FRESH_COUNTER.lock().unwrap();
    *counter_handle += 1;
    let val = *counter_handle;

    let name = format!("__fresh_{}", val);
    CharVar { name }
}

#[cfg(test)]
#[test]
fn test_get_fresh_var() {
    let var1 = get_fresh_var();
    let var2 = get_fresh_var();
    assert_eq!(var1.name, "__fresh_1");
    assert_eq!(var2.name, "__fresh_2");
}
