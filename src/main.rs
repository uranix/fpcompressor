use std::fs::File;
use std::io;
use std::io::{Read};
use bitstream_io::{BitWriter, BigEndian, BitWrite};

mod varint;
mod chunk;

fn main() -> io::Result<()> {

    let mut file = File::open("/home/uranix/YZ.STD")?;
    let mut value : [u8; 4] = [0, 0, 0, 0];
    let mut eprev : u8 = 0;

    let buf = Vec::with_capacity(10);
    let mut stream = BitWriter::endian(buf, BigEndian);

    let sz = 10000;
    for _ in 0..sz {
        match file.read_exact(&mut value) {
            Err(_) => break,
            Ok(_) => {},
        }
        // let float = f32::from_le_bytes(value);
        let e : u8 = (value[3] << 1) | (value[2] >> 7);
        let deltae : i16 = e.wrapping_sub(eprev).into();
        let sdeltae = if deltae < 128 { deltae } else { deltae - 255 };

        varint::write(sdeltae, &mut stream)?;

        eprev = e;
        // println!("{:08b} {:08b} {:08b} {:08b} {} {}", value[3], value[2], value[1], value[0], e as i16 - 127, float);
    }

    stream.byte_align()?;
    println!("{} bits/point", 8.0f32 * stream.into_writer().len() as f32 / sz as f32);

    Ok(())
}
