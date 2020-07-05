

//! Module dedicated to [Parameter](struct.Parameter.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::*;
use super::r#type::Type;
use super::value::Value;

/// Structure describing a textual parameter.
/// 
/// It owns a name, and optionnal [Type](../type/struct.Type.html) and/or [Value](../value/enum.Value.html). There is no logical dependency between them at this point.
#[derive(Clone)]
pub struct Parameter {
    pub name: PositionnedString,
    pub r#type: Option<Type>,
    pub value: Option<Value>,
}

impl Parameter {
    /// Build a parameter by parsing words, starting when named [Type](../type/struct.Type.html) is expected.
    /// 
    /// * `name`: The name already parsed for the `Parameter` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Type](../type/struct.Type.html).
    ///
    /// ```
    /// # use melodium_rust::script::text::parameter::*;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::value::Value;
    /// let words = get_words("myParameter: Vec<Int> = [1, 3, 5, 7, 11]").unwrap();
    /// let mut iter = words.iter();
    /// 
    /// // Taking 'myParameter' in name.
    /// let name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;
    /// // Checking and discarding ':'.
    /// expect_word_kind(Kind::Colon, "Colon expected.", &mut iter)?;
    /// 
    /// let parameter = Parameter::build_from_type(name, &mut iter)?;
    /// 
    /// assert!(parameter.r#type.is_some());
    /// assert_eq!(parameter.r#type.unwrap().name.string, "Int");
    /// assert!(parameter.value.is_some());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build_from_type(name: PositionnedString, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let r#type = Type::build(&mut iter)?;

        // We _clone_ the iterator (in case next word doesn't rely on Parameter) and doesn't make our expectation to fail if not satisfied.
        let possible_equal = expect_word_kind(Kind::Equal, "", &mut iter.clone());
        if possible_equal.is_ok() {
            // We discard the equal sign.
            iter.next();

            let value = Value::build_from_first_item(&mut iter)?;

            Ok(Self {
                name,
                r#type: Some(r#type),
                value: Some(value),
            })
        }
        else {
            Ok(Self {
                name,
                r#type: Some(r#type),
                value: None,
            })
        }
    }

    /// Build a parameter by parsing words, starting when a [Value](../value/enum.Value.html) is expected.
    ///
    /// * `name`: The name already parsed for the `Parameter` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Value](../value/enum.Value.html).
    /// 
    /// ```
    /// # use melodium_rust::script::text::parameter::*;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::value::Value;
    /// let words = get_words("myParameter = 0.248").unwrap();
    /// let mut iter = words.iter();
    /// 
    /// // Taking 'myParameter' in name.
    /// let name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;
    /// // Checking and discarding '='.
    /// expect_word_kind(Kind::Equal, "Equal expected.", &mut iter)?;
    /// 
    /// let parameter = Parameter::build_from_value(name, &mut iter)?;
    /// 
    /// assert!(parameter.value.is_some());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build_from_value(name: PositionnedString, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let value = Value::build_from_first_item(&mut iter)?;

        Ok(Self {
            name,
            r#type: None,
            value: Some(value),
        })
    }
}
