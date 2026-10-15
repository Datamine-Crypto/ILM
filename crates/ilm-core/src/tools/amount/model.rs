use crate::base::keys::{KeyNumbers, NO_NUMBER, fields_of, packed};
use crate::base::lanes::{on_lanes, shards};
use crate::base::symbols::Symbols;
use crate::tools::amount::calcs::AMOUNT_PAIR;
use crate::tools::amount::counts::{AMOUNT_NEAR_FLAG, AMOUNT_PARTS, AmountCounts, amount_count};
use crate::tools::amount::learn::AmountLesson;
use crate::tools::amount::shared::{AmountBase, AmountHand};
use crate::tools::amount::steps::AmountWay;
use patterns::{because, source};
use std::cmp::Ordering;

pub struct AmountSure;
source!(AmountSure, "the sure test: a rule is a key with a certain outcome. A form with a calculation is a rule when it stands in enough lessons and nearly all of them give the true number, in the first part of the lessons and again in the part that is held back; the rules are tried in a fixed order, the one with the higher share first");

pub(super) const AMOUNT_SURE_LESSONS: u32 = 20;
because!(AMOUNT_SURE_LESSONS, AmountSure, "the least lessons of the first part in which a form must stand to be a rule: fewer readings do not show that an outcome is certain");

pub(super) const AMOUNT_SURE_SHARE: f64 = 0.98;
because!(AMOUNT_SURE_SHARE, AmountSure, "the least share of the lessons of a form that must give the true number: a rule may fail for a rare lesson, which a fault of a set can cause, and not more");

pub(super) const AMOUNT_PRIOR: u32 = 3;
because!(AMOUNT_PRIOR, AmountSure, "the readings that are added to the lessons of a form before its share is taken, of which one counts as right: a form with few lessons then has a lower share than a form with many lessons and the same faults");

pub(super) const AMOUNT_EXPECT_HITS: u32 = 5;
because!(AMOUNT_EXPECT_HITS, AmountSure, "the least right lessons of a rule before the tool says that its result must stand in the text: fewer do not show a habit of the set");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AmountKind {
    Window,
    Examples,
    Steps,
    WordSteps,
}
because!(AmountKind, AmountSure, "how a rule gets its number: from the operands at the place of its form, from the examples inside the case, from the steps of a written sum, or from steps over the answer words");

#[derive(Clone, Debug)]
pub(super) struct AmountRule {
    pub(super) key: u128,
    pub(super) kind: AmountKind,
    pub(super) calc: usize,
    pub(super) share: f64,
    pub(super) lessons: u32,
    pub(super) hits: u32,
    pub(super) expects: bool,
    pub(super) shape: u32,
    pub(super) counted: u32,
}
because!(AmountRule, AmountSure, "a rule that an answer can come from: its form, its calculation, its share of right lessons with the added readings, its lessons and right lessons, whether its result must stand in the text outside the operands, the form of its answer, and its number in the counts, under which the words of its answer are kept");

pub(super) fn amount_ranks(symbols: &Symbols) -> Vec<u32> {
    let mut order: Vec<u32> = (0..symbols.len() as u32).collect();
    order.sort_unstable_by(|left, right| symbols.name(*left).cmp(symbols.name(*right)));
    let mut ranks = vec![0u32; symbols.len()];
    for (rank, number) in order.iter().enumerate() {
        ranks[*number as usize] = rank as u32;
    }
    ranks
}
because!(amount_ranks, AmountSure, "the place of each token in the order of the texts of all tokens: equal rules are put in order by the texts of their forms, so the order of the rules does not depend on the order of the lessons, and with these places two forms are compared as numbers");

#[derive(Debug)]
pub struct AmountModel {
    pub(super) base: AmountBase,
    pub(super) default: Vec<String>,
    pub(super) default_share: f64,
    pub(super) ranks: Vec<u32>,
    pub(super) forms: KeyNumbers,
    pub(super) best: Vec<u32>,
    pub(super) always: Vec<u32>,
    pub(super) rules: Vec<AmountRule>,
    pub(super) sure: Vec<u32>,
    pub(super) sure_head: Vec<u32>,
    pub(super) sure_next: Vec<u32>,
    pub(super) parts: KeyNumbers,
    pub(super) part_source: Vec<u32>,
    pub(super) way: Option<AmountWay>,
    pub(super) word_way: Option<AmountWay>,
    pub(super) word_values: Vec<u32>,
}
because!(AmountModel, AmountSure, "what the tool learned, as plain tables: the forms with the best calculation of each, the sure rules in the order in which they are tried (by their share right, the form with more fixed tokens first), the words of each answer, and the ways of steps. An answer comes from the first sure rule that gives a number which can be written; with no such rule from the form and calculation with the best share, when that share is above the share of the most frequent answer of the lessons; else it is the most frequent answer. The note of an answer names the form, the calculation, its right lessons and lessons, and which of the two lists gave it");

impl AmountModel {
    pub(super) fn settle(base: AmountBase, counts: &AmountCounts, default: Vec<String>, default_share: f64) -> AmountModel {
        let ranks = amount_ranks(&base.symbols);
        let rule_count = counts.rules.keys().len();
        let rule_form: Vec<usize> = counts.rules.keys().iter().map(|key| fields_of(*key)[0] as usize).collect();
        let rule_calc: Vec<usize> = counts.rules.keys().iter().map(|key| fields_of(*key)[1] as usize).collect();
        let stood = |part: usize, form: usize| amount_count(&counts.stands[part], form);
        let right = |part: usize, rule: usize| amount_count(&counts.right[part], rule);
        let hits = |rule: usize| right(0, rule) + right(1, rule);
        let mut best = vec![NO_NUMBER; counts.forms.keys().len()];
        let mut used = vec![false; rule_count];
        let mut sure_rules: Vec<usize> = Vec::new();
        for rule in 0..rule_count {
            let form = rule_form[rule];
            let held = best[form] as usize;
            if best[form] == NO_NUMBER || hits(rule) > hits(held) || (hits(rule) == hits(held) && rule_calc[rule] < rule_calc[held]) {
                best[form] = rule as u32;
            }
            let (early, later) = (stood(0, form), stood(1, form));
            if early >= AMOUNT_SURE_LESSONS && f64::from(right(0, rule)) >= AMOUNT_SURE_SHARE * f64::from(early) && later > 0 && f64::from(right(1, rule)) >= AMOUNT_SURE_SHARE * f64::from(later) {
                sure_rules.push(rule);
                used[rule] = true;
            }
        }
        for rule in &best {
            if *rule != NO_NUMBER {
                used[*rule as usize] = true;
            }
        }
        let mut top_shape = counts.shape.clone();
        let mut top_count = counts.shape_count.clone();
        let mut top_first = counts.shape_first.clone();
        for (other, key) in counts.others.keys().iter().enumerate() {
            let [rule, shape, ..] = fields_of(*key);
            let rule = rule as usize;
            let (count, first) = (counts.other_count[other], counts.other_first[other]);
            if used[rule] && (count > top_count[rule] || (count == top_count[rule] && first < top_first[rule])) {
                top_shape[rule] = shape;
                top_count[rule] = count;
                top_first[rule] = first;
            }
        }
        let mut parts = KeyNumbers::default();
        let mut part_source: Vec<u32> = Vec::new();
        let mut part_count: Vec<u32> = Vec::new();
        for (number, key) in counts.sources.keys().iter().enumerate() {
            let [rule, shape, place, from] = fields_of(*key);
            if !used[rule as usize] || top_shape[rule as usize] != shape {
                continue;
            }
            let count = counts.source_count[number];
            let part = parts.number(packed(&[rule, place])) as usize;
            if part == part_source.len() {
                part_source.push(from);
                part_count.push(count);
                continue;
            }
            let held = part_source[part];
            let later = match (from >= AMOUNT_NEAR_FLAG, held >= AMOUNT_NEAR_FLAG) {
                (true, true) => from > held,
                (false, false) => ranks[from as usize] > ranks[held as usize],
                (near, _) => near,
            };
            if count > part_count[part] || (count == part_count[part] && later) {
                part_source[part] = from;
                part_count[part] = count;
            }
        }
        let mut rules: Vec<AmountRule> = Vec::new();
        let mut stat_of = vec![NO_NUMBER; rule_count];
        for rule in 0..rule_count {
            if !used[rule] {
                continue;
            }
            let form = rule_form[rule];
            let key = counts.forms.key(form as u32);
            let lessons = stood(0, form) + stood(1, form);
            let kind = if fields_of(key)[0] == base.marks.examples { AmountKind::Examples } else { AmountKind::Window };
            stat_of[rule] = rules.len() as u32;
            rules.push(AmountRule {
                key,
                kind,
                calc: rule_calc[rule],
                share: f64::from(hits(rule) + 1) / f64::from(lessons + AMOUNT_PRIOR),
                lessons,
                hits: hits(rule),
                expects: hits(rule) >= AMOUNT_EXPECT_HITS && f64::from(amount_count(&counts.outside, rule)) >= AMOUNT_SURE_SHARE * f64::from(hits(rule)),
                shape: top_shape[rule],
                counted: rule as u32,
            });
        }
        let mut forms = KeyNumbers::default();
        let mut best_stat: Vec<u32> = Vec::new();
        for (form, rule) in best.iter().enumerate() {
            if *rule != NO_NUMBER {
                forms.number(counts.forms.key(form as u32));
                best_stat.push(stat_of[*rule as usize]);
            }
        }
        let sure: Vec<u32> = sure_rules.iter().map(|rule| stat_of[*rule]).collect();
        let mut model = AmountModel { base, default, default_share, ranks, forms, best: best_stat, always: Vec::new(), rules, sure, sure_head: Vec::new(), sure_next: Vec::new(), parts, part_source, way: None, word_way: None, word_values: Vec::new() };
        model.index();
        model
    }

    pub(super) fn form_order(&self, left: u128, right: u128) -> Ordering {
        for (mine, theirs) in fields_of(left).into_iter().zip(fields_of(right)) {
            let found = match (mine == NO_NUMBER, theirs == NO_NUMBER) {
                (true, true) => return Ordering::Equal,
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (false, false) => self.ranks[mine as usize].cmp(&self.ranks[theirs as usize]),
            };
            if found != Ordering::Equal {
                return found;
            }
        }
        Ordering::Equal
    }

    pub(super) fn fixed(&self, key: u128) -> usize {
        fields_of(key).iter().filter(|slot| **slot != NO_NUMBER && **slot != self.base.marks.open).count()
    }

    pub(super) fn sure_order(&self, left: &AmountRule, right: &AmountRule) -> Ordering {
        let name = |rule: &AmountRule| self.ranks[self.base.marks.names[rule.calc] as usize];
        right
            .share
            .total_cmp(&left.share)
            .then_with(|| self.fixed(right.key).cmp(&self.fixed(left.key)))
            .then_with(|| right.lessons.cmp(&left.lessons))
            .then_with(|| self.form_order(left.key, right.key))
            .then_with(|| name(left).cmp(&name(right)))
    }

    pub(super) fn best_order(&self, left: &AmountRule, right: &AmountRule) -> Ordering {
        right.share.total_cmp(&left.share).then_with(|| right.lessons.cmp(&left.lessons)).then_with(|| self.form_order(left.key, right.key))
    }

    pub(super) fn index(&mut self) {
        let mut sure = std::mem::take(&mut self.sure);
        sure.sort_by(|left, right| self.sure_order(&self.rules[*left as usize], &self.rules[*right as usize]));
        self.sure_head = vec![NO_NUMBER; self.forms.keys().len()];
        self.sure_next = vec![NO_NUMBER; sure.len()];
        for (rank, stat) in sure.iter().enumerate().rev() {
            if let Some(form) = self.forms.find(self.rules[*stat as usize].key) {
                self.sure_next[rank] = self.sure_head[form as usize];
                self.sure_head[form as usize] = rank as u32;
            }
        }
        self.sure = sure;
    }

    pub(super) fn add_rule(&mut self, mark: u32, kind: AmountKind, seen: [u32; AMOUNT_PARTS], right: [u32; AMOUNT_PARTS], expects: bool) {
        let hits: u32 = right.iter().sum();
        let lessons: u32 = seen.iter().sum();
        let key = packed(&[mark]);
        let stat = self.rules.len() as u32;
        self.rules.push(AmountRule { key, kind, calc: 0, share: f64::from(hits + 1) / f64::from(lessons + AMOUNT_PRIOR), lessons, hits, expects, shape: 0, counted: NO_NUMBER });
        let form = self.forms.number(key);
        self.best.push(stat);
        self.always.push(form);
        let [early, later] = seen;
        let [early_right, later_right] = right;
        if early >= AMOUNT_SURE_LESSONS && f64::from(early_right) >= AMOUNT_SURE_SHARE * f64::from(early) && later > 0 && f64::from(later_right) >= AMOUNT_SURE_SHARE * f64::from(later) {
            self.sure.push(stat);
        }
        self.index();
    }

    pub(super) fn add_steps(&mut self, lessons: &[AmountLesson], lanes: usize) {
        let first: Vec<&AmountLesson> = lessons.iter().filter(|lesson| lesson.counted && !lesson.late).collect();
        if first.is_empty() {
            return;
        }
        let ranges = shards(first.len(), lanes);
        let answered: usize = on_lanes(ranges.len(), lanes, &|piece| {
            let mut hand = AmountHand::default();
            ranges[piece]
                .clone()
                .filter(|place| {
                    let got = self.sure_number(&first[*place].cut, &mut hand);
                    got.is_some() && got == first[*place].truth
                })
                .count()
        })
        .iter()
        .sum();
        if (answered as f64) >= AMOUNT_SURE_SHARE * (first.len() as f64) {
            return;
        }
        let Some(way) = self.base.find_steps(lessons, lanes) else { return };
        let mut seen = [0u32; AMOUNT_PARTS];
        let mut right = [0u32; AMOUNT_PARTS];
        let mut in_text = 0u32;
        let mut items = Vec::new();
        for lesson in lessons.iter().filter(|lesson| lesson.counted) {
            let Some(got) = self.base.steps_value(&lesson.cut, &way, &mut items) else { continue };
            let part = usize::from(lesson.late);
            seen[part] += 1;
            if Some(got) == lesson.truth {
                right[part] += 1;
                in_text += u32::from(lesson.cut.stands(got, 0, 0));
            }
        }
        let hits: u32 = right.iter().sum();
        if hits == 0 {
            return;
        }
        self.way = Some(way);
        self.add_rule(self.base.marks.steps, AmountKind::Steps, seen, right, hits >= AMOUNT_EXPECT_HITS && f64::from(in_text) >= AMOUNT_SURE_SHARE * f64::from(hits));
    }

    pub(super) fn add_word_steps(&mut self, lessons: &[AmountLesson], cases: usize, lanes: usize) {
        let mut both: Vec<u32> = Vec::new();
        let mut worded = 0usize;
        for word in lessons.iter().filter_map(|lesson| lesson.word) {
            worded += 1;
            if !both.contains(&word) {
                both.push(word);
            }
        }
        if (worded as f64) < AMOUNT_SURE_SHARE * (cases as f64) || both.len() != AMOUNT_PAIR {
            return;
        }
        both.sort_by(|left, right| self.base.symbols.name(*left).cmp(self.base.symbols.name(*right)));
        let stands = |word: &u32| lessons.iter().filter(|lesson| lesson.word.is_some() && lesson.cut.words.contains(word)).count() >= AMOUNT_SURE_LESSONS as usize;
        if !both.iter().all(stands) {
            return;
        }
        let Some(way) = self.base.find_word_steps(lessons, &both, lanes) else { return };
        let mut seen = [0u32; AMOUNT_PARTS];
        let mut right = [0u32; AMOUNT_PARTS];
        let mut items = Vec::new();
        for lesson in lessons.iter().filter(|lesson| lesson.word.is_some()) {
            let Some(got) = self.base.word_value(&lesson.cut.words, &both, &way, &mut items) else { continue };
            let part = usize::from(lesson.late);
            seen[part] += 1;
            right[part] += u32::from(Some(both[usize::from(got)]) == lesson.word);
        }
        if right.iter().sum::<u32>() == 0 {
            return;
        }
        self.word_way = Some(way);
        self.word_values = both;
        self.add_rule(self.base.marks.word_steps, AmountKind::WordSteps, seen, right, false);
    }
}
