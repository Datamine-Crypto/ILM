use crate::base::exact::Amount;
use crate::tools::amount::tokens::AMOUNT_NO_TOKEN;
use patterns::{because, source};

pub struct AmountCalculations;
source!(AmountCalculations, "a fixed small set of exact calculations that a text can mean for two or three of its numbers: every calculation of one step, and every calculation of two steps that uses each operand once. A learner chooses among them by counting and never makes a new one, so what it learned is a name of this set");

pub(super) const AMOUNT_PAIR: usize = 2;
because!(AMOUNT_PAIR, AmountCalculations, "the operands of a calculation of one step, and the answer words of a kind whose answers are calculated in steps of words");

pub(super) const AMOUNT_TRIPLE: usize = 3;
because!(AMOUNT_TRIPLE, AmountCalculations, "the operands of a calculation of two steps, and the items of one step in a written sum: a number, a token, a number");

pub(super) const AMOUNT_LETTERS: &str = "xyz";
because!(AMOUNT_LETTERS, AmountCalculations, "the names of the operands in the name of a calculation, in the order of the text");

pub(super) const AMOUNT_ORDERS: &[&str] = &["xyz", "xzy", "yxz", "yzx", "zxy", "zyx"];
because!(AMOUNT_ORDERS, AmountCalculations, "every order in which a calculation of two steps can take its operands. The order of the list is fixed: of two writings with the same results the first one stays, and its name is what a note shows");

pub(super) const AMOUNT_PROBES: &[[i128; AMOUNT_TRIPLE]] = &[[7, 11, 13], [17, 5, 3], [2, 29, 23]];
because!(AMOUNT_PROBES, AmountCalculations, "the operands on which two writings of a calculation are compared: different primes, so two calculations with the same results on all of them are one calculation");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AmountSign {
    Plus,
    Minus,
    Times,
    Over,
}
because!(AmountSign, AmountCalculations, "the four steps of arithmetic that a calculation is made of");

pub(super) const AMOUNT_SIGNS: &[(AmountSign, &str)] = &[(AmountSign::Plus, "+"), (AmountSign::Minus, "-"), (AmountSign::Times, "*"), (AmountSign::Over, "/")];
because!(AMOUNT_SIGNS, AmountCalculations, "the steps of arithmetic with the sign of each in the name of a calculation. The order of the list is fixed: it decides which of two writings of one calculation stays");

pub(super) const AMOUNT_PAIRS: &[(AmountSign, bool)] = &[(AmountSign::Plus, false), (AmountSign::Minus, false), (AmountSign::Minus, true), (AmountSign::Times, false), (AmountSign::Over, false), (AmountSign::Over, true)];
because!(AMOUNT_PAIRS, AmountCalculations, "the calculations of one step, each with the mark that it takes its operands in the other order; a sum and a product have one order, and the order of the list decides which calculation stays of two with the same share");

pub(super) fn amount_step(sign: AmountSign, left: Amount, right: Amount) -> Option<Amount> {
    match sign {
        AmountSign::Plus => left.plus(right),
        AmountSign::Minus => left.minus(right),
        AmountSign::Times => left.times(right),
        AmountSign::Over => left.over(right),
    }
}
because!(amount_step, AmountCalculations, "one step of arithmetic on two exact amounts, with no result for a division by nothing");

pub(super) fn amount_sign_text(sign: AmountSign) -> &'static str {
    AMOUNT_SIGNS.iter().find(|(held, _)| *held == sign).map_or(AMOUNT_NO_TOKEN, |(_, text)| *text)
}
because!(amount_sign_text, AmountCalculations, "the sign of a step of arithmetic in the name of a calculation");

#[derive(Clone, Copy, Debug)]
pub(super) struct AmountCalc {
    pub(super) first: AmountSign,
    pub(super) second: Option<AmountSign>,
    pub(super) places: [usize; AMOUNT_TRIPLE],
    pub(super) inner_left: bool,
}
because!(AmountCalc, AmountCalculations, "one calculation of the fixed set: a first step on two of the operands, and for a calculation of two steps a second step that takes the result of the first on its left or on its right. run gives the result for the operands of a place, and no result when a step has none or an operand left the width of the amounts");

impl AmountCalc {
    pub(super) fn run(&self, given: &[Option<Amount>]) -> Option<Amount> {
        let [first, second, third] = self.places;
        let inner = amount_step(self.first, given[first]?, given[second]?)?;
        match self.second {
            None => Some(inner),
            Some(sign) if self.inner_left => amount_step(sign, inner, given[third]?),
            Some(sign) => amount_step(sign, given[third]?, inner),
        }
    }
}

pub(super) fn amount_calcs() -> (Vec<AmountCalc>, Vec<String>) {
    let letter = |place: usize| &AMOUNT_LETTERS[place..=place];
    let mut calcs: Vec<AmountCalc> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for (sign, turned) in AMOUNT_PAIRS {
        let places = if *turned { [1, 0, 0] } else { [0, 1, 0] };
        let [first, second, _] = places;
        calcs.push(AmountCalc { first: *sign, second: None, places, inner_left: true });
        names.push(format!("{} {} {}", letter(first), amount_sign_text(*sign), letter(second)));
    }
    let probes: Vec<Vec<Option<Amount>>> = AMOUNT_PROBES.iter().map(|probe| probe.iter().map(|value| Some(Amount::whole(*value))).collect()).collect();
    let mut seen: Vec<Vec<Option<Amount>>> = Vec::new();
    for order in AMOUNT_ORDERS {
        let mut places = [0usize; AMOUNT_TRIPLE];
        for (slot, name) in order.chars().enumerate() {
            places[slot] = AMOUNT_LETTERS.find(name).unwrap_or(0);
        }
        let [first, second, third] = places.map(letter);
        for (inner, inner_text) in AMOUNT_SIGNS {
            for (outer, outer_text) in AMOUNT_SIGNS {
                for inner_left in [true, false] {
                    let calc = AmountCalc { first: *inner, second: Some(*outer), places, inner_left };
                    let mark: Vec<Option<Amount>> = probes.iter().map(|probe| calc.run(probe)).collect();
                    if seen.contains(&mark) {
                        continue;
                    }
                    seen.push(mark);
                    calcs.push(calc);
                    names.push(if inner_left { format!("({first} {inner_text} {second}) {outer_text} {third}") } else { format!("{third} {outer_text} ({first} {inner_text} {second})") });
                }
            }
        }
    }
    (calcs, names)
}
because!(amount_calcs, AmountCalculations, "the fixed set of calculations with their names: those of one step, then every calculation of two steps that uses each of its operands once, where two writings with the same results on the probe operands are kept once, the first one in the fixed order of the lists");
