use patterns::{because, source};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ExactAmounts;
source!(ExactAmounts, "arithmetic on fractions of whole numbers in lowest terms, as Knuth gives it for rational numbers in The Art of Computer Programming: the common divisors are taken out before each product, so the parts of a result stay small; an amount of a text is never rounded, so a result is the number of an answer or it is not");

pub struct BinaryDivisor;
source!(BinaryDivisor, "the greatest common divisor by the binary way of Stein: shifts and subtractions in place of divisions, which is the faster way for numbers that fit one machine word");

pub struct WholeRange;
source!(WholeRange, "whole numbers of a fixed width: a step whose result leaves the width gives no result and is counted, so a run can say that no amount of its texts needed more room");

static BEYOND_RANGE: AtomicU64 = AtomicU64::new(0);
because!(BEYOND_RANGE, WholeRange, "the steps of this run of the program whose result left the width of the whole numbers");

const RADIX: u32 = 10;
because!(RADIX, ExactAmounts, "the base in which a text writes its numbers");

const NEGATIVE: char = '-';
because!(NEGATIVE, ExactAmounts, "the sign that a text writes before an amount below nothing");

const POINT: char = '.';
because!(POINT, ExactAmounts, "the mark between the whole part of a written amount and its part below one");

pub fn steps_beyond_range() -> u64 {
    BEYOND_RANGE.load(Ordering::Relaxed)
}
because!(steps_beyond_range, WholeRange, "how many steps of this run left the width of the whole numbers; a run that reports none lost no result to the width");

fn in_range<T>(found: Option<T>) -> Option<T> {
    if found.is_none() {
        BEYOND_RANGE.fetch_add(1, Ordering::Relaxed);
    }
    found
}
because!(in_range, WholeRange, "the result of a checked step as it is, and one more counted step when it left the width");

fn short_divisor(mut left: u64, mut right: u64) -> u64 {
    if left == 0 {
        return right;
    }
    if right == 0 {
        return left;
    }
    let shared = (left | right).trailing_zeros();
    left >>= left.trailing_zeros();
    loop {
        right >>= right.trailing_zeros();
        if left > right {
            std::mem::swap(&mut left, &mut right);
        }
        right -= left;
        if right == 0 {
            return left << shared;
        }
    }
}
because!(short_divisor, BinaryDivisor, "the greatest common divisor of two numbers of one machine word");

fn long_divisor(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}
because!(long_divisor, ExactAmounts, "the greatest common divisor of two wide numbers by the remainders of Euclid, for the rare amount that does not fit one machine word");

pub(crate) fn common_divisor(left: u128, right: u128) -> u128 {
    match (u64::try_from(left), u64::try_from(right)) {
        (Ok(left), Ok(right)) => u128::from(short_divisor(left, right)),
        _ => long_divisor(left, right),
    }
}
because!(common_divisor, ExactAmounts, BinaryDivisor, "the greatest common divisor of two parts of amounts, by the fast way when both fit one machine word");

fn digits_value(start: i128, digits: &str) -> Option<i128> {
    let mut value = start;
    for digit in digits.chars() {
        value = in_range(value.checked_mul(i128::from(RADIX)))?;
        value = in_range(value.checked_add(i128::from(digit.to_digit(RADIX)?)))?;
    }
    Some(value)
}
because!(digits_value, ExactAmounts, WholeRange, "the whole number that a run of digits writes, after the digits that came before it");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Amount {
    top: i128,
    bottom: i128,
}
because!(Amount, ExactAmounts, WholeRange, "an exact amount: a whole number over a whole number above nothing, with no common divisor, so equal amounts have equal parts. whole makes the amount of a count; from_digits reads the digits of a text before and after its point; plus, minus, times and over are the four steps, each with no result when it leaves the width, and over with no result when it divides by nothing; a step on two whole amounts takes no divisor; written gives the digits of a whole amount, or the decimal that ends within the given places, and nothing for an amount whose decimal does not end there; two amounts compare by their values, by the whole parts first and then by the parts below one turned over, as Euclid takes the remainders, so no product can leave the width; written_over gives the whole number over the whole number, with a mark between them, for an amount whose decimal does not end; whole_part gives the largest whole number that is not above the amount, so that it rounds down also below nothing (the whole part of minus two and a half is minus three)");

impl Ord for Amount {
    fn cmp(&self, other: &Amount) -> std::cmp::Ordering {
        let (mut top, mut bottom, mut other_top, mut other_bottom) = (self.top, self.bottom, other.top, other.bottom);
        let mut turned = false;
        loop {
            let (whole, other_whole) = (top.div_euclid(bottom), other_top.div_euclid(other_bottom));
            let (rest, other_rest) = (top.rem_euclid(bottom), other_top.rem_euclid(other_bottom));
            let found = match (whole.cmp(&other_whole), rest == 0, other_rest == 0) {
                (std::cmp::Ordering::Equal, true, true) => std::cmp::Ordering::Equal,
                (std::cmp::Ordering::Equal, true, false) => std::cmp::Ordering::Less,
                (std::cmp::Ordering::Equal, false, true) => std::cmp::Ordering::Greater,
                (std::cmp::Ordering::Equal, false, false) => {
                    (top, bottom, other_top, other_bottom) = (bottom, rest, other_bottom, other_rest);
                    turned = !turned;
                    continue;
                }
                (order, _, _) => order,
            };
            return if turned { found.reverse() } else { found };
        }
    }
}

impl PartialOrd for Amount {
    fn partial_cmp(&self, other: &Amount) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Amount {
    pub const fn whole(count: i128) -> Amount {
        Amount { top: count, bottom: 1 }
    }

    pub fn is_whole(&self) -> bool {
        self.bottom == 1
    }

    fn lowest(top: i128, bottom: i128) -> Amount {
        let shared = common_divisor(top.unsigned_abs(), bottom.unsigned_abs()) as i128;
        Amount { top: top / shared, bottom: bottom / shared }
    }

    pub fn from_digits(before: &str, after: &str) -> Option<Amount> {
        let top = digits_value(digits_value(0, before)?, after)?;
        let mut bottom: i128 = 1;
        for _ in after.chars() {
            bottom = in_range(bottom.checked_mul(i128::from(RADIX)))?;
        }
        Some(Amount::lowest(top, bottom))
    }

    pub fn negated(self) -> Option<Amount> {
        Some(Amount { top: in_range(self.top.checked_neg())?, bottom: self.bottom })
    }

    pub fn plus(self, other: Amount) -> Option<Amount> {
        if self.bottom == 1 && other.bottom == 1 {
            return in_range(self.top.checked_add(other.top)).map(Amount::whole);
        }
        let shared = common_divisor(self.bottom.unsigned_abs(), other.bottom.unsigned_abs()) as i128;
        let mine = self.bottom / shared;
        let theirs = other.bottom / shared;
        let top = in_range(self.top.checked_mul(theirs).zip(other.top.checked_mul(mine)).and_then(|(left, right)| left.checked_add(right)))?;
        if top == 0 {
            return Some(Amount::whole(0));
        }
        let rest = common_divisor(top.unsigned_abs(), shared.unsigned_abs()) as i128;
        let bottom = in_range(mine.checked_mul(other.bottom / rest))?;
        Some(Amount { top: top / rest, bottom })
    }

    pub fn minus(self, other: Amount) -> Option<Amount> {
        self.plus(other.negated()?)
    }

    pub fn times(self, other: Amount) -> Option<Amount> {
        if self.bottom == 1 && other.bottom == 1 {
            return in_range(self.top.checked_mul(other.top)).map(Amount::whole);
        }
        if self.top == 0 || other.top == 0 {
            return Some(Amount::whole(0));
        }
        let left = common_divisor(self.top.unsigned_abs(), other.bottom.unsigned_abs()) as i128;
        let right = common_divisor(other.top.unsigned_abs(), self.bottom.unsigned_abs()) as i128;
        let top = in_range((self.top / left).checked_mul(other.top / right))?;
        let bottom = in_range((self.bottom / right).checked_mul(other.bottom / left))?;
        Some(Amount { top, bottom })
    }

    pub fn over(self, other: Amount) -> Option<Amount> {
        if other.top == 0 {
            return None;
        }
        if self.bottom == 1 && other.bottom == 1 && self.top.checked_rem(other.top) == Some(0) {
            return in_range(self.top.checked_div(other.top)).map(Amount::whole);
        }
        let bottom = in_range(other.top.checked_abs())?;
        let top = if other.top < 0 { -other.bottom } else { other.bottom };
        self.times(Amount { top, bottom })
    }

    pub fn whole_part(&self) -> Amount {
        Amount::whole(self.top.div_euclid(self.bottom))
    }

    pub fn written_over(&self, mark: char) -> String {
        format!("{}{mark}{}", self.top, self.bottom)
    }

    pub fn written(&self, most_places: usize) -> Option<String> {
        if self.bottom == 1 {
            return Some(self.top.to_string());
        }
        let nought = char::from_digit(0, RADIX)?;
        let mut scale: i128 = 1;
        for places in 1..=most_places {
            scale = in_range(scale.checked_mul(i128::from(RADIX)))?;
            if scale % self.bottom != 0 {
                continue;
            }
            let digits = in_range(self.top.unsigned_abs().checked_mul((scale / self.bottom).unsigned_abs()))?.to_string();
            let mut text = String::with_capacity(digits.len() + places);
            if self.top < 0 {
                text.push(NEGATIVE);
            }
            let whole = digits.len().saturating_sub(places);
            if whole == 0 {
                text.push(nought);
            }
            text.push_str(&digits[..whole]);
            text.push(POINT);
            text.extend(std::iter::repeat_n(nought, places.saturating_sub(digits.len())));
            text.push_str(&digits[whole..]);
            return Some(text);
        }
        None
    }
}
