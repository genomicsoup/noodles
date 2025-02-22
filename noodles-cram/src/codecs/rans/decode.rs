mod order_0;
mod order_1;

use std::io::{self, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use super::Order;

pub fn rans_decode<R>(reader: &mut R) -> io::Result<Vec<u8>>
where
    R: Read,
{
    let (order, _, data_len) = read_header(reader)?;

    let mut buf = vec![0; data_len];

    match order {
        Order::Zero => order_0::decode(reader, &mut buf)?,
        Order::One => order_1::decode(reader, &mut buf)?,
    }

    Ok(buf)
}

fn read_header<R>(reader: &mut R) -> io::Result<(Order, usize, usize)>
where
    R: Read,
{
    let order = reader.read_u8().and_then(|order| {
        Order::try_from(order).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })?;

    let compressed_len = reader.read_u32::<LittleEndian>().and_then(|n| {
        usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })?;

    let data_len = reader.read_u32::<LittleEndian>().and_then(|n| {
        usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })?;

    Ok((order, compressed_len, data_len))
}

pub fn rans_get_cumulative_freq(r: u32) -> u32 {
    r & 0x0fff
}

pub fn rans_advance_step(r: u32, c: u32, f: u32) -> u32 {
    f * (r >> 12) + (r & 0x0fff) - c
}

pub fn rans_renorm<R>(reader: &mut R, mut r: u32) -> io::Result<u32>
where
    R: Read,
{
    while r < (1 << 23) {
        r = (r << 8) + reader.read_u8().map(u32::from)?;
    }

    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_header() -> io::Result<()> {
        let data = [
            0x00, // order = 0
            0x25, 0x00, 0x00, 0x00, // compressed size = 37
            0x07, 0x00, 0x00, 0x00, // data size = 7
        ];

        let mut reader = &data[..];
        assert_eq!(read_header(&mut reader)?, (Order::Zero, 37, 7));

        Ok(())
    }

    #[test]
    fn test_rans_decode_with_order_0() -> io::Result<()> {
        let expected = b"noodles";

        let data = vec![
            0x00, 0x25, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x64, 0x82, 0x49, 0x65, 0x00,
            0x82, 0x49, 0x6c, 0x82, 0x49, 0x6e, 0x82, 0x49, 0x6f, 0x00, 0x84, 0x92, 0x73, 0x82,
            0x49, 0x00, 0xe2, 0x06, 0x83, 0x18, 0x74, 0x7b, 0x41, 0x0c, 0x2b, 0xa9, 0x41, 0x0c,
            0x25, 0x31, 0x80, 0x03,
        ];

        let mut reader = &data[..];
        let actual = rans_decode(&mut reader)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_rans_decode_with_order_1() -> io::Result<()> {
        let expected = b"noodles";

        let data = vec![
            0x01, 0x3b, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x64, 0x84, 0x00, 0x6e,
            0x84, 0x00, 0x6f, 0x00, 0x87, 0xff, 0x00, 0x64, 0x6c, 0x8f, 0xff, 0x00, 0x65, 0x00,
            0x73, 0x8f, 0xff, 0x00, 0x6c, 0x65, 0x8f, 0xff, 0x00, 0x6e, 0x6f, 0x8f, 0xff, 0x00,
            0x6f, 0x00, 0x64, 0x87, 0xff, 0x6f, 0x88, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x02,
            0x02, 0x28, 0x00, 0x01, 0x02, 0x28, 0x00, 0x01, 0x02, 0x60, 0x00, 0x02,
        ];

        let mut reader = &data[..];
        let actual = rans_decode(&mut reader)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
