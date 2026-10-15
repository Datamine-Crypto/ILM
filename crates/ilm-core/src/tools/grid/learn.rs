use crate::base::first_sight::FirstSightTable;
use crate::base::symbols::Symbols;
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::frequent::frequent_answer;
use crate::tools::grid::read::{GridPuzzle, STATEMENT_THINGS, answer_place, fillers, find_mark, grid_puzzle, joiner, without_fillers};
use crate::tools::grid::solve::{PlaceMeaning, PlaceTest, kind_arrangements, pair_meanings, single_meanings};
use patterns::{because, source};

pub struct GridSearch;
source!(GridSearch, "the meaning of each statement form is found as a problem of constraints, by backtracking: in every lesson the told statements hold and the asked thing stands at the place of the answer in some arrangement, and once every statement has a meaning the told statements leave the asked thing only that place; a meaning is kept only while nearly every lesson that it touches is so explained. The places are numbered, so no direction of the order is free. The fewest lessons that may stay unexplained are tried first");

const GRID_LEAST_PUZZLES: usize = 10;
because!(GRID_LEAST_PUZZLES, GridSearch, "the tool learns a kind only from this many lessons that it can read or more: with fewer, a meaning of a form rests on chance");

const GRID_SLACK_PARTS: usize = 20;
because!(GRID_SLACK_PARTS, GridSearch, "at most one lesson in this many may stay unexplained by the meanings: a set may hold a few lessons that were written wrong, and no more than a few");

const GRID_MOST_NODES: usize = 100_000;
because!(GRID_MOST_NODES, GridSearch, "the most meanings that the search tries: enough for the forms of the public sets, and a bound on the time when no meanings explain the lessons");

const GRID_SLACK_GROWTH: usize = 2;
because!(GRID_SLACK_GROWTH, GridSearch, "the lessons that may stay unexplained grow by this factor at each step");

const FIRST_ARRANGEMENT: usize = 1;
because!(FIRST_ARRANGEMENT, GridSearch, "a test of a lesson stops as soon as an arrangement passes, since that is enough to explain the lesson or to show that another place is open");

const GRID_MOST_ARRANGEMENTS: usize = 5_000;
because!(GRID_MOST_ARRANGEMENTS, GridSearch, "the most arrangements that are counted for an answer: a puzzle that its clues decide has one, and a bound on the time of a story that they leave open");

const GRID_NO_READ: &str = "the grid tool does not read this kind: its questions do not ask for the place of a thing of told lists";
because!(GRID_NO_READ, GridSearch, "what the tool says for lessons that it cannot read as things in places");

const GRID_NO_MEANINGS: &str = "the grid tool does not read this kind: no meanings of the statement forms explain the lessons";
because!(GRID_NO_MEANINGS, GridSearch, "what the tool says when the search finds no meanings");

const GRID_NOTE: &str = "arrangements of the things in their places";
because!(GRID_NOTE, GridSearch, "the note of an answer that the arrangements give");

const GRID_NOTE_NONE: &str = "no arrangement of the things";
because!(GRID_NOTE_NONE, GridSearch, "the note of an answer for a case that no arrangement explains");

struct GridLesson {
    puzzle: GridPuzzle,
    answer: usize,
}
because!(GridLesson, GridSearch, "a lesson read for the tool, with the place of its answer");

fn told_tests(puzzle: &GridPuzzle, meanings: &[Option<PlaceMeaning>]) -> Vec<PlaceTest> {
    puzzle.told.iter().filter_map(|told| told.form.and_then(|form| meanings[form as usize]).map(|meaning| PlaceTest::new(meaning, &told.things, true, puzzle.places))).collect()
}
because!(told_tests, GridSearch, "the tests of the told statements whose forms have a meaning");

fn grid_explained(lesson: &GridLesson, meanings: &[Option<PlaceMeaning>], complete: bool, witness: &mut Vec<usize>) -> bool {
    let puzzle = &lesson.puzzle;
    let mut tests = told_tests(puzzle, meanings);
    tests.push(PlaceTest::new(PlaceMeaning::At(lesson.answer), &[puzzle.asked], true, puzzle.places));
    if witness.is_empty() || !tests.iter().all(|test| test.passes(witness)) {
        let mut found: Option<Vec<usize>> = None;
        kind_arrangements(puzzle.places, puzzle.kinds, &tests, FIRST_ARRANGEMENT, &mut |at| found = Some(at.to_vec()));
        match found {
            Some(at) => *witness = at,
            None => return false,
        }
    }
    if !complete {
        return true;
    }
    if let Some(last) = tests.last_mut() {
        *last = PlaceTest::new(PlaceMeaning::At(lesson.answer), &[puzzle.asked], false, puzzle.places);
    }
    kind_arrangements(puzzle.places, puzzle.kinds, &tests, FIRST_ARRANGEMENT, &mut |_| {}) == 0
}
because!(grid_explained, GridSearch, "whether a lesson is explained under the meanings so far: some arrangement passes its told tests with the asked thing at the place of the answer, and when the meaning of every form of the lesson is decided, no arrangement puts the asked thing elsewhere; the last arrangement found for the lesson is tried first, and a search runs only when it fails a test");

struct GridHunt<'a> {
    lessons: &'a [GridLesson],
    order: &'a [u32],
    choices: &'a [Vec<PlaceMeaning>],
    touched: &'a [Vec<usize>],
    decided: &'a [usize],
    meanings: Vec<Option<PlaceMeaning>>,
    witnesses: Vec<Vec<usize>>,
    bad: Vec<bool>,
    slack: usize,
    nodes: usize,
}
because!(GridHunt, GridSearch, "the state of the search for meanings, where a form that no meaning fits is left with none and its statements test nothing: the forms in their order with the meanings of each and the lessons that hold each, the step after which every form of each lesson is decided, the meaning of every form so far, the last arrangement found for each lesson, the lessons that stay unexplained, how many may, and how many meanings were tried");

impl GridHunt<'_> {
    fn step(&mut self, at: usize, missed: usize) -> bool {
        if at == self.order.len() {
            return true;
        }
        let form = self.order[at] as usize;
        for meaning in &self.choices[at] {
            self.nodes += 1;
            if self.nodes > GRID_MOST_NODES {
                return false;
            }
            self.meanings[form] = Some(*meaning);
            if self.checked_step(at, missed) {
                return true;
            }
        }
        self.meanings[form] = None;
        self.nodes += 1;
        self.nodes <= GRID_MOST_NODES && self.checked_step(at, missed)
    }

    fn checked_step(&mut self, at: usize, missed: usize) -> bool {
        let mut newly: Vec<usize> = Vec::new();
        for lesson in &self.touched[at] {
            if !self.bad[*lesson] && !grid_explained(&self.lessons[*lesson], &self.meanings, self.decided[*lesson] <= at, &mut self.witnesses[*lesson]) {
                newly.push(*lesson);
            }
        }
        if missed + newly.len() > self.slack {
            return false;
        }
        newly.iter().for_each(|lesson| self.bad[*lesson] = true);
        if self.step(at + 1, missed + newly.len()) {
            return true;
        }
        newly.iter().for_each(|lesson| self.bad[*lesson] = false);
        false
    }
}

pub struct GridModel {
    mark: String,
    joined: Option<String>,
    optional: Vec<Vec<String>>,
    forms: Symbols,
    meanings: Vec<Option<PlaceMeaning>>,
    default: Vec<String>,
}
because!(GridModel, GridSearch, "what the tool learned: the word that starts the choices, the word that joins the last piece of a list, the optional runs of the forms, the forms of the statements with their meanings, and the most frequent answer of the lessons");

impl Learner for GridModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let none = || Said { words: self.default.clone(), note: GRID_NOTE_NONE.to_string() };
        let case = crate::cases::Case { sentences: sentences.to_vec(), question: question.to_string(), answer: String::new() };
        let Some(puzzle) = grid_puzzle(&case, &self.mark, self.joined.as_deref(), &mut |text| self.forms.find(&without_fillers(text, &self.optional))) else { return none() };
        let meanings: Vec<Option<PlaceMeaning>> = (0..self.forms.len()).map(|form| self.meanings.get(form).copied().flatten()).collect();
        let tests = told_tests(&puzzle, &meanings);
        let mut held: Vec<usize> = vec![0; puzzle.places];
        let found = kind_arrangements(puzzle.places, puzzle.kinds, &tests, GRID_MOST_ARRANGEMENTS, &mut |at| held[at[puzzle.asked]] += 1);
        let mut best: Option<usize> = None;
        for (place, count) in held.iter().enumerate() {
            if best.is_none_or(|top| *count > held[top]) {
                best = Some(place);
            }
        }
        match best.filter(|_| found > 0) {
            Some(place) => Said { words: puzzle.choices[place].clone(), note: format!("{GRID_NOTE}: {found}; the place holds the asked thing in {}", held[place]) },
            None => none(),
        }
    }
}

pub(super) fn learn_grid(lessons: Lessons, _lanes: usize) -> Learned {
    let Some(mark) = find_mark(lessons.cases) else { return Err(GRID_NO_READ.to_string()) };
    let joined = joiner(lessons.cases, &mark);
    let mut texts = Symbols::new();
    for case in lessons.cases {
        grid_puzzle(case, &mark, joined.as_deref(), &mut |text| Some(texts.number(text)));
    }
    let optional = fillers(texts.names());
    let mut forms = Symbols::new();
    let mut read: Vec<GridLesson> = Vec::new();
    for case in lessons.cases {
        let Some(puzzle) = grid_puzzle(case, &mark, joined.as_deref(), &mut |text| Some(forms.number(&without_fillers(text, &optional)))) else { continue };
        if let Some(answer) = answer_place(case, &puzzle) {
            read.push(GridLesson { puzzle, answer });
        }
    }
    if read.len() < GRID_LEAST_PUZZLES {
        return Err(GRID_NO_READ.to_string());
    }
    let most_places = read.iter().map(|lesson| lesson.puzzle.places).max().unwrap_or(0);
    let mut counts: FirstSightTable<u32, u32> = FirstSightTable::new();
    let mut arity: Vec<usize> = vec![0; forms.len()];
    for lesson in &read {
        for told in &lesson.puzzle.told {
            if let Some(form) = told.form {
                counts.add(form, 1);
                arity[form as usize] = told.things.len();
            }
        }
    }
    let mut order: Vec<u32> = counts.rows().iter().map(|(form, _)| *form).collect();
    order.sort_by_key(|form| std::cmp::Reverse(counts.get(form).copied().unwrap_or(0)));
    let choices: Vec<Vec<PlaceMeaning>> = order.iter().map(|form| if arity[*form as usize] == STATEMENT_THINGS { pair_meanings(most_places) } else { single_meanings(most_places) }).collect();
    let touched: Vec<Vec<usize>> = order.iter().map(|form| (0..read.len()).filter(|lesson| read[*lesson].puzzle.told.iter().any(|told| told.form == Some(*form))).collect()).collect();
    let mut decided: Vec<usize> = vec![0; read.len()];
    for (at, lessons) in touched.iter().enumerate() {
        lessons.iter().for_each(|lesson| decided[*lesson] = at);
    }
    let most = read.len() / GRID_SLACK_PARTS;
    let mut slack = 0usize;
    loop {
        let mut hunt = GridHunt { lessons: &read, order: &order, choices: &choices, touched: &touched, decided: &decided, meanings: vec![None; forms.len()], witnesses: vec![Vec::new(); read.len()], bad: vec![false; read.len()], slack, nodes: 0 };
        let ok = hunt.step(0, 0);
        if ok {
            return Ok(Box::new(GridModel { mark, joined, optional, forms, meanings: hunt.meanings, default: frequent_answer(lessons.cases).0 }));
        }
        if slack >= most {
            return Err(GRID_NO_MEANINGS.to_string());
        }
        slack = (slack * GRID_SLACK_GROWTH).max(1).min(most);
    }
}
because!(learn_grid, GridSearch, "the tool on lessons: the word that starts the choices and the word that joins a list, every lesson read as things in places with the place of its answer, and the meanings of the forms by frequency of the forms, the most frequent first; the tool says that it does not read a kind where it reads too few lessons or finds no meanings");
