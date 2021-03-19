use std::io::{Read, Result, Write};

use bitstream_io::{BigEndian, BitRead, BitReader, BitWrite, BitWriter};

pub fn write<W: Write>(x: i16, stream: &mut BitWriter<W, BigEndian>) -> Result<()> {
    if x == 0 {
        stream.write(2, 0b00u8)
    } else if x == 1 {
        stream.write(2, 0b01u8)
    } else if x == -1 {
        stream.write(2, 0b10u8)
    } else {
        let neg: bool = x < 0;
        let absx: u16 = (if neg { -x } else { x }) as u16;
        if absx < 18 {
            stream.write(3, 0b110u8)?;
            stream.write_bit(neg)?;
            stream.write(4, absx - 2)
        } else if absx < 146 {
            stream.write(4, 0b1110u8)?;
            stream.write_bit(neg)?;
            stream.write(7, absx - 18)
        } else if absx < 1170 {
            stream.write(5, 0b11110u8)?;
            stream.write_bit(neg)?;
            stream.write(10, absx - 146)
        } else {
            panic!("Unsupported integer {} in write_varint", x);
        }
    }
}

fn read_sign_and_value<R: Read>(stream: &mut BitReader<R, BigEndian>) -> Result<(bool, u16)> {
    if stream.read_bit()? {
        if stream.read_bit()? {
            if stream.read_bit()? {
                // 0b11111 prefix - reserved
                panic!("Unsupported bit sequence (reserved)");
            } else {
                // 0b11110 prefix
                let neg: bool = stream.read_bit()?;
                let absx: u16 = stream.read::<u16>(10)? + 146;
                Ok((neg, absx))
            }
        } else {
            // 0b1110 prefix
            let neg: bool = stream.read_bit()?;
            let absx: u16 = stream.read::<u16>(7)? + 18;
            Ok((neg, absx))
        }
    } else {
        // 0b110 prefix
        let neg: bool = stream.read_bit()?;
        let absx: u16 = stream.read::<u16>(4)? + 2;
        Ok((neg, absx))
    }
}

pub fn read<R: Read>(stream: &mut BitReader<R, BigEndian>) -> Result<i16> {
    let prefix: u8 = stream.read::<u8>(2)?;
    if prefix == 0b00u8 {
        Ok(0)
    } else if prefix == 0b01u8 {
        Ok(1)
    } else if prefix == 0b10u8 {
        Ok(-1)
    } else {
        let (neg, absx) = read_sign_and_value(stream)?;
        if neg {
            Ok(-(absx as i16))
        } else {
            Ok(absx as i16)
        }
    }
}

#[test]
fn rtt() -> Result<()> {
    let mut buff = Vec::with_capacity(10);
    let mut write_stream = BitWriter::endian(buff, BigEndian);

    let span = -1024..=1024;

    for x in span.clone() {
        write(x, &mut write_stream)?;
    }

    write_stream.byte_align()?;
    write_stream.flush()?;

    buff = write_stream.into_writer();

    let mut read_stream = BitReader::endian(buff.as_slice(), BigEndian);

    for y in span {
        assert_eq!(y, read(&mut read_stream)?);
    }

    Ok(())
}
