use std::io;
use std::io::Write;
use std::marker::PhantomData;

use bitstream_io::{BigEndian, BitWrite, BitWriter};

use crate::ieee_float::IEEEFloat;
use crate::varint;

pub struct WriteChunk<F: IEEEFloat> {
    size: usize,
    signs: BitWriter<Vec<u8>, BigEndian>,
    exps: BitWriter<Vec<u8>, BigEndian>,
    mantissas: Vec<BitWriter<Vec<u8>, BigEndian>>,
    prev_sign: bool,
    prev_exp: u16,
    _phantom: PhantomData<F>,
}

impl<F: IEEEFloat> WriteChunk<F> {
    pub fn new(capacity: usize) -> WriteChunk<F> {
        let cap_8 : usize = (capacity + 7) / 8;

        let mantissas : Vec<BitWriter<Vec<u8>, BigEndian>> = (0..F::MANTISSA_BITS)
            .map(|_| BitWriter::endian(Vec::with_capacity(cap_8), BigEndian))
            .collect();

        WriteChunk {
            size: 0,
            signs: BitWriter::endian(Vec::with_capacity(cap_8), BigEndian),
            exps: BitWriter::endian(Vec::with_capacity(16 * cap_8), BigEndian),
            mantissas,
            prev_sign: false,
            prev_exp: 0,
            _phantom: Default::default(),
        }
    }

    pub fn write(self: &mut WriteChunk<F>, value: F) -> io::Result<()> {
        let sign = value.get_sign();
        let exp = value.get_exp();
        let mantissa = value.get_mantissa();

        let exp_even = (exp & 1u16) == 0;

        self.size += 1;
        self.signs.write_bit(sign ^ self.prev_sign)?;
        varint::write(F::exp_diff(self.prev_exp, exp), &mut self.exps)?;
        for i in 0..F::MANTISSA_BITS {
            self.mantissas[i].write_bit(((mantissa & (1u64 << i)) == 0) ^ exp_even)?;
        }

        self.prev_sign = sign;
        self.prev_exp = exp;

        Ok(())
    }

    pub fn complete(self: &mut WriteChunk<F>) -> io::Result<()> {
        self.signs.byte_align()?;
        self.exps.byte_align()?;
        for mantissa in &mut self.mantissas {
            mantissa.byte_align()?;
        }
        Ok(())
    }

    pub fn get_size(self: &mut WriteChunk<F>) -> usize {
        self.size
    }

    pub fn serialize<W: Write>(self: WriteChunk<F>, ostream: &mut W) -> io::Result<()> {
        ostream.write(&(self.size as u32).to_be_bytes()[0..4])?;
        // TODO: checksum, compression, etc
        ostream.write(&self.signs.into_writer())?;
        ostream.write(&self.exps.into_writer())?;
        for mantissa in self.mantissas {
            ostream.write(&mantissa.into_writer())?;
        }

        Ok(())
    }
}


