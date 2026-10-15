use crate::base::first_sight::FirstSightTable;
use crate::base::lanes::on_lanes;
use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::tools::arrange::read::{MOST_PLACES, Puzzle, ChoiceStatement, choice_mark, puzzle};
use crate::tools::arrange::solve::{Meaning, Test, arrangements, holds};
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::frequent::frequent_answer;
use patterns::{because, source};

pub struct MeaningSearch;
source!(MeaningSearch, "the meaning of each statement form is found as a problem of constraints, by backtracking: in every lesson the told statements and the answer hold and the other choices fail in some arrangement of the things, so a meaning is kept only while nearly every lesson that it touches can still be arranged. Forms that stand in the same lessons are found together; the order of the places is free, so the most frequent form about two things is taken to mean before. The fewest lessons that may stay unexplained are tried first");

const LEAST_PUZZLES: usize = 10;
because!(LEAST_PUZZLES, MeaningSearch, "the tool learns a kind only from this many lessons that it can read or more: with fewer, a meaning of a form rests on chance");

const SLACK_PARTS: usize = 20;
because!(SLACK_PARTS, MeaningSearch, "at most one lesson in this many of a group of forms may stay unexplained by the meanings: a set may hold a few lessons that were written wrong, and no more than a few");

const MOST_NODES: usize = 200_000;
because!(MOST_NODES, MeaningSearch, "the most meanings that the search tries for one group of forms: enough for the groups of the public sets, which are found in a few hundred tries, and a bound on the time of a group that no meanings explain");

const SLACK_GROWTH: usize = 2;
because!(SLACK_GROWTH, MeaningSearch, "the lessons that may stay unexplained grow by this factor at each step, so that few steps reach the most that are allowed");

const ONE_ARRANGEMENT: usize = 1;
because!(ONE_ARRANGEMENT, MeaningSearch, "the arrangements that a test of a lesson looks for: the lesson is explained as soon as some arrangement passes its tests, so the search stops at the first");

const MOST_ARRANGEMENTS: usize = 40_320;
because!(MOST_ARRANGEMENTS, MeaningSearch, "the most arrangements that are counted for an answer: every arrangement of the most things that the tool reads");

const NO_CHOICES: &str = "the arrange tool does not read this kind: its questions do not list choices among things";
because!(NO_CHOICES, MeaningSearch, "what the tool says for lessons whose questions offer no choice among the things of the story");

const NO_MEANINGS: &str = "the arrange tool does not read this kind: no meanings of the statement forms explain the lessons";
because!(NO_MEANINGS, MeaningSearch, "what the tool says when no form gets a meaning");

const NOTE_ARRANGED: &str = "arrangements of the things";
because!(NOTE_ARRANGED, MeaningSearch, "the note of an answer that the arrangements of the things give");

const ARRANGE_NOTE_NONE: &str = "no arrangement or no choice with a meaning";
because!(ARRANGE_NOTE_NONE, MeaningSearch, "the note of an answer for a case that no arrangement explains or whose choices have no meaning");

#[derive(Clone, Debug)]
struct ArrangeLesson {
    puzzle: Puzzle,
    answer: usize,
}
because!(ArrangeLesson, MeaningSearch, "a lesson read for the tool, with the place of its answer among its choices");

fn tests_of(lesson: &ArrangeLesson, meanings: &[Option<Meaning>]) -> Vec<Test> {
    let meaning_of = |statement: &ChoiceStatement| statement.form.and_then(|form| meanings[form as usize]);
    let mut tests: Vec<Test> = lesson.puzzle.told.iter().filter_map(|told| meaning_of(told).map(|meaning| Test { meaning, slots: told.slots.clone(), wanted: true })).collect();
    for (place, (choice, _)) in lesson.puzzle.choices.iter().enumerate() {
        if let Some((meaning, statement)) = choice.as_ref().and_then(|statement| meaning_of(statement).map(|meaning| (meaning, statement))) {
            tests.push(Test { meaning, slots: statement.slots.clone(), wanted: place == lesson.answer });
        }
    }
    tests
}
because!(tests_of, MeaningSearch, "the tests of a lesson under the meanings found so far: every told statement holds, the answer holds, and every other choice fails; a statement whose form has no meaning yet tests nothing");

fn explained(lesson: &ArrangeLesson, meanings: &[Option<Meaning>]) -> bool {
    let tests = tests_of(lesson, meanings);
    let complete = statements(&lesson.puzzle).all(|statement| statement.form.is_some_and(|form| meanings[form as usize].is_some()));
    if !complete {
        return arrangements(lesson.puzzle.things, &tests, ONE_ARRANGEMENT, &mut |_| {}) > 0;
    }
    let told = &tests[..lesson.puzzle.told.len()];
    let all = arrangements(lesson.puzzle.things, &tests, MOST_ARRANGEMENTS, &mut |_| {});
    all > 0 && arrangements(lesson.puzzle.things, told, MOST_ARRANGEMENTS, &mut |_| {}) == all
}
because!(explained, MeaningSearch, "whether a lesson is explained under the meanings found so far: while a statement of it has no meaning, some arrangement must pass all its tests; when every statement has one, the told statements must leave only arrangements in which the answer holds and every other choice fails, so that the told text decides the answer");

fn statements(puzzle: &Puzzle) -> impl Iterator<Item = &ChoiceStatement> {
    puzzle.told.iter().chain(puzzle.choices.iter().filter_map(|(choice, _)| choice.as_ref()))
}
because!(statements, MeaningSearch, "the told statements and the choices of a case that the tool reads");

struct Group {
    order: Vec<u32>,
    lessons: Vec<usize>,
    touched: Vec<Vec<usize>>,
}
because!(Group, MeaningSearch, "forms that stand in the same lessons: the forms in the order of the search, the lessons of the group, and for each form of the order the lessons of the group that hold it");

struct Hunt<'a> {
    lessons: &'a [ArrangeLesson],
    group: &'a Group,
    choices: &'a [Vec<Meaning>],
    meanings: Vec<Option<Meaning>>,
    bad: Vec<bool>,
    slack: usize,
    nodes: usize,
}
because!(Hunt, MeaningSearch, "the state of the search for the meanings of one group: the meaning of every form so far, the lessons that stay unexplained, how many may, and how many meanings were tried");

impl Hunt<'_> {
    fn step(&mut self, at: usize, missed: usize) -> bool {
        if at == self.group.order.len() {
            return true;
        }
        let form = self.group.order[at] as usize;
        for meaning in &self.choices[at] {
            self.nodes += 1;
            if self.nodes > MOST_NODES {
                return false;
            }
            self.meanings[form] = Some(*meaning);
            let mut newly: Vec<usize> = Vec::new();
            for lesson in &self.group.touched[at] {
                if !self.bad[*lesson] && !explained(&self.lessons[*lesson], &self.meanings) {
                    newly.push(*lesson);
                }
            }
            if missed + newly.len() <= self.slack {
                newly.iter().for_each(|lesson| self.bad[*lesson] = true);
                if self.step(at + 1, missed + newly.len()) {
                    return true;
                }
                newly.iter().for_each(|lesson| self.bad[*lesson] = false);
            }
        }
        self.meanings[form] = None;
        false
    }
}

fn groups(lessons: &[ArrangeLesson], counts: &FirstSightTable<u32, u32>) -> Vec<Group> {
    let mut root: Vec<usize> = (0..counts.rows().len()).collect();
    let find = |root: &Vec<usize>, mut at: usize| {
        while root[at] != at {
            at = root[at];
        }
        at
    };
    for lesson in lessons {
        let places: Vec<usize> = statements(&lesson.puzzle).filter_map(|statement| statement.form.and_then(|form| counts.place(&form))).collect();
        for place in places.iter().skip(1) {
            let (one, other) = (find(&root, places[0]), find(&root, *place));
            if one != other {
                root[other.max(one)] = other.min(one);
            }
        }
    }
    let mut found: Vec<Group> = Vec::new();
    let mut head: Vec<Option<usize>> = vec![None; root.len()];
    for place in 0..root.len() {
        let top = find(&root, place);
        let at = *head[top].get_or_insert_with(|| {
            found.push(Group { order: Vec::new(), lessons: Vec::new(), touched: Vec::new() });
            found.len() - 1
        });
        found[at].order.push(counts.rows()[place].0);
    }
    for group in &mut found {
        group.order.sort_by_key(|form| std::cmp::Reverse(counts.get(form).copied().unwrap_or(0)));
    }
    for (number, lesson) in lessons.iter().enumerate() {
        let Some(first) = statements(&lesson.puzzle).find_map(|statement| statement.form.and_then(|form| counts.place(&form))) else { continue };
        let top = find(&root, first);
        if let Some(at) = head[top] {
            found[at].lessons.push(number);
        }
    }
    for group in &mut found {
        group.touched = group.order.iter().map(|form| group.lessons.iter().copied().filter(|lesson| statements(&lessons[*lesson].puzzle).any(|statement| statement.form == Some(*form))).collect()).collect();
    }
    found
}
because!(groups, MeaningSearch, "the groups of forms that stand in the same lessons, as parts of a graph whose edges join the forms of a lesson, each with its forms by falling count (the first of equals in the order of first sight) and its lessons");

fn search_group(lessons: &[ArrangeLesson], group: &Group, forms: usize, unary: &[Meaning]) -> Vec<Option<Meaning>> {
    let arity: Vec<usize> = group.order.iter().map(|form| lessons.iter().flat_map(|lesson| statements(&lesson.puzzle)).find(|statement| statement.form == Some(*form)).map_or(0, |statement| statement.slots.len())).collect();
    let seed = arity.iter().position(|slots| *slots == MOST_PLACES);
    let mut order: Vec<u32> = Vec::with_capacity(group.order.len());
    let mut choices: Vec<Vec<Meaning>> = Vec::with_capacity(group.order.len());
    let mut touched: Vec<Vec<usize>> = Vec::with_capacity(group.order.len());
    if let Some(seed) = seed {
        order.push(group.order[seed]);
        choices.push(vec![Meaning::Before]);
        touched.push(group.touched[seed].clone());
    }
    for (at, form) in group.order.iter().enumerate() {
        if Some(at) == seed {
            continue;
        }
        order.push(*form);
        choices.push(if arity[at] == MOST_PLACES { vec![Meaning::Before, Meaning::After] } else { unary.to_vec() });
        touched.push(group.touched[at].clone());
    }
    let ordered = Group { order, lessons: group.lessons.clone(), touched };
    let most = group.lessons.len() / SLACK_PARTS;
    let mut slack = 0usize;
    loop {
        let mut hunt = Hunt { lessons, group: &ordered, choices: &choices, meanings: vec![None; forms], bad: vec![false; lessons.len()], slack, nodes: 0 };
        if hunt.step(0, 0) {
            return hunt.meanings;
        }
        if slack >= most {
            return vec![None; forms];
        }
        slack = (slack * SLACK_GROWTH).max(1).min(most);
    }
}
because!(search_group, MeaningSearch, "the meanings of the forms of one group: the most frequent form about two things means before, and the other forms are tried in their order with every meaning of their kind; the search keeps the first meanings with which at most the allowed lessons stay unexplained, and allows more lessons step by step up to a small share; no meanings when none are found");

pub struct ArrangeModel {
    mark: String,
    forms: Symbols,
    meanings: Vec<Option<Meaning>>,
    default: Vec<String>,
}
because!(ArrangeModel, MeaningSearch, "what the tool learned: the word that starts the choices, the forms of the statements with their meanings, and the most frequent answer of the lessons for a case that no arrangement explains");

impl Learner for ArrangeModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let none = || Said { words: self.default.clone(), note: ARRANGE_NOTE_NONE.to_string() };
        let Some(read) = puzzle(sentences, question, &self.mark, &mut |text| self.forms.find(text)) else { return none() };
        let meaning_of = |statement: &ChoiceStatement| statement.form.and_then(|form| self.meanings.get(form as usize).copied().flatten());
        let tests: Vec<Test> = read.told.iter().filter_map(|told| meaning_of(told).map(|meaning| Test { meaning, slots: told.slots.clone(), wanted: true })).collect();
        let asked: Vec<Option<(Meaning, &ChoiceStatement)>> = read.choices.iter().map(|(choice, _)| choice.as_ref().and_then(|statement| meaning_of(statement).map(|meaning| (meaning, statement)))).collect();
        let mut holding: Vec<usize> = vec![0; read.choices.len()];
        let found = arrangements(read.things, &tests, MOST_ARRANGEMENTS, &mut |places| {
            for (place, choice) in asked.iter().enumerate() {
                if let Some((meaning, statement)) = choice {
                    holding[place] += usize::from(holds(*meaning, places, &statement.slots, read.things));
                }
            }
        });
        let mut best: Option<usize> = None;
        for (place, count) in holding.iter().enumerate() {
            if asked[place].is_some() && best.is_none_or(|top| *count > holding[top]) {
                best = Some(place);
            }
        }
        match best.filter(|_| found > 0) {
            Some(place) => Said { words: read.choices[place].1.clone(), note: format!("{NOTE_ARRANGED}: {found}; the choice holds in {}", holding[place]) },
            None => none(),
        }
    }
}

pub(super) fn learn_arrange(lessons: Lessons, lanes: usize) -> Learned {
    let Some(mark) = choice_mark(lessons.cases) else { return Err(NO_CHOICES.to_string()) };
    let mut forms = Symbols::new();
    let mut read: Vec<ArrangeLesson> = Vec::new();
    for case in lessons.cases {
        let Some(puzzle) = puzzle(&case.sentences, &case.question, &mark, &mut |text| Some(forms.number(text))) else { continue };
        let truth = answer_words(&case.answer);
        if let Some(answer) = puzzle.choices.iter().position(|(_, said)| *said == truth) {
            read.push(ArrangeLesson { puzzle, answer });
        }
    }
    if read.len() < LEAST_PUZZLES {
        return Err(NO_CHOICES.to_string());
    }
    let mut counts: FirstSightTable<u32, u32> = FirstSightTable::new();
    for lesson in &read {
        for statement in statements(&lesson.puzzle) {
            if let Some(form) = statement.form {
                counts.add(form, 1);
            }
        }
    }
    let most_things = read.iter().map(|lesson| lesson.puzzle.things).max().unwrap_or(0);
    let one_size = read.iter().all(|lesson| lesson.puzzle.things == most_things);
    let mut unary: Vec<Meaning> = (0..most_things).map(Meaning::At).collect();
    if !one_size {
        unary.extend((0..most_things).map(Meaning::FromEnd));
    }
    let found = groups(&read, &counts);
    let parts: Vec<Vec<Option<Meaning>>> = on_lanes(found.len(), lanes, &|group| search_group(&read, &found[group], forms.len(), &unary));
    let mut meanings: Vec<Option<Meaning>> = vec![None; forms.len()];
    for part in parts {
        for (form, meaning) in part.into_iter().enumerate() {
            if meaning.is_some() {
                meanings[form] = meaning;
            }
        }
    }
    if meanings.iter().all(Option::is_none) {
        return Err(NO_MEANINGS.to_string());
    }
    Ok(Box::new(ArrangeModel { mark, forms, meanings, default: frequent_answer(lessons.cases).0 }))
}
because!(learn_arrange, MeaningSearch, "the tool on lessons: the word that starts the choices, every lesson read as a puzzle with the place of its answer, the groups of forms, and the meanings of each group found side by side on the threads; a place counted from the end is tried only when the lessons hold different numbers of things, because with one number it is a place counted from the start");
