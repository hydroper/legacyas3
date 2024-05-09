use crate::ns::*;

use num_traits::FromPrimitive;
// use num_traits::{One, Zero};
// use num_bigint::BigInt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};

/// Represents a numeric value represented as one of the data types
/// `Number`, `float`, `uint`, or `int`.
#[derive(Clone, PartialEq)]
pub enum NumberVariant {
    Number(f64),
    Float(f32),
    // BigInt(BigInt),
    Int(i32),
    Uint(u32),
}

impl Add for NumberVariant {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(v + rhs)
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(v + rhs)
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v.checked_add(rhs).unwrap_or(0))
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v.checked_add(rhs).unwrap_or(0))
            },
        }
    }
}

impl Sub for NumberVariant {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(v - rhs)
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(v - rhs)
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v.checked_sub(rhs).unwrap_or(0))
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v.checked_sub(rhs).unwrap_or(0))
            },
        }
    }
}

impl Mul for NumberVariant {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(v * rhs)
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(v * rhs)
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v.checked_mul(rhs).unwrap_or(0))
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v.checked_mul(rhs).unwrap_or(0))
            },
        }
    }
}

impl Div for NumberVariant {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(v / rhs)
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(v / rhs)
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v.checked_div(rhs).unwrap_or(0))
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v.checked_div(rhs).unwrap_or(0))
            },
        }
    }
}

impl Rem for NumberVariant {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(v % rhs)
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(v % rhs)
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v.checked_rem(rhs).unwrap_or(0))
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v.checked_rem(rhs).unwrap_or(0))
            },
        }
    }
}

impl std::ops::Neg for NumberVariant {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Float(v) => Self::Float(-v),
            Self::Number(v) => Self::Number(-v),
            Self::Int(v) => Self::Int(-v),
            Self::Uint(v) => Self::Uint(v),
        }
    }
}

impl BitAnd for NumberVariant {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(f32::from_u32(unsafe { v.to_int_unchecked::<u32>() } & unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0.0))
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(f64::from_u32(unsafe { v.to_int_unchecked::<u32>() } & unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0.0))
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v & rhs)
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v & rhs)
            },
        }
    }
}

impl BitXor for NumberVariant {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(f32::from_u32(unsafe { v.to_int_unchecked::<u32>() } ^ unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0.0))
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(f64::from_u32(unsafe { v.to_int_unchecked::<u32>() } ^ unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0.0))
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v ^ rhs)
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v ^ rhs)
            },
        }
    }
}

impl BitOr for NumberVariant {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(f32::from_u32(unsafe { v.to_int_unchecked::<u32>() } | unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0.0))
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(f64::from_u32(unsafe { v.to_int_unchecked::<u32>() } | unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0.0))
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v | rhs)
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v | rhs)
            },
        }
    }
}

impl Shl for NumberVariant {
    type Output = Self;
    fn shl(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(f32::from_u32(unsafe { v.to_int_unchecked::<u32>() }.checked_shl(unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0)).unwrap_or(0.0))
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(f64::from_u32(unsafe { v.to_int_unchecked::<u32>() }.checked_shl(unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0)).unwrap_or(0.0))
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v.checked_shl(rhs.try_into().unwrap_or(0)).unwrap_or(0))
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v.checked_shl(rhs).unwrap_or(0))
            },
        }
    }
}

impl Shr for NumberVariant {
    type Output = Self;
    fn shr(self, rhs: Self) -> Self::Output {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(f32::from_u32(unsafe { v.to_int_unchecked::<u32>() }.checked_shr(unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0)).unwrap_or(0.0))
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(f64::from_u32(unsafe { v.to_int_unchecked::<u32>() }.checked_shr(unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0)).unwrap_or(0.0))
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                Self::Int(v.checked_shr(rhs.try_into().unwrap_or(0)).unwrap_or(0))
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v.checked_shr(rhs).unwrap_or(0))
            },
        }
    }
}

impl NumberVariant {
    pub fn zero(type_thing: &Thingy, host: &SemanticHost) -> Self {
        if type_thing == &host.number_type() {
            Self::Number(0.0)
        } else if type_thing == &host.int_type() {
            Self::Int(0)
        } else if type_thing == &host.uint_type() {
            Self::Uint(0)
        /*
        } else if type_thing == &host.big_int_type() {
            Self::BigInt(BigInt::zero())
        */
        } else if type_thing == &host.float_type() {
            Self::Float(0.0)
        } else {
            panic!()
        }
    }

    pub fn nan(type_thing: &Thingy, host: &SemanticHost) -> Self {
        if type_thing == &host.number_type() {
            Self::Number(f64::NAN)
        } else if type_thing == &host.float_type() {
            Self::Float(f32::NAN)
        } else {
            panic!("Type does not support NaN.");
        }
    }

    pub fn one(type_thing: &Thingy, host: &SemanticHost) -> Self {
        if type_thing == &host.number_type() {
            Self::Number(1.0)
        } else if type_thing == &host.int_type() {
            Self::Int(1)
        } else if type_thing == &host.uint_type() {
            Self::Uint(1)
        /*
        } else if type_thing == &host.big_int_type() {
            Self::BigInt(BigInt::one())
        */
        } else if type_thing == &host.float_type() {
            Self::Float(1.0)
        } else {
            panic!()
        }
    }

    pub fn minimum_value(type_thing: &Thingy, host: &SemanticHost) -> Self {
        if type_thing == &host.number_type() {
            Self::Number(f64::NEG_INFINITY)
        } else if type_thing == &host.int_type() {
            Self::Int(i32::MIN)
        } else if type_thing == &host.uint_type() {
            Self::Uint(0)
        /*
        } else if type_thing == &host.big_int_type() {
            panic!("BigInt has no minimum value")
        */
        } else if type_thing == &host.float_type() {
            Self::Float(f32::NEG_INFINITY)
        } else {
            panic!()
        }
    }

    pub fn maximum_value(type_thing: &Thingy, host: &SemanticHost) -> Self {
        if type_thing == &host.number_type() {
            Self::Number(f64::INFINITY)
        } else if type_thing == &host.int_type() {
            Self::Int(i32::MAX)
        } else if type_thing == &host.uint_type() {
            Self::Uint(u32::MAX)
        /*
        } else if type_thing == &host.big_int_type() {
            panic!("BigInt has no maximum value")
        */
        } else if type_thing == &host.float_type() {
            Self::Float(f32::INFINITY)
        } else {
            panic!()
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Float(v) => v == &0.0,
            Self::Number(v) => v == &0.0,
            // Self::BigInt(v) => v.is_zero(),
            Self::Int(v) => v == &0,
            Self::Uint(v) => v == &0,
        }
    }

    pub fn is_one(&self) -> bool {
        match self {
            Self::Float(v) => v == &1.0,
            Self::Number(v) => v == &1.0,
            // Self::BigInt(v) => v.is_one(),
            Self::Int(v) => v == &1,
            Self::Uint(v) => v == &1,
        }
    }

    pub fn multiply_per_two(&self) -> Self {
        match self {
            Self::Float(v) => Self::Float(v * 2.0),
            Self::Number(v) => Self::Number(v * 2.0),
            // Self::BigInt(v) => Self::BigInt(v * 2),
            Self::Int(v) => Self::Int(v * 2),
            Self::Uint(v) => Self::Uint(v * 2),
        }
    }

    pub fn increase_by_one(&self) -> Self {
        match self {
            Self::Float(v) => Self::Float(v + 1.0),
            Self::Number(v) => Self::Number(v + 1.0),
            // Self::BigInt(v) => Self::BigInt(v + 1),
            Self::Int(v) => Self::Int(v + 1),
            Self::Uint(v) => Self::Uint(v + 1),
        }
    }

    /// Performs bitwise OR if `value` is true or erases bits with the `erase_bits()` method otherwise.
    pub fn apply_bits(&self, bits: &Self, value: bool) -> Self {
        if value { self.clone() | bits.clone() } else { self.erase_bits(bits) }
    }

    /// Erases bits if all of such bits are included in the base value.
    pub fn erase_bits(&self, bits: &Self) -> Self {
        if self.includes_bits(bits) { self.clone() ^ bits.clone() } else { self.clone() }
    }

    pub fn bitwise_not(&self) -> Self {
        match self {
            Self::Float(v) => Self::Float(f32::from_u32(unsafe { !v.to_int_unchecked::<u32>() }).unwrap_or(0.0)),
            Self::Number(v) => Self::Number(f64::from_u32(unsafe { !v.to_int_unchecked::<u32>() }).unwrap_or(0.0)),
            Self::Int(v) => Self::Int(!v),
            Self::Uint(v) => Self::Uint(!v),
        }
    }
    
    pub fn shift_right_unsigned(&self, rhs: &Self) -> Self {
        match self {
            Self::Float(v) => {
                let rhs = rhs.as_float().unwrap();
                Self::Float(f32::from_u32(unsafe { v.to_int_unchecked::<u32>() }.checked_shr(unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0)).unwrap_or(0.0))
            },
            Self::Number(v) => {
                let rhs = rhs.as_double().unwrap();
                Self::Number(f64::from_u32(unsafe { v.to_int_unchecked::<u32>() }.checked_shr(unsafe { rhs.to_int_unchecked::<u32>() }).unwrap_or(0)).unwrap_or(0.0))
            },
            Self::Int(v) => {
                let rhs = rhs.as_int().unwrap();
                let uint1: u32 = (*v).try_into().unwrap_or(0);
                let uint2: u32 = rhs.try_into().unwrap_or(0);
                let v = uint1.checked_shr(uint2).unwrap_or(0);
                let v: i32 = v.try_into().unwrap_or(0);
                Self::Int(v)
            },
            Self::Uint(v) => {
                let rhs = rhs.as_uint().unwrap();
                Self::Uint(v.checked_shr(rhs).unwrap_or(0))
            },
        }
    }

    pub fn includes_bits(&self, rhs: &Self) -> bool {
        match self {
            Self::Float(v) => {
                let Self::Float(rhs) = rhs else { panic!(); };
                (unsafe { v.to_int_unchecked::<u32>() } & unsafe { rhs.to_int_unchecked::<u32>() } != 0)
            },
            Self::Number(v) => {
                let Self::Number(rhs) = rhs else { panic!(); };
                (unsafe { v.to_int_unchecked::<u32>() } & unsafe { rhs.to_int_unchecked::<u32>() } != 0)
            },
            /*
            Self::BigInt(v) => {
                let Self::BigInt(ref rhs) = rhs else { panic!(); };
                !(v.clone() & rhs.clone()).is_zero()
            },
            */
            Self::Int(v) => {
                let Self::Int(rhs) = rhs else { panic!(); };
                v & rhs != 0
            },
            Self::Uint(v) => {
                let Self::Uint(rhs) = rhs else { panic!(); };
                v & rhs != 0
            },
        }
    }

    pub fn is_power_of_two(&self) -> bool {
        // Based on https://stackoverflow.com/a/600306
        match self {
            Self::Float(v) => {
                let v = unsafe { v.to_int_unchecked::<u32>() };
                (v != 0) && ((v & (v - 1)) == 0)
            },
            Self::Number(v) => {
                let v = unsafe { v.to_int_unchecked::<u32>() };
                (v != 0) && ((v & (v - 1)) == 0)
            },
            /*
            Self::BigInt(v) => {
                !v.is_zero() && ((v & (v - BigInt::one())).is_zero())
            },
            */
            Self::Int(v) => (v != &0) && ((v & (v - 1)) == 0),
            Self::Uint(v) => (v != &0) && ((v & (v - 1)) == 0),
        }
    }

    pub fn convert_type(&self, target_type: &Thingy, host: &SemanticHost) -> Result<Self, DeferError> {
        let number_type = host.number_type().defer()?;
        let float_type = host.float_type().defer()?;
        let int_type = host.int_type().defer()?;
        let uint_type = host.int_type().defer()?;
        // let big_int_type = host.big_int_type().defer()?;

        Ok(if target_type == &number_type {
            match self {
                Self::Float(v) => Self::Number(*v as f64),
                Self::Number(v) => Self::Number(*v),
                /*
                Self::BigInt(v) => {
                    let v: Result<u32, _> = v.try_into();
                    Self::Number(v.map(|v| v as f64).unwrap_or(f64::NAN))
                },
                */
                Self::Int(v) => {
                    let v: Result<i32, _> = (*v).try_into();
                    Self::Number(v.map(|v| v as f64).unwrap_or(f64::NAN))
                },
                Self::Uint(v) => {
                    let v: Result<u32, _> = (*v).try_into();
                    Self::Number(v.map(|v| v as f64).unwrap_or(f64::NAN))
                },
            }
        } else if target_type == &float_type {
            match self {
                Self::Float(v) => Self::Float(*v),
                Self::Number(v) => Self::Float(*v as f32),
                /*
                Self::BigInt(v) => {
                    let v: Result<u32, _> = v.try_into();
                    Self::Float(v.map(|v| v as f32).unwrap_or(f32::NAN))
                },
                */
                Self::Int(v) => {
                    let v: Result<i32, _> = (*v).try_into();
                    Self::Float(v.map(|v| v as f32).unwrap_or(f32::NAN))
                },
                Self::Uint(v) => {
                    let v: Result<u32, _> = (*v).try_into();
                    Self::Float(v.map(|v| v as f32).unwrap_or(f32::NAN))
                },
            }
        } else if target_type == &int_type {
            match self {
                Self::Float(v) => Self::Int(if v.is_infinite() {
                    if v.is_sign_negative() { i32::MIN } else { i32::MAX }
                } else if v.is_nan() { 0 } else { unsafe { v.to_int_unchecked() } }),
                Self::Number(v) => Self::Int(if v.is_infinite() {
                    if v.is_sign_negative() { i32::MIN } else { i32::MAX }
                } else if v.is_nan() { 0 } else { unsafe { v.to_int_unchecked() } }),
                // Self::BigInt(v) => Self::Int(v.try_into().unwrap_or(0)),
                Self::Int(v) => Self::Int((*v).try_into().unwrap_or(0)),
                Self::Uint(v) => Self::Int((*v).try_into().unwrap_or(0)),
            }
        } else if target_type == &uint_type {
            match self {
                Self::Float(v) => Self::Uint(if v.is_infinite() {
                    if v.is_sign_negative() { u32::MIN } else { u32::MAX }
                } else if v.is_nan() { 0 } else { unsafe { v.to_int_unchecked() } }),
                Self::Number(v) => Self::Uint(if v.is_infinite() {
                    if v.is_sign_negative() { u32::MIN } else { u32::MAX }
                } else if v.is_nan() { 0 } else { unsafe { v.to_int_unchecked() } }),
                // Self::BigInt(v) => Self::Int(v.try_into().unwrap_or(0)),
                Self::Int(v) => Self::Uint((*v).try_into().unwrap_or(0)),
                Self::Uint(v) => Self::Uint((*v).try_into().unwrap_or(0)),
            }
        /*
        } else if target_type == &big_int_type {
            match self {
                Self::Float(v) => Self::BigInt(BigInt::from_f32(*v).unwrap_or(BigInt::zero())),
                Self::Number(v) => Self::BigInt(BigInt::from_f64(*v).unwrap_or(BigInt::zero())),
                Self::BigInt(v) => Self::BigInt(v.clone()),
                Self::Int(v) => Self::BigInt((*v).into()),
            }
        */
        } else {
            panic!()
        })
    }

    pub fn is_nan(&self) -> bool {
        match self {
            Self::Number(f) => f.is_nan(),
            Self::Float(f) => f.is_nan(),
            _ => false,
        }
    }

    pub fn is_negative_infinity(&self) -> bool {
        match self {
            Self::Number(f) => f == &f64::NEG_INFINITY,
            Self::Float(f) => f == &f32::NEG_INFINITY,
            _ => false,
        }
    }

    pub fn is_positive_infinity(&self) -> bool {
        match self {
            Self::Number(f) => f == &f64::INFINITY,
            Self::Float(f) => f == &f32::INFINITY,
            _ => false,
        }
    }

    pub fn as_double(&self) -> Option<f64> {
        if let NumberVariant::Number(v) = self { Some(*v) } else { None }
    }

    pub fn as_float(&self) -> Option<f32> {
        if let NumberVariant::Float(v) = self { Some(*v) } else { None }
    }

    pub fn as_int(&self) -> Option<i32> {
        if let NumberVariant::Int(v) = self { Some(*v) } else { None }
    }

    pub fn as_uint(&self) -> Option<u32> {
        if let NumberVariant::Uint(v) = self { Some(*v) } else { None }
    }
}