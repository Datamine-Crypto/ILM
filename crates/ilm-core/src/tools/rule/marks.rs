use crate::base::keys::packed;
use crate::base::runs::RunNumbers;
use crate::base::sums::compensated;
use crate::base::symbols::Symbols;
use crate::tools::rule::forms::{RULE_ANY, RuleFact, RulePart, RuleRole};
use crate::tools::rule::lessons::{RULE_KIND_ORDER, RuleKind, RuleLesson, rule_kind_of, rule_words};
use crate::tools::rule::pieces::{RuleHand, RuleRead, rule_thing_number};
use crate::tools::rule::tables::{RULE_CLASSES, RULE_KINDS, RULE_LEAST, RULE_MAJORITY, RULE_SURE, RuleReader, RuleWay, rule_thing_words};
use crate::tools::rule::tokens::RuleSentence;
use crate::base::runs::run_place;
use patterns::{because, source};

pub struct ReadingByMarks;
source!(ReadingByMarks, "a sentence read by its marks, for text that people wrote, where almost no form comes twice: naive Bayes, the independence model of Maron, chooses the kind of the sentence from its words that are no things and from its place in the theory, and the things of the sentence are then the parts of a fact or of a rule by their place. It is used only where nearly all told parts of the lessons have one relation and one of yes and no, so that the marks need not say them");

pub struct NewNames;
source!(NewNames, "a word that the lessons never held can be the thing that a sentence speaks about, a new name, and never what is said about it; what it is, is counted from the places where the things of the lessons stand: how the word is written and which tokens stand next to it");

const RULE_SURE_MARK: u32 = 20;
because!(RULE_SURE_MARK, ReadingByMarks, "the least readings of a mark that decides the kind of a sentence alone: fewer do not show that the mark goes with one kind");

const RULE_PRIOR: f64 = 20.0;
because!(RULE_PRIOR, ReadingByMarks, "the readings that the share of a mark over all kinds counts for inside each kind: a kind with few lessons then gets no floor of its own for a mark that it never had");

const RULE_HALF: f64 = 0.5;
because!(RULE_HALF, ReadingByMarks, "the reading that every kind has before the lessons, as the estimate of Krichevsky and Trofimov gives it for one of two outcomes; a new word is also a subject thing only when more than this share of the things at its place were");

const RULE_CLEAR: f64 = 0.9;
because!(RULE_CLEAR, NewNames, "the share at which the place of a new first word decides alone: the first word of a sentence has a capital letter in any case, so its place says less than a place inside the sentence");

const RULE_MOST_LEFT: u32 = 9;
because!(RULE_MOST_LEFT, ReadingByMarks, "more told sentences after a sentence count as this many: far from the end of a theory the exact number says nothing more");

const RULE_PLACE_KEYS: usize = 4;
because!(RULE_PLACE_KEYS, NewNames, "the keys of the place of a word with the tokens next to it, from the fullest to the shortest: the token before and the next tokens after; the next tokens after; the token before and the token after; the token after");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuleClass {
    Subject,
    Object,
    Mark,
    Open,
}
because!(RuleClass, ReadingByMarks, NewNames, "what a token of a sentence is for the reading by marks: a subject thing, an object thing, a mark, or a new first word that stays open until the marks of its sentence decide");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuleWritten {
    First,
    Capital,
    Small,
}
because!(RuleWritten, NewNames, "how a word is written at its place: as the first word of the sentence, with a capital letter inside the sentence, or small");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuleBefore {
    Named,
    Any,
    Nothing,
}
because!(RuleBefore, ReadingByMarks, "what stands before a sentence as the subject of the part before: a named thing, any thing, or nothing. The order is the order of the marks in the sum of their logarithms, which is fixed so that a sentence has the same score in every run");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuleMark {
    Word(u32),
    Before(RuleBefore),
    Later(bool),
    Left(u32),
    Subject(bool),
}
because!(RuleMark, ReadingByMarks, "one mark of a sentence for its kind: a word that is no thing; what stands before the sentence; whether a later sentence of the theory holds a subject thing; how many told sentences stand after it; whether it holds a subject thing itself. place gives the place of the mark in the counts, the marks of the place of the sentence first and the words after them; order gives the place of the mark in the sum of a score, the words first");

impl RuleMark {
    fn place(&self) -> usize {
        let laters = RuleBefore::Nothing as usize + 1;
        let lefts = laters + usize::from(true) + 1;
        let subjects = lefts + RULE_MOST_LEFT as usize + 1;
        let words = subjects + usize::from(true) + 1;
        match self {
            RuleMark::Before(before) => *before as usize,
            RuleMark::Later(later) => laters + usize::from(*later),
            RuleMark::Left(left) => lefts + *left as usize,
            RuleMark::Subject(subject) => subjects + usize::from(*subject),
            RuleMark::Word(word) => words + *word as usize,
        }
    }

    fn order(&self, words: &[u32]) -> (bool, usize) {
        match self {
            RuleMark::Word(word) => (false, words.get(*word as usize).copied().unwrap_or(0) as usize),
            _ => (true, self.place()),
        }
    }
}

const RULE_QUOTE: char = '\'';
because!(RULE_QUOTE, ReadingByMarks, "the mark that stands around a word in the order of the marks");

const RULE_OTHER_QUOTE: char = '"';
because!(RULE_OTHER_QUOTE, ReadingByMarks, "the mark that stands around a word which holds the usual one, so such a word comes before all others in the order of the marks");

const RULE_ESCAPE: char = '\\';
because!(RULE_ESCAPE, ReadingByMarks, "the mark that stands before a mark of a word which could end the word, and before the number of a control mark");

const RULE_NUMBERED: char = 'x';
because!(RULE_NUMBERED, ReadingByMarks, "the letter after which a control mark is written as its number");

const RULE_NUMBER_BASE: u32 = 16;
because!(RULE_NUMBER_BASE, ReadingByMarks, "the base in which the number of a control mark is written, with one digit for each half of its byte");

fn rule_written_word(word: &str) -> String {
    let quote = if word.contains(RULE_QUOTE) && !word.contains(RULE_OTHER_QUOTE) { RULE_OTHER_QUOTE } else { RULE_QUOTE };
    let mut out = String::new();
    out.push(quote);
    for mark in word.chars() {
        if mark.is_control() {
            let number = u32::from(mark);
            out.push(RULE_ESCAPE);
            out.push(RULE_NUMBERED);
            out.extend(char::from_digit(number / RULE_NUMBER_BASE, RULE_NUMBER_BASE));
            out.extend(char::from_digit(number % RULE_NUMBER_BASE, RULE_NUMBER_BASE));
            continue;
        }
        if mark == quote || mark == RULE_ESCAPE {
            out.push(RULE_ESCAPE);
        }
        out.push(mark);
    }
    out.push(quote);
    out
}
because!(rule_written_word, ReadingByMarks, "a word as it is written for the order of the marks: between quotes, with the marks that would end it and the control marks written out, so that the order is the order of plain texts and does not depend on how a table numbers its words");

pub(super) fn rule_mark_order(symbols: &Symbols) -> Vec<u32> {
    let written: Vec<String> = symbols.names().iter().map(|name| rule_written_word(name)).collect();
    let mut order: Vec<usize> = (0..written.len()).collect();
    order.sort_by(|left, right| written[*left].cmp(&written[*right]));
    let mut places = vec![0u32; written.len()];
    for (place, word) in order.iter().enumerate() {
        places[*word] = place as u32;
    }
    places
}
because!(rule_mark_order, ReadingByMarks, "the place of each word in the order in which the marks of a sentence are summed: a sum of logarithms depends on its order in the last digit, so the order must be the same for every sentence and every run");

impl RuleReader {
    fn is_subject(&self, word: u32) -> bool {
        RULE_MAJORITY * self.as_subject[word as usize] > self.slots[word as usize]
    }

    fn place_keys(&self, sent: &RuleSentence, at: usize) -> (RuleWritten, [u128; RULE_PLACE_KEYS], u128) {
        let toks = &sent.toks;
        let written = if at == 0 {
            RuleWritten::First
        } else if sent.caps.get(at).copied().unwrap_or(false) {
            RuleWritten::Capital
        } else {
            RuleWritten::Small
        };
        let before = at.checked_sub(1).map_or(self.marks.start, |place| toks[place]);
        let rest = toks.get(at + 1..).unwrap_or(&[]);
        let after = rest.first().copied().unwrap_or(self.marks.end);
        let then = rest.get(1).copied().unwrap_or(self.marks.end);
        let shape = |number: usize| (written as usize * (RULE_PLACE_KEYS + 1) + number) as u32;
        let mut keys = [0u128; RULE_PLACE_KEYS];
        for (number, [first, second, third]) in [[before, after, then], [after, then, after], [before, after, before], [after, after, after]].into_iter().enumerate() {
            keys[number] = packed(&[shape(number), first, second, third]);
        }
        (written, keys, packed(&[shape(RULE_PLACE_KEYS)]))
    }

    fn new_word(&self, sent: &RuleSentence, at: usize) -> RuleClass {
        let (written, keys, alone) = self.place_keys(sent, at);
        let counted = |key: &u128| self.stands.find(*key).map(|number| self.stand_counts[number as usize]).filter(|counts| counts.iter().sum::<u32>() >= RULE_LEAST);
        let mut counts = keys.iter().find_map(counted);
        if counts.is_none() && written == RuleWritten::Capital {
            counts = self.stands.find(alone).map(|number| self.stand_counts[number as usize]);
        }
        let Some(counts) = counts else { return if written == RuleWritten::First { RuleClass::Open } else { RuleClass::Mark } };
        let share = f64::from(counts[RuleClass::Subject as usize]) / f64::from(counts.iter().sum::<u32>());
        if written != RuleWritten::First {
            return if share > RULE_HALF { RuleClass::Subject } else { RuleClass::Mark };
        }
        if share >= RULE_CLEAR {
            RuleClass::Subject
        } else if share <= f64::from(1u8) - RULE_CLEAR {
            RuleClass::Mark
        } else {
            RuleClass::Open
        }
    }

    pub(super) fn classes(&self, sent: &RuleSentence) -> Vec<RuleClass> {
        let mut out: Vec<RuleClass> = Vec::with_capacity(sent.toks.len());
        for (at, word) in sent.toks.iter().enumerate() {
            out.push(if self.seen_of(*word) == 0 {
                self.new_word(sent, at)
            } else if !self.is_slot(*word, false) {
                RuleClass::Mark
            } else if self.is_subject(*word) {
                RuleClass::Subject
            } else {
                RuleClass::Object
            });
        }
        if out.contains(&RuleClass::Subject) {
            for class in &mut out {
                if *class == RuleClass::Open {
                    *class = RuleClass::Mark;
                }
            }
        }
        out
    }

    fn marks_of(&self, sent: &RuleSentence, before: Option<u32>, classes: &[RuleClass]) -> Vec<RuleMark> {
        let mut marks: Vec<RuleMark> = Vec::new();
        for (word, class) in sent.toks.iter().zip(classes) {
            if *class == RuleClass::Mark && !marks.contains(&RuleMark::Word(*word)) {
                marks.push(RuleMark::Word(*word));
            }
        }
        if !classes.contains(&RuleClass::Open) {
            marks.push(RuleMark::Subject(classes.contains(&RuleClass::Subject)));
        }
        marks.extend(sent.later.map(RuleMark::Later));
        marks.extend(sent.left.map(|left| RuleMark::Left(left.min(RULE_MOST_LEFT))));
        marks.push(RuleMark::Before(match before {
            None => RuleBefore::Nothing,
            Some(RULE_ANY) => RuleBefore::Any,
            Some(_) => RuleBefore::Named,
        }));
        marks
    }

    fn mark_counts(&self, mark: &RuleMark) -> [u32; RULE_KINDS] {
        self.kind_marks.get(mark.place()).copied().unwrap_or_default()
    }

    pub(super) fn count_marks(&mut self, lessons: &[&RuleLesson], terms: &RunNumbers) {
        for lesson in lessons.iter().filter(|lesson| lesson.told) {
            let (subjects, things) = rule_thing_words(&lesson.parts, terms);
            for (at, word) in lesson.sent.toks.iter().enumerate() {
                let what = if subjects.contains(word) {
                    RuleClass::Subject
                } else if things.contains(word) {
                    RuleClass::Object
                } else if self.seen_of(*word) == 1 {
                    RuleClass::Mark
                } else {
                    continue;
                };
                let (_, keys, alone) = self.place_keys(&lesson.sent, at);
                for key in keys.iter().chain([&alone]) {
                    let number = self.stands.number(*key) as usize;
                    if number == self.stand_counts.len() {
                        self.stand_counts.push([0; RULE_CLASSES]);
                    }
                    self.stand_counts[number][what as usize] += 1;
                }
            }
        }
        for lesson in lessons.iter().filter(|lesson| lesson.told) {
            let toks = &lesson.sent.toks;
            let Some(kind) = rule_kind_of(toks, &lesson.parts, lesson.before, terms) else { continue };
            let Some(first) = toks.first() else { continue };
            self.kinds[kind as usize] += 1;
            let begins = if kind == RuleKind::ItsOwn { lesson.parts.iter().any(|(fact, _)| rule_words(terms, fact.subject).contains(first)) } else { self.seen_of(*first) == 1 };
            self.begins[kind as usize] += u32::from(begins);
            let classes = self.classes(&lesson.sent);
            for mark in self.marks_of(&lesson.sent, lesson.before, &classes) {
                if self.kind_marks.len() <= mark.place() {
                    self.kind_marks.resize(mark.place() + 1, [0; RULE_KINDS]);
                }
                self.kind_marks[mark.place()][kind as usize] += 1;
            }
            if kind == RuleKind::ARule && lesson.parts.iter().all(|(_, role)| *role != RuleRole::More) {
                let mut order: Vec<&RulePart> = lesson.parts.iter().collect();
                order.sort_by_key(|(fact, _)| run_place(toks, rule_words(terms, fact.thing), 0).unwrap_or(0));
                for (from_end, (_, role)) in order.iter().rev().enumerate() {
                    self.places[from_end.min(1)][*role as usize] += 1;
                }
            }
        }
    }

    fn runs_of(&self, sent: &RuleSentence, classes: &[RuleClass], wanted: RuleClass) -> Vec<(usize, usize)> {
        let toks = &sent.toks;
        let mut runs = Vec::new();
        let mut at = 0;
        while at < toks.len() {
            if classes[at] == RuleClass::Mark {
                at += 1;
                continue;
            }
            let mut end = at + 1;
            while end < toks.len() && classes[end] == classes[at] && self.one_thing(toks[end - 1], toks[end]) {
                end += 1;
            }
            if classes[at] == wanted {
                runs.push((at, end));
            }
            at = end;
        }
        runs
    }

    fn kind_by_marks(&self, marks: &[RuleMark], able: &[RuleKind], new_start: bool) -> RuleKind {
        let mut sure: Option<(u32, usize)> = None;
        for mark in marks {
            let counts = self.mark_counts(mark);
            let total: u32 = counts.iter().sum();
            let mut top = 0;
            for kind in 0..RULE_KINDS {
                if counts[kind] >= counts[top] {
                    top = kind;
                }
            }
            let Some(place) = able.iter().position(|kind| *kind as usize == top) else { continue };
            if total >= RULE_SURE_MARK && f64::from(counts[top]) >= RULE_SURE * f64::from(total) && sure.is_none_or(|(held, at)| total > held || (total == held && place < at)) {
                sure = Some((total, place));
            }
        }
        if let Some((_, place)) = sure
            && !new_start
        {
            return able[place];
        }
        let total: u32 = able.iter().map(|kind| self.kinds[*kind as usize]).sum();
        let every = f64::from(self.kinds.iter().sum::<u32>());
        let score = |kind: &RuleKind| {
            let lessons = self.kinds[*kind as usize];
            let shares = marks.iter().map(|mark| {
                let counts = self.mark_counts(mark);
                ((f64::from(counts[*kind as usize]) + RULE_PRIOR * f64::from(counts.iter().sum::<u32>()) / every) / (f64::from(lessons) + RULE_PRIOR)).ln()
            });
            let mut out = ((f64::from(lessons) + RULE_HALF) / (f64::from(total) + RULE_HALF * (able.len() as f64))).ln() + compensated(shares);
            if new_start {
                out += ((f64::from(self.begins[*kind as usize]) + RULE_HALF) / f64::from(lessons + 1)).ln();
            }
            out
        };
        let mut best = able[0];
        let mut top = score(&best);
        for kind in &able[1..] {
            let found = score(kind);
            if found > top {
                best = *kind;
                top = found;
            }
        }
        best
    }

    pub(super) fn by_marks(&self, sent: &RuleSentence, before: Option<u32>, known: &RunNumbers, hand: &mut RuleHand) -> Option<RuleRead> {
        let (relation, yes) = self.usual?;
        if self.kinds.iter().all(|count| *count == 0) {
            return None;
        }
        let toks = &sent.toks;
        let classes = self.classes(sent);
        let mut subjects = self.runs_of(sent, &classes, RuleClass::Subject);
        let open_ones = self.runs_of(sent, &classes, RuleClass::Open);
        let objects = self.runs_of(sent, &classes, RuleClass::Object);
        if objects.is_empty() {
            return Some(RuleRead { parts: Vec::new(), way: RuleWay::Marks, used: Vec::new(), before });
        }
        let fits = |kind: &RuleKind| match kind {
            RuleKind::ARule => objects.len() > 1,
            RuleKind::ItsOwn => !subjects.is_empty() || !open_ones.is_empty(),
            RuleKind::TheSame => before.is_some_and(|thing| thing != RULE_ANY),
            RuleKind::GoesOn => before == Some(RULE_ANY) && subjects.is_empty(),
        };
        let able: Vec<RuleKind> = RULE_KIND_ORDER.iter().copied().filter(|kind| fits(kind) && self.kinds[*kind as usize] > 0).collect();
        if able.is_empty() {
            return None;
        }
        let mut marks: Vec<RuleMark> = self.marks_of(sent, before, &classes).into_iter().filter(|mark| able.iter().any(|kind| self.mark_counts(mark)[*kind as usize] > 0)).collect();
        marks.sort_by_key(|mark| mark.order(&self.mark_order));
        let kind = self.kind_by_marks(&marks, &able, !open_ones.is_empty() && subjects.is_empty());
        if subjects.is_empty() {
            subjects = open_ones;
        }
        let mut parts: Vec<RulePart> = Vec::new();
        for (number, (from, to)) in objects.iter().enumerate() {
            let thing = rule_thing_number(known, &mut hand.more, &toks[*from..*to]);
            let (subject, role) = match kind {
                RuleKind::GoesOn => (RULE_ANY, RuleRole::More),
                RuleKind::ARule => {
                    let counts = self.places[(objects.len() - 1 - number).min(1)];
                    let mut top: Option<usize> = None;
                    for role in 0..counts.len() {
                        if counts[role] > 0 && top.is_none_or(|held| counts[role] >= counts[held]) {
                            top = Some(role);
                        }
                    }
                    (RULE_ANY, [RuleRole::Condition, RuleRole::Fact, RuleRole::Head, RuleRole::More][top?])
                }
                RuleKind::ItsOwn => {
                    let near = subjects.iter().min_by_key(|(start, end)| (start.abs_diff(*to).min(from.abs_diff(*end)), *start))?;
                    (rule_thing_number(known, &mut hand.more, &toks[near.0..near.1]), RuleRole::Fact)
                }
                RuleKind::TheSame => (before?, RuleRole::Fact),
            };
            parts.push((RuleFact { subject, relation, thing, yes }, role));
        }
        if kind == RuleKind::ARule {
            let heads = parts.iter().filter(|(_, role)| *role == RuleRole::Head).count();
            let conditions = parts.iter().filter(|(_, role)| *role == RuleRole::Condition).count();
            if heads != 1 || conditions + 1 != parts.len() {
                return None;
            }
        }
        let last = parts.last().map(|(fact, _)| fact.subject);
        Some(RuleRead { parts, way: RuleWay::Marks, used: Vec::new(), before: last })
    }
}
