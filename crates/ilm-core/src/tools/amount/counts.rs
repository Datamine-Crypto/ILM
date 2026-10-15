use crate::base::keys::{KeyNumbers, fields_of, packed};
use crate::tools::amount::calcs::AMOUNT_PAIR;
use crate::tools::amount::forms::amount_examples;
use crate::tools::amount::learn::AmountLesson;
use crate::tools::amount::shared::{AmountBase, AmountHand};
use patterns::{because, source};
use std::ops::Range;

pub struct AmountCounting;
source!(AmountCounting, "counting over the lessons which calculation a form means: for each form the lessons where it stands, and for each calculation of a form the lessons where its result is the true number of the answer. Counts add, so the lessons are counted in runs side by side and the counts of the runs are summed");

pub(super) const AMOUNT_PARTS: usize = 2;
because!(AMOUNT_PARTS, AmountCounting, "the parts of the lessons that are counted side by side: the first part and the part that was held back");

pub(super) const AMOUNT_REACH: isize = 2;
because!(AMOUNT_REACH, AmountCounting, "how far from an operand a word of the answer is looked for: the unit of an amount stands next to its number or one token further");

pub(super) const AMOUNT_NEAR_FLAG: u32 = 0x8000_0000;
because!(AMOUNT_NEAR_FLAG, AmountCounting, "the high bit of the number that says where a word of the answer comes from: with it the number is a place near an operand, without it the number of a fixed word");

pub(super) fn amount_bump(counts: &mut Vec<u32>, at: usize, by: u32) {
    if counts.len() <= at {
        counts.resize(at + 1, 0);
    }
    counts[at] += by;
}
because!(amount_bump, AmountCounting, "more readings for one place of a flat array of counts, which grows to the place when a new key got its number");

pub(super) fn amount_count(counts: &[u32], at: usize) -> u32 {
    counts.get(at).copied().unwrap_or(0)
}
because!(amount_count, AmountCounting, "the readings of one place of a flat array of counts, and none for a place that the array never grew to");

pub(super) fn amount_near() -> impl Iterator<Item = isize> {
    (-AMOUNT_REACH..=AMOUNT_REACH).filter(|step| *step != 0)
}
because!(amount_near, AmountCounting, "the places next to an operand where a word of the answer is looked for, as steps from the operand, in rising order");

#[derive(Debug, Default)]
pub(super) struct AmountCounts {
    pub(super) forms: KeyNumbers,
    pub(super) stands: [Vec<u32>; AMOUNT_PARTS],
    pub(super) rules: KeyNumbers,
    pub(super) right: [Vec<u32>; AMOUNT_PARTS],
    pub(super) outside: Vec<u32>,
    pub(super) shape: Vec<u32>,
    pub(super) shape_count: Vec<u32>,
    pub(super) shape_first: Vec<u32>,
    pub(super) others: KeyNumbers,
    pub(super) other_count: Vec<u32>,
    pub(super) other_first: Vec<u32>,
    pub(super) sources: KeyNumbers,
    pub(super) source_count: Vec<u32>,
}
because!(AmountCounts, AmountCounting, "the counts over the lessons: for each form the lessons of each part where it stands; for each form with a calculation the lessons of each part where the result is the true number, and those where the true number also stands in the text outside the operands; for each such rule the forms of the answer with their lessons and their first lesson, where the form of the first right lesson of a rule is kept in the place of the rule, because nearly every rule has one form of answer and then needs no key of its own for it; for each word of such an answer where it comes from. hit counts one right lesson of a rule; absorb adds the counts of a later run of lessons, so runs that were counted side by side give the counts of all lessons");

impl AmountCounts {
    pub(super) fn shaped(&mut self, rule: usize, shape: u32, count: u32, first: u32) {
        if rule == self.shape.len() {
            self.shape.push(shape);
            self.shape_count.push(count);
            self.shape_first.push(first);
            return;
        }
        if self.shape[rule] == shape {
            self.shape_count[rule] += count;
            self.shape_first[rule] = self.shape_first[rule].min(first);
            return;
        }
        let other = self.others.number(packed(&[rule as u32, shape])) as usize;
        if other == self.other_first.len() {
            self.other_first.push(first);
            self.other_count.push(0);
        }
        self.other_count[other] += count;
        self.other_first[other] = self.other_first[other].min(first);
    }

    pub(super) fn hit(&mut self, form: u32, calc: u32, part: usize, outside: bool, shape: u32, lesson: u32, wheres: &[(u32, u32)]) {
        let rule = self.rules.number(packed(&[form, calc]));
        amount_bump(&mut self.right[part], rule as usize, 1);
        amount_bump(&mut self.outside, rule as usize, u32::from(outside));
        self.shaped(rule as usize, shape, 1, lesson);
        for (place, from) in wheres {
            let held = self.sources.number(packed(&[rule, shape, *place, *from]));
            amount_bump(&mut self.source_count, held as usize, 1);
        }
    }

    pub(super) fn absorb(&mut self, other: &AmountCounts) {
        let forms: Vec<u32> = other.forms.keys().iter().map(|key| self.forms.number(*key)).collect();
        let mut rules: Vec<u32> = Vec::with_capacity(other.rules.keys().len());
        for (local, key) in other.rules.keys().iter().enumerate() {
            let [form, calc, ..] = fields_of(*key);
            let rule = self.rules.number(packed(&[forms[form as usize], calc]));
            self.shaped(rule as usize, other.shape[local], other.shape_count[local], other.shape_first[local]);
            rules.push(rule);
        }
        for part in 0..AMOUNT_PARTS {
            for (local, count) in other.stands[part].iter().enumerate() {
                amount_bump(&mut self.stands[part], forms[local] as usize, *count);
            }
            for (local, count) in other.right[part].iter().enumerate() {
                amount_bump(&mut self.right[part], rules[local] as usize, *count);
            }
        }
        for (local, count) in other.outside.iter().enumerate() {
            amount_bump(&mut self.outside, rules[local] as usize, *count);
        }
        for (local, key) in other.others.keys().iter().enumerate() {
            let [rule, shape, ..] = fields_of(*key);
            self.shaped(rules[rule as usize] as usize, shape, other.other_count[local], other.other_first[local]);
        }
        for (local, key) in other.sources.keys().iter().enumerate() {
            let [rule, shape, place, from] = fields_of(*key);
            let held = self.sources.number(packed(&[rules[rule as usize], shape, place, from]));
            amount_bump(&mut self.source_count, held as usize, other.source_count[local]);
        }
    }
}

pub(super) type AmountRight = Option<(u128, bool, usize, usize)>;
because!(AmountRight, AmountCounting, "what a lesson says about one place of operands, found once for all forms of the place: the calculations whose result is the true number as bits in the order of the fixed set, whether the true number also stands outside the operands, and the run of the entries that say where the words of the answer come from");

impl AmountBase {
    pub(super) fn right(&self, lesson: &AmountLesson, window: (u32, u32), wheres: &mut Vec<(u32, u32)>) -> (u128, bool, usize, usize) {
        let nothing = (0, false, 0, 0);
        let Some(truth) = lesson.truth else { return nothing };
        let (from, size) = (window.0 as usize, window.1 as usize);
        let given = &lesson.cut.values[from..from + size];
        let tried = if size == AMOUNT_PAIR { 0..self.pairs } else { self.pairs..self.calcs.len() };
        let mut mask: u128 = 0;
        for calc in tried {
            if self.calcs[calc].run(given) == Some(truth) {
                mask |= 1 << calc;
            }
        }
        if mask == 0 {
            return nothing;
        }
        let start = wheres.len();
        let near = amount_near().count();
        for (place, word) in lesson.said.iter().enumerate() {
            if *word == self.marks.number {
                continue;
            }
            wheres.push((place as u32, *word));
            for operand in 0..size {
                for (index, step) in amount_near().enumerate() {
                    let at = lesson.cut.spots[from + operand] as isize + step;
                    if at >= 0 && lesson.cut.words.get(at as usize) == Some(word) {
                        wheres.push((place as u32, AMOUNT_NEAR_FLAG + (operand * near + index) as u32));
                    }
                }
            }
        }
        (mask, lesson.cut.stands(truth, from, size), start, wheres.len())
    }

    pub(super) fn count_lesson(&self, lesson: &AmountLesson, place: u32, counts: &mut AmountCounts, hand: &mut AmountHand) {
        let part = usize::from(lesson.late);
        hand.forms.fill(&lesson.cut, &self.marks);
        hand.right.clear();
        hand.right.resize(hand.forms.windows.len(), None);
        hand.wheres.clear();
        for number in 0..hand.forms.seen.keys().len() {
            let window = hand.forms.owner[number] as usize;
            let form = counts.forms.number(hand.forms.seen.key(number as u32));
            amount_bump(&mut counts.stands[part], form as usize, 1);
            let (mask, outside, from, to) = match hand.right[window] {
                Some(found) => found,
                None => {
                    let found = self.right(lesson, hand.forms.windows[window], &mut hand.wheres);
                    hand.right[window] = Some(found);
                    found
                }
            };
            let mut left = mask;
            while left != 0 {
                counts.hit(form, left.trailing_zeros(), part, outside, lesson.shape, place, &hand.wheres[from..to]);
                left &= left - 1;
            }
        }
        amount_examples(&lesson.cut, &self.marks, &self.calcs[..self.pairs], &mut hand.shown);
        if hand.shown.is_empty() {
            return;
        }
        hand.fixed.clear();
        hand.fixed.extend(lesson.said.iter().enumerate().filter(|(_, word)| **word != self.marks.number).map(|(at, word)| (at as u32, *word)));
        for (calc, got) in &hand.shown {
            let form = counts.forms.number(packed(&[self.marks.examples, self.marks.names[*calc]]));
            amount_bump(&mut counts.stands[part], form as usize, 1);
            if Some(*got) == lesson.truth {
                counts.hit(form, *calc as u32, part, lesson.cut.stands(*got, 0, 0), lesson.shape, place, &hand.fixed);
            }
        }
    }

    pub(super) fn count(&self, lessons: &[AmountLesson], range: Range<usize>) -> AmountCounts {
        let mut counts = AmountCounts::default();
        let mut hand = AmountHand::default();
        for place in range {
            if lessons[place].counted {
                self.count_lesson(&lessons[place], place as u32, &mut counts, &mut hand);
            }
        }
        counts
    }
}
