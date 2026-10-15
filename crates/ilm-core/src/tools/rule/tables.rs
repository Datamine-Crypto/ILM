use crate::base::keys::{KeyNumbers, NO_NUMBER, fields_of, packed};
use crate::base::runs::RunNumbers;
use crate::base::symbols::Symbols;
use crate::tools::rule::forms::{RULE_ANY, RulePart, RuleRole};
use crate::tools::rule::lessons::{RULE_SOURCE_ORDER, RULE_SOURCES, RuleCut, RuleLesson, RuleRest, RuleSource, rule_cut, rule_words};
use crate::tools::rule::tokens::RuleMarks;
use patterns::{because, source};

pub struct PieceTables;
source!(PieceTables, "a reader counted from lesson sentences with their formal forms: for the key of each piece the values that the lessons gave it, and a value counts when nearly every reading of the key has it. Beside the whole key of a piece there are shorter keys, the marks before the piece cut in two, which read a piece whose whole key no lesson held; a shorter key is used only where the lessons show that its value does not depend on what stands next to it");

pub(super) const RULE_SURE: f64 = 0.98;
because!(RULE_SURE, PieceTables, "the least share of the readings of a key that must have one value: a rule may fail for a rare lesson, which a fault of a set can cause, and not more");

pub(super) const RULE_LEAST: u32 = 3;
because!(RULE_LEAST, PieceTables, "the least readings of a shorter key, and of a word that names a relation: fewer do not show that the value belongs to the key and not to one sentence");

pub(super) const RULE_WAYS: usize = 5;
because!(RULE_WAYS, PieceTables, "the ways of reading a sentence, from the whole key of each piece to the marks of the sentence");

pub(super) const RULE_TABLES: usize = 3;
because!(RULE_TABLES, PieceTables, "the tables of keys: the whole key of a piece, the first part of the marks before it, and the rest of these marks with the piece");

pub(super) const RULE_KINDS: usize = 4;
because!(RULE_KINDS, PieceTables, "the kinds of a told sentence that the reading by marks chooses among");

pub(super) const RULE_ROLES: usize = 4;
because!(RULE_ROLES, PieceTables, "the roles that a part of a sentence can have");

pub(super) const RULE_CLASSES: usize = 3;
because!(RULE_CLASSES, PieceTables, "what is counted at the place of a word: a subject thing, an object thing, or a word that the lessons hold once");

const RULE_NEIGHBOURS: usize = 2;
because!(RULE_NEIGHBOURS, PieceTables, "the tokens of a sentence that are looked at together to count whether slots which follow one another are one thing");

pub(super) const RULE_BOTH: usize = 2;
because!(RULE_BOTH, PieceTables, "the outcomes of a count of yes and no: slots that follow one another as one thing or as separate things, a thing of a rule at its end or before its end, a sentence read right or wrong");

pub(super) const RULE_MAJORITY: u32 = 2;
because!(RULE_MAJORITY, PieceTables, "a word is taken for what it was in most of its places: its places of one sort times this number are compared with all its places");

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum RuleWay {
    Whole,
    Word,
    Halves,
    HalvesWord,
    Marks,
}
because!(RuleWay, PieceTables, "how a piece or a sentence was read, from the surest way to the last: the whole key; the whole key with another relation word; the two halves of the marks; the two halves with another relation word; the marks of the sentence. A sentence is read by the cut whose pieces need the least of the later ways");

pub(super) const RULE_ALL_WAYS: &[RuleWay] = &[RuleWay::Whole, RuleWay::Word, RuleWay::Halves, RuleWay::HalvesWord, RuleWay::Marks];
because!(RULE_ALL_WAYS, PieceTables, "every way of reading, in the order from the surest to the last");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuleTableName {
    Whole,
    Role,
    Body,
}
because!(RuleTableName, PieceTables, "the three tables of keys: the whole key of a piece; the first part of the marks before a piece with the word whether the piece is the last, which gives the role; the rest of the marks with the piece, which gives the relation and yes or no");

#[derive(Clone, Debug, Default)]
pub(super) struct RuleStat {
    pub(super) readings: u32,
    pub(super) rests: Vec<(RuleRest, u32)>,
    pub(super) fits: [u32; RULE_SOURCES],
    pub(super) side: u32,
    pub(super) two_sides: bool,
}
because!(RuleStat, PieceTables, "the readings of one key: how many, the values that they gave with the count of each, for each place that a subject can come from the readings that fit it, and whether two different tokens stood next to the key over its readings. best gives the value and the place of the subject when nearly all readings agree on each, and nothing else");

impl RuleStat {
    fn add(&mut self, rest: RuleRest, fits: &[bool; RULE_SOURCES], side: u32) {
        if self.readings == 0 {
            self.side = side;
        }
        self.readings += 1;
        match self.rests.iter_mut().find(|(held, _)| *held == rest) {
            Some((_, count)) => *count += 1,
            None => self.rests.push((rest, 1)),
        }
        for (place, fit) in fits.iter().enumerate() {
            self.fits[place] += u32::from(*fit);
        }
        self.two_sides = self.two_sides || (side != NO_NUMBER && self.side != NO_NUMBER && side != self.side);
        if self.side == NO_NUMBER {
            self.side = side;
        }
    }

    pub(super) fn best(&self, least: u32, with_source: bool) -> Option<(RuleRest, RuleSource)> {
        if self.readings < least {
            return None;
        }
        let mut top: Option<(RuleRest, u32)> = None;
        for (rest, count) in &self.rests {
            if top.is_none_or(|(_, held)| *count > held) {
                top = Some((*rest, *count));
            }
        }
        let (rest, count) = top?;
        let needed = RULE_SURE * f64::from(self.readings);
        if f64::from(count) < needed {
            return None;
        }
        let mut name = RuleSource::Own;
        for source in RULE_SOURCE_ORDER {
            if self.fits[*source as usize] > self.fits[name as usize] {
                name = *source;
            }
        }
        if with_source && f64::from(self.fits[name as usize]) < needed {
            return None;
        }
        Some((rest, name))
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct RuleTable {
    pub(super) runs: RunNumbers,
    pub(super) stats: Vec<RuleStat>,
    pub(super) dropped: Vec<bool>,
}
because!(RuleTable, PieceTables, "one table of keys: each key is a run of tokens with its readings, and a key can be dropped when it read too many sentences wrong on lessons that it was not counted from. add counts one reading; known gives the number of a key with its sure value, and nothing for a key that is not there, that was dropped, or whose readings do not agree");

impl RuleTable {
    fn add(&mut self, run: &[u32], rest: RuleRest, fits: &[bool; RULE_SOURCES], side: u32) {
        let number = self.runs.number(run) as usize;
        if number == self.stats.len() {
            self.stats.push(RuleStat::default());
            self.dropped.push(false);
        }
        self.stats[number].add(rest, fits, side);
    }

    pub(super) fn known(&self, run: &[u32], least: u32, with_source: bool) -> Option<(u32, RuleRest, RuleSource)> {
        let number = self.runs.find(run)?;
        if self.dropped[number as usize] {
            return None;
        }
        self.stats[number as usize].best(least, with_source).map(|(rest, source)| (number, rest, source))
    }
}

pub(super) fn rule_whole_run(lead: &[u32], inner: &[u32], tail: &[u32], run: &mut Vec<u32>) {
    run.clear();
    run.extend_from_slice(lead);
    run.push(NO_NUMBER);
    run.extend_from_slice(inner);
    run.push(NO_NUMBER);
    run.extend_from_slice(tail);
}
because!(rule_whole_run, PieceTables, "the whole key of a piece as one run: the marks before it, its slots with the marks between them, and the marks after it, with a number that no token has between the three");

pub(super) fn rule_role_run(start: &[u32], last: bool, run: &mut Vec<u32>) {
    run.clear();
    run.extend_from_slice(start);
    run.push(NO_NUMBER);
    run.push(u32::from(last));
}
because!(rule_role_run, PieceTables, "the key that gives the role of a piece: the first part of the marks before it, and whether the piece is the last of its sentence");

pub(super) fn rule_body_run(rest: &[u32], inner: &[u32], run: &mut Vec<u32>) {
    run.clear();
    run.extend_from_slice(rest);
    run.push(NO_NUMBER);
    run.extend_from_slice(inner);
}
because!(rule_body_run, PieceTables, "the key that gives the relation of a piece and yes or no: the rest of the marks before it, and its slots with the marks between them");

#[derive(Clone, Debug, Default)]
pub(super) struct RuleReader {
    pub(super) marks: RuleMarks,
    pub(super) ways: [bool; RULE_WAYS],
    pub(super) new_is_slot: bool,
    pub(super) tables: [RuleTable; RULE_TABLES],
    pub(super) seen: Vec<u32>,
    pub(super) slots: Vec<u32>,
    pub(super) as_subject: Vec<u32>,
    pub(super) pairs: KeyNumbers,
    pub(super) pair_counts: Vec<[u32; RULE_BOTH]>,
    pub(super) joined: [u32; RULE_BOTH],
    pub(super) relation_of: Vec<u32>,
    pub(super) relation_words: Vec<(u32, u32)>,
    pub(super) many_relations: bool,
    pub(super) usual: Option<(u32, bool)>,
    pub(super) stands: KeyNumbers,
    pub(super) stand_counts: Vec<[u32; RULE_CLASSES]>,
    pub(super) kinds: [u32; RULE_KINDS],
    pub(super) begins: [u32; RULE_KINDS],
    pub(super) kind_marks: Vec<[u32; RULE_KINDS]>,
    pub(super) places: [[u32; RULE_ROLES]; RULE_BOTH],
    pub(super) mark_order: Vec<u32>,
    pub(super) not_cut: u32,
}
because!(RuleReader, PieceTables, "what the reader counted from its lesson sentences, as plain tables: the three tables of keys; for each word how often it stood, how often as a word of a thing and how often in a subject; for each pair of slots that follow one another whether they were one thing; which word names which relation, and whether the words name more than one relation, because a key with another relation word is looked for only then; the relation and the yes or no that nearly all told parts have, when there is one; and the counts of the reading by marks (where things stand, the kinds of sentences, the marks of each kind, the roles of the things of a rule by their place from the end). count makes the tables from lesson sentences; the ways of reading, the dropped keys and what a new word is taken for are set after it");

impl RuleReader {
    pub(super) fn seen_of(&self, word: u32) -> u32 {
        self.seen.get(word as usize).copied().unwrap_or(0)
    }

    fn bump_pair(&mut self, first: u32, second: u32, one: bool) {
        let number = self.pairs.number(packed(&[first, second])) as usize;
        if number == self.pair_counts.len() {
            self.pair_counts.push([0; RULE_BOTH]);
        }
        self.pair_counts[number][usize::from(one)] += 1;
        self.joined[usize::from(one)] += 1;
    }

    fn count_piece_keys(&mut self, cut: &RuleCut, means: &mut KeyNumbers, mean_counts: &mut Vec<u32>, run: &mut Vec<u32>) {
        let no_fits = [false; RULE_SOURCES];
        for piece in &cut.pieces {
            rule_whole_run(&piece.lead, &piece.inner, &piece.tail, run);
            self.tables[RuleTableName::Whole as usize].add(run, piece.rest, &piece.fits, NO_NUMBER);
            for split in 1..=piece.lead.len() {
                rule_role_run(&piece.lead[..split], !piece.tail.is_empty(), run);
                let side = piece.lead.get(split).copied().unwrap_or(self.marks.edge);
                self.tables[RuleTableName::Role as usize].add(run, RuleRest { relation: NO_NUMBER, yes: false, role: piece.rest.role }, &no_fits, side);
                rule_body_run(&piece.lead[split..], &piece.inner, run);
                self.tables[RuleTableName::Body as usize].add(run, RuleRest { role: RuleRole::Fact, ..piece.rest }, &piece.fits, piece.lead[split - 1]);
            }
            let words = piece.lead.iter().chain(&piece.inner);
            for (place, word) in words.clone().enumerate() {
                if *word == self.marks.slot || *word == self.marks.start || words.clone().take(place).any(|held| held == word) {
                    continue;
                }
                let number = means.number(packed(&[*word, piece.rest.relation])) as usize;
                if number == mean_counts.len() {
                    mean_counts.push(0);
                }
                mean_counts[number] += 1;
            }
        }
    }

    fn name_relations(&mut self, means: &KeyNumbers, mean_counts: &[u32], symbols: &Symbols) {
        let mut total = vec![0u32; symbols.len()];
        let mut best = vec![(0u32, NO_NUMBER); symbols.len()];
        for (number, key) in means.keys().iter().enumerate() {
            let [word, relation, ..] = fields_of(*key);
            let count = mean_counts[number];
            let (held, named) = best[word as usize];
            total[word as usize] += count;
            if count > held || (count == held && (named == NO_NUMBER || symbols.name(relation) > symbols.name(named))) {
                best[word as usize] = (count, relation);
            }
        }
        self.relation_of = vec![NO_NUMBER; symbols.len()];
        for word in 0..symbols.len() {
            let (count, relation) = best[word];
            if total[word] >= RULE_LEAST && f64::from(count) >= RULE_SURE * f64::from(total[word]) {
                self.relation_of[word] = relation;
                self.relation_words.push((word as u32, relation));
            }
        }
        self.relation_words.sort_by(|left, right| symbols.name(left.0).cmp(symbols.name(right.0)));
        self.many_relations = self.relation_words.iter().any(|(_, relation)| self.relation_words.first().is_some_and(|(_, first)| first != relation));
    }

    pub(super) fn count(lessons: &[&RuleLesson], symbols: &Symbols, marks: &RuleMarks, terms: &RunNumbers, mark_order: &[u32]) -> RuleReader {
        let mut reader = RuleReader { marks: *marks, ways: [true; RULE_WAYS], new_is_slot: true, seen: vec![0; symbols.len()], slots: vec![0; symbols.len()], as_subject: vec![0; symbols.len()], mark_order: mark_order.to_vec(), ..RuleReader::default() };
        let mut means = KeyNumbers::default();
        let mut mean_counts: Vec<u32> = Vec::new();
        let mut content: Vec<((u32, bool), u32)> = Vec::new();
        let mut run: Vec<u32> = Vec::new();
        let mut inside: Vec<u32> = Vec::new();
        for lesson in lessons {
            let toks = &lesson.sent.toks;
            let (subjects, things) = rule_thing_words(&lesson.parts, terms);
            for word in toks {
                let in_subject = subjects.contains(word);
                reader.seen[*word as usize] += 1;
                reader.slots[*word as usize] += u32::from(in_subject || things.contains(word));
                reader.as_subject[*word as usize] += u32::from(in_subject);
            }
            if lesson.told {
                for (fact, _) in &lesson.parts {
                    match content.iter_mut().find(|(held, _)| *held == (fact.relation, fact.yes)) {
                        Some((_, count)) => *count += 1,
                        None => content.push(((fact.relation, fact.yes), 1)),
                    }
                }
            }
            let cut = if lesson.parts.is_empty() { Some(RuleCut::default()) } else { rule_cut(toks, &lesson.parts, lesson.before, marks, terms) };
            let Some(cut) = cut else {
                reader.not_cut += 1;
                continue;
            };
            inside.clear();
            inside.resize(toks.len(), NO_NUMBER);
            for (number, (from, to)) in cut.spans.iter().enumerate() {
                inside[*from..*to].fill(number as u32);
            }
            for (place, pair) in toks.windows(RULE_NEIGHBOURS).enumerate() {
                if let [first, second] = pair
                    && inside[place] != NO_NUMBER
                    && inside[place + 1] != NO_NUMBER
                {
                    reader.bump_pair(*first, *second, inside[place] == inside[place + 1]);
                }
            }
            reader.count_piece_keys(&cut, &mut means, &mut mean_counts, &mut run);
        }
        reader.name_relations(&means, &mean_counts, symbols);
        let told: u32 = content.iter().map(|(_, count)| *count).sum();
        let mut usual: Option<((u32, bool), u32)> = None;
        for (value, count) in &content {
            if usual.is_none_or(|(_, held)| *count > held) {
                usual = Some((*value, *count));
            }
        }
        reader.usual = usual.filter(|(_, count)| f64::from(*count) >= RULE_SURE * f64::from(told)).map(|(value, _)| value);
        if reader.usual.is_some() {
            reader.count_marks(lessons, terms);
        }
        reader
    }
}

pub(super) fn rule_thing_words(parts: &[RulePart], terms: &RunNumbers) -> (Vec<u32>, Vec<u32>) {
    let mut subjects: Vec<u32> = Vec::new();
    let mut things: Vec<u32> = Vec::new();
    for (fact, _) in parts {
        if fact.subject != RULE_ANY {
            subjects.extend_from_slice(rule_words(terms, fact.subject));
        }
        things.extend_from_slice(rule_words(terms, fact.thing));
    }
    (subjects, things)
}
because!(rule_thing_words, PieceTables, "the words of the subjects and the words of the objects of the parts of a lesson sentence: the words of a sentence that are words of its things are its slots");

