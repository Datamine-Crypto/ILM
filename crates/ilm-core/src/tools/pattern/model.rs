use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::codes::{Point, Step};
use crate::tools::pattern::finder::{Finder, Steps, words_at};
use crate::tools::pattern::groups::Groups;
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::list::Rule;
use crate::tools::pattern::names::WordTexts;
use crate::tools::pattern::splits::QuestionSplit;
use crate::tools::pattern::story::Query;
use crate::tools::pattern::striped::{Frozen, Seek};
use crate::tools::pattern::table::{Places, Table};
use crate::tools::pattern::texts::{asked_text, place_text, step_text};
use patterns::{because, source};

pub struct PatternAnswers;
source!(PatternAnswers, "the answer of a counted model: the first rule of the covering list that fits the case; with no such rule the fitting pattern with the best share of hits, marked as a guess; with no fitting pattern the answer of plain counting where that beat the most frequent answer on lessons that it did not count, else the most frequent answer");

pub const SURE_READINGS: u32 = 20;
because!(SURE_READINGS, PatternAnswers, "a pattern is a rule only with this many readings or more: with fewer, a place of the answer that every reading shares is still likely to be chance");

pub const SURE_SHARE: f64 = 0.98;
because!(SURE_SHARE, PatternAnswers, "the share of the readings of a rule that must have one place of the answer: nearly all, so that a few lessons that were written wrong do not stop a rule, and no more than a few");

#[derive(Clone, Copy, Debug)]
pub struct Why {
    pub pattern: u32,
    pub place: u32,
    pub hit: u32,
    pub readings: u32,
    pub list: bool,
}
because!(Why, PatternAnswers, "how an answer came: the pattern, the place of the answer, the hits and the readings, and if the pattern is a rule of the list or the pattern with the best share, which is a guess");

#[derive(Default)]
pub struct Fit {
    pub rows: Vec<(u32, u32, u32)>,
    pub slots: Vec<(u32, u32)>,
    at: CheapMap<u32, u32>,
    pub unknown: bool,
}
because!(Fit, PatternAnswers, "the known patterns that fit one case, in the order in which the search finds them, each with its free slots in one flat row; a pattern that is found again keeps its place and takes the later slots; unknown says that a part learner had no rule for a candidate. put adds a pattern, slots_of gives the slots of a row");

impl Fit {
    pub fn clear(&mut self) {
        self.rows.clear();
        self.slots.clear();
        self.at.clear();
        self.unknown = false;
    }

    pub fn put(&mut self, pattern: u32, slots: &[(u32, u32)], more: &[(u32, u32)]) {
        let from = self.slots.len() as u32;
        self.slots.extend(slots.iter().chain(more));
        let row = (pattern, from, self.slots.len() as u32 - from);
        match self.at.get(&pattern) {
            Some(held) => self.rows[*held as usize] = row,
            None => {
                self.at.insert(pattern, self.rows.len() as u32);
                self.rows.push(row);
            }
        }
    }

    pub fn slots_of(&self, row: usize) -> &[(u32, u32)] {
        let (_, from, len) = self.rows[row];
        &self.slots[from as usize..(from + len) as usize]
    }
}

pub struct Part {
    pub asked: u32,
    pub kind: u32,
    pub model: PatternModel,
    pub null: Vec<u32>,
    pub used: bool,
}
because!(Part, QuestionSplit, "one split: the question form and the kind of its candidates, the part learner, the answers for a candidate that add nothing to the answer of the whole case, and if a rule of the list uses the split");

pub struct PatternModel {
    pub depth: u32,
    pub star: u32,
    pub hash: u32,
    pub forms: Frozen<Vec<u32>>,
    pub steps: Frozen<Step>,
    pub asked: Frozen<Vec<u32>>,
    pub places: Places,
    pub table: Table,
    pub place: Vec<u32>,
    pub hit: Vec<u32>,
    pub rank: Vec<u32>,
    pub order: Vec<Rule>,
    pub left: usize,
    pub default: Vec<u32>,
    pub point_kinds: Vec<u32>,
    pub parts: Vec<Part>,
}
because!(PatternModel, PatternAnswers, QuestionSplit, "one counted model after its lessons: the tables of forms, steps, question forms and places, the patterns with the most frequent place of each and its hits, the list of rules with the place of each pattern in it, the lessons that the list left open, the most frequent answer, the kinds that a pointing may use, and the splits. hands gives the working tables for one thread; fitting finds the known patterns that fit a case; answer_case answers a case that was read; readings gives the lessons of a question form; text and place_name give the texts of a pattern and of a place");

pub struct Hands<'m> {
    finder: Finder<'m, Seek<'m>>,
    steps: Steps,
    one: Groups,
    two: Groups,
    pub fit: Fit,
    parts: Vec<Hands<'m>>,
    ids: Vec<u32>,
    kids: Vec<Option<u32>>,
    pub keep: bool,
}
because!(Hands, PatternAnswers, "the working tables of one thread for one model and for the part learners of its splits: the search for sentences, the ways to point, the groups of patterns, the patterns that fit, and the steps of the ways to point with the longer patterns of one pattern for each of them, which are all looked up before any is used so that the reads of memory do not wait for each other; they are made once for a call and used for every case of it; keep says that the cases are questions of one story, so the forms of its sentences stay from one case to the next");

#[derive(Clone, Copy)]
pub struct Setting {
    pub depth: u32,
    pub per_form: bool,
    pub split: bool,
    pub lanes: usize,
    pub star: u32,
    pub hash: u32,
}
because!(Setting, PatternAnswers, "how one model is counted: the most sentences of a pattern, if each question form widens its own most read patterns, if splits are learned, the threads, and the numbers of the two marks that the model needs (the word that is a star, and the mark before a candidate in the question of a part learner)");

impl PatternModel {
    pub fn hands<'m>(&'m self, words: &'m PatternWords) -> Hands<'m> {
        Hands {
            finder: Finder::over(words, &self.point_kinds, self.star, Seek { forms: &self.forms, steps: &self.steps }),
            steps: Steps::default(),
            one: Groups::default(),
            two: Groups::default(),
            fit: Fit::default(),
            parts: self.parts.iter().map(|part| part.model.hands(words)).collect(),
            ids: Vec::new(),
            kids: Vec::new(),
            keep: false,
        }
    }

    pub fn readings(&self, form: &[u32]) -> u32 {
        let root = self.asked.seek(form).and_then(|asked| self.table.child.get(ABSENT, asked));
        root.map_or(0, |pattern| self.table.count[pattern as usize])
    }

    pub fn fitting<N: WordTexts>(&self, words: &PatternWords, names: &mut N, hands: &mut Hands, case: &Query, light: bool) {
        let Hands { finder, steps, one, two, fit, parts, ids, kids, keep } = hands;
        fit.clear();
        let asked = self.asked.seek(case.form.as_slice());
        let Some(root) = asked.and_then(|asked| self.table.child.get(ABSENT, asked)) else { return };
        fit.put(root, &[], &[]);
        if *keep {
            finder.reopen(case);
        } else {
            finder.open(case);
        }
        finder.first_steps(case, steps);
        one.clear();
        ids.clear();
        ids.extend(steps.rows.iter().map(|found| found.step));
        self.table.child.get_all(root, ids, kids);
        for (found, kid) in steps.rows.iter().zip(kids.iter()) {
            let Some(full) = *kid else { continue };
            fit.put(full, steps.slots_of(found), &[]);
            if self.table.has_child[full as usize] {
                one.add((found.sent, ABSENT), steps.slots_of(found), &[], full);
            }
        }
        for step in FIRST_LATER..=self.depth {
            two.clear();
            for group in one.all() {
                let chosen = one.chosen(group);
                finder.next_steps(case, step, &chosen, one.slots_of(group), steps);
                ids.clear();
                ids.extend(steps.rows.iter().map(|found| found.step));
                for key in one.keys(group) {
                    self.table.child.get_all(key, ids, kids);
                    for (found, kid) in steps.rows.iter().zip(kids.iter()) {
                        let Some(full) = *kid else { continue };
                        fit.put(full, one.slots_of(group), steps.slots_of(found));
                        if self.table.has_child[full as usize] {
                            two.add((chosen[0], found.sent), one.slots_of(group), steps.slots_of(found), full);
                        }
                    }
                }
            }
            std::mem::swap(one, two);
        }
        for (number, part) in self.parts.iter().enumerate() {
            if Some(part.asked) != asked || (light && !part.used) {
                continue;
            }
            let Some(outs) = self.outcomes(words, names, &mut parts[number], case, number, light) else {
                fit.unknown = true;
                continue;
            };
            let (keys, slots) = self.split_keys(names, number, &outs);
            for key in keys {
                if let Some(full) = self.steps.seek(&key).and_then(|step| self.table.child.get(root, step)) {
                    fit.put(full, &slots, &[]);
                }
            }
        }
    }

    pub fn answer_case<N: WordTexts>(&self, words: &PatternWords, names: &mut N, hands: &mut Hands, case: &Query, light: bool) -> (Vec<u32>, Option<Why>) {
        self.fitting(words, names, hands, case, light);
        let fit = &hands.fit;
        if fit.unknown {
            return (Vec::new(), None);
        }
        let said = |place: u32, row: usize| self.places.row(place).and_then(|parts| words_at(parts, case, fit.slots_of(row))).filter(|got| !got.is_empty());
        let mut first: Option<(u32, Vec<u32>)> = None;
        for (row, (pattern, _, _)) in fit.rows.iter().enumerate() {
            let rank = self.rank[*pattern as usize];
            if rank != ABSENT && first.as_ref().is_none_or(|(top, _)| rank < *top) {
                if let Some(got) = said(self.order[rank as usize].place, row) {
                    first = Some((rank, got));
                }
            }
        }
        if let Some((rank, got)) = first {
            let rule = self.order[rank as usize];
            return (got, Some(Why { pattern: rule.pattern, place: rule.place, hit: rule.hit, readings: rule.readings, list: true }));
        }
        if light {
            return (Vec::new(), None);
        }
        let mut best: Option<(Rule, Vec<u32>)> = None;
        for (row, (pattern, _, _)) in fit.rows.iter().enumerate() {
            let rule = Rule { pattern: *pattern, place: self.place[*pattern as usize], hit: self.hit[*pattern as usize], readings: self.table.count[*pattern as usize] };
            let better = best.as_ref().is_none_or(|(top, _)| rule.share() > top.share() || (rule.share() == top.share() && rule.readings > top.readings));
            if better {
                if let Some(got) = said(rule.place, row) {
                    best = Some((rule, got));
                }
            }
        }
        match best {
            Some((rule, got)) => (got, Some(Why { pattern: rule.pattern, place: rule.place, hit: rule.hit, readings: rule.readings, list: false })),
            None => (self.default.clone(), None),
        }
    }

    pub fn text<N: WordTexts>(&self, words: &PatternWords, names: &N, pattern: u32) -> String {
        let root = self.table.root(pattern);
        let mut out = asked_text(names, self.asked.held(self.table.step[root as usize]));
        for step in self.table.chain(pattern) {
            let key = self.steps.held(step);
            let form: &[u32] = if matches!(key.point, Point::Any | Point::Last | Point::End | Point::Near) { self.forms.held(key.item) } else { &[] };
            out.push_str(" || ");
            out.push_str(&step_text(words, names, key, form));
        }
        out
    }

    pub fn place_name<N: WordTexts>(&self, names: &N, place: u32) -> String {
        self.places.row(place).map_or_else(|| "?".to_string(), |row| place_text(names, row))
    }
}

pub const FIRST_LATER: u32 = 2;
because!(FIRST_LATER, PatternAnswers, "the number of the first sentence of a pattern that is found from an earlier sentence, where the search for longer patterns starts");
