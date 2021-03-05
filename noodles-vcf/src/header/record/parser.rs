use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag, take_till, take_until},
    character::complete::{alpha1, none_of},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, separated_pair},
    IResult,
};

use super::{Value, PREFIX};

fn string(input: &str) -> IResult<&str, String> {
    delimited(
        tag("\""),
        map(
            opt(escaped_transform(
                none_of("\\\""),
                '\\',
                alt((tag("\\"), tag("\""))),
            )),
            |s| s.unwrap_or_default(),
        ),
        tag("\""),
    )(input)
}

fn value(input: &str) -> IResult<&str, String> {
    map(take_till(|c| matches!(c, '\"' | ',' | '>')), |s: &str| {
        s.into()
    })(input)
}

fn field_key(input: &str) -> IResult<&str, &str> {
    alpha1(input)
}

fn field_value(input: &str) -> IResult<&str, String> {
    alt((string, value))(input)
}

fn field(input: &str) -> IResult<&str, (String, String)> {
    map(
        separated_pair(field_key, tag("="), field_value),
        |(k, v): (&str, String)| (k.into(), v),
    )(input)
}

fn string_field(input: &str) -> IResult<&str, (String, String)> {
    map(
        separated_pair(field_key, tag("="), string),
        |(k, v): (&str, String)| (k.into(), v),
    )(input)
}

fn value_field(input: &str) -> IResult<&str, (String, String)> {
    map(
        separated_pair(field_key, tag("="), value),
        |(k, v): (&str, String)| (k.into(), v),
    )(input)
}

fn extra_fields<'a>(
    mut input: &'a str,
    fields: &mut Vec<(String, String)>,
) -> IResult<&'a str, ()> {
    loop {
        match tag(",")(input) {
            Ok((i, _)) => {
                let (i, f) = string_field(i)?;
                fields.push(f);
                input = i;
            }
            Err(nom::Err::Error(_)) => break,
            Err(e) => return Err(e),
        }
    }

    Ok((input, ()))
}

fn info_structure(input: &str) -> IResult<&str, Value> {
    let mut fields = Vec::new();

    let (input, _) = tag("<")(input)?;

    // ID
    let (input, f) = value_field(input)?;
    fields.push(f);

    // Number
    let (input, _) = tag(",")(input)?;
    let (input, f) = value_field(input)?;
    fields.push(f);

    // Type
    let (input, _) = tag(",")(input)?;
    let (input, f) = value_field(input)?;
    fields.push(f);

    // Description
    let (input, _) = tag(",")(input)?;
    let (input, f) = string_field(input)?;
    fields.push(f);

    let (input, _) = extra_fields(input, &mut fields)?;
    let (input, _) = tag(">")(input)?;

    Ok((input, Value::Struct(fields)))
}

fn filter_structure(input: &str) -> IResult<&str, Value> {
    let mut fields = Vec::new();

    let (input, _) = tag("<")(input)?;

    // ID
    let (input, f) = value_field(input)?;
    fields.push(f);

    // Description
    let (input, _) = tag(",")(input)?;
    let (input, f) = string_field(input)?;
    fields.push(f);

    let (input, _) = extra_fields(input, &mut fields)?;
    let (input, _) = tag(">")(input)?;

    Ok((input, Value::Struct(fields)))
}

fn format_structure(input: &str) -> IResult<&str, Value> {
    let mut fields = Vec::new();

    let (input, _) = tag("<")(input)?;

    // ID
    let (input, f) = value_field(input)?;
    fields.push(f);

    // Number
    let (input, _) = tag(",")(input)?;
    let (input, f) = value_field(input)?;
    fields.push(f);

    // Type
    let (input, _) = tag(",")(input)?;
    let (input, f) = value_field(input)?;
    fields.push(f);

    // Description
    let (input, _) = tag(",")(input)?;
    let (input, f) = string_field(input)?;
    fields.push(f);

    let (input, _) = extra_fields(input, &mut fields)?;
    let (input, _) = tag(">")(input)?;

    Ok((input, Value::Struct(fields)))
}

fn meta_list(input: &str) -> IResult<&str, &str> {
    delimited(tag("["), take_until("]"), tag("]"))(input)
}

fn meta_values_field(input: &str) -> IResult<&str, (String, String)> {
    map(
        separated_pair(tag("Values"), tag("="), meta_list),
        |(k, v): (&str, &str)| (k.into(), v.into()),
    )(input)
}

fn meta_structure(input: &str) -> IResult<&str, Value> {
    let mut fields = Vec::new();

    let (input, _) = tag("<")(input)?;

    // ID
    let (input, f) = field(input)?;
    fields.push(f);

    // Type
    let (input, _) = tag(",")(input)?;
    let (input, f) = field(input)?;
    fields.push(f);

    // Number
    let (input, _) = tag(",")(input)?;
    let (input, f) = field(input)?;
    fields.push(f);

    // Values
    let (input, _) = tag(",")(input)?;
    let (input, f) = meta_values_field(input)?;
    fields.push(f);

    let (input, _) = tag(">")(input)?;

    Ok((input, Value::Struct(fields)))
}

fn generic_structure(input: &str) -> IResult<&str, Value> {
    map(
        delimited(tag("<"), separated_list1(tag(","), field), tag(">")),
        Value::Struct,
    )(input)
}

fn generic_value(input: &str) -> IResult<&str, Value> {
    map(alt((string, value)), Value::String)(input)
}

fn record(input: &str) -> IResult<&str, (String, Value)> {
    let (input, key) = delimited(tag(PREFIX), take_until("="), tag("="))(input)?;

    let (input, value) = match key {
        "INFO" => info_structure(input)?,
        "FILTER" => filter_structure(input)?,
        "FORMAT" => format_structure(input)?,
        "META" => meta_structure(input)?,
        _ => alt((generic_structure, generic_value))(input)?,
    };

    Ok((input, (key.into(), value)))
}

pub fn parse(input: &str) -> IResult<&str, (String, Value)> {
    record(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert!(parse("").is_err());
        assert!(parse("fileformat=VCFv4.3").is_err());
        assert!(parse("#fileformat=VCFv4.3").is_err());
    }

    #[test]
    fn test_parse_with_record_string_value() -> Result<(), Box<dyn std::error::Error>> {
        let (_, (key, value)) = parse("##fileformat=VCFv4.3")?;
        assert_eq!(key, "fileformat");
        assert_eq!(value, Value::String(String::from("VCFv4.3")));

        let (_, (key, value)) = parse("##fileDate=20200502")?;
        assert_eq!(key, "fileDate");
        assert_eq!(value, Value::String(String::from("20200502")));

        let (_, (key, value)) = parse("##reference=file:///tmp/ref.fasta")?;
        assert_eq!(key, "reference");
        assert_eq!(value, Value::String(String::from("file:///tmp/ref.fasta")));

        Ok(())
    }

    #[test]
    fn test_parse_with_record_struct_value() -> Result<(), Box<dyn std::error::Error>> {
        let (_, (key, value)) = parse(
            r#"##ALT=<ID=NON_REF,Description="Represents any possible alternative allele at this location">"#,
        )?;

        assert_eq!(key, "ALT");
        assert_eq!(
            value,
            Value::Struct(vec![
                (String::from("ID"), String::from("NON_REF")),
                (
                    String::from("Description"),
                    String::from("Represents any possible alternative allele at this location")
                ),
            ])
        );

        let (_, (key, value)) = parse(
            r#"##INFO=<ID=NS,Number=1,Type=Integer,Description="Number of samples with data">"#,
        )?;

        assert_eq!(key, "INFO");
        assert_eq!(
            value,
            Value::Struct(vec![
                (String::from("ID"), String::from("NS")),
                (String::from("Number"), String::from("1")),
                (String::from("Type"), String::from("Integer")),
                (
                    String::from("Description"),
                    String::from("Number of samples with data")
                ),
            ])
        );

        let (_, (key, value)) = parse(r#"##FILTER=<ID=PASS,Description="">"#)?;

        assert_eq!(key, "FILTER");
        assert_eq!(
            value,
            Value::Struct(vec![
                (String::from("ID"), String::from("PASS")),
                (String::from("Description"), String::from("")),
            ])
        );

        let (_, (key, value)) =
            parse(r#"##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">"#)?;

        assert_eq!(key, "FORMAT");
        assert_eq!(
            value,
            Value::Struct(vec![
                (String::from("ID"), String::from("GT")),
                (String::from("Number"), String::from("1")),
                (String::from("Type"), String::from("String")),
                (String::from("Description"), String::from("Genotype")),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_parse_with_meta_record_struct_value() -> Result<(), Box<dyn std::error::Error>> {
        let (_, (key, value)) =
            parse("##META=<ID=Assay,Type=String,Number=.,Values=[WholeGenome, Exome]>")?;

        assert_eq!(key, "META");
        assert_eq!(
            value,
            Value::Struct(vec![
                (String::from("ID"), String::from("Assay")),
                (String::from("Type"), String::from("String")),
                (String::from("Number"), String::from(".")),
                (String::from("Values"), String::from("WholeGenome, Exome")),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_parse_with_invalid_info_record() {
        assert!(
            parse(
                r#"##INFO=<ID="NS",Number=1,Type=Integer,Description="Number of samples with data">"#
            )
            .is_err(),
            "INFO: ID must be a value"
        );

        assert!(
            parse(
                r#"##INFO=<ID=NS,Number="1",Type=Integer,Description="Number of samples with data">"#
            )
            .is_err(),
            "INFO: Number must be a value"
        );

        assert!(
            parse(
                r#"##INFO=<ID=NS,Number=1,Type="Integer",Description="Number of samples with data">"#
            )
            .is_err(),
            "INFO: Type must be a value"
        );

        assert!(
            parse(
                r#"##INFO=<ID=NS,Number=1,Type=Integer,Description=Number of samples with data>"#
            )
            .is_err(),
            "INFO: Description must be a string"
        );

        assert!(
            parse(
                r#"##INFO=<ID=NS,Number=1,Type=Integer,Description="Number of samples with data",Source=dbsnp>"#
            )
            .is_err(),
            "INFO: extra fields must be a string"
        );
    }

    #[test]
    fn test_parse_with_invalid_filter_record() {
        assert!(
            parse(r#"##FILTER=<ID="PASS",Description="All filters passed">"#).is_err(),
            "FILTER: ID must be a value"
        );

        assert!(
            parse(r#"##FILTER=<ID=PASS,Description=All filters passed>"#).is_err(),
            "FILTER: Description must be a string"
        );

        assert!(
            parse(r#"##FILTER=<ID=PASS,Description="All filters passed",Color=green>"#).is_err(),
            "FILTER: extra fields must be a string"
        );
    }

    #[test]
    fn test_parse_with_invalid_format_record() {
        assert!(
            parse(r#"##FORMAT=<ID="GT",Number=1,Type=String,Description="Genotype">"#).is_err(),
            "FORMAT: ID must be a value"
        );

        assert!(
            parse(r#"##FORMAT=<ID=GT,Number="1",Type=String,Description="Genotype">"#).is_err(),
            "FORMAT: Number must be a value"
        );

        assert!(
            parse(r#"##FORMAT=<ID=GT,Number=1,Type="String",Description="Genotype">"#).is_err(),
            "FORMAT: Type must be a value"
        );

        assert!(
            parse(r#"##FORMAT=<ID=GT,Number=1,Type=String,Description=Genotype>"#).is_err(),
            "FORMAT: Description must be a string"
        );

        assert!(
            parse(
                r#"##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype",Comment=noodles>"#
            )
            .is_err(),
            "FORMAT: extra fields must be a string"
        );
    }

    #[test]
    fn test_parse_with_invalid_meta_record() {
        assert_eq!(
            parse("##META=<ID=Assay,Type=String,Number=.,Values=WholeGenome>"),
            Err(nom::Err::Error(nom::error::Error::new(
                "WholeGenome>",
                nom::error::ErrorKind::Tag,
            ))),
            "Values missing '[]' delimiters"
        );
    }
}
