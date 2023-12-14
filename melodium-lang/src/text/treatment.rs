//! Module dedicated to [Treatment](Treatment) parsing.

use core::slice::Windows;
use std::collections::HashMap;

use super::common::{
    parse_configuration_declarations, parse_generics, parse_parameters_declarations,
};
use super::connection::Connection;
use super::instanciation::Instanciation;
use super::parameter::Parameter;
use super::requirement::Requirement;
use super::word::{Kind, Word};
use super::{CommentsAnnotations, Generic, PositionnedString};
use crate::ScriptError;

/// Structure describing a textual treatment.
///
/// It owns the name, and the attributes of the treatment, as well as its internal treatments instanciations and connections. There is no logical dependency between them at this point.
#[derive(Clone, Debug)]
pub struct Treatment {
    pub annotations: Option<CommentsAnnotations>,
    pub name: PositionnedString,
    pub generics: Vec<Generic>,
    pub configuration: Vec<Parameter>,
    pub parameters: Vec<Parameter>,
    pub models: Vec<Instanciation>,
    pub requirements: Vec<Requirement>,
    pub inputs: Vec<Parameter>,
    pub outputs: Vec<Parameter>,
    pub treatments: Vec<Instanciation>,
    pub connections: Vec<Connection>,
}

impl Treatment {
    /// Build a treatment by parsing words.
    ///
    /// * `iter`: Iterator over words list, next() being expected to be the name of the treatment.
    ///
    pub fn build(
        mut iter: &mut Windows<Word>,
        mut self_annotations: Option<CommentsAnnotations>,
        global_annotations: &mut HashMap<Word, CommentsAnnotations>,
    ) -> Result<Self, ScriptError> {
        let word_name = iter
            .next()
            .map(|s| &s[0])
            .ok_or_else(|| ScriptError::end_of_script(31))
            .and_then(|w| {
                if w.kind != Some(Kind::Name) {
                    Err(ScriptError::word(32, w.clone(), &[Kind::Name]))
                } else {
                    Ok(w.clone())
                }
            })?;
        let name: PositionnedString = (&word_name).into();

        let mut generics_parsed = false;
        let mut generics = Vec::new();
        let mut configuration_parsed = false;
        let mut configuration = Vec::new();
        let parameters;
        loop {
            match iter.next().map(|s| &s[0]) {
                Some(w) if w.kind == Some(Kind::OpeningChevron) && !generics_parsed => {
                    generics_parsed = true;
                    generics = parse_generics(&mut iter, global_annotations)?;
                }
                Some(w) if w.kind == Some(Kind::OpeningBracket) && !configuration_parsed => {
                    generics_parsed = true;
                    configuration_parsed = true;
                    configuration =
                        parse_configuration_declarations(&mut iter, global_annotations)?;
                }
                Some(w) if w.kind == Some(Kind::OpeningParenthesis) => {
                    parameters = parse_parameters_declarations(&mut iter, global_annotations)?;
                    break;
                }
                Some(w) => {
                    return Err(ScriptError::word(
                        33,
                        w.clone(),
                        &[
                            Kind::OpeningChevron,
                            Kind::OpeningBracket,
                            Kind::OpeningParenthesis,
                        ],
                    ))
                }
                None => return Err(ScriptError::end_of_script(34)),
            }
        }

        let mut models = Vec::new();
        let mut requirements = Vec::new();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        /*
            We examine the presence (or abscence) of origin, inputs, outputs, and requirements declarations.
        */
        loop {
            match iter.next().map(|s| &s[0]) {
                Some(w) if w.kind == Some(Kind::OpeningBrace) => break,
                Some(w) if w.kind == Some(Kind::Name) => match w.text.as_str() {
                    "input" => {
                        let input_name = iter
                            .next()
                            .map(|s| &s[0])
                            .ok_or_else(|| ScriptError::end_of_script(39))
                            .and_then(|w| {
                                if w.kind != Some(Kind::Name) {
                                    Err(ScriptError::word(40, w.clone(), &[Kind::Name]))
                                } else {
                                    Ok(w.into())
                                }
                            })?;

                        iter.next()
                            .map(|s| &s[0])
                            .ok_or_else(|| ScriptError::end_of_script(41))
                            .and_then(|w| {
                                if w.kind != Some(Kind::Colon) {
                                    Err(ScriptError::word(42, w.clone(), &[Kind::Colon]))
                                } else {
                                    Ok(())
                                }
                            })?;

                        inputs.push(Parameter::build_from_type(
                            global_annotations.remove(w),
                            None,
                            input_name,
                            &mut iter,
                            global_annotations,
                        )?);
                    }
                    "output" => {
                        let output_name = iter
                            .next()
                            .map(|s| &s[0])
                            .ok_or_else(|| ScriptError::end_of_script(43))
                            .and_then(|w| {
                                if w.kind != Some(Kind::Name) {
                                    Err(ScriptError::word(44, w.clone(), &[Kind::Name]))
                                } else {
                                    Ok(w.into())
                                }
                            })?;

                        iter.next()
                            .map(|s| &s[0])
                            .ok_or_else(|| ScriptError::end_of_script(45))
                            .and_then(|w| {
                                if w.kind != Some(Kind::Colon) {
                                    Err(ScriptError::word(46, w.clone(), &[Kind::Colon]))
                                } else {
                                    Ok(())
                                }
                            })?;

                        outputs.push(Parameter::build_from_type(
                            global_annotations.remove(w),
                            None,
                            output_name,
                            &mut iter,
                            global_annotations,
                        )?);
                    }
                    "model" => models.push(Instanciation::build(
                        global_annotations.remove(&w),
                        &mut iter,
                        global_annotations,
                    )?),
                    "require" => requirements.push(Requirement::build(&mut iter)?),
                    _ => {
                        return Err(ScriptError::description_element_expected(
                            47,
                            w.clone(),
                            word_name.clone(),
                            &["input", "output", "model", "require"],
                        ))
                    }
                },
                Some(w) => {
                    return Err(ScriptError::word(
                        37,
                        w.clone(),
                        &[Kind::OpeningBrace, Kind::Name],
                    ))
                }
                None => return Err(ScriptError::end_of_script(38)),
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
        let mut last_connection_name_end_point: Option<PositionnedString> = None;
        let mut may_be_connection_data_out = false; //  (1)
        let mut may_be_connection_end_point = false; // (2)

        loop {
            // Those are convenience variables, in case we're not continuing a connection chain,
            // reused in "else" block later.
            let element_name: PositionnedString;
            let mut element_annotations = None;
            let determinant;

            match iter.next().map(|s| &s[0]) {
                // In case a continuation of connection with data transmission (1) is possible,
                // we check if word is a comma.
                Some(w) if w.kind == Some(Kind::Comma) && may_be_connection_data_out => {
                    let connection = Connection::build_from_name_data_out(
                        element_annotations,
                        last_connection_name_end_point.unwrap(),
                        &mut iter,
                    )?;
                    last_connection_name_end_point = Some(connection.name_end_point.clone());
                    connections.push(connection);

                    // Redundant assignation, as will stay as true
                    // may_be_connection_data_out = true;

                    // And nothing is to do later in that iteration.
                    continue;
                }
                // In case a continuation of connection that only chain treatments (2) is possible,
                // we check if word is a right arrow '-->'.
                Some(w) if w.kind == Some(Kind::RightArrow) && may_be_connection_end_point => {
                    // So it means we expect continuing a connection that only chains treatments (2).
                    let connection = Connection::build_from_name_end_point(
                        element_annotations,
                        last_connection_name_end_point.unwrap(),
                        &mut iter,
                    )?;
                    last_connection_name_end_point = Some(connection.name_end_point.clone());
                    connections.push(connection);

                    // Redundant assignation, as will stay as true
                    // may_be_connection_end_point = true;

                    // And nothing is to do later in that iteration.
                    continue;
                }
                Some(w) => {
                    // We're not continuing a connection, so resetting those ones.
                    last_connection_name_end_point = None;
                    may_be_connection_data_out = false;
                    may_be_connection_end_point = false;

                    // If we're not continuing a connection, word have to be the name of an element.
                    if w.kind == Some(Kind::Name) {
                        element_name = w.into();
                        element_annotations = global_annotations.remove(w);

                        // And the next word is determinant of what can follow.
                        determinant = iter.next();
                    }
                    // Or a closing brace, ending the treatment.
                    else if w.kind == Some(Kind::ClosingBrace) {
                        break;
                    } else {
                        return Err(ScriptError::word(
                            48,
                            w.clone(),
                            &[Kind::Name, Kind::ClosingBrace],
                        ));
                    }
                }
                None => return Err(ScriptError::end_of_script(49)),
            }

            match determinant.map(|s| &s[0]) {
                // If determinant is ':', '<', '[', or '(', we are in a treatment declaration.
                Some(w) if w.kind == Some(Kind::Colon) => {
                    treatments.push(Instanciation::build_from_type(
                        element_annotations,
                        element_name.clone(),
                        &mut iter,
                        global_annotations,
                    )?)
                }
                Some(w) if w.kind == Some(Kind::OpeningChevron) => {
                    treatments.push(Instanciation::build_from_generics(
                        element_annotations,
                        element_name.clone(),
                        element_name.clone(),
                        &mut iter,
                        global_annotations,
                    )?)
                }
                Some(w) if w.kind == Some(Kind::OpeningBracket) => {
                    treatments.push(Instanciation::build_from_configuration(
                        element_annotations,
                        element_name.clone(),
                        element_name.clone(),
                        Vec::new(),
                        &mut iter,
                        global_annotations,
                    )?)
                }
                Some(w) if w.kind == Some(Kind::OpeningParenthesis) => {
                    treatments.push(Instanciation::build_from_parameters(
                        element_annotations,
                        element_name.clone(),
                        element_name.clone(),
                        Vec::new(),
                        Vec::new(),
                        &mut iter,
                        global_annotations,
                    )?)
                }
                // If determinant is a dot '.', we are in a connection declaration, with data transmission (1).
                Some(w) if w.kind == Some(Kind::Dot) => {
                    let connection = Connection::build_from_name_data_out(
                        element_annotations,
                        element_name,
                        &mut iter,
                    )?;
                    last_connection_name_end_point = Some(connection.name_end_point.clone());
                    connections.push(connection);
                    // We remind that next iteration may be a continuation of connections.
                    may_be_connection_data_out = true;
                }
                // If determinant is an arrow '-->', we are in a connection declaration, without data transmission (2).
                Some(w) if w.kind == Some(Kind::RightArrow) => {
                    let connection = Connection::build_from_name_end_point(
                        element_annotations,
                        element_name,
                        &mut iter,
                    )?;
                    last_connection_name_end_point = Some(connection.name_end_point.clone());
                    connections.push(connection);
                    // We remind that next iteration may be a continuation of connections.
                    may_be_connection_end_point = true;
                }
                Some(w) => {
                    return Err(ScriptError::word(
                        50,
                        w.clone(),
                        &[
                            Kind::Colon,
                            Kind::OpeningChevron,
                            Kind::OpeningBracket,
                            Kind::OpeningParenthesis,
                            Kind::Dot,
                            Kind::RightArrow,
                        ],
                    ))
                }
                None => return Err(ScriptError::end_of_script(51)),
            }
        }

        if let Some(doc) = self_annotations.as_mut().and_then(|sa| sa.doc.as_mut()) {
            doc.remove_indent();
        }

        Ok(Self {
            annotations: self_annotations,
            name,
            generics,
            configuration,
            parameters,
            models,
            requirements,
            inputs,
            outputs,
            treatments,
            connections,
        })
    }
}
