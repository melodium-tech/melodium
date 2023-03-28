use melodium_core::*;
use melodium_macro::mel_function;

/// Return the replacement character.
///
/// `U+FFFD REPLACEMENT CHARACTER` (�) is used in Unicode to represent a decoding error.
#[mel_function]
pub fn replacement_character() -> char {
    char::REPLACEMENT_CHARACTER
}

/// Return the highest code point `char` can contains.
///
/// The highest code point is `U+10FFFF`.
#[mel_function]
pub fn max() -> char {
    char::MAX
}
