use crate::base::keys::{NO_NUMBER, packed};
use crate::base::runs::RunNumbers;
use crate::tools::rule::forms::{RULE_ANY, RuleFact, RulePart, RuleRole};
use crate::tools::rule::lessons::{RuleRest, RuleSource};
use crate::tools::rule::tables::{RULE_LEAST, RULE_MAJORITY, RuleReader, RuleTableName, RuleWay, rule_body_run, rule_role_run, rule_whole_run};
use crate::tools::rule::tokens::RuleSentence;
use patterns::{because, source};

pub struct ReadingByPieces;
source!(ReadingByPieces, "a sentence read by its pieces: the words that the lessons hold as things are the slots, slots that follow one another are one thing or two as the lessons show, and the sentence is cut into pieces of one thing or of two. Every cut is tried, and the sentence is read by the cut whose pieces need the least of the weaker ways of reading; the pieces must be all facts, or conditions with one head");

const RULE_MOST_THINGS: usize = 12;
because!(RULE_MOST_THINGS, ReadingByPieces, "a sentence with more things is not cut: the cuts to try grow fast with the things, and a sentence of a rule that people read is shorter");

const RULE_WIDTHS: &[usize] = &[2, 1];
because!(RULE_WIDTHS, ReadingByPieces, "the things that a piece can hold, a subject with its object or an object alone, in the order in which the cuts are tried: of two cuts with the same score the one that was tried first stays");

const RULE_SOURCE_NAMES: &[RuleSource] = &[RuleSource::Any, RuleSource::Near, RuleSource::Own, RuleSource::Same];
because!(RULE_SOURCE_NAMES, ReadingByPieces, "the places that a subject can come from in the order of their names: where two values of a key with another relation word both fit, the first in a fixed order is taken, so the same sentence is read the same way in every run");

#[derive(Clone, Debug)]
pub(super) struct RuleGot {
    pub(super) rest: RuleRest,
    pub(super) source: RuleSource,
    pub(super) way: RuleWay,
    pub(super) used: Vec<(usize, u32)>,
}
because!(RuleGot, ReadingByPieces, "one piece as it was read: what its marks say, where its subject comes from, the way of reading, and the keys that were used, each as its table and its number");

#[derive(Clone, Debug)]
pub(super) struct RuleRead {
    pub(super) parts: Vec<RulePart>,
    pub(super) way: RuleWay,
    pub(super) used: Vec<(usize, u32)>,
    pub(super) before: Option<u32>,
}
because!(RuleRead, ReadingByPieces, "one sentence as it was read: the parts of its formal form, the weakest way of reading that a piece needed, the keys that were used, and the subject of its last part");

#[derive(Debug, Default)]
pub(super) struct RuleHand {
    pub(super) more: RunNumbers,
    pub(super) run: Vec<u32>,
    pub(super) lead: Vec<u32>,
    pub(super) inner: Vec<u32>,
    pub(super) tail: Vec<u32>,
}
because!(RuleHand, ReadingByPieces, "the working memory of one reading: the things that the lessons did not name, each with a number after the numbers of the things of the lessons, so that a thing has one number in all sentences of a theory; and the runs of tokens of the piece that is looked up");

pub(super) fn rule_thing_number(known: &RunNumbers, more: &mut RunNumbers, words: &[u32]) -> u32 {
    match known.find(words) {
        Some(number) => number + 1,
        None => (known.count() as u32) + more.number(words) + 1,
    }
}
because!(rule_thing_number, ReadingByPieces, "the number of a thing that a sentence names: its number among the things of the lessons, or a number after them for a thing that no lesson named");

impl RuleReader {
    pub(super) fn is_slot(&self, word: u32, flip: bool) -> bool {
        let seen = self.seen_of(word);
        if seen == 0 {
            return self.new_is_slot != flip;
        }
        RULE_MAJORITY * self.slots[word as usize] >= seen
    }

    pub(super) fn one_thing(&self, first: u32, second: u32) -> bool {
        let [apart, joined] = self.pairs.find(packed(&[first, second])).map_or(self.joined, |number| self.pair_counts[number as usize]);
        joined >= apart
    }

    pub(super) fn things(&self, toks: &[u32], flip: bool) -> Vec<(usize, usize)> {
        let mut things = Vec::new();
        let mut at = 0;
        while at < toks.len() {
            if !self.is_slot(toks[at], flip) {
                at += 1;
                continue;
            }
            let mut end = at + 1;
            while end < toks.len() && self.is_slot(toks[end], flip) && self.one_thing(toks[end - 1], toks[end]) {
                end += 1;
            }
            things.push((at, end));
            at = end;
        }
        things
    }

    fn other_word(&self, table: RuleTableName, run: &mut [u32], first: usize, second: usize, way: RuleWay) -> Option<RuleGot> {
        if !self.many_relations {
            return None;
        }
        for at in (0..first).chain(first + 1..first + 1 + second) {
            let word = run[at];
            let mine = self.relation_of.get(word as usize).copied().unwrap_or(NO_NUMBER);
            if mine == NO_NUMBER {
                continue;
            }
            let mut found: Vec<(bool, RuleRole, usize, Vec<u32>, Vec<(usize, u32)>)> = Vec::new();
            for (other, relation) in &self.relation_words {
                if *other == word {
                    continue;
                }
                run[at] = *other;
                let Some((number, rest, source)) = self.tables[table as usize].known(run, 1, true) else { continue };
                if rest.relation != *relation {
                    continue;
                }
                let name = RULE_SOURCE_NAMES.iter().position(|held| *held == source).unwrap_or(0);
                let place = match found.iter().position(|(yes, role, held, _, _)| *yes == rest.yes && *role == rest.role && *held == name) {
                    Some(place) => place,
                    None => {
                        found.push((rest.yes, rest.role, name, Vec::new(), Vec::new()));
                        found.len() - 1
                    }
                };
                if !found[place].3.contains(relation) {
                    found[place].3.push(*relation);
                }
                found[place].4.push((table as usize, number));
            }
            run[at] = word;
            found.sort_by_key(|(yes, role, name, _, _)| (*yes, *role, *name));
            if let Some((yes, role, name, _, used)) = found.into_iter().find(|(_, _, _, relations, _)| relations.len() > 1) {
                return Some(RuleGot { rest: RuleRest { relation: mine, yes, role }, source: RULE_SOURCE_NAMES[name], way, used });
            }
        }
        None
    }

    fn piece(&self, hand: &mut RuleHand) -> Option<RuleGot> {
        let (lead, inner, tail) = (&hand.lead, &hand.inner, &hand.tail);
        rule_whole_run(lead, inner, tail, &mut hand.run);
        if self.ways[RuleWay::Whole as usize]
            && let Some((number, rest, source)) = self.tables[RuleTableName::Whole as usize].known(&hand.run, 1, true)
        {
            return Some(RuleGot { rest, source, way: RuleWay::Whole, used: vec![(RuleTableName::Whole as usize, number)] });
        }
        if self.ways[RuleWay::Word as usize]
            && let Some(got) = self.other_word(RuleTableName::Whole, &mut hand.run, lead.len(), inner.len(), RuleWay::Word)
        {
            return Some(got);
        }
        if !self.ways[RuleWay::Halves as usize] && !self.ways[RuleWay::HalvesWord as usize] {
            return None;
        }
        for split in 1..=lead.len() {
            rule_role_run(&lead[..split], !tail.is_empty(), &mut hand.run);
            let roles = &self.tables[RuleTableName::Role as usize];
            let Some((start, role, _)) = roles.known(&hand.run, RULE_LEAST, false) else { continue };
            if !roles.stats[start as usize].two_sides {
                continue;
            }
            rule_body_run(&lead[split..], inner, &mut hand.run);
            let bodies = &self.tables[RuleTableName::Body as usize];
            if self.ways[RuleWay::Halves as usize]
                && let Some((body, rest, source)) = bodies.known(&hand.run, RULE_LEAST, true)
                && bodies.stats[body as usize].two_sides
            {
                return Some(RuleGot { rest: RuleRest { role: role.role, ..rest }, source, way: RuleWay::Halves, used: vec![(RuleTableName::Role as usize, start), (RuleTableName::Body as usize, body)] });
            }
            if self.ways[RuleWay::HalvesWord as usize]
                && let Some(mut got) = self.other_word(RuleTableName::Body, &mut hand.run, lead.len() - split, inner.len(), RuleWay::HalvesWord)
            {
                got.rest.role = role.role;
                got.used.insert(0, (RuleTableName::Role as usize, start));
                return Some(got);
            }
        }
        None
    }

    fn piece_at(&self, toks: &[u32], things: &[(usize, usize)], thing: usize, width: usize, hand: &mut RuleHand) -> Option<RuleGot> {
        let end = thing.checked_sub(1).map_or(0, |before| things[before].1);
        let stop = things[thing + width - 1].1;
        hand.lead.clear();
        if thing == 0 {
            hand.lead.push(self.marks.start);
        }
        hand.lead.extend_from_slice(&toks[end..things[thing].0]);
        hand.inner.clear();
        hand.inner.push(self.marks.slot);
        if width > 1 {
            hand.inner.extend_from_slice(&toks[things[thing].1..things[thing + 1].0]);
            hand.inner.push(self.marks.slot);
        }
        hand.tail.clear();
        if thing + width == things.len() {
            hand.tail.extend_from_slice(&toks[stop..]);
            hand.tail.push(self.marks.end);
        }
        self.piece(hand)
    }

    fn walk(&self, things: usize, thing: usize, got: &[Vec<Option<RuleGot>>], taken: &mut Vec<(usize, usize)>, best: &mut Option<((usize, usize), Vec<(usize, usize)>)>) {
        if thing == things {
            let roles: Vec<RuleRole> = taken.iter().filter_map(|(at, width)| got[*at][*width - 1].as_ref().map(|piece| piece.rest.role)).collect();
            let heads = roles.iter().filter(|role| **role == RuleRole::Head).count();
            let conditions = roles.iter().filter(|role| **role == RuleRole::Condition).count();
            let fine = roles.iter().all(|role| *role == RuleRole::Fact) || roles.iter().all(|role| *role == RuleRole::More) || (roles.len() > 1 && heads == 1 && conditions + 1 == roles.len());
            if !fine {
                return;
            }
            let ways: usize = taken.iter().filter_map(|(at, width)| got[*at][*width - 1].as_ref().map(|piece| piece.way as usize)).sum();
            let score = (ways, taken.len());
            if best.as_ref().is_none_or(|(held, _)| score < *held) {
                *best = Some((score, taken.clone()));
            }
            return;
        }
        for width in RULE_WIDTHS {
            if thing + width > things || got[thing][*width - 1].is_none() {
                continue;
            }
            taken.push((thing, *width));
            self.walk(things, thing + width, got, taken, best);
            taken.pop();
        }
    }

    fn cuts(&self, toks: &[u32], before: Option<u32>, flip: bool, plain: Option<usize>, known: &RunNumbers, hand: &mut RuleHand) -> Option<RuleRead> {
        let mut things = self.things(toks, flip);
        if let Some(plain) = plain {
            things.remove(plain);
            if things.is_empty() {
                return None;
            }
        }
        if things.is_empty() {
            return Some(RuleRead { parts: Vec::new(), way: RuleWay::Whole, used: Vec::new(), before });
        }
        if things.len() > RULE_MOST_THINGS {
            return None;
        }
        let mut got: Vec<Vec<Option<RuleGot>>> = Vec::with_capacity(things.len());
        for thing in 0..things.len() {
            let mut widths: Vec<Option<RuleGot>> = Vec::new();
            for width in 1..=RULE_WIDTHS.len() {
                let piece = if thing + width > things.len() { None } else { self.piece_at(toks, &things, thing, width, hand) };
                widths.push(piece.filter(|piece| plain.is_none() || piece.way == RuleWay::Whole));
            }
            got.push(widths);
        }
        let mut best: Option<((usize, usize), Vec<(usize, usize)>)> = None;
        self.walk(things.len(), 0, &got, &mut Vec::new(), &mut best);
        let (_, taken) = best?;
        let own: Vec<Option<u32>> = taken.iter().map(|(thing, width)| (*width > 1).then(|| rule_thing_number(known, &mut hand.more, &toks[things[*thing].0..things[*thing].1]))).collect();
        let mut read = RuleRead { parts: Vec::new(), way: RuleWay::Whole, used: Vec::new(), before };
        for (number, (thing, width)) in taken.iter().enumerate() {
            let piece = got[*thing][*width - 1].as_ref()?;
            let subject = match piece.source {
                RuleSource::Own => own[number],
                RuleSource::Any => Some(RULE_ANY),
                RuleSource::Same => read.before,
                RuleSource::Near => own.iter().enumerate().filter_map(|(other, held)| held.map(|held| (other.abs_diff(number), other, held))).min().map(|(_, _, held)| held),
            }?;
            let (from, to) = things[thing + width - 1];
            let object = rule_thing_number(known, &mut hand.more, &toks[from..to]);
            read.parts.push((RuleFact { subject, relation: piece.rest.relation, thing: object, yes: piece.rest.yes }, piece.rest.role));
            read.used.extend_from_slice(&piece.used);
            read.way = read.way.max(piece.way);
            read.before = Some(subject);
        }
        Some(read)
    }

    pub(super) fn sentence(&self, sent: &RuleSentence, before: Option<u32>, told: bool, known: &RunNumbers, hand: &mut RuleHand) -> Option<RuleRead> {
        let toks = &sent.toks;
        let mut got = self.cuts(toks, before, false, None, known, hand);
        if got.is_none() && toks.iter().any(|word| self.seen_of(*word) == 0) {
            got = self.cuts(toks, before, true, None, known, hand);
        }
        if got.is_none() && self.ways[RuleWay::Whole as usize] {
            for plain in 0..self.things(toks, false).len() {
                got = self.cuts(toks, before, false, Some(plain), known, hand);
                if got.is_some() {
                    break;
                }
            }
        }
        if got.is_none() && told && self.ways[RuleWay::Marks as usize] {
            got = self.by_marks(sent, before, known, hand);
        }
        got
    }
}
