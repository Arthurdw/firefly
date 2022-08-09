use std::string::FromUtf8Error;

use anyhow::{anyhow, Result};
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

/// All possible query types
#[derive(Debug, Clone, Copy, EnumCountMacro, PartialEq, Eq)]
pub enum QueryType {
    New,
    Get,
    GetValue,
    GetTTL,
    Drop,
    DropAll,
    QueryTypeString,
    QueryTypeBitwise,
}

impl QueryType {
    /// Simply get the associated query type from a ascii character.
    /// Returns `None` if the query type is not found.
    ///
    /// Arguments
    ///
    /// * `byte` - The byte which should be able to represent the query type.
    pub fn from_byte(byte: u8) -> Option<QueryType> {
        match byte as char {
            '0' => Some(QueryType::New),
            '1' => Some(QueryType::Get),
            '2' => Some(QueryType::GetValue),
            '3' => Some(QueryType::GetTTL),
            '4' => Some(QueryType::Drop),
            '5' => Some(QueryType::DropAll),
            '6' => Some(QueryType::QueryTypeString),
            '7' => Some(QueryType::QueryTypeBitwise),
            _ => None,
        }
    }

    /// Get the query identifier from a query type.
    pub fn as_byte(&self) -> u8 {
        match &self {
            QueryType::New => '0' as u8,
            QueryType::Get => '1' as u8,
            QueryType::GetValue => '2' as u8,
            QueryType::GetTTL => '3' as u8,
            QueryType::Drop => '4' as u8,
            QueryType::DropAll => '5' as u8,
            QueryType::QueryTypeString => '6' as u8,
            QueryType::QueryTypeBitwise => '7' as u8,
        }
    }
}

/// Each query type its defining syntax
const BINARY_VALUES: [(&[u8], QueryType); QueryType::COUNT] = [
    ("NEW".as_bytes(), QueryType::New),
    ("GET".as_bytes(), QueryType::Get),
    ("GETVALUE".as_bytes(), QueryType::GetValue),
    ("GETTTL".as_bytes(), QueryType::GetTTL),
    ("DROP".as_bytes(), QueryType::Drop),
    ("DROPALL".as_bytes(), QueryType::DropAll),
    ("QUERYTYPESTRING".as_bytes(), QueryType::QueryTypeString),
    ("QUERYTYPEBITWISE".as_bytes(), QueryType::QueryTypeBitwise),
];

/// Deduct the query type.
///
/// Arguments
///
/// * `query` - The query which should be identifiable.
pub(crate) fn get_query_type(query: String) -> Option<QueryType> {
    let mut matchable: Vec<(&[u8], QueryType)> = BINARY_VALUES.into();
    let mut last_match: Option<QueryType> = None;

    let mut byte_index = 0;

    for byte in query.bytes() {
        if byte == '\n' as u8 || byte == ' ' as u8 {
            continue;
        }

        matchable = matchable
            .into_iter()
            .filter(|(bytes, query_type)| {
                if bytes.len() - 1 == byte_index {
                    last_match = Some(*query_type);
                } else if bytes.len() - 1 < byte_index {
                    return false;
                }

                bytes[byte_index] == byte.to_ascii_uppercase()
            })
            .collect();

        byte_index += 1;

        if matchable.len() == 1 {
            return Some(matchable[0].1);
        } else if matchable.len() == 0 {
            if let Some(_) = last_match {
                return last_match;
            }
            break;
        }
    }

    None
}

const SINGLE_QUOTE: u8 = '\'' as u8;
const DOUBLE_QUOTE: u8 = '"' as u8;
const END_QUERY: u8 = ';' as u8;

/// Retrieve all query arguments.
///
/// Arguments
///
/// * `query` - The string which contains the arguments.
pub(crate) fn get_arguments(query: String) -> Result<Vec<String>, FromUtf8Error> {
    let mut arguments = Vec::new();
    let mut current_argument = Vec::new();
    let mut is_within_value = false;
    let mut end_byte = SINGLE_QUOTE;

    for byte in query.bytes() {
        let is_delimiter = byte == SINGLE_QUOTE || byte == DOUBLE_QUOTE;

        if is_delimiter {
            if current_argument.len() == 0 {
                is_within_value = true;
                end_byte = byte;
                continue;
            } else if byte == end_byte {
                arguments.push(String::from_utf8(current_argument)?);
                current_argument = Vec::new();
                is_within_value = false;
            }
        }

        if is_within_value {
            current_argument.push(byte);
        } else if byte == END_QUERY {
            break;
        }
    }

    return Ok(arguments);
}

// TODO: Write some documentation
pub(crate) fn parse_query_arguments(
    arguments_fetcher: impl Fn(String) -> Result<Vec<String>, FromUtf8Error>,
    query_type: QueryType,
    query: String,
) -> Result<(QueryType, Vec<String>)> {
    let expected_arg_count = match query_type {
        QueryType::New => 3,
        QueryType::QueryTypeString => 0,
        QueryType::QueryTypeBitwise => 0,
        _ => 1,
    };

    let mut arguments = if expected_arg_count == 0 {
        Vec::new()
    } else {
        arguments_fetcher(query)?
    };

    if query_type == QueryType::New && arguments.len() == 2 {
        arguments.push('0'.to_string());
    }

    if expected_arg_count != arguments.len() {
        return Err(anyhow!("Invalid amount of arguments: {}", arguments.len()));
    }

    Ok((query_type, arguments))
}

/// Parse a query, this means that it will get the query type and its
/// arguments.
///
/// Arguments
///
/// * `query` - The Firefly query.
pub fn parse_query(query: String) -> Result<(QueryType, Vec<String>)> {
    match get_query_type(query.clone()) {
        Some(query_type) => parse_query_arguments(get_arguments, query_type, query),
        None => Err(anyhow!("Invalid query type")),
    }
}
