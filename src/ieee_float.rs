use std::fmt::Debug;

#[cfg(test)]
use num::traits::FloatConst;

#[cfg(test)]
use rand::Rng;

pub trait IEEEFloat: Sized + Copy + PartialEq + Debug {
    const EXP_BITS: usize;
    const MANTISSA_BITS: usize;

    const SIGN_MASK: u64 = (1u64 << (Self::EXP_BITS + Self::MANTISSA_BITS));
    const MANTISSA_MASK: u64 = (1u64 << Self::MANTISSA_BITS) - 1;
    const EXP_MASK: u64 = (Self::SIGN_MASK - 1) & !Self::MANTISSA_MASK;
    const EXP_BOUND: u16 = 1u16 << Self::EXP_BITS;
    const EXP_SHIFT: i16 = (1u16 << (Self::EXP_BITS - 1)) as i16;

    fn to_u64(self: Self) -> u64;
    fn from_u64(value: u64) -> Self;

    fn from_parts(sign: bool, exp: u16, mantissa: u64) -> Self {
        let x1: u64 = if sign { Self::SIGN_MASK } else { 0u64 };
        let x2: u64 = (exp as u64) << Self::MANTISSA_BITS;
        Self::from_u64(x1 | x2 | mantissa)
    }

    fn get_sign(self: Self) -> bool {
        self.to_u64() & Self::SIGN_MASK != 0
    }

    fn get_exp(self: Self) -> u16 {
        ((self.to_u64() & Self::EXP_MASK) >> Self::MANTISSA_BITS) as u16
    }

    fn get_mantissa(self: Self) -> u64 {
        self.to_u64() & Self::MANTISSA_MASK
    }

    fn exp_diff(eprev: u16, ecurr: u16) -> i16 {
        let ediff = (((ecurr + Self::EXP_BOUND) - eprev) % Self::EXP_BOUND) as i16;
        if ediff < Self::EXP_SHIFT {
            ediff
        } else {
            ediff - (Self::EXP_BOUND as i16)
        }
    }

    fn exp_adv(eprev: u16, ediff: i16) -> u16 {
        let ediff_unsigned = (ediff + Self::EXP_BOUND as i16) as u16;
        (eprev + ediff_unsigned) % Self::EXP_BOUND
    }
}

impl IEEEFloat for f32 {
    const EXP_BITS: usize = 8;
    const MANTISSA_BITS: usize = 23;

    fn to_u64(self: Self) -> u64 {
        unsafe { std::mem::transmute::<f32, u32>(self) as u64 }
    }

    fn from_u64(value: u64) -> Self {
        unsafe { std::mem::transmute::<u32, f32>(value as u32) }
    }
}

impl IEEEFloat for f64 {
    const EXP_BITS: usize = 11;
    const MANTISSA_BITS: usize = 52;

    fn to_u64(self: Self) -> u64 {
        unsafe { std::mem::transmute(self) }
    }

    fn from_u64(value: u64) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[cfg(test)]
fn test_binary<F: IEEEFloat>(value: F, sign: bool, exp: u16, mantissa: u64) {
    assert_eq!(value.get_sign(), sign);
    assert_eq!(value.get_exp(), exp);
    assert_eq!(value.get_mantissa(), mantissa);

    assert_eq!(F::from_parts(sign, exp, mantissa), value);
}

#[test]
fn binary32() {
    test_binary(0f32, false, 0, 0);
    test_binary(f32::PI(), false, 128, 0x490fdb);
    test_binary(-1.0f32/3.0f32, true, 125, 0x2aaaab);
    test_binary(f32::NEG_INFINITY, true, 255, 0);
}

#[test]
fn binary64() {
    test_binary(0f64, false, 0, 0);
    test_binary(f64::PI(), false, 1024, 0x921fb54442d18);
    test_binary(-1.0f64/3.0f64, true, 1021, 0x5_5555_5555_5555);
    test_binary(f64::NEG_INFINITY, true, 2047, 0);
}

#[cfg(test)]
fn fuzz_binary<F: IEEEFloat>() {
    let mut rng = rand::thread_rng();

    for _ in 0..1_000_000 {
        let sign : bool = rng.gen();
        let exp : u16 = rng.gen_range(0..(1u16 << F::EXP_BITS));
        let mantissa : u64 = rng.gen_range(0..(1u64 << F::MANTISSA_BITS));

        let value = F::from_parts(sign, exp, mantissa);
        assert_eq!(value.get_sign(), sign);
        assert_eq!(value.get_exp(), exp);
        assert_eq!(value.get_mantissa(), mantissa);
    }
}

#[test]
fn fuzz_binary32() {
    fuzz_binary::<f32>();
}

#[test]
fn fuzz_binary64() {
    fuzz_binary::<f64>();
}

#[cfg(test)]
fn test_exp_modular<F: IEEEFloat>() {
    for e1 in 0..F::EXP_BOUND {
        for e2 in 0..F::EXP_BOUND {
            let ed : i16 = F::exp_diff(e1, e2);
            assert!(ed < F::EXP_SHIFT);
            assert!(ed >= -F::EXP_SHIFT);
            let er = F::exp_adv(e1, ed);
            assert_eq!(e2, er)
        }
    }
}

#[test]
fn exp_modular32() {
    test_exp_modular::<f32>();
}

#[test]
fn exp_modular64() {
    test_exp_modular::<f64>();
}
