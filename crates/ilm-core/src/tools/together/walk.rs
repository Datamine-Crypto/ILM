use crate::base::exact::Amount;
use crate::base::first_sight::FirstSightTable;
use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::amount::learn::amount_late;
use crate::tools::amount::tokens::AmountCut;
use crate::tools::call::Said;
use crate::tools::pattern::names::{Fresh, WordTexts};
use crate::tools::state::reader::{FormIds, Forms, MoreForms, StateReader};
use crate::tools::together::tokens::{RESULT_PLACES, answer_amount, token_amount};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct PlaceWalk;
source!(PlaceWalk, "a story whose sentences move a place by amounts and turn a facing is read by three tools at once: the state tool reads each told sentence into units with forms, the amount tool gives the amount of a unit and makes each sum, and this tool holds the place on two lines and the facing; for each key of unit it searches the operation that the unit does, by running whole lessons and comparing their answers; only a program that is right for nearly all lessons of both parts of the lessons answers");

pub struct WalkSearch;
source!(WalkSearch, "the program of a walk is searched a group at a time: the keys that stand together in the most lessons are tried together, every choice of their operations, and the choice with the most lessons right stays; of equal choices the one where the words in one place of one form do the most different things, then the one with the most units as expected (a unit with an amount moves, one with no amount does nothing), then the first in the fixed order; then the next keys, with the first ones fixed");

const WALK_SURE_COUNT: usize = 20;
because!(WALK_SURE_COUNT, PlaceWalk, "the lessons that a form of told sentence, and a program that is a rule, need: the sure count of the tools");

const WALK_SURE_SHARE: f64 = 0.98;
because!(WALK_SURE_SHARE, PlaceWalk, "the share of told sentences with a frequent form, of answers held by the readings of the start, and of lessons right, that a walk needs: the sure share of the tools");

const MOST_KEYS: usize = 40;
because!(MOST_KEYS, PlaceWalk, "a kind with more keys of units is not read as a walk: the search tries every choice of operations for each group of keys, and a walk of the public sets has fewer keys");

const MOST_FORMS_SHARE: usize = 2;
because!(MOST_FORMS_SHARE, PlaceWalk, "a kind whose told sentences have more forms than this many times the most keys is no walk, looked at before the state tool is asked");

const TEST_PARTS: usize = 2;
because!(TEST_PARTS, PlaceWalk, "the parts of the sure test of a walk: the first part of the lessons, where the program is searched, and the part held back, each needing the sure count");

const MOST_GROUP: usize = 6;
because!(MOST_GROUP, WalkSearch, "the most keys whose operations are tried together: five to the power of this many choices is the most that one group costs");

const TOP_ANSWERS: usize = 2;
because!(TOP_ANSWERS, PlaceWalk, "the reading of whether the place is the start has this many results, so the answers of the lessons must be nearly all of this many kinds when they are no numbers");

const FACINGS: u8 = 4;
because!(FACINGS, PlaceWalk, "the facings of a walk: ahead, to one side, back and to the other side");

const AHEAD: u8 = 0;
because!(AHEAD, PlaceWalk, "the operation that moves the place ahead by the amount of the unit");

const BACK: u8 = 1;
because!(BACK, PlaceWalk, "the operation that moves the place back by the amount of the unit");

const ONE_SIDE: u8 = 2;
because!(ONE_SIDE, PlaceWalk, "the operation that moves the place to one side by the amount of the unit");

const OTHER_SIDE: u8 = 3;
because!(OTHER_SIDE, PlaceWalk, "the operation that moves the place to the other side by the amount of the unit");

const QUARTER_ONE: u8 = 4;
because!(QUARTER_ONE, PlaceWalk, "the operation that turns the facing a quarter to one side");

const QUARTER_OTHER: u8 = 5;
because!(QUARTER_OTHER, PlaceWalk, "the operation that turns the facing a quarter to the other side");

const TURN_HALF: u8 = 6;
because!(TURN_HALF, PlaceWalk, "the operation that turns the facing half round");

const NOTHING: u8 = 7;
because!(NOTHING, PlaceWalk, "the operation of a unit that does nothing");

const UNSET: u8 = u8::MAX;
because!(UNSET, PlaceWalk, "the mark of a key that the program has no operation for, so a story with such a unit is not read");

const MOVES: &[u8] = &[AHEAD, BACK, ONE_SIDE, OTHER_SIDE, NOTHING];
because!(MOVES, WalkSearch, "the operations of a unit with an amount, in the fixed order of the search");

const TURNS: &[u8] = &[QUARTER_ONE, QUARTER_OTHER, TURN_HALF, NOTHING];
because!(TURNS, WalkSearch, "the operations of a unit with no amount, in the fixed order of the search");

const OPERATION_TEXTS: &[&str] = &["ahead", "back", "to one side", "to the other side", "a quarter to one side", "a quarter to the other side", "a half", "nothing"];
because!(OPERATION_TEXTS, PlaceWalk, "the name of each operation in the note of an answer");

const SIDE_OF: &[u8] = &[0, 2, 1, 3];
because!(SIDE_OF, PlaceWalk, "for each move the quarter turns from the facing to the way it moves: ahead none, back half round, to one side one quarter, to the other side three");

const TURN_OF: &[u8] = &[1, 3, 2];
because!(TURN_OF, PlaceWalk, "for each turn the quarters that it adds to the facing");

const LINES: &[(i8, i8)] = &[(0, 1), (1, 0), (0, -1), (-1, 0)];
because!(LINES, PlaceWalk, "a facing as the step on the line across and the step on the line ahead");

const NUMBER_FORM: &str = "#";
because!(NUMBER_FORM, PlaceWalk, "the mark of an amount in the form of a key, as the amount tool marks a number");

const ANSWER_GAP: &str = " ";
because!(ANSWER_GAP, PlaceWalk, "the mark between the words of an answer when they are read again as one text");

const WALK_PARTS: &str = "state tool: the sentences as units with forms; amount tool: the amounts and each sum; together tool: the place, the facing, the program";
because!(WALK_PARTS, PlaceWalk, "the part of the note that says which tool did which step of an answer of the walk");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitAmount {
    Absent,
    One(Amount),
    Many,
}
because!(UnitAmount, PlaceWalk, "the amount of a unit: none, the one amount of its tokens, or more than one");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WalkResult {
    Ahead(Amount),
    Start(bool),
}
because!(WalkResult, PlaceWalk, "the result of a walk that is read: the amount on the line ahead, or whether the place is the start");

type WalkKey = (Vec<String>, Vec<String>);
because!(WalkKey, PlaceWalk, "the key of a unit: its form with the mark of an amount in the place of an amount, and the words in its other places of kinds");

type WalkSteps = Vec<(u32, UnitAmount)>;
because!(WalkSteps, PlaceWalk, "the units of a story as the numbers of their keys and their amounts");

type WalkLesson = (WalkSteps, Vec<String>);
because!(WalkLesson, PlaceWalk, "a lesson of the walk: the units of its story and the words of its answer");

pub struct Walk {
    pub rule: bool,
    reader: StateReader,
    keys: HashMap<WalkKey, u32>,
    texts: Vec<WalkKey>,
    program: Vec<u8>,
    ahead: bool,
    said: Vec<(bool, Vec<String>)>,
}
because!(Walk, PlaceWalk, WalkSearch, "a walk learned from lessons: whether it is a rule, the state tool that reads stories into units, the keys of the units by their texts, the operation of each key, whether the answer is the amount ahead or whether the place is the start, and the answer of each result of the start. learn makes it, or nothing when the lessons are no walk; answer gives the words of a story and the steps of the walk, or nothing when the program does not read the story");

fn sentence_forms(cases: &[Case]) -> bool {
    let mut forms: FirstSightTable<Vec<u32>, u32> = FirstSightTable::new();
    let mut told: Vec<Vec<u32>> = Vec::new();
    let mut table = Symbols::new();
    let number = table.number(NUMBER_FORM);
    let mut lower = String::new();
    for case in cases {
        let mut mine: Vec<Vec<u32>> = Vec::with_capacity(case.sentences.len());
        for sentence in &case.sentences {
            let mut cut = AmountCut::default();
            cut.scan(sentence, number, &mut lower, &mut |word: &str| table.number(word));
            mine.push(cut.words);
        }
        let mut distinct = mine.clone();
        distinct.sort_unstable();
        distinct.dedup();
        for form in distinct {
            forms.add(form, 1);
        }
        told.extend(mine);
    }
    if told.is_empty() || forms.rows().len() > MOST_FORMS_SHARE * MOST_KEYS {
        return false;
    }
    let frequent = told.iter().filter(|form| forms.get(form).is_some_and(|count| *count as usize >= WALK_SURE_COUNT)).count();
    frequent as f64 >= WALK_SURE_SHARE * told.len() as f64
}
because!(sentence_forms, PlaceWalk, "the first look before the state tool is asked: the told sentences have few forms with each number as one mark, and nearly all of them have a form that enough lessons hold");

fn is_number_answer(words: &[String]) -> bool {
    answer_amount(&words.join(ANSWER_GAP)).is_some()
}
because!(is_number_answer, PlaceWalk, "whether the words of an answer, read again as one text, are one number and nothing else");

impl Walk {
    fn intern(&mut self, key: WalkKey) -> u32 {
        if let Some(found) = self.keys.get(&key) {
            return *found;
        }
        let number = self.texts.len() as u32;
        self.keys.insert(key.clone(), number);
        self.texts.push(key);
        number
    }

    fn units<N: WordTexts, F: FormIds>(&self, names: &mut N, forms: &mut F, sentences: &[String]) -> Vec<(WalkKey, UnitAmount)> {
        let none: HashSet<Vec<u32>> = HashSet::new();
        let mut out: Vec<(WalkKey, UnitAmount)> = Vec::new();
        for sentence in sentences {
            let cut = self.reader.cut_of(names, sentence, &[]);
            for unit in self.reader.units(names, forms, &cut, &[], &none) {
                let shape = forms.shape(unit.fid).to_vec();
                let mut form: Vec<String> = Vec::with_capacity(shape.len());
                let mut words: Vec<String> = Vec::new();
                let mut given: Vec<Amount> = Vec::new();
                for (token, entry) in unit.tokens.iter().zip(&shape) {
                    let text = names.name(*token).to_string();
                    let name = self.reader.shape_text(names, *entry);
                    match token_amount(&text) {
                        Some(amount) => {
                            given.push(amount);
                            form.push(NUMBER_FORM.to_string());
                        }
                        None => {
                            if name != text {
                                words.push(text);
                            }
                            form.push(name);
                        }
                    }
                }
                let amount = match given.as_slice() {
                    [] => UnitAmount::Absent,
                    [one] => UnitAmount::One(*one),
                    _ => UnitAmount::Many,
                };
                out.push(((form, words), amount));
            }
        }
        out
    }

    fn run(&self, steps: &[(u32, UnitAmount)], program: &[u8], mut trace: Option<&mut Vec<(u8, UnitAmount)>>) -> Option<(Amount, Amount)> {
        let (mut across, mut ahead, mut facing) = (Amount::whole(0), Amount::whole(0), 0u8);
        for (key, amount) in steps {
            let operation = program.get(*key as usize).copied().unwrap_or(UNSET);
            if operation == UNSET {
                return None;
            }
            if operation <= OTHER_SIDE {
                if let UnitAmount::One(amount) = amount {
                    let (step_across, step_ahead) = LINES[((facing + SIDE_OF[operation as usize]) % FACINGS) as usize];
                    if step_across != 0 {
                        across = if step_across > 0 { across.plus(*amount)? } else { across.minus(*amount)? };
                    }
                    if step_ahead != 0 {
                        ahead = if step_ahead > 0 { ahead.plus(*amount)? } else { ahead.minus(*amount)? };
                    }
                }
            } else if operation < NOTHING {
                facing = (facing + TURN_OF[(operation - QUARTER_ONE) as usize]) % FACINGS;
            }
            if let Some(trace) = trace.as_mut() {
                trace.push((operation, *amount));
            }
        }
        Some((across, ahead))
    }

    fn result(&self, steps: &[(u32, UnitAmount)], program: &[u8]) -> Option<WalkResult> {
        let (across, ahead) = self.run(steps, program, None)?;
        Some(if self.ahead { WalkResult::Ahead(ahead) } else { WalkResult::Start(across == Amount::whole(0) && ahead == Amount::whole(0)) })
    }

    fn words_of(&self, result: &WalkResult) -> Option<Vec<String>> {
        match result {
            WalkResult::Ahead(amount) => amount.written(RESULT_PLACES).map(|text| answer_words(&text)),
            WalkResult::Start(start) => self.said.iter().find(|(held, _)| held == start).map(|(_, words)| words.clone()),
        }
    }

    fn count(&self, lessons: &[WalkLesson], program: &[u8], known: Option<&[bool]>) -> (usize, Vec<(bool, Vec<String>)>) {
        let mut table: FirstSightTable<bool, FirstSightTable<Vec<String>, u32>> = FirstSightTable::new();
        let mut number = 0usize;
        for (steps, answer) in lessons {
            if let Some(known) = known
                && steps.iter().any(|(key, _)| !known[*key as usize])
            {
                continue;
            }
            let Some(got) = self.result(steps, program) else { continue };
            match got {
                WalkResult::Ahead(amount) => number += usize::from(amount.written(RESULT_PLACES).is_some_and(|text| answer_words(&text) == *answer)),
                WalkResult::Start(start) => table.entry(start, FirstSightTable::new()).add(answer.clone(), 1),
            }
        }
        if self.ahead {
            return (number, Vec::new());
        }
        let right = table.rows().iter().map(|(_, answers)| answers.rows().iter().map(|(_, count)| *count).max().unwrap_or_default() as usize).sum();
        let said = table
            .rows()
            .iter()
            .filter_map(|(start, answers)| {
                let mut ranked: Vec<&(Vec<String>, u32)> = answers.rows().iter().collect();
                ranked.sort_by(|one, other| other.1.cmp(&one.1).then_with(|| one.0.cmp(&other.0)));
                ranked.first().map(|(words, _)| (*start, words.clone()))
            })
            .collect();
        (right, said)
    }

    fn search(&mut self, lessons: &[WalkLesson]) -> bool {
        let keys = self.texts.len();
        let mut has_amount = vec![false; keys];
        for (steps, _) in lessons {
            for (key, amount) in steps {
                has_amount[*key as usize] |= *amount != UnitAmount::Absent;
            }
        }
        let mut program = vec![UNSET; keys];
        loop {
            let mut groups: FirstSightTable<Vec<u32>, u32> = FirstSightTable::new();
            for (steps, _) in lessons {
                let mut group: Vec<u32> = steps.iter().map(|(key, _)| *key).filter(|key| program[*key as usize] == UNSET).collect();
                group.sort_by(|one, other| self.texts[*one as usize].cmp(&self.texts[*other as usize]));
                group.dedup();
                if !group.is_empty() {
                    groups.add(group, 1);
                }
            }
            let top = groups.rows().iter().max_by(|one, other| {
                one.1.cmp(&other.1).then(other.0.len().cmp(&one.0.len())).then_with(|| {
                    let left = one.0.iter().map(|key| &self.texts[*key as usize]);
                    let right = other.0.iter().map(|key| &self.texts[*key as usize]);
                    left.cmp(right)
                })
            });
            let Some((top, _)) = top else { break };
            let group: Vec<u32> = top.iter().take(MOST_GROUP).copied().collect();
            let mut known: Vec<bool> = program.iter().map(|operation| *operation != UNSET).collect();
            for key in &group {
                known[*key as usize] = true;
            }
            let choices: Vec<&[u8]> = group.iter().map(|key| if has_amount[*key as usize] { MOVES } else { TURNS }).collect();
            let mut picks = vec![0usize; group.len()];
            let mut best: Option<((usize, usize, usize), Vec<u8>)> = None;
            let mut trial = program.clone();
            loop {
                for ((key, pick), choice) in group.iter().zip(&picks).zip(&choices) {
                    trial[*key as usize] = choice[*pick];
                }
                let right = self.count(lessons, &trial, Some(&known)).0;
                let mut by_form: FirstSightTable<&Vec<String>, Vec<u8>> = FirstSightTable::new();
                let mut expected = 0usize;
                for (key, operation) in trial.iter().enumerate() {
                    if *operation == UNSET {
                        continue;
                    }
                    let held = by_form.entry(&self.texts[key].0, Vec::new());
                    if !held.contains(operation) {
                        held.push(*operation);
                    }
                    expected += usize::from((*operation != NOTHING) == has_amount[key]);
                }
                let different = by_form.rows().iter().map(|(_, operations)| operations.len()).sum();
                let score = (right, different, expected);
                if best.as_ref().is_none_or(|(top, _)| score > *top) {
                    best = Some((score, group.iter().map(|key| trial[*key as usize]).collect()));
                }
                let mut place = group.len();
                let mut more = false;
                while place > 0 {
                    place -= 1;
                    if picks[place] + 1 < choices[place].len() {
                        picks[place] += 1;
                        more = true;
                        break;
                    }
                    picks[place] = 0;
                }
                if !more {
                    break;
                }
            }
            let Some(((right, _, _), chosen)) = best else { break };
            for (key, operation) in group.iter().zip(&chosen) {
                program[*key as usize] = *operation;
            }
            let held = lessons.iter().filter(|(steps, _)| steps.iter().all(|(key, _)| known[*key as usize])).count();
            if (held - right) as f64 > (f64::from(1u8) - WALK_SURE_SHARE) * lessons.len() as f64 {
                self.program = program;
                return false;
            }
        }
        self.program = program;
        self.said = self.count(lessons, &self.program, None).1;
        true
    }

    fn score(&self, lessons: &[WalkLesson]) -> (usize, usize) {
        let right = lessons.iter().filter(|(steps, answer)| self.result(steps, &self.program).and_then(|got| self.words_of(&got)).is_some_and(|words| words == *answer)).count();
        (right, lessons.len())
    }

    pub fn learn(cases: &[Case], symbols: &mut Symbols) -> Option<Walk> {
        if cases.len() < WALK_SURE_COUNT * TEST_PARTS || !sentence_forms(cases) {
            return None;
        }
        let late = amount_late(cases);
        let mut answers: FirstSightTable<Vec<String>, u32> = FirstSightTable::new();
        for (case, late) in cases.iter().zip(&late) {
            if !late {
                answers.add(answer_words(&case.answer), 1);
            }
        }
        let numbers = answers.rows().iter().all(|(words, _)| is_number_answer(words));
        let all: u32 = answers.rows().iter().map(|(_, count)| *count).sum();
        let top: u32 = answers.by_count().iter().take(TOP_ANSWERS).map(|(_, count)| *count).sum();
        if !numbers && f64::from(top) < WALK_SURE_SHARE * f64::from(all) {
            return None;
        }
        let reader = StateReader::learn(cases, symbols);
        let mut walk = Walk { rule: false, reader, keys: HashMap::new(), texts: Vec::new(), program: Vec::new(), ahead: false, said: Vec::new() };
        let mut forms: Forms = std::mem::take(&mut walk.reader.forms);
        let mut first: Vec<WalkLesson> = Vec::new();
        let mut last: Vec<WalkLesson> = Vec::new();
        for (case, late) in cases.iter().zip(&late) {
            let units = walk.units(symbols, &mut forms, &case.sentences);
            let steps: WalkSteps = units.into_iter().map(|(key, amount)| (walk.intern(key), amount)).collect();
            let lesson = (steps, answer_words(&case.answer));
            if *late {
                last.push(lesson);
            } else {
                first.push(lesson);
            }
        }
        walk.reader.forms = forms;
        let mut seen: Vec<u32> = first.iter().flat_map(|(steps, _)| steps.iter().map(|(key, _)| *key)).collect();
        seen.sort_unstable();
        seen.dedup();
        if seen.is_empty() || seen.len() > MOST_KEYS {
            return None;
        }
        walk.ahead = first.iter().all(|(_, answer)| is_number_answer(answer));
        if !walk.search(&first) {
            return None;
        }
        let (right, count) = walk.score(&first);
        let (right_last, count_last) = walk.score(&last);
        walk.rule = count >= WALK_SURE_COUNT && right as f64 >= WALK_SURE_SHARE * count as f64 && count_last > 0 && right_last as f64 >= WALK_SURE_SHARE * count_last as f64;
        Some(walk)
    }

    pub fn answer(&self, symbols: &Symbols, sentences: &[String]) -> Option<Said> {
        let mut names = Fresh::over(symbols);
        let mut forms = MoreForms::from(&self.reader.forms);
        let units = self.units(&mut names, &mut forms, sentences);
        let mut steps: WalkSteps = Vec::with_capacity(units.len());
        for (key, amount) in units {
            steps.push((*self.keys.get(&key)?, amount));
        }
        let mut trace: Vec<(u8, UnitAmount)> = Vec::new();
        let (across, ahead) = self.run(&steps, &self.program, Some(&mut trace))?;
        let result = if self.ahead { WalkResult::Ahead(ahead) } else { WalkResult::Start(across == Amount::whole(0) && ahead == Amount::whole(0)) };
        let words = self.words_of(&result).filter(|words| !words.is_empty())?;
        let told: Vec<String> = trace
            .iter()
            .map(|(operation, amount)| {
                let name = OPERATION_TEXTS[*operation as usize];
                match amount {
                    UnitAmount::Absent => name.to_string(),
                    UnitAmount::One(amount) => format!("{name} {}", amount.written(RESULT_PLACES).unwrap_or_default()),
                    UnitAmount::Many => format!("{name} many"),
                }
            })
            .collect();
        let place = format!("{} -> the place {} across, {} ahead", told.join("; "), across.written(RESULT_PLACES).unwrap_or_default(), ahead.written(RESULT_PLACES).unwrap_or_default());
        Some(Said { words, note: format!("{place} | {WALK_PARTS} | list") })
    }
}
