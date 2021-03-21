use std::fs::File;
use std::io;
use std::io::Read;

use crate::chunk::WriteChunk;

mod varint;
mod chunk;
mod ieee_float;

fn main() -> io::Result<()> {

    let mut file = File::open("/home/uranix/out.ssh.std")?;
    let mut value : [u8; 4] = [0, 0, 0, 0];
    let sz = 100_000_000;

    let mut chunk : WriteChunk<f32> = WriteChunk::new(sz);

    for _ in 0..sz {
        match file.read_exact(&mut value) {
            Err(_) => break,
            Ok(_) => {},
        }
        chunk.write(f32::from_le_bytes(value))?;
    }

    chunk.complete()?;

    println!("{}", chunk.get_size());

    let mut ofile = File::create("/home/uranix/res.fpz")?;
    chunk.serialize(&mut ofile)?;

    Ok(())
}
