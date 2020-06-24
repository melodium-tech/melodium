
//! Module dedicated to [Sequence](struct.Sequence.html) parsing.

use crate::script::error::ScriptError;

use super::word::{expect_word, expect_word_kind, Kind, Word};
use super::parameter::Parameter;
use super::requirement::Requirement;
use super::treatment::Treatment;
use super::connection::Connection;

/// Structure describing a textual sequence.
/// 
/// It owns the name, and the attributes of the sequence, as well as its internal treatments and connections. There is no logical dependency between them at this point.
#[derive(Clone)]
pub struct Sequence {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub requirements: Vec<Requirement>,
    pub origin: Option<Treatment>,
    pub inputs: Vec<Parameter>,
    pub outputs: Vec<Parameter>,
    pub treatments: Vec<Treatment>,
    pub connections: Vec<Connection>,
}

impl Sequence {
    /// Build a sequence by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the name of the sequence.
    /// 
    /// ```
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::sequence::Sequence;
    /// 
    /// let text = r##"
    /// sequence PrepareAudioFiles(path: Vec<String>, sampleRate: Int = 44100, frameSize: Int = 4096, hopSize: Int = 2048, windowingType: String)
	///     origin AudioFiles(path=path, sampleRate=sampleRate)
	///     output spectrum: Mat<Int>
    ///     require @File
    ///     require @Signal
    /// {
	/// MakeSpectrum(frameSize = frameSize, hopSize = hopSize, windowingType = windowingType)
	/// 
	/// AudioFiles.signal -> MakeSpectrum.signal,spectrum -> Self.spectrum
    /// }
    /// "##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let sequence_keyword = expect_word_kind(Kind::Name, "Keyword expected.", &mut iter)?;
    /// assert_eq!(sequence_keyword, "sequence");
    /// 
    /// let sequence = Sequence::build(&mut iter)?;
    /// 
    /// assert_eq!(sequence.parameters.len(), 5);
    /// assert_eq!(sequence.requirements.len(), 2);
    /// assert!(sequence.origin.is_some());
    /// assert_eq!(sequence.inputs.len(), 0);
    /// assert_eq!(sequence.outputs.len(), 1);
    /// assert_eq!(sequence.treatments.len(), 1);
    /// assert_eq!(sequence.connections.len(), 2);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name = expect_word_kind(Kind::Name, "Sequence name expected.", &mut iter)?;

        /*
            First of all, we parse sequence's parameters.
        */
        expect_word_kind(Kind::OpeningParenthesis, "Sequence parameters declaration expected '('.", &mut iter)?;

        let mut parameters = Vec::new();

        let mut first_param = true;
        loop {

            let word = expect_word("Unexpected end of script.", &mut iter)?;

            if first_param && word.kind == Some(Kind::ClosingParenthesis) {
                break;
            }
            else if word.kind == Some(Kind::Name) {
                first_param = false;

                expect_word_kind(Kind::Colon, "Parameter type declaration expected.", &mut iter)?;
                parameters.push(Parameter::build_from_type(word.text, &mut iter)?);

                let delimiter = expect_word("Unexpected end of script.", &mut iter)?;
                
                if delimiter.kind == Some(Kind::Comma) {
                    continue;
                }
                else if delimiter.kind == Some(Kind::ClosingParenthesis) {
                    break;
                }
                else {
                    return Err(ScriptError::new("Comma or closing parenthesis expected.".to_string(), delimiter.text, delimiter.line, delimiter.line_position, delimiter.absolute_position));
                }
            }
            else {
                return Err(ScriptError::new("Parameter declaration expected.".to_string(), word.text, word.line, word.line_position, word.absolute_position));
            }
        }

        let mut origin = None;
        let mut requirements = Vec::new();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        /*
            We examine the presence (or abscence) of origin, inputs, outputs, and requirements declarations.
        */
        loop {
            let word = expect_word("Unexpected end of script.", &mut iter)?;

            if word.kind == Some(Kind::OpeningBrace) {
                break;
            }
            else if word.kind == Some(Kind::Name) {
                if word.text == "input" {

                    let input_name = expect_word_kind(Kind::Name, "Input name expected.", &mut iter)?;
                    expect_word_kind(Kind::Colon, "Input type declaration expected.", &mut iter)?;
                    inputs.push(Parameter::build_from_type(input_name, &mut iter)?);
                }
                else if word.text == "output" {

                    let output_name = expect_word_kind(Kind::Name, "Output name expected.", &mut iter)?;
                    expect_word_kind(Kind::Colon, "Output type declaration expected.", &mut iter)?;
                    outputs.push(Parameter::build_from_type(output_name, &mut iter)?);
                }
                else if word.text == "require" {

                    requirements.push(Requirement::build(&mut iter)?);
                }
                else if word.text == "origin" {

                    if origin.is_none() {
                        let origin_name = expect_word_kind(Kind::Name, "Origin name expected.", &mut iter)?;
                        expect_word_kind(Kind::OpeningParenthesis, "Origin parameters declaration '(' expected.", &mut iter)?;
                        origin = Some(Treatment::build_from_parameters(origin_name, &mut iter)?);
                    }
                    else {
                        return Err(ScriptError::new("Origin already declared.".to_string(), word.text, word.line, word.line_position, word.absolute_position));
                    }
                }
            }
            else {
                return Err(ScriptError::new("Sequence attributes or content declaration expected.".to_string(), word.text, word.line, word.line_position, word.absolute_position));
            }
        }

        let mut treatments = Vec::new();
        let mut connections = Vec::new();

        /*
            We prepare variables able to tell if the last built element were a connection.
            Because connections are chainable, and that possibility of chain depends on
            the type of last connection, we have to keep track on (1) was the last
            connection including data transmission, or (2) was it only a chain of treatments.
        */
        let mut last_connection_name_end_point: Option<String> = None;
        let mut may_be_connection_data_out = false; //  (1)
        let mut may_be_connection_end_point = false; // (2)
        
        loop {
            // Those are convenience variables, in case we're not continuing a connection chain,
            // reused in "else" block later.
            let element_name;
            let determinant;

            // We DO want a word there, a sequence can only be terminated using '}'.
            let word = expect_word("Unexpected end of script.", &mut iter)?;

            // In case a continuation of connection with data transmission (1) is possible,
            // we check if word is a comma.
            if may_be_connection_data_out &&
                word.kind == Some(Kind::Comma) {
                    // So it means we expect continuing a connection with data tramsission (1).
                    let connection = Connection::build_from_name_data_out(last_connection_name_end_point.unwrap(), &mut iter)?;
                    last_connection_name_end_point = Some(connection.name_end_point.to_string());
                    connections.push(connection);
                    
                    // Redundant assignation, as will stay as true
                    // may_be_connection_data_out = true;

                    // And nothing is to do later in that iteration.
                    continue;
            }

            // In case a continuation of connection that only chain treatments (2) is possible,
            // we check if word is a right arrow '-->'.
            else if may_be_connection_end_point &&
                word.kind == Some(Kind::RightArrow) {
                    // So it means we expect continuing a connection that only chains treatments (2).
                    let connection = Connection::build_from_name_end_point(last_connection_name_end_point.unwrap(), &mut iter)?;
                    last_connection_name_end_point = Some(connection.name_end_point.to_string());
                    connections.push(connection);

                    // Redundant assignation, as will stay as true
                    // may_be_connection_end_point = true;

                    // And nothing is to do later in that iteration.
                    continue;
            }

            else {
                // We're not continuing a connection, so resetting those ones.
                last_connection_name_end_point = None;
                may_be_connection_data_out = false;
                may_be_connection_end_point = false;

                // If we're not continuing a connection, word have to be the name of an element.
                if word.kind == Some(Kind::Name) {
                    
                    element_name = word.text;

                    // And the next word is determinant of what can follow.
                    determinant = expect_word("Unexpected end of script.", &mut iter)?;
                }
                // Or a closing brace, ending the sequence.
                else if word.kind == Some(Kind::ClosingBrace) {
                    break;
                }
                else {
                    return Err(ScriptError::new("Element name expected.".to_string(), word.text, word.line, word.line_position, word.absolute_position));
                }
            }

            // If determinant is an opening parenthesis '(', we are in a treatment declaration.
            if determinant.kind == Some(Kind::OpeningParenthesis) {
                treatments.push(Treatment::build_from_parameters(element_name, &mut iter)?);
            }
            // If determinant is a dot '.', we are in a connection declaration, with data transmission (1).
            else if determinant.kind == Some(Kind::Dot) {
                let connection = Connection::build_from_name_data_out(element_name, &mut iter)?;
                last_connection_name_end_point = Some(connection.name_end_point.to_string());
                connections.push(connection);
                // We remind that next iteration may be a continuation of connections.
                may_be_connection_data_out = true;
            }
            // If determinant is an arrow '-->', we are in a connection declaration, without data transmission (2).
            else if determinant.kind == Some(Kind::RightArrow) {
                let connection = Connection::build_from_name_end_point(element_name, &mut iter)?;
                last_connection_name_end_point = Some(connection.name_end_point.to_string());
                connections.push(connection);
                // We remind that next iteration may be a continuation of connections.
                may_be_connection_end_point = true;
            }
            // In other cases, we're not getting what's expected.
            else {
                return Err(ScriptError::new("Symbol expected.".to_string(), determinant.text, determinant.line, determinant.line_position, determinant.absolute_position));
            }
        }

        Ok(Self {
            name,
            parameters,
            requirements,
            origin,
            inputs,
            outputs,
            treatments,
            connections,
        })
    }
}
