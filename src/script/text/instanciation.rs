
//! Module dedicated to [Instanciation](struct.Instanciation.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::{expect_word, expect_word_kind, Kind, Word};
use super::common::{parse_configuration_assignations, parse_parameters_assignations};
use super::parameter::Parameter;

/// Structure describing a textual instanciation.
/// 
/// This match the conceptual syntax of calling a model, sequence, or treatment.
/// It owns a name, a type (treatment or model type, not [data type](../type/struct.Type.html)), and list of [parameters](../parameter/struct.Parameter.html).
#[derive(Clone, Debug)]
pub struct Instanciation {
    pub name: PositionnedString,
    pub r#type: PositionnedString,
    pub configuration: Vec<Parameter>,
    pub parameters: Vec<Parameter>,
}

impl Instanciation {
    /// Build an instanciation by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the name.
    /// 
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::instanciation::Instanciation;
    /// let text = r##"MakeSpectrum: Spectrum[Audio=AudioConnection](frameSize = 1024, hopSize = 512, windowingType = "blackmanharris92")"##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let instanciation = Instanciation::build(&mut iter)?;
    /// 
    /// assert_eq!(instanciation.name.string, "MakeSpectrum");
    /// assert_eq!(instanciation.r#type.string, "Spectrum");
    /// assert_eq!(instanciation.configuration.len(), 1);
    /// assert_eq!(instanciation.parameters.len(), 3);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;

        let determinant = expect_word("Unexpected end of script.", &mut iter)?;
        // If determinant is colon, we expect to have instanciation type.
        if determinant.kind == Some(Kind::Colon) {

            Self::build_from_type(name.clone(), &mut iter)
        }
        // Bracket, the configuration.
        else if determinant.kind == Some(Kind::OpeningBracket) {

            Self::build_from_configuration(name.clone(), name.clone(), &mut iter)
        }
        // Else we expect parameters.
        else if determinant.kind == Some(Kind::OpeningParenthesis) {
            
            Self::build_from_parameters(name.clone(), name.clone(), Vec::new(), &mut iter)
        }
        else {
            Err(ScriptError::word("Configuration or parameter declaration expected.".to_string(), determinant.text, determinant.position))
        }
    }

    /// Build an instanciation by parsing words, starting when its type is expected.
    /// 
    /// * `name`: The name already parsed for the `Instanciation` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be the type name.
    /// 
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::instanciation::Instanciation;
    /// let text = r##"MakeSpectrum: Spectrum(frameSize = 1024, hopSize = 512, windowingType = "blackmanharris92")"##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let instanciation_name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;
    /// expect_word_kind(Kind::Colon, "Colon ':' expected.", &mut iter)?;
    /// 
    /// let instanciation = Instanciation::build_from_type(instanciation_name, &mut iter)?;
    /// 
    /// assert_eq!(instanciation.name.string, "MakeSpectrum");
    /// assert_eq!(instanciation.r#type.string, "Spectrum");
    /// assert_eq!(instanciation.configuration.len(), 0);
    /// assert_eq!(instanciation.parameters.len(), 3);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build_from_type(name: PositionnedString, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let r#type = expect_word_kind(Kind::Name, "Type or instanciation name expected.", &mut iter)?;
            
        let determinant = expect_word("Unexpected end of script.", &mut iter)?;

        // If determinant is bracket, the configuration.
        if determinant.kind == Some(Kind::OpeningBracket) {

            Self::build_from_configuration(name, r#type, &mut iter)
        }
        // Else we expect parameters.
        else if determinant.kind == Some(Kind::OpeningParenthesis) {
            
            Self::build_from_parameters(name, r#type, Vec::new(), &mut iter)
        }
        else {
            Err(ScriptError::word("Configuration or parameter declaration expected.".to_string(), determinant.text, determinant.position))
        }
    }

    /// Build an instanciation by parsing words, starting when configuration [Parameter](../parameter/struct.Parameter.html) is expected.
    /// 
    /// * `name`: The name already parsed for the `Instanciation` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Parameter](../type/struct.Parameter.html).
    /// 
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::instanciation::Instanciation;
    /// let text = r##"MakeSpectrum: Spectrum[Audio=AudioConnection](frameSize = 1024, hopSize = 512, windowingType = "blackmanharris92")"##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let instanciation_name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;
    /// expect_word_kind(Kind::Colon, "Colon ':' expected.", &mut iter)?;
    /// 
    /// let instanciation_type = expect_word_kind(Kind::Name, "Type expected.", &mut iter)?;
    /// expect_word_kind(Kind::OpeningBracket, "Configuration expected '['.", &mut iter)?;
    /// 
    /// let instanciation = Instanciation::build_from_configuration(instanciation_name, instanciation_type, &mut iter)?;
    /// 
    /// assert_eq!(instanciation.name.string, "MakeSpectrum");
    /// assert_eq!(instanciation.r#type.string, "Spectrum");
    /// assert_eq!(instanciation.configuration.len(), 1);
    /// assert_eq!(instanciation.parameters.len(), 3);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build_from_configuration(name: PositionnedString, r#type: PositionnedString, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let configuration = parse_configuration_assignations(&mut iter)?;

        // We expect parameters in any cases.
        expect_word_kind(Kind::OpeningParenthesis, "Parameters expected '('.", &mut iter)?;
        Self::build_from_parameters(name, r#type, configuration, &mut iter)
    }

    /// Build an instanciation by parsing words, starting when [Parameter](../parameter/struct.Parameter.html) is expected.
    /// 
    /// * `name`: The name already parsed for the `Instanciation` (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be about [Parameter](../type/struct.Parameter.html).
    /// 
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::instanciation::Instanciation;
    /// let text = r##"MakeSpectrum: Spectrum(frameSize = 1024, hopSize = 512, windowingType = "blackmanharris92")"##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let instanciation_name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;
    /// expect_word_kind(Kind::Colon, "Colon ':' expected.", &mut iter)?;
    /// 
    /// let instanciation_type = expect_word_kind(Kind::Name, "Type expected.", &mut iter)?;
    /// expect_word_kind(Kind::OpeningParenthesis, "Parameters expected '('.", &mut iter)?;
    /// 
    /// let instanciation = Instanciation::build_from_parameters(instanciation_name, instanciation_type, Vec::new(), &mut iter)?;
    /// 
    /// assert_eq!(instanciation.name.string, "MakeSpectrum");
    /// assert_eq!(instanciation.r#type.string, "Spectrum");
    /// assert_eq!(instanciation.configuration.len(), 0);
    /// assert_eq!(instanciation.parameters.len(), 3);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build_from_parameters(name: PositionnedString, r#type: PositionnedString, configuration: Vec<Parameter>, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {
        
        let parameters = parse_parameters_assignations(&mut iter)?;

        Ok(Self {
            name,
            r#type,
            configuration,
            parameters,
        })
    }
}
