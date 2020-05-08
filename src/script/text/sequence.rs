
use crate::script::error::ScriptError;

use super::word::{expect_word, expect_word_kind, Kind, Word};
use super::treatment::Treatment;
use super::connection::Connection;

pub struct Sequence {
    pub name: String,
    pub treatments: Vec<Treatment>,
    pub connections: Vec<Connection>,
}

impl Sequence {
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name = expect_word_kind(Kind::Name, "Sequence name expected.", &mut iter)?;
        expect_word_kind(Kind::OpeningBrace, "Sequence content declaration expected '{'.", &mut iter)?;

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
                treatments.push(Treatment::build(element_name, &mut iter)?);
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
            treatments,
            connections,
        })
    }
}
