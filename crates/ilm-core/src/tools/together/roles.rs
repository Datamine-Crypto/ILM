use crate::base::exact::Amount;
use patterns::{because, source};
use std::cmp::Ordering;

pub struct NumberRoles;
source!(NumberRoles, "each number of a problem has one role in the calculation, stated before the lessons: added, taken away, a factor, a divisor, added after the factors, taken away after the factors, or not used; the value of a problem is the sum of its added and taken numbers (one when it has none), times its factors, over its divisors, then its late numbers; one writing for one calculation: a sum has two numbers or more or stands alone, a product with no sum before it has two numbers or more and one factor, and late numbers come after a product");

const ROLE_SIGNS: &[&str] = &["+", "-", "*", "/"];
because!(ROLE_SIGNS, NumberRoles, "the sign with which each role of a sum or a product is written in a calculation");

const ROLE_RANKS: &[u8] = &[1, 3, 0, 5, 2, 4, 6];
because!(ROLE_RANKS, NumberRoles, "the place of each role in the order of the short texts that name the roles (a factor, added, added late, taken, taken late, a divisor, not used), by which two ways of roles are ordered");

pub const ADDED: u8 = 0;
because!(ADDED, NumberRoles, "the first role of the fixed order: a number of the sum before the factors, with a plus");

pub const TAKEN: u8 = 1;
because!(TAKEN, NumberRoles, "the role of a number that is taken away");

pub const FACTOR: u8 = 2;
because!(FACTOR, NumberRoles, "the role of a number that the value is multiplied by");

pub const DIVISOR: u8 = 3;
because!(DIVISOR, NumberRoles, "the role of a number that the value is divided by");

pub const LATE_ADDED: u8 = 4;
because!(LATE_ADDED, NumberRoles, "the role of a number that is added after the factors and the divisors");

pub const LATE_TAKEN: u8 = 5;
because!(LATE_TAKEN, NumberRoles, "the role of a number that is taken away after the factors and the divisors");

pub const UNUSED: u8 = 6;
because!(UNUSED, NumberRoles, "the role of a number that the calculation does not use");

pub const MOST_NUMBERS: usize = 5;
because!(MOST_NUMBERS, NumberRoles, "a problem with more numbers is not read by the roles: the ways of roles grow sevenfold with each number, and the problems of the public sets that the roles read have fewer");

pub fn role_order(one: &[u8], other: &[u8]) -> Ordering {
    let ranks = |roles: &[u8]| roles.iter().map(|role| ROLE_RANKS[*role as usize]).collect::<Vec<u8>>();
    ranks(one).cmp(&ranks(other))
}
because!(role_order, NumberRoles, "the order of two ways of roles by the texts that name their roles, role by role, so that a choice among equal votes is the same in every run");

pub fn role_shapes(count: usize) -> Vec<Vec<u8>> {
    let kinds = ROLE_RANKS.len();
    let mut out: Vec<Vec<u8>> = Vec::new();
    let mut roles = vec![ADDED; count];
    loop {
        let has = |role: u8| roles.iter().filter(|held| **held == role).count();
        let early = has(ADDED) + has(TAKEN);
        let scale = has(FACTOR) + has(DIVISOR);
        let late = has(LATE_ADDED) + has(LATE_TAKEN);
        let fits = !(early > 0 && has(ADDED) == 0) && !(early == 1 && (scale > 0 || late > 0)) && !(early == 0 && (scale <= 1 || has(FACTOR) == 0)) && !(late > 0 && scale == 0);
        if fits {
            out.push(roles.clone());
        }
        let mut place = count;
        loop {
            if place == 0 {
                return out;
            }
            place -= 1;
            if (roles[place] as usize) + 1 < kinds {
                roles[place] += 1;
                break;
            }
            roles[place] = ADDED;
        }
    }
}
because!(role_shapes, NumberRoles, "every way to give a count of numbers their roles in the one writing of each calculation, in the order of the product of the roles, the last number changing fastest");

pub fn role_value(roles: &[u8], amounts: &[Amount]) -> Option<Amount> {
    let mut value: Option<Amount> = None;
    for (role, amount) in roles.iter().zip(amounts) {
        if *role == ADDED {
            value = Some(match value {
                None => *amount,
                Some(held) => held.plus(*amount)?,
            });
        }
    }
    for (role, amount) in roles.iter().zip(amounts) {
        if *role == TAKEN {
            value = Some(value?.minus(*amount)?);
        }
    }
    let mut value = value.unwrap_or(Amount::whole(1));
    for (role, amount) in roles.iter().zip(amounts) {
        if *role == FACTOR {
            value = value.times(*amount)?;
        }
    }
    for (role, amount) in roles.iter().zip(amounts) {
        if *role == DIVISOR {
            value = value.over(*amount)?;
        }
    }
    for (role, amount) in roles.iter().zip(amounts) {
        if *role == LATE_ADDED {
            value = value.plus(*amount)?;
        }
    }
    for (role, amount) in roles.iter().zip(amounts) {
        if *role == LATE_TAKEN {
            value = value.minus(*amount)?;
        }
    }
    Some(value)
}
because!(role_value, NumberRoles, "the exact value of one way of roles for the amounts of a problem, each step a step of the exact amounts, and none when it divides by nought");

pub fn role_ways(shapes: &[Vec<u8>], amounts: &[Amount], truth: Amount) -> Vec<Vec<u8>> {
    shapes.iter().filter(|roles| role_value(roles, amounts) == Some(truth)).cloned().collect()
}
because!(role_ways, NumberRoles, "the ways of roles whose exact value is the true number, in the order of the ways");

pub fn written_way(roles: &[u8], texts: &[String]) -> String {
    let pick = |wanted: &[u8]| -> Vec<(u8, &str)> {
        let mut out: Vec<(u8, &str)> = Vec::new();
        for role in wanted {
            for (held, text) in roles.iter().zip(texts) {
                if held == role {
                    out.push((*role, text.as_str()));
                }
            }
        }
        out
    };
    let chain = |items: &[(u8, &str)], bare: bool| -> String {
        items.iter().enumerate().map(|(at, (role, text))| if at == 0 && bare { (*text).to_string() } else { format!(" {} {text}", ROLE_SIGNS[*role as usize]) }).collect()
    };
    let early = pick(&[ADDED, TAKEN]);
    let scale = pick(&[FACTOR, DIVISOR]);
    let mut out = chain(&early, true);
    if early.len() > 1 && !scale.is_empty() {
        out = format!("({out})");
    }
    out += &chain(&scale, early.is_empty());
    for (held, text) in roles.iter().zip(texts) {
        let sign = match *held {
            LATE_ADDED => ADDED,
            LATE_TAKEN => TAKEN,
            _ => continue,
        };
        out += &format!(" {} {text}", ROLE_SIGNS[sign as usize]);
    }
    out
}
because!(written_way, NumberRoles, "a way of roles as a calculation with the numbers of the problem: the added numbers, then the taken ones, then the factors and the divisors, then the late numbers in their order, with brackets round a sum of more than one number before a product");
