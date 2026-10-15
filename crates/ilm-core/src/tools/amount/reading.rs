use crate::base::exact::Amount;
use crate::base::words::answer_words;
use crate::tools::amount::calcs::AMOUNT_PAIR;
use crate::tools::amount::counts::{AMOUNT_NEAR_FLAG, amount_near};
use crate::base::keys::{NO_NUMBER, fields_of, packed};
use crate::tools::amount::forms::amount_examples;
use crate::tools::amount::model::{AmountKind, AmountModel, AmountRule};
use crate::tools::amount::shared::{AMOUNT_HANDS, AmountHand};
use crate::tools::amount::steps::AmountWay;
use crate::tools::amount::tokens::{AMOUNT_BETWEEN, AmountCut, AmountUnknown, amount_cut_case};
use crate::tools::call::{Learner, Said};
use patterns::{because, source};

pub struct AmountAnswer;
source!(AmountAnswer, "the answer of a case from what was counted: the first sure rule that gives a number which can be written; with no such rule the form and calculation with the best share, when that share is above the share of the most frequent answer of the lessons; else the most frequent answer. The answer has the form of the answers of the lessons of its rule: the number alone, or with the words that stand with it");

pub(super) const AMOUNT_MOST_PLACES: usize = 12;
because!(AMOUNT_MOST_PLACES, AmountAnswer, "a result with more digits after the point is not written: no answer of a set is so long, and a longer result is most often a fraction that does not end");

pub(super) const AMOUNT_BY_LIST: &str = "list";
because!(AMOUNT_BY_LIST, AmountAnswer, "in the note of an answer: a sure rule gave it");

pub(super) const AMOUNT_BY_BEST: &str = "best";
because!(AMOUNT_BY_BEST, AmountAnswer, "in the note of an answer: no sure rule answered, and the form with the best share gave it");

#[derive(Clone, Copy, Debug)]
pub(super) enum AmountGot {
    Number(Amount),
    Word(u32),
}
because!(AmountGot, AmountAnswer, "what a rule gives for a case: an exact amount, or for steps over words the token of an answer word");

impl AmountModel {
    pub(super) fn gather(&self, cut: &AmountCut, hand: &mut AmountHand) {
        hand.forms.fill(cut, &self.base.marks);
        amount_examples(cut, &self.base.marks, &self.base.calcs[..self.base.pairs], &mut hand.shown);
        hand.found.clear();
        for (number, key) in hand.forms.seen.keys().iter().enumerate() {
            if let Some(form) = self.forms.find(*key) {
                hand.found.push((form, hand.forms.owner[number]));
            }
        }
        for (place, (calc, _)) in hand.shown.iter().enumerate() {
            if let Some(form) = self.forms.find(packed(&[self.base.marks.examples, self.base.marks.names[*calc]])) {
                hand.found.push((form, place as u32));
            }
        }
        hand.found.extend(self.always.iter().map(|form| (*form, 0)));
    }

    pub(super) fn listed(&self, hand: &mut AmountHand) {
        hand.tried.clear();
        for (form, at) in &hand.found {
            let mut rank = self.sure_head[*form as usize];
            while rank != NO_NUMBER {
                hand.tried.push((rank, self.sure[rank as usize], *at));
                rank = self.sure_next[rank as usize];
            }
        }
        hand.tried.sort_unstable();
    }

    pub(super) fn result(&self, stat: u32, at: u32, cut: &AmountCut, hand: &mut AmountHand) -> Option<(AmountGot, usize, usize)> {
        let rule = &self.rules[stat as usize];
        match rule.kind {
            AmountKind::Window => {
                let (from, size) = hand.forms.windows[at as usize];
                let (from, size) = (from as usize, size as usize);
                Some((AmountGot::Number(self.base.calcs[rule.calc].run(&cut.values[from..from + size])?), from, size))
            }
            AmountKind::Examples => Some((AmountGot::Number(hand.shown[at as usize].1), 0, 0)),
            AmountKind::Steps => Some((AmountGot::Number(self.base.steps_value(cut, self.way.as_ref()?, &mut hand.items)?), 0, 0)),
            AmountKind::WordSteps => {
                let value = self.base.word_value(&cut.words, &self.word_values, self.word_way.as_ref()?, &mut hand.word_items)?;
                Some((AmountGot::Word(self.word_values[usize::from(value)]), 0, 0))
            }
        }
    }

    pub(super) fn held(got: AmountGot, from: usize, size: usize, cut: &AmountCut) -> bool {
        match got {
            AmountGot::Number(value) => cut.stands(value, from, size),
            AmountGot::Word(_) => false,
        }
    }

    pub(super) fn sure_number(&self, cut: &AmountCut, hand: &mut AmountHand) -> Option<Amount> {
        self.gather(cut, hand);
        self.listed(hand);
        for place in 0..hand.tried.len() {
            let (_, stat, at) = hand.tried[place];
            if let Some((AmountGot::Number(value), from, size)) = self.result(stat, at, cut, hand)
                && (!self.rules[stat as usize].expects || cut.stands(value, from, size))
            {
                return Some(value);
            }
        }
        None
    }

    pub(super) fn token<'a>(&'a self, number: u32, unknown: &'a AmountUnknown) -> &'a str {
        match (number as usize).checked_sub(self.base.symbols.len()) {
            Some(place) => unknown.name(place),
            None => self.base.symbols.name(number),
        }
    }

    pub(super) fn text(&self, rule: &AmountRule, got: AmountGot, from: usize, size: usize, cut: &AmountCut, hand: &mut AmountHand) -> Option<()> {
        let near = amount_near().count();
        hand.text.clear();
        for (place, is_number) in self.base.shapes[rule.shape as usize].iter().enumerate() {
            if place > 0 {
                hand.text.push_str(AMOUNT_BETWEEN);
            }
            if *is_number {
                match got {
                    AmountGot::Number(value) => hand.text.push_str(&value.written(AMOUNT_MOST_PLACES)?),
                    AmountGot::Word(token) => hand.text.push_str(self.token(token, &hand.unknown)),
                }
                continue;
            }
            let part = self.part_source[self.parts.find(packed(&[rule.counted, place as u32]))? as usize];
            let Some(code) = part.checked_sub(AMOUNT_NEAR_FLAG) else {
                hand.text.push_str(self.base.symbols.name(part));
                continue;
            };
            let operand = code as usize / near;
            if operand >= size {
                return None;
            }
            let at = cut.spots[from + operand] as isize + amount_near().nth(code as usize % near)?;
            let word = *cut.words.get(usize::try_from(at).ok()?)?;
            hand.text.push_str(self.token(word, &hand.unknown));
        }
        Some(())
    }

    pub(super) fn speak(&self, stat: u32, at: u32, by: &str, cut: &AmountCut, hand: &mut AmountHand) -> Option<Said> {
        let rule = &self.rules[stat as usize];
        let (got, from, size) = self.result(stat, at, cut, hand)?;
        if rule.expects && !AmountModel::held(got, from, size, cut) {
            return None;
        }
        self.text(rule, got, from, size, cut, hand)?;
        let form: Vec<&str> = fields_of(rule.key).iter().filter(|slot| **slot != NO_NUMBER).map(|slot| self.base.symbols.name(*slot)).collect();
        let name = match rule.kind {
            AmountKind::Window | AmountKind::Examples => self.base.names[rule.calc].clone(),
            AmountKind::Steps => self.way_name(self.way.as_ref()?, false),
            AmountKind::WordSteps => self.way_name(self.word_way.as_ref()?, true),
        };
        Some(Said { words: answer_words(&hand.text), note: format!("{} -> {}: {} of {} lessons, {}", form.join(AMOUNT_BETWEEN), name, rule.hits, rule.lessons, by) })
    }

    pub(super) fn way_name(&self, way: &AmountWay, of_words: bool) -> String {
        let mut places: Vec<usize> = (0..way.count).collect();
        places.sort_by(|left, right| self.base.symbols.name(way.tokens[*left]).cmp(self.base.symbols.name(way.tokens[*right])));
        let said: Vec<String> = places
            .iter()
            .map(|place| {
                let token = self.base.symbols.name(way.tokens[*place]);
                let choice = way.choices[*place] as usize;
                if of_words {
                    let table: String = self.base.tables[choice].iter().map(|value| self.base.symbols.name(self.word_values[usize::from(*value)]).chars().next().unwrap_or_default()).collect();
                    format!("{token}: takes {}, {table}, level {}", self.base.tables[choice].len() / AMOUNT_PAIR, way.levels[*place])
                } else {
                    format!("{token}: {}, level {}", self.base.names[choice], way.levels[*place])
                }
            })
            .collect();
        said.join("; ")
    }

    pub(super) fn reply(&self, cut: &AmountCut, hand: &mut AmountHand) -> Option<Said> {
        self.gather(cut, hand);
        self.listed(hand);
        for place in 0..hand.tried.len() {
            let (_, stat, at) = hand.tried[place];
            if let Some(said) = self.speak(stat, at, AMOUNT_BY_LIST, cut, hand) {
                return Some(said);
            }
        }
        hand.tried.clear();
        for (form, at) in &hand.found {
            let stat = self.best[*form as usize];
            if self.rules[stat as usize].share > self.default_share {
                hand.tried.push((0, stat, *at));
            }
        }
        hand.tried.sort_by(|left, right| self.best_order(&self.rules[left.1 as usize], &self.rules[right.1 as usize]));
        for place in 0..hand.tried.len() {
            let (_, stat, at) = hand.tried[place];
            if let Some(said) = self.speak(stat, at, AMOUNT_BY_BEST, cut, hand) {
                return Some(said);
            }
        }
        None
    }

    pub(super) fn said(&self, sentences: &[String], question: &str, hand: &mut AmountHand) -> Said {
        let mut cut = std::mem::take(&mut hand.cut);
        let known = self.base.symbols.len();
        let symbols = &self.base.symbols;
        let unknown = &mut hand.unknown;
        unknown.clear();
        amount_cut_case(sentences, question, &self.base.marks, &mut cut, &mut hand.lower, &mut |word: &str| symbols.find(word).unwrap_or_else(|| unknown.number(word, known)));
        let found = self.reply(&cut, hand);
        hand.cut = cut;
        found.unwrap_or_else(|| Said { words: self.default.clone(), note: String::new() })
    }
}

impl Learner for AmountModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        AMOUNT_HANDS.with_borrow_mut(|hand| self.said(sentences, question, hand))
    }
}
