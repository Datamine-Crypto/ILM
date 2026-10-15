use crate::tools::parts::language::{PartAnswers, PARTS_OF_FORM, Program, ROW_MOST, WordRow, TURNED, FIRST_OF, LESS_FIRST, AnswerView, Word};
use crate::tools::parts::ordered::Kept;
use crate::tools::parts::says::WordSays;
use crate::tools::pattern::hash::{CheapMap, CheapSet};
use patterns::{because, source};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct FormsInOrder;
source!(FormsInOrder, "a grammar as a rewriting system that is used in a fixed order, in the way of the operator precedence of Floyd: each word of a text is a part with an answer, a fixed word of a form, or left out; a form takes the parts next to its fixed word and gives one part with the answer of its program; the forms have levels, the form of the first level that fits is used at its place most to the left, until one part is left");

pub const ROW_KEY: Word = 0;
because!(ROW_KEY, FormsInOrder, "the key of the form with no fixed word, two parts in a row: the first number of the table of words, which stands for the empty text, so that no word of a text has it and it comes first where keys are put in the order of their texts");

pub const UNSOLVED: Word = 1;
because!(UNSOLVED, FormsInOrder, "the number that stands in a row for the result of the program that a search solves; next after the key of the form with no fixed word, so that no word of a text has it");

pub const UNSOLVED_TURNED: Word = 2;
because!(UNSOLVED_TURNED, FormsInOrder, "the number that stands in a row for the unknown result with its words in the opposite order; the last of the numbers that are kept from the words of the texts");

pub const KEPT_NAMES: &[&str] = &["", " ", "  "];
because!(KEPT_NAMES, FormsInOrder, "the texts of the numbers that are kept from the words: the empty text and texts of blanks alone, which the cut of a text never gives");

pub const LEVEL_PLACES: usize = 2;
because!(LEVEL_PLACES, FormsInOrder, "the places that a form can take at one level of the order of use: a level of its own before that level, or a place in the level beside its forms");

const TWO_FIXED_SPAN: usize = 4;
because!(TWO_FIXED_SPAN, FormsInOrder, "the things of a text that a form with a second fixed word and a part beside each fixed word takes: each fixed word and each part");

const VIEW_WEIGHT: usize = 2;
because!(VIEW_WEIGHT, FormsInOrder, "an operation on an answer weighs as much as this many pieces when programs with the same gain are compared: a turn or a cut is more of a claim than a piece more");

const LOOK_WEIGHT: usize = 3;
because!(LOOK_WEIGHT, FormsInOrder, "a looked-up piece weighs as much as this many pieces, more than an operation: a table is the largest thing that a program can hold");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShapeKind {
    InARow,
    Between,
    After,
    Before,
    BeforeTwo,
    BetweenTwo,
    Around,
    Pair,
    SecondTwo,
}
because!(ShapeKind, FormsInOrder, "the shapes that a form can have, the simpler first: two parts in a row; one fixed word between two parts, after a part, before a part; two fixed words before each of two parts, after each of two parts, round one part; two fixed words in a row that say nothing and leave the text; and two fixed words before each of two parts where the form is kept under the second of them");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FormShape {
    pub kind: ShapeKind,
    pub second: Option<Word>,
}
because!(FormShape, FormsInOrder, "the shape of a form with its second fixed word when it has one. parts is the number of parts that the shape takes; owner is the fixed word that comes first in the text, which owns the tables of the form");

impl FormShape {
    pub fn plain(kind: ShapeKind) -> FormShape {
        FormShape { kind, second: None }
    }

    pub fn two(kind: ShapeKind, second: Word) -> FormShape {
        FormShape { kind, second: Some(second) }
    }

    pub fn parts(&self) -> usize {
        match self.kind {
            ShapeKind::InARow | ShapeKind::Between | ShapeKind::BeforeTwo | ShapeKind::BetweenTwo | ShapeKind::SecondTwo => PARTS_OF_FORM,
            ShapeKind::Pair => 0,
            _ => 1,
        }
    }

    pub fn owner(&self, key: Word) -> Word {
        match (self.kind, self.second) {
            (ShapeKind::SecondTwo, Some(second)) => second,
            _ => key,
        }
    }
}

pub type Slot = Option<Program>;
because!(Slot, FormsInOrder, "the program of a form, or nothing while a search solves it from a lesson");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormState {
    pub shape: FormShape,
    pub program: Slot,
    pub place: usize,
    pub said: Option<WordRow>,
}
because!(FormState, FormsInOrder, "the state of a word that has a form: the shape, the program, the place in the order of use, and what the word says when it says something beside its form");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum State {
    NotKnown,
    Out,
    Table(WordRow),
    Form(FormState),
    Mark,
}
because!(State, FormsInOrder, "the one state of a word: not known, left out, a part with a row of answer words, the fixed word of a form, or the second fixed word of a form of another word. content is the state without its place in the order of use, for the question whether two states are the same thing");

impl State {
    pub fn content(&self) -> State {
        match self {
            State::Form(form) => State::Form(FormState { shape: form.shape, program: form.program.clone(), place: 0, said: form.said.clone() }),
            other => other.clone(),
        }
    }

    pub fn form(shape: FormShape, program: Slot, place: usize, said: Option<WordRow>) -> State {
        State::Form(FormState { shape, program, place, said })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weight {
    Form { shape: ShapeKind, size: isize, program: Program, beside: usize, place: usize },
    Table { length: usize, row: Vec<u32> },
    Other,
}
because!(Weight, FormsInOrder, "the order of two changes with the same gain: a form comes before a table word, and a table word before a word that is left out; of two forms the simpler shape, then the smaller program, then a level of its own before a place beside other forms; of two table words the shorter row, then the order of their texts");

pub fn weight_of(state: &State, rank: &dyn Fn(Word) -> u32) -> Weight {
    match state {
        State::Table(row) => Weight::Table { length: row.len(), row: row.iter().map(|word| rank(*word)).collect() },
        State::Form(FormState { shape, program: Some(program), place, .. }) => {
            let used: HashSet<u8> = program.pieces.iter().map(|piece| piece.slot).filter(|slot| (*slot as usize) < PARTS_OF_FORM).collect();
            let views: usize = program.pieces.iter().map(|piece| piece.view.size()).sum();
            let looks = program.pieces.iter().filter(|piece| piece.looked()).count();
            let size = program.pieces.len() + usize::from(program.times > 1) + VIEW_WEIGHT * views + LOOK_WEIGHT * looks;
            Weight::Form { shape: shape.kind, size: size as isize + shape.parts() as isize - used.len() as isize, program: program.clone(), beside: place % LEVEL_PLACES, place: *place }
        }
        _ => Weight::Other,
    }
}
because!(weight_of, FormsInOrder, "the weight of a state; the size of a form is its pieces, one more for a row that is said several times, the operations and the tables with their weights, and one more for each part that the program leaves out");

pub type ProgramTables = HashMap<usize, HashMap<WordRow, WordRow>>;
because!(ProgramTables, FormsInOrder, "the tables of the looked-up pieces of one program: the number of the piece, then the key, then the word that the answer holds for it");

pub type Lookup = HashMap<(Word, Program), ProgramTables>;
because!(Lookup, FormsInOrder, "the tables of all forms with looked-up pieces: the forms of one first fixed word that have the same program share their tables");

pub enum Stop {
    TooLong,
    NotLooked,
    NoWay,
    FirstUse(PartAnswers),
}
because!(Stop, FormsInOrder, "why the use of a form ended a reading: the row grew past its limit; a table does not hold the key; the lesson cannot be solved for one unknown result; or the reading was asked to stop at the first use of the unknown form and gives its parts");

pub enum Apply<'a> {
    Run,
    Solving(&'a mut Option<PartAnswers>),
    FirstUse,
}
because!(Apply, FormsInOrder, "how a reading uses the forms: it runs their programs; or one program is unknown, its result is a mark in the row and its parts are kept; or it stops at the first use of the unknown form");

#[derive(Default)]
pub struct Meter {
    pub words: Cell<u64>,
    pub tried: Cell<u64>,
}
because!(Meter, FormsInOrder, "the work of a learning run as two counts: the words of the texts that were read, which ends a search on a kind that it cannot read, and the places of answers that the search for one program has tried");

fn run_program(program: &Slot, given: &PartAnswers, tables: Option<&ProgramTables>) -> Result<WordRow, Stop> {
    let Some(program) = program else { return Err(Stop::NoWay) };
    let mut out: WordRow = Vec::new();
    for (number, piece) in program.pieces.iter().enumerate() {
        let Some(part) = given.row(piece.slot as usize) else { return Err(Stop::NotLooked) };
        let row = piece.view.on(part);
        if piece.looked() {
            match tables.and_then(|tables| tables.get(&number)).and_then(|table| table.get(&row)) {
                Some(word) => out.extend(word),
                None => return Err(Stop::NotLooked),
            }
        } else {
            out.extend(row);
        }
    }
    if out.len() * program.times > ROW_MOST {
        return Err(Stop::TooLong);
    }
    Ok(out.repeat(program.times))
}
because!(run_program, FormsInOrder, "the row that a program makes from the answers of its parts; a part that is not there, or a key that the table of a piece does not hold, ends the reading");

fn unsolved_in(row: &[Word]) -> bool {
    row.iter().any(|word| *word == UNSOLVED || *word == UNSOLVED_TURNED)
}
because!(unsolved_in, FormsInOrder, "whether a row holds the mark of the unknown result");

fn follow(row: &[Word], view: AnswerView) -> Result<WordRow, Stop> {
    let mut row: WordRow = row.to_vec();
    for operation in view.operations() {
        if operation == TURNED {
            row.reverse();
            for word in row.iter_mut() {
                *word = if *word == UNSOLVED { UNSOLVED_TURNED } else if *word == UNSOLVED_TURNED { UNSOLVED } else { *word };
            }
            continue;
        }
        let end = if operation == LESS_FIRST || operation == FIRST_OF { row.first() } else { row.last() };
        match end {
            Some(word) if *word != UNSOLVED && *word != UNSOLVED_TURNED => row = AnswerView { first: operation, then: 0 }.on(&row),
            _ => return Err(Stop::NoWay),
        }
    }
    Ok(row)
}
because!(follow, FormsInOrder, "a row that holds the unknown result under a view: a turn gives the row in the opposite order with the mark of the turned result; a cut is followed when the word that it takes or leaves is a known word, and the lesson cannot be solved when an end of the unknown row is cut");

fn solving_program(program: &Slot, given: PartAnswers, tables: Option<&ProgramTables>, kept: &mut Option<PartAnswers>) -> Result<WordRow, Stop> {
    let Some(program) = program else {
        if given.rows.iter().flatten().any(|part| unsolved_in(part)) {
            return Err(Stop::NoWay);
        }
        match kept {
            None => *kept = Some(given),
            Some(first) if *first != given => return Err(Stop::NoWay),
            _ => {}
        }
        return Ok(vec![UNSOLVED]);
    };
    let mut out: WordRow = Vec::new();
    for (number, piece) in program.pieces.iter().enumerate() {
        let Some(part) = given.row(piece.slot as usize) else { return Err(Stop::NoWay) };
        if piece.looked() {
            let row = piece.view.on(part);
            if unsolved_in(&row) {
                return Err(Stop::NoWay);
            }
            match tables.and_then(|tables| tables.get(&number)).and_then(|table| table.get(&row)) {
                Some(word) => out.extend(word),
                None => return Err(Stop::NoWay),
            }
        } else if !piece.view.plain() && unsolved_in(part) {
            out.extend(follow(part, piece.view)?);
        } else {
            out.extend(piece.view.on(part));
        }
    }
    if out.len() * program.times > ROW_MOST {
        return Err(Stop::TooLong);
    }
    Ok(out.repeat(program.times))
}
because!(solving_program, FormsInOrder, "the use of a form while one program is unknown: the unknown form gives the mark of its result and its parts are kept, and a second use with other parts cannot be solved; a known program is run on rows that may hold the mark");

pub fn solved_row(last: &[Word], target: &[Word]) -> Option<WordRow> {
    let marks = last.iter().filter(|word| **word == UNSOLVED || **word == UNSOLVED_TURNED).count();
    if marks == 0 || target.len() + marks < last.len() {
        return None;
    }
    let extra = target.len() + marks - last.len();
    if extra % marks != 0 {
        return None;
    }
    let size = extra / marks;
    let mut row: Option<WordRow> = None;
    let mut at = 0;
    for word in last {
        if *word == UNSOLVED || *word == UNSOLVED_TURNED {
            let mut piece: WordRow = target[at.min(target.len())..(at + size).min(target.len())].to_vec();
            if *word == UNSOLVED_TURNED {
                piece.reverse();
            }
            match &row {
                None => row = Some(piece),
                Some(first) if *first != piece => return None,
                _ => {}
            }
            at += size;
        } else {
            if at >= target.len() || target[at] != *word {
                return None;
            }
            at += 1;
        }
    }
    row.filter(|_| at == target.len())
}
because!(solved_row, FormsInOrder, "the row that the unknown result must be so that the row of a reading is the true answer, or nothing: every mark stands for the same row, and the turned mark for that row turned");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Item {
    Part(WordRow),
    Fixed(Word),
}
because!(Item, FormsInOrder, "one thing of a text while it is read: a part with its answer, or a fixed word of a form");

impl Item {
    fn part(&self) -> Option<&WordRow> {
        match self {
            Item::Part(row) => Some(row),
            Item::Fixed(_) => None,
        }
    }

    fn fixed(&self) -> Option<Word> {
        match self {
            Item::Part(_) => None,
            Item::Fixed(word) => Some(*word),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Grammar {
    pub table: WordSays,
    pub out: Arc<CheapSet<Word>>,
    pub forms: Arc<Kept<Word, (FormShape, Slot)>>,
    pub row: Option<Slot>,
    pub order: Vec<Vec<Word>>,
    pub copy: bool,
    pub copies: bool,
    pub marks: Arc<CheapSet<Word>>,
    pub fixed: Arc<CheapSet<Word>>,
    pub lookup: Arc<Lookup>,
    pub two: bool,
    pub rowed: bool,
    pub looks: bool,
    pub extra: Option<Vec<Word>>,
    pub right: usize,
    pub seen: usize,
    pub stopped: bool,
    pub base: usize,
    level_of: CheapMap<Word, usize>,
    looked: CheapSet<Word>,
    says: bool,
}
because!(Grammar, FormsInOrder, "what is known of a kind of text: what each word says, the words that are left out, the forms with their order of use as a list of levels, the second fixed words, the tables of the looked-up pieces, and whether a word that is not known says itself. state gives the state of a word; with_state gives a copy with another state of one word; place and places say where a form stands and can stand in the order of use; read gives the answer of a text as a row and left gives what is left of a text when no form fits any more");

pub struct Reading<'a> {
    pub strict: bool,
    pub apply: Apply<'a>,
    pub used: Option<&'a mut HashSet<Word>>,
    pub hold: &'a [Word],
}
because!(Reading, FormsInOrder, "how one text is read: strict gives no answer when a word is not known, when a fixed word is left whose form does not fit, or when parts are left with no form for two parts in a row; the use of the forms; the set that takes the words and forms that the reading uses; and words that are not known and stay in the text as fixed words");

impl<'a> Reading<'a> {
    pub fn plain(strict: bool) -> Reading<'a> {
        Reading { strict, apply: Apply::Run, used: None, hold: &[] }
    }

    pub fn solving(kept: &'a mut Option<PartAnswers>) -> Reading<'a> {
        Reading { strict: true, apply: Apply::Solving(kept), used: None, hold: &[] }
    }

    fn note(&mut self, key: Word) {
        if let Some(used) = self.used.as_mut() {
            used.insert(key);
        }
    }

    fn apply(&mut self, program: &Slot, given: PartAnswers, tables: Option<&ProgramTables>) -> Result<WordRow, Stop> {
        match &mut self.apply {
            Apply::Run => run_program(program, &given, tables),
            Apply::Solving(kept) => solving_program(program, given, tables, kept),
            Apply::FirstUse => match program {
                None => Err(Stop::FirstUse(given)),
                Some(_) => run_program(program, &given, tables),
            },
        }
    }
}

impl Grammar {
    pub fn place(&self, key: Word) -> Option<usize> {
        self.order.iter().enumerate().find(|(_, level)| level.contains(&key)).map(|(number, level)| LEVEL_PLACES * number + usize::from(level.len() > 1))
    }

    pub fn places(&self, key: Word, words: Option<&[Word]>) -> Vec<usize> {
        let levels: Vec<Vec<Word>> = self.order.iter().map(|level| level.iter().copied().filter(|other| *other != key).collect::<Vec<Word>>()).filter(|level| !level.is_empty()).collect();
        let every: Vec<usize> = (0..=levels.len()).map(|number| LEVEL_PLACES * number).chain((0..levels.len()).map(|number| LEVEL_PLACES * number + 1)).collect();
        let Some(words) = words else { return every };
        let live: Vec<bool> = levels.iter().map(|level| level.iter().any(|other| *other == ROW_KEY || words.contains(other))).collect();
        let mut seen: Vec<(usize, bool)> = Vec::new();
        let mut found: Vec<usize> = Vec::new();
        for place in every {
            let number = place / LEVEL_PLACES;
            let sort = (live[..number].iter().filter(|alive| **alive).count(), place % LEVEL_PLACES == 1 && live[number]);
            if !seen.contains(&sort) {
                seen.push(sort);
                found.push(place);
            }
        }
        found
    }

    pub fn state(&self, word: Word) -> State {
        if word == ROW_KEY {
            return match &self.row {
                Some(program) => State::form(FormShape::plain(ShapeKind::InARow), program.clone(), self.place(ROW_KEY).unwrap_or_default(), None),
                None => State::NotKnown,
            };
        }
        if let Some((shape, program)) = self.forms.get(&word) {
            return State::form(*shape, program.clone(), self.place(word).unwrap_or_default(), self.table.get(&word).cloned());
        }
        if let Some(row) = self.table.get(&word) {
            return State::Table(row.clone());
        }
        if self.marks.contains(&word) {
            return State::Mark;
        }
        if self.out.contains(&word) { State::Out } else { State::NotKnown }
    }

    pub fn with_state(&self, word: Word, state: &State) -> Grammar {
        let mut next = Grammar {
            table: self.table.clone(),
            out: self.out.clone(),
            forms: self.forms.clone(),
            row: self.row.clone(),
            order: self.order.iter().map(|level| level.iter().copied().filter(|key| *key != word).collect::<Vec<Word>>()).filter(|level| !level.is_empty()).collect(),
            copy: self.copy,
            copies: self.copies,
            marks: self.marks.clone(),
            fixed: self.fixed.clone(),
            lookup: self.lookup.clone(),
            two: self.two,
            rowed: self.rowed,
            looks: self.looks,
            ..Grammar::default()
        };
        if next.fixed.contains(&word) {
            Arc::make_mut(&mut next.fixed).remove(&word);
        }
        if word == ROW_KEY {
            next.row = None;
        } else {
            if next.table.get(&word).is_some() {
                next.table.set(word, None);
            }
            if next.out.contains(&word) {
                Arc::make_mut(&mut next.out).remove(&word);
            }
            if next.forms.holds(&word) {
                Arc::make_mut(&mut next.forms).take(&word);
            }
        }
        match state {
            State::NotKnown => {}
            State::Out => {
                Arc::make_mut(&mut next.out).insert(word);
            }
            State::Mark => {
                Arc::make_mut(&mut next.fixed).insert(word);
            }
            State::Table(row) => {
                next.table.set(word, Some(row.clone()));
            }
            State::Form(form) => {
                if word == ROW_KEY {
                    next.row = Some(form.program.clone());
                } else {
                    Arc::make_mut(&mut next.forms).put(word, (form.shape, form.program.clone()));
                    if let Some(said) = &form.said {
                        next.table.set(word, Some(said.clone()));
                    }
                    next.two = next.two || form.shape.second.is_some();
                    next.rowed = next.rowed || form.said.is_some();
                    next.looks = next.looks || form.program.as_ref().is_some_and(Program::looks);
                }
                let level = form.place / LEVEL_PLACES;
                if form.place % LEVEL_PLACES == 1 {
                    next.order[level].push(word);
                } else {
                    let at = level.min(next.order.len());
                    next.order.insert(at, vec![word]);
                }
            }
        }
        if next.two {
            let mut marks: CheapSet<Word> = next.forms.values().filter_map(|(shape, _)| shape.second).collect();
            marks.extend(next.fixed.iter().copied());
            next.marks = Arc::new(marks);
        } else {
            next.marks = next.fixed.clone();
        }
        next.settle();
        next
    }

    pub fn settle(&mut self) {
        self.level_of = self.order.iter().enumerate().flat_map(|(number, level)| level.iter().map(move |key| (*key, number))).collect();
        self.looked = if self.looks { self.forms.pairs().filter(|(_, (_, program))| program.as_ref().is_some_and(Program::looks)).map(|(key, _)| *key).collect() } else { CheapSet::default() };
        self.says = (self.two || self.rowed || self.looks) && (!self.looked.is_empty() || self.forms.keys().any(|key| self.table.get(key).is_some()) || self.marks.iter().any(|word| self.table.get(word).is_some()));
    }

    pub fn seconds(&self, words: &[Word], word: Word) -> Vec<Word> {
        let Some(at) = words.iter().position(|other| *other == word) else { return Vec::new() };
        let mut found: Vec<Word> = Vec::new();
        for other in &words[at + 1..] {
            if *other != word && !found.contains(other) && (!self.table.get(other).is_some() || self.marks.contains(other)) && !self.forms.holds(other) && !self.out.contains(other) {
                found.push(*other);
            }
        }
        found
    }

    pub fn firsts(&self, words: &[Word], word: Word) -> Vec<Word> {
        let Some(at) = words.iter().position(|other| *other == word) else { return Vec::new() };
        let mut found: Vec<Word> = Vec::new();
        for other in &words[..at] {
            if *other != word && !found.contains(other) && (!self.table.get(other).is_some() || self.marks.contains(other)) && !self.out.contains(other) {
                found.push(*other);
            }
        }
        found
    }

    pub fn unknown(&self, words: &[Word]) -> Vec<Word> {
        let mut found: Vec<Word> = Vec::new();
        if self.copy {
            return found;
        }
        for word in words {
            if !self.table.get(word).is_some() && !self.forms.holds(word) && !self.out.contains(word) && !self.marks.contains(word) && !found.contains(word) {
                found.push(*word);
            }
        }
        found
    }

    pub fn read(&self, words: &[Word], meter: &Meter, mut how: Reading) -> Result<Option<WordRow>, Stop> {
        meter.words.set(meter.words.get() + words.len() as u64);
        let mut items: Vec<Item> = Vec::with_capacity(words.len());
        for word in words {
            let row = self.table.get(word);
            let fixed = self.forms.holds(word) || self.marks.contains(word);
            match row {
                Some(row) if !fixed => {
                    items.push(Item::Part(row.clone()));
                    how.note(*word);
                }
                _ if row.is_some() || fixed || how.hold.contains(word) => items.push(Item::Fixed(*word)),
                _ if self.out.contains(word) => {}
                _ if self.copy => items.push(Item::Part(vec![*word])),
                _ if how.strict => return Ok(None),
                _ => {}
            }
        }
        let left = match self.reduce(items, &mut how) {
            Ok(left) => left,
            Err(Stop::TooLong | Stop::NotLooked) => return Ok(None),
            Err(stop) => return Err(stop),
        };
        if left.is_empty() {
            return Ok(Some(Vec::new()));
        }
        if let [Item::Part(row)] = left.as_slice() {
            return Ok(Some(row.clone()));
        }
        if how.strict {
            return Ok(None);
        }
        let parts: Vec<Item> = left.into_iter().filter(|item| item.part().is_some()).collect();
        match self.reduce(parts, &mut how) {
            Ok(parts) => Ok(Some(parts.iter().filter_map(Item::part).flatten().copied().collect())),
            Err(Stop::TooLong | Stop::NotLooked) => Ok(None),
            Err(stop) => Err(stop),
        }
    }

    pub fn left(&self, words: &[Word], meter: &Meter) -> Option<Vec<Item>> {
        meter.words.set(meter.words.get() + words.len() as u64);
        let mut items: Vec<Item> = Vec::with_capacity(words.len());
        for word in words {
            if let Some(row) = self.table.get(word) {
                items.push(Item::Part(row.clone()));
            } else if self.forms.holds(word) || self.marks.contains(word) {
                items.push(Item::Fixed(*word));
            }
        }
        self.reduce(items, &mut Reading::plain(true)).ok()
    }

    fn fits(&self, items: &[Item], at: usize) -> Option<(usize, Word)> {
        let level = |key: Word, best: Option<usize>| self.level_of.get(&key).copied().filter(|level| best.is_none_or(|top| *level < top));
        let is_part = |place: usize| items.get(place).is_some_and(|item| item.part().is_some());
        let is_text_word = |place: usize, word: Option<Word>| word.is_some() && items.get(place).and_then(Item::fixed) == word;
        let shape = |key: Word| self.forms.get(&key).map(|(shape, _)| *shape);
        let mut best: Option<(usize, Word)> = None;
        let top = |best: &Option<(usize, Word)>| best.map(|(level, _)| level);
        match (&items[at], &items[at + 1]) {
            (Item::Part(_), Item::Part(_)) => {
                if let Some(found) = level(ROW_KEY, None) {
                    best = Some((found, ROW_KEY));
                }
            }
            (Item::Part(_), Item::Fixed(word)) => {
                if let (Some(found), Some(shape)) = (level(*word, None), shape(*word)) {
                    let between = is_part(at + PARTS_OF_FORM) && (shape.kind == ShapeKind::Between || (shape.kind == ShapeKind::BetweenTwo && is_text_word(at + PARTS_OF_FORM + 1, shape.second)));
                    if shape.kind == ShapeKind::After || between {
                        best = Some((found, *word));
                    }
                }
            }
            (Item::Fixed(word), Item::Part(_)) => {
                if let (Some(found), Some(shape)) = (level(*word, None), shape(*word)) {
                    let round = shape.second.is_some() && is_text_word(at + PARTS_OF_FORM, shape.second) && (shape.kind == ShapeKind::Around || (shape.kind == ShapeKind::BeforeTwo && is_part(at + PARTS_OF_FORM + 1)));
                    if shape.kind == ShapeKind::Before || round {
                        best = Some((found, *word));
                    }
                }
                if self.says {
                    if let Some(other) = items.get(at + PARTS_OF_FORM).and_then(Item::fixed).filter(|_| at + PARTS_OF_FORM + 1 < items.len()) {
                        if let (Some(found), Some(shape)) = (level(other, top(&best)), shape(other)) {
                            if shape.kind == ShapeKind::SecondTwo && shape.second == Some(*word) && is_part(at + PARTS_OF_FORM + 1) {
                                best = Some((found, other));
                            }
                        }
                    }
                }
            }
            (Item::Fixed(word), Item::Fixed(next)) => {
                if let (Some(found), Some(shape)) = (level(*word, None), shape(*word)) {
                    if shape.kind == ShapeKind::Pair && shape.second == Some(*next) {
                        best = Some((found, *word));
                    }
                }
            }
        }
        best
    }

    fn reduce(&self, mut items: Vec<Item>, how: &mut Reading) -> Result<Vec<Item>, Stop> {
        loop {
            let mut fitted: Vec<Option<(usize, Word)>> = (0..items.len().saturating_sub(1)).map(|at| self.fits(&items, at)).collect();
            while items.len() > 1 {
                let mut best: Option<(usize, usize, Word)> = None;
                for (at, found) in fitted.iter().enumerate() {
                    if let Some((level, key)) = *found {
                        if best.is_none_or(|(top, _, _)| level < top) {
                            best = Some((level, at, key));
                        }
                    }
                    if best.is_some_and(|(top, _, _)| top == 0) {
                        break;
                    }
                }
                let Some((_, at, key)) = best else { break };
                let before = items.len();
                self.use_form(&mut items, at, key, how)?;
                how.note(key);
                let taken = before - items.len();
                let low = at.saturating_sub(PARTS_OF_FORM + 1);
                let fresh: Vec<Option<(usize, Word)>> = (low..(at + 1).min(items.len().saturating_sub(1))).map(|place| self.fits(&items, place)).collect();
                fitted.splice(low..(at + taken + 1).min(fitted.len()), fresh);
            }
            if !self.says {
                break;
            }
            let said = items.iter().enumerate().find_map(|(place, item)| item.fixed().and_then(|word| self.table.get(&word).map(|row| (place, word, row.clone()))));
            let Some((place, word, row)) = said else { break };
            items[place] = Item::Part(row);
            how.note(word);
        }
        Ok(items)
    }

    fn use_form(&self, items: &mut Vec<Item>, at: usize, key: Word, how: &mut Reading) -> Result<(), Stop> {
        let part = |place: usize| items.get(place).and_then(Item::part).cloned();
        if key == ROW_KEY {
            let given = PartAnswers::of(vec![part(at), part(at + 1)]);
            let Some(program) = &self.row else { return Err(Stop::NoWay) };
            let row = how.apply(program, given, None)?;
            items.splice(at..at + PARTS_OF_FORM, [Item::Part(row)]);
            return Ok(());
        }
        let Some((shape, program)) = self.forms.get(&key) else { return Err(Stop::NoWay) };
        let (mut rows, span) = match shape.kind {
            ShapeKind::After => (vec![part(at)], PARTS_OF_FORM),
            ShapeKind::Between => (vec![part(at), part(at + PARTS_OF_FORM)], PARTS_OF_FORM + 1),
            ShapeKind::Before => (vec![part(at + 1)], PARTS_OF_FORM),
            ShapeKind::Pair => {
                items.drain(at..at + PARTS_OF_FORM);
                return Ok(());
            }
            ShapeKind::Around => (vec![part(at + 1)], PARTS_OF_FORM + 1),
            ShapeKind::BeforeTwo | ShapeKind::SecondTwo => (vec![part(at + 1), part(at + PARTS_OF_FORM + 1)], TWO_FIXED_SPAN),
            _ => (vec![part(at), part(at + PARTS_OF_FORM)], TWO_FIXED_SPAN),
        };
        let mut tables: Option<&ProgramTables> = None;
        if self.says {
            let said = self.table.get(&key).cloned();
            let second = shape.second.and_then(|word| self.table.get(&word).cloned());
            let looked = self.looked.contains(&key);
            if said.is_some() || second.is_some() || looked {
                rows.resize(PARTS_OF_FORM, None);
                rows.push(said);
                rows.push(second);
                if looked {
                    if let Some(program) = program {
                        tables = self.lookup.get(&(shape.owner(key), program.clone()));
                    }
                }
            }
        }
        let row = how.apply(program, PartAnswers::of(rows), tables)?;
        items.splice(at..at + span, [Item::Part(row)]);
        Ok(())
    }

    pub fn turns_and_cuts(&self, things: &[Word]) -> (Vec<(Word, State)>, usize) {
        let mut sorted: Vec<Word> = things.to_vec();
        sorted.sort_unstable();
        let mut shapes: Vec<(Word, State)> = Vec::new();
        let mut views = 0;
        for thing in sorted {
            match self.state(thing) {
                State::Form(form) => {
                    views += form.program.iter().flat_map(|program| program.pieces.iter()).map(|piece| piece.view.size()).sum::<usize>();
                    shapes.push((thing, State::form(form.shape, None, 0, None)));
                }
                State::Table(_) => shapes.push((thing, State::Table(Vec::new()))),
                other => shapes.push((thing, other)),
            }
        }
        (shapes, views)
    }

    pub fn key(&self, things: &[Word]) -> GrammarKey {
        let mut states: Vec<(Word, State)> = things.iter().map(|thing| (*thing, self.state(*thing).content())).collect();
        states.sort_by_key(|(thing, _)| *thing);
        let order: Vec<Vec<Word>> = self.order.iter().map(|level| {
            let mut level = level.clone();
            level.sort_unstable();
            level
        }).collect();
        GrammarKey { states, order }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GrammarKey {
    pub states: Vec<(Word, State)>,
    pub order: Vec<Vec<Word>>,
}
because!(GrammarKey, FormsInOrder, "the states of some things and the order of use: two grammars with one key read every text alike when they differ in nothing else");
