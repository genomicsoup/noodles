//! GTF record and fields.

pub mod attributes;
pub mod strand;

pub use self::{attributes::Attributes, strand::Strand};

use std::{error, fmt, num, str::FromStr};

pub(crate) const NULL_FIELD: &str = ".";

/// A GTF record.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    reference_sequence_name: String,
    source: String,
    ty: String,
    start: i32,
    end: i32,
    score: Option<f32>,
    strand: Option<Strand>,
    frame: Option<String>,
    attributes: Attributes,
}

/// An error returned when a raw GTF record fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// The reference sequence name is missing.
    MissingReferenceSequenceName,
    /// The source is missing.
    MissingSource,
    /// The type is missing.
    MissingType,
    /// The start is missing.
    MissingStart,
    /// The start is invalid.
    InvalidStart(num::ParseIntError),
    /// The end is missing.
    MissingEnd,
    /// The end is invalid.
    InvalidEnd(num::ParseIntError),
    /// The score is missing.
    MissingScore,
    /// Thes score is invalid.
    InvalidScore(num::ParseFloatError),
    /// The strand is missing.
    MissingStrand,
    /// The strand is invalid.
    InvalidStrand(strand::ParseError),
    /// The frame is missing.
    MissingFrame,
    /// The frame is invalid.
    InvalidFrame,
    /// The attributes are missing.
    MissingAttributes,
    /// The attributes are invalid.
    InvalidAttributes(attributes::ParseError),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty input"),
            Self::MissingReferenceSequenceName => write!(f, "missing reference sequence name"),
            Self::MissingSource => write!(f, "missing source"),
            Self::MissingType => write!(f, "missing type"),
            Self::MissingStart => write!(f, "missing start"),
            Self::InvalidStart(e) => write!(f, "invalid start: {}", e),
            Self::MissingEnd => write!(f, "missing end"),
            Self::InvalidEnd(e) => write!(f, "invalid end: {}", e),
            Self::MissingScore => write!(f, "missing score"),
            Self::InvalidScore(e) => write!(f, "invalid score: {}", e),
            Self::MissingStrand => write!(f, "missing strand"),
            Self::InvalidStrand(e) => write!(f, "invalid strand: {}", e),
            Self::MissingFrame => write!(f, "missing frame"),
            Self::InvalidFrame => write!(f, "invalid frame"),
            Self::MissingAttributes => write!(f, "missing attributes"),
            Self::InvalidAttributes(e) => write!(f, "invalid attributes: {}", e),
        }
    }
}

impl FromStr for Record {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const FIELD_DELIMITER: char = '\t';
        const MAX_FIELDS: usize = 9;

        let mut fields = s.splitn(MAX_FIELDS, FIELD_DELIMITER);

        let reference_sequence_name = fields
            .next()
            .map(|s| s.into())
            .ok_or(ParseError::MissingReferenceSequenceName)?;

        let source = fields
            .next()
            .map(|s| s.into())
            .ok_or(ParseError::MissingSource)?;

        let ty = fields
            .next()
            .map(|s| s.into())
            .ok_or(ParseError::MissingType)?;

        let start = fields
            .next()
            .ok_or(ParseError::MissingStart)
            .and_then(|s| s.parse().map_err(ParseError::InvalidStart))?;

        let end = fields
            .next()
            .ok_or(ParseError::MissingEnd)
            .and_then(|s| s.parse().map_err(ParseError::InvalidEnd))?;

        let score = fields
            .next()
            .ok_or(ParseError::MissingScore)
            .and_then(parse_score)?;

        let strand = fields
            .next()
            .ok_or(ParseError::MissingStrand)
            .and_then(parse_strand)?;

        let frame = fields
            .next()
            .ok_or(ParseError::MissingFrame)
            .and_then(parse_frame)?;

        let attributes = fields
            .next()
            .ok_or(ParseError::MissingAttributes)
            .and_then(|s| s.parse().map_err(ParseError::InvalidAttributes))?;

        Ok(Self {
            reference_sequence_name,
            source,
            ty,
            start,
            end,
            score,
            strand,
            frame,
            attributes,
        })
    }
}

fn parse_score(s: &str) -> Result<Option<f32>, ParseError> {
    if s == NULL_FIELD {
        Ok(None)
    } else {
        s.parse().map(Some).map_err(ParseError::InvalidScore)
    }
}

fn parse_strand(s: &str) -> Result<Option<Strand>, ParseError> {
    if s == NULL_FIELD {
        Ok(None)
    } else {
        s.parse().map(Some).map_err(ParseError::InvalidStrand)
    }
}

fn parse_frame(s: &str) -> Result<Option<String>, ParseError> {
    match s {
        NULL_FIELD => Ok(None),
        "1" | "2" | "3" => Ok(Some(s.into())),
        _ => Err(ParseError::InvalidFrame),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        use attributes::Entry;

        let s = "sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id \"g0\"; transcript_id \"t0\";";

        assert_eq!(
            s.parse(),
            Ok(Record {
                reference_sequence_name: String::from("sq0"),
                source: String::from("NOODLES"),
                ty: String::from("gene"),
                start: 8,
                end: 13,
                score: None,
                strand: Some(Strand::Forward),
                frame: None,
                attributes: Attributes::from(vec![
                    Entry::new("gene_id", "g0"),
                    Entry::new("transcript_id", "t0"),
                ])
            })
        );
    }
}