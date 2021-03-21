use bitstream_io::{BigEndian, BitWriter};



pub struct WriteChunk<F: Float> {
    size: u32,
    signs: BitWriter<Vec<u8>, BigEndian>,
    exps: BitWriter<Vec<u8>, BigEndian>,
    mantissas: Vec<BitWriter<Vec<u8>, BigEndian>>,
}


