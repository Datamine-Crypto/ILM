use crate::base::exact::common_divisor;
use patterns::{because, source};
use std::cmp::Ordering;

pub struct LongWholes;
source!(LongWholes, "whole numbers of any length as rows of machine words, with the long multiplication and the long division of Knuth (The Art of Computer Programming, volume two, Algorithm D), so that a sum of shares is exact however many terms it has");

pub struct ExactShares;
source!(ExactShares, "the shares of a vote are fractions of whole numbers of any length and never rounded, so two shares are equal when they are equal and a vote has the same winner in every run; a sum of many small fractions keeps the least common multiple of their denominators, so it stays as short as the sum needs; the float of a share is the nearest double, ties to the even one, as an exact division gives it");

const LIMB_BITS: u32 = 64;
because!(LIMB_BITS, LongWholes, "the bits of one machine word of a long whole number, the widest word whose product the machine holds in a word of twice the width");

const DOUBLE_BITS: u64 = 53;
because!(DOUBLE_BITS, ExactShares, "the bits of the mantissa of a double, with its hidden bit");

const QUOTIENT_BITS: u64 = 55;
because!(QUOTIENT_BITS, ExactShares, "the bits of the quotient from which a double is rounded: more than the mantissa, so the bit after the last one and a bit below it decide the rounding together with the rest");

const DOUBLE_BASE: f64 = 2.0;
because!(DOUBLE_BASE, ExactShares, "the base of the exponent of a double: a mantissa times a whole power of it is exact while the power stays within the range of the doubles, and the shares of a vote lie far inside it");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LongWhole {
    limbs: Vec<u64>,
}
because!(LongWhole, LongWholes, "a whole number not below nought as machine words, the lowest word first, with no word of nought at the top. of makes it from one word; plus, minus (of a smaller number), times and divided (quotient and rest) are the steps of school done word by word; the small forms take one machine word as the other number; common is the greatest common divisor of Euclid; bits is the length in bits, nought for nought; shifted multiplies by a power of two; low_word gives the lowest word");

impl Ord for LongWhole {
    fn cmp(&self, other: &LongWhole) -> Ordering {
        self.limbs.len().cmp(&other.limbs.len()).then_with(|| self.limbs.iter().rev().cmp(other.limbs.iter().rev()))
    }
}

impl PartialOrd for LongWhole {
    fn partial_cmp(&self, other: &LongWhole) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl LongWhole {
    pub fn of(value: u64) -> LongWhole {
        let mut made = LongWhole { limbs: vec![value] };
        made.trim();
        made
    }

    fn trim(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }

    pub fn bits(&self) -> u64 {
        self.limbs.last().map_or(0, |top| (self.limbs.len() as u64 - 1) * u64::from(LIMB_BITS) + u64::from(LIMB_BITS - top.leading_zeros()))
    }

    pub fn low_word(&self) -> u64 {
        self.limbs.first().copied().unwrap_or_default()
    }

    pub fn plus(&self, other: &LongWhole) -> LongWhole {
        let (long, short) = if self.limbs.len() >= other.limbs.len() { (self, other) } else { (other, self) };
        let mut out: Vec<u64> = Vec::with_capacity(long.limbs.len() + 1);
        let mut carry = 0u64;
        for (at, limb) in long.limbs.iter().enumerate() {
            let (sum, over) = limb.overflowing_add(short.limbs.get(at).copied().unwrap_or_default());
            let (sum, again) = sum.overflowing_add(carry);
            out.push(sum);
            carry = u64::from(over) + u64::from(again);
        }
        if carry > 0 {
            out.push(carry);
        }
        LongWhole { limbs: out }
    }

    pub fn minus(&self, other: &LongWhole) -> LongWhole {
        let mut out: Vec<u64> = Vec::with_capacity(self.limbs.len());
        let mut borrow = 0u64;
        for (at, limb) in self.limbs.iter().enumerate() {
            let (left, under) = limb.overflowing_sub(other.limbs.get(at).copied().unwrap_or_default());
            let (left, again) = left.overflowing_sub(borrow);
            out.push(left);
            borrow = u64::from(under) + u64::from(again);
        }
        let mut made = LongWhole { limbs: out };
        made.trim();
        made
    }

    pub fn times(&self, other: &LongWhole) -> LongWhole {
        if self.limbs.is_empty() || other.limbs.is_empty() {
            return LongWhole::default();
        }
        let mut out = vec![0u64; self.limbs.len() + other.limbs.len()];
        for (at, limb) in self.limbs.iter().enumerate() {
            let mut carry: u128 = 0;
            for (step, factor) in other.limbs.iter().enumerate() {
                let now = u128::from(*limb) * u128::from(*factor) + u128::from(out[at + step]) + carry;
                out[at + step] = now as u64;
                carry = now >> LIMB_BITS;
            }
            out[at + other.limbs.len()] = carry as u64;
        }
        let mut made = LongWhole { limbs: out };
        made.trim();
        made
    }

    pub fn times_small(&self, factor: u64) -> LongWhole {
        self.times(&LongWhole::of(factor))
    }

    pub fn divided_small(&self, divisor: u64) -> (LongWhole, u64) {
        let mut out = vec![0u64; self.limbs.len()];
        let mut rest: u128 = 0;
        for at in (0..self.limbs.len()).rev() {
            let now = (rest << LIMB_BITS) | u128::from(self.limbs[at]);
            out[at] = (now / u128::from(divisor)) as u64;
            rest = now % u128::from(divisor);
        }
        let mut made = LongWhole { limbs: out };
        made.trim();
        (made, rest as u64)
    }

    fn shifted_limbs(&self, shift: u32, extra: bool) -> Vec<u64> {
        let mut out: Vec<u64> = Vec::with_capacity(self.limbs.len() + 1);
        let mut carry = 0u64;
        for limb in &self.limbs {
            out.push(if shift == 0 { *limb } else { (limb << shift) | carry });
            carry = if shift == 0 { 0 } else { limb >> (LIMB_BITS - shift) };
        }
        if extra {
            out.push(carry);
        }
        out
    }

    pub fn divided(&self, divisor: &LongWhole) -> (LongWhole, LongWhole) {
        if divisor.limbs.len() == 1 {
            let (quotient, rest) = self.divided_small(divisor.limbs[0]);
            return (quotient, LongWhole::of(rest));
        }
        if self < divisor {
            return (LongWhole::default(), self.clone());
        }
        let shift = divisor.limbs[divisor.limbs.len() - 1].leading_zeros();
        let lower = divisor.shifted_limbs(shift, false);
        let mut upper = self.shifted_limbs(shift, true);
        let size = lower.len();
        let steps = upper.len() - size;
        let mut quotient = vec![0u64; steps];
        let top = u128::from(lower[size - 1]);
        let next = u128::from(lower[size - 1 - 1]);
        for at in (0..steps).rev() {
            let head = (u128::from(upper[at + size]) << LIMB_BITS) | u128::from(upper[at + size - 1]);
            let mut guess = head / top;
            let mut rest = head % top;
            while guess >> LIMB_BITS != 0 || guess * next > ((rest << LIMB_BITS) | u128::from(upper[at + size - 1 - 1])) {
                guess -= 1;
                rest += top;
                if rest >> LIMB_BITS != 0 {
                    break;
                }
            }
            let mut borrow = 0u64;
            let mut carry = 0u64;
            for place in 0..size {
                let product = guess * u128::from(lower[place]) + u128::from(carry);
                carry = (product >> LIMB_BITS) as u64;
                let (left, under) = upper[at + place].overflowing_sub(product as u64);
                let (left, again) = left.overflowing_sub(borrow);
                upper[at + place] = left;
                borrow = u64::from(under) + u64::from(again);
            }
            let (left, under) = upper[at + size].overflowing_sub(carry);
            let (left, again) = left.overflowing_sub(borrow);
            upper[at + size] = left;
            if under || again {
                guess -= 1;
                let mut carry = 0u64;
                for place in 0..size {
                    let (sum, over) = upper[at + place].overflowing_add(lower[place]);
                    let (sum, more) = sum.overflowing_add(carry);
                    upper[at + place] = sum;
                    carry = u64::from(over) + u64::from(more);
                }
                upper[at + size] = upper[at + size].wrapping_add(carry);
            }
            quotient[at] = guess as u64;
        }
        let mut rest: Vec<u64> = Vec::with_capacity(size);
        for place in 0..size {
            let high = if shift == 0 { 0 } else { upper[place + 1] << (LIMB_BITS - shift) };
            rest.push((upper[place] >> shift) | high);
        }
        let mut quotient = LongWhole { limbs: quotient };
        quotient.trim();
        let mut rest = LongWhole { limbs: rest };
        rest.trim();
        (quotient, rest)
    }

    pub fn common(&self, other: &LongWhole) -> LongWhole {
        let (mut left, mut right) = (self.clone(), other.clone());
        while !right.limbs.is_empty() {
            let rest = left.divided(&right).1;
            left = right;
            right = rest;
        }
        left
    }

    pub fn shifted(&self, bits: u64) -> LongWhole {
        let mut limbs = vec![0u64; (bits / u64::from(LIMB_BITS)) as usize];
        limbs.extend(self.shifted_limbs((bits % u64::from(LIMB_BITS)) as u32, true));
        let mut made = LongWhole { limbs };
        made.trim();
        made
    }
}

#[derive(Clone, Debug)]
pub struct Share {
    top: LongWhole,
    bottom: LongWhole,
}
because!(Share, ExactShares, "a fraction not below nought: a long whole number over a long whole number above nought, not always in lowest terms. whole and part make one; plus is the exact sum, complement the rest of a share not above one to one, times and over exact products; add_part adds a small fraction and keeps the least common multiple of the denominators; over_small divides by a count; is_zero says whether it is nought; order compares the values of two shares; float gives the nearest double");

impl Default for Share {
    fn default() -> Share {
        Share::whole(0)
    }
}

impl Share {
    pub fn whole(count: u64) -> Share {
        Share { top: LongWhole::of(count), bottom: LongWhole::of(1) }
    }

    pub fn part(top: u64, bottom: u64) -> Share {
        Share { top: LongWhole::of(top), bottom: LongWhole::of(bottom) }
    }

    pub fn is_zero(&self) -> bool {
        self.top.bits() == 0
    }

    pub fn order(&self, other: &Share) -> Ordering {
        self.top.times(&other.bottom).cmp(&other.top.times(&self.bottom))
    }

    pub fn plus(&self, other: &Share) -> Share {
        if self.bottom == other.bottom {
            return Share { top: self.top.plus(&other.top), bottom: self.bottom.clone() };
        }
        Share { top: self.top.times(&other.bottom).plus(&other.top.times(&self.bottom)), bottom: self.bottom.times(&other.bottom) }
    }

    pub fn complement(&self) -> Share {
        Share { top: self.bottom.minus(&self.top), bottom: self.bottom.clone() }
    }

    pub fn times(&self, other: &Share) -> Share {
        Share { top: self.top.times(&other.top), bottom: self.bottom.times(&other.bottom) }
    }

    fn turned(&self) -> Share {
        Share { top: self.bottom.clone(), bottom: self.top.clone() }
    }

    pub fn over(&self, other: &Share) -> Share {
        self.times(&other.turned())
    }

    pub fn over_small(&self, count: u64) -> Share {
        Share { top: self.top.clone(), bottom: self.bottom.times_small(count) }
    }

    pub fn add_part(&mut self, top: u64, bottom: u64) {
        let rest = self.bottom.divided_small(bottom).1;
        let shared = common_divisor(u128::from(rest), u128::from(bottom)) as u64;
        let widen = bottom / shared;
        let (part_scale, _) = self.bottom.divided_small(shared);
        self.top = self.top.times_small(widen).plus(&part_scale.times_small(top));
        self.bottom = self.bottom.times_small(widen);
    }

    pub fn float(&self) -> f64 {
        if self.top.bits() == 0 {
            return f64::default();
        }
        let gap = self.top.bits() as i64 - self.bottom.bits() as i64;
        let scale = QUOTIENT_BITS as i64 - gap;
        let (top, bottom) = if scale >= 0 { (self.top.shifted(scale as u64), self.bottom.clone()) } else { (self.top.clone(), self.bottom.shifted((-scale) as u64)) };
        let (quotient, rest) = top.divided(&bottom);
        let extra = quotient.bits() - DOUBLE_BITS;
        let low = quotient.low_word();
        let mut mantissa = low >> extra;
        let dropped = low & ((1u64 << extra) - 1);
        let half = 1u64 << (extra - 1);
        if dropped > half || (dropped == half && (rest.bits() > 0 || mantissa & 1 == 1)) {
            mantissa += 1;
        }
        (mantissa as f64) * DOUBLE_BASE.powi((extra as i64 - scale) as i32)
    }
}
