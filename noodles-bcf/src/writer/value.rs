mod ty;

use self::ty::write_type;

use std::io::{self, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::record::value::{Int16, Int32, Int8, Type, Value};

#[allow(dead_code)]
pub fn write_value<W>(writer: &mut W, value: Option<Value>) -> io::Result<()>
where
    W: Write,
{
    match value {
        None => write_missing(writer),
        Some(Value::Int8(v)) => write_int8(writer, v),
        Some(Value::Int8Array(v)) => write_int8_array(writer, &v),
        Some(Value::Int16(v)) => write_int16(writer, v),
        Some(Value::Int16Array(v)) => write_int16_array(writer, &v),
        Some(Value::Int32(v)) => write_int32(writer, v),
        Some(Value::Int32Array(v)) => write_int32_array(writer, &v),
        _ => todo!(),
    }
}

pub fn write_missing<W>(writer: &mut W) -> io::Result<()>
where
    W: Write,
{
    write_type(writer, None)
}

pub fn write_int8<W>(writer: &mut W, value: Option<Int8>) -> io::Result<()>
where
    W: Write,
{
    match value {
        None => {
            write_type(writer, Some(Type::Int8(0)))?;
        }
        Some(v) => {
            write_type(writer, Some(Type::Int8(1)))?;
            writer.write_i8(i8::from(v))?;
        }
    }

    Ok(())
}

pub fn write_int8_array<W>(writer: &mut W, values: &[i8]) -> io::Result<()>
where
    W: Write,
{
    write_type(writer, Some(Type::Int8(values.len())))?;

    for &value in values {
        writer.write_i8(value)?;
    }

    Ok(())
}

pub fn write_int16<W>(writer: &mut W, value: Option<Int16>) -> io::Result<()>
where
    W: Write,
{
    match value {
        None => {
            write_type(writer, Some(Type::Int16(0)))?;
        }
        Some(v) => {
            write_type(writer, Some(Type::Int16(1)))?;
            writer.write_i16::<LittleEndian>(i16::from(v))?;
        }
    }

    Ok(())
}

pub fn write_int16_array<W>(writer: &mut W, values: &[i16]) -> io::Result<()>
where
    W: Write,
{
    write_type(writer, Some(Type::Int16(values.len())))?;

    for &value in values {
        writer.write_i16::<LittleEndian>(value)?;
    }

    Ok(())
}

pub fn write_int32<W>(writer: &mut W, value: Option<Int32>) -> io::Result<()>
where
    W: Write,
{
    match value {
        None => {
            write_type(writer, Some(Type::Int32(0)))?;
        }
        Some(v) => {
            write_type(writer, Some(Type::Int32(1)))?;
            writer.write_i32::<LittleEndian>(i32::from(v))?;
        }
    }

    Ok(())
}

pub fn write_int32_array<W>(writer: &mut W, values: &[i32]) -> io::Result<()>
where
    W: Write,
{
    write_type(writer, Some(Type::Int32(values.len())))?;

    for &value in values {
        writer.write_i32::<LittleEndian>(value)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_value() -> io::Result<()> {
        let mut buf = Vec::new();
        write_value(&mut buf, None)?;
        assert_eq!(buf, [0x00]);

        buf.clear();
        write_value(&mut buf, Some(Value::Int8(None)))?;
        assert_eq!(buf, [0x01]);

        buf.clear();
        write_value(&mut buf, Some(Value::Int8(Some(Int8::Value(5)))))?;
        assert_eq!(buf, [0x11, 0x05]);

        buf.clear();
        write_value(&mut buf, Some(Value::Int8Array(vec![5, 8, 13])))?;
        assert_eq!(buf, [0x31, 0x05, 0x08, 0x0d]);

        buf.clear();
        write_value(&mut buf, Some(Value::Int16(None)))?;
        assert_eq!(buf, [0x02]);

        buf.clear();
        write_value(&mut buf, Some(Value::Int16(Some(Int16::Value(377)))))?;
        assert_eq!(buf, [0x12, 0x79, 0x01]);

        buf.clear();
        write_value(&mut buf, Some(Value::Int16Array(vec![377, 610, 987])))?;
        assert_eq!(buf, [0x32, 0x79, 0x01, 0x62, 0x02, 0xdb, 0x03]);

        buf.clear();
        write_value(&mut buf, Some(Value::Int32(None)))?;
        assert_eq!(buf, [0x03]);

        buf.clear();
        write_value(&mut buf, Some(Value::Int32(Some(Int32::Value(75025)))))?;
        assert_eq!(buf, [0x13, 0x11, 0x25, 0x01, 0x00]);

        buf.clear();
        write_value(
            &mut buf,
            Some(Value::Int32Array(vec![75025, 121393, 196418])),
        )?;
        assert_eq!(
            buf,
            [0x33, 0x11, 0x25, 0x01, 0x00, 0x31, 0xda, 0x01, 0x00, 0x42, 0xff, 0x02, 0x00]
        );

        // TODO: Float
        // TODO: FloatArray
        // TODO: String

        Ok(())
    }
}