//!
//! A few utility functions on `char`
//!

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
