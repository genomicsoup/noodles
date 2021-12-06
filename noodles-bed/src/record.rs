//! BED record and fields.

use std::{error, fmt, num, str::FromStr};

/// A list of raw optional fields.
pub type OptionalFields = Vec<String>;

#[derive(Clone, Debug, Eq, PartialEq)]
struct StandardFields {
    reference_sequence_name: String,
    start_position: u64,
    end_position: u64,
}

impl StandardFields {
    fn new<N>(reference_sequence_name: N, start_position: u64, end_position: u64) -> Self
    where
        N: Into<String>,
    {
        Self {
            reference_sequence_name: reference_sequence_name.into(),
            start_position,
            end_position,
        }
    }
}

/// A BED record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record<const N: u8> {
    standard_fields: StandardFields,
    optional_fields: OptionalFields,
}

impl<const N: u8> Record<N> {
    /// Returns the reference sequence name (`chrom`).
    pub fn reference_sequence_name(&self) -> &str {
        &self.standard_fields.reference_sequence_name
    }

    /// Returns the feature start position (`chromStart`).
    pub fn start_position(&self) -> u64 {
        self.standard_fields.start_position
    }

    /// Returns the feature end position (`chromEnd`).
    pub fn end_position(&self) -> u64 {
        self.standard_fields.end_position
    }

    /// Returns the list of raw optional fields.
    pub fn optional_fields(&self) -> &OptionalFields {
        &self.optional_fields
    }
}

/// An error returned when a raw BED record fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The reference sequence name is missing.
    MissingReferenceSequenceName,
    /// The start position is missing.
    MissingStartPosition,
    /// The start position is invalid.
    InvalidStartPosition(num::ParseIntError),
    /// The end position is missing.
    MissingEndPosition,
    /// The end position is invalid.
    InvalidEndPosition(num::ParseIntError),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingReferenceSequenceName => f.write_str("missing reference sequence name"),
            Self::MissingStartPosition => f.write_str("missing start position"),
            Self::InvalidStartPosition(e) => write!(f, "invalid start position: {}", e),
            Self::MissingEndPosition => f.write_str("missing end position"),
            Self::InvalidEndPosition(e) => write!(f, "invalid end position: {}", e),
        }
    }
}

impl FromStr for Record<3> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const DELIMITER: char = '\t';

        let mut fields = s.split(DELIMITER);

        let standard_fields = parse_mandatory_fields(&mut fields)?;
        let optional_fields = parse_optional_fields(&mut fields);

        Ok(Self {
            standard_fields,
            optional_fields,
        })
    }
}

fn parse_mandatory_fields<'a, I>(fields: &mut I) -> Result<StandardFields, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    let reference_sequence_name = fields
        .next()
        .ok_or(ParseError::MissingReferenceSequenceName)?;

    let start_position = fields
        .next()
        .ok_or(ParseError::MissingStartPosition)
        .and_then(|s| s.parse().map_err(ParseError::InvalidStartPosition))?;

    let end_position = fields
        .next()
        .ok_or(ParseError::MissingEndPosition)
        .and_then(|s| s.parse().map_err(ParseError::InvalidEndPosition))?;

    Ok(StandardFields::new(
        reference_sequence_name,
        start_position,
        end_position,
    ))
}

fn parse_optional_fields<'a, I>(fields: &mut I) -> OptionalFields
where
    I: Iterator<Item = &'a str>,
{
    fields.map(|s| s.into()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_for_record_3() {
        let actual = "sq0\t8\t13".parse();

        let standard_fields = StandardFields::new("sq0", 8, 13);
        let expected = Ok(Record {
            standard_fields,
            optional_fields: Vec::new(),
        });

        assert_eq!(actual, expected);
    }
}
