use melodium_core::*;
use melodium_macro::mel_function;

/// Return the replacement character.
///
/// `U+FFFD REPLACEMENT CHARACTER` (�) is used in Unicode to represent a decoding error.
#[mel_function]
pub fn replacement_character() -> char {
    char::REPLACEMENT_CHARACTER
}
