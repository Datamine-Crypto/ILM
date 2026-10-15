use crate::tools::pattern::classes::{ABSENT, KIND_MARK};
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::PatternWords;
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::WordTexts;
use patterns::{because, source};
use std::collections::HashSet;

pub struct ToldStatements;
source!(ToldStatements, "a told sentence can hold statements, and a rule holds statements of which one is what it gives and the others are what it needs; the forms of the statements are the forms of the questions that are asked often and have few answers, so the reader is counted from the questions and holds no word of a set");

pub struct OppositeForms;
source!(OppositeForms, "a statement form with more fixed words than another statement form is its opposite, as a form with not is the opposite of the form without it; an opposite is read as the absence of its statement, which is negation as failure in the sense of Clark, or as a statement of its own");

pub struct ChoiceByFit;
source!(ChoiceByFit, "each way to read the told sentences is a choice that is kept when the table of answers fits more lessons with it: for each row of the table the lessons that a guess by the shares of the answers of the row gets right, added over the rows, which is the expected accuracy of a guess by the row");

pub const PENDING: u32 = 0x4000_0000;
because!(PENDING, ToldStatements, "the bit of a value that is a place of a kind that nothing has filled yet: a later named word of the sentence, or the last named word of the sentences before, fills it");

pub const TOLD_LEAST: u32 = 5;
because!(TOLD_LEAST, ToldStatements, "the questions that a form needs to be a statement form, and the lessons that a row of the table of answers needs to give an answer: with fewer, a share of answers is not a share");

pub const LEAST_STORIES: usize = 3;
because!(LEAST_STORIES, ToldStatements, "the stories that must hold a word before it is read as an open word or as a word after a name, and the stories that must hold a form of sentence before it can be dropped: a word or a form of one or two stories is chance");

pub const TOLD_ROUNDS: usize = 12;
because!(TOLD_ROUNDS, ToldStatements, "the most rounds in which the rules of one story are used on what rules gave: the chains of the public sets are shorter, and the limit bounds the work for a story whose rules never end");

pub const TOLD_LESSONS: usize = 1000;
because!(TOLD_LESSONS, ChoiceByFit, "the lessons that the reader is counted on: enough for every choice of the reader, and a bound on its work for a large set of lessons");

pub const TRIED_STORIES: usize = 150;
because!(TRIED_STORIES, ChoiceByFit, "the stories on which a choice is tried first: each try reads the stories again, and this many give the fit of a choice with little noise");

pub const MOST_REFERS: usize = 60;
because!(MOST_REFERS, ChoiceByFit, "the words that are tried as words for a named word, the ones with the most stories first: a word for a named word is common, so a few dozen are enough");

pub const MOST_HOLDERS: usize = 40;
because!(MOST_HOLDERS, ChoiceByFit, "the stories with a word on which the choice about that word is tried first, spread over all its stories: a cheap first test before the test on all of them");

pub const FIT_GAIN: f64 = 1.0;
because!(FIT_GAIN, ChoiceByFit, "a choice is taken when the fit rises by more than this many lessons: a smaller rise is within the noise of the guesses");

pub const MOST_MISSES: f64 = 0.1;
because!(MOST_MISSES, ChoiceByFit, "the share of the statements with an answer that the reading may miss before a row that holds such misses is no longer sure");

pub const MOST_ANSWERS: usize = 3;
because!(MOST_ANSWERS, ToldStatements, "the answers that a statement form may have: true, false and one answer for a statement that the story does not decide");

pub const MOST_ALIKE: usize = 3;
because!(MOST_ALIKE, ToldStatements, "the other words of the kind of a fixed word of a statement form that are read as that word: a few forms of one verb, as is and are, and not a kind of many words");

pub const TINY: f64 = 0.000_000_001;
because!(TINY, ChoiceByFit, "a rise of the fit smaller than this is a rounding of the sum of shares and no rise");

pub const QUICK_SHARE: f64 = 0.9;
because!(QUICK_SHARE, ChoiceByFit, "the share of the asked statements that stand in the facts that one answer must have before the reader is tried at all; below it the reader can not tell the statements of the facts from the others");

pub const CHOICE_TURNS: usize = 3;
because!(CHOICE_TURNS, ChoiceByFit, "the turns in which every choice is tried again after a change: the choices settle in a turn or two");

pub type Shape = Vec<u32>;
because!(Shape, ToldStatements, "the form of a read text: a word as its number, a word of a kind as the kind with the bit of a kind");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Statement {
    pub absent: bool,
    pub base: Shape,
    pub values: Vec<u32>,
}
because!(Statement, ToldStatements, "one statement of a told sentence: whether a rule needs it to be absent, its statement form, and its values, each a word, an open place of a kind, or a place that nothing has filled yet");

pub type Known = (Shape, Vec<u32>);
because!(Known, ToldStatements, "a statement with all its places filled by words: its form and its words");

#[derive(Clone, Debug)]
pub struct Implication {
    pub needs: Vec<Statement>,
    pub head: Statement,
}
because!(Implication, ToldStatements, "a told rule: the statements that it needs and the statement that it gives");

pub type TableRow = (bool, bool, bool);
because!(TableRow, ChoiceByFit, "a row of the table of answers: the question asks the absence of a statement; the asked form stands in the facts; its other form stands in the facts");

#[derive(Clone, Debug)]
pub struct Sure {
    pub answer: Vec<u32>,
    pub lessons: u32,
    pub hit: u32,
    pub sure: bool,
}
because!(Sure, ChoiceByFit, "the answer of a row of the table: the words, the lessons of the row, the lessons with this answer, and whether the row is sure by itself or only up to misses of the reading");

#[derive(Clone, Debug, Default)]
pub struct View {
    pub extra: Vec<(u32, u32)>,
    pub new: Vec<u32>,
}
because!(View, ToldStatements, "the kinds of the words of one story beside the kinds that were counted: the kinds of the learner for the words that no lesson held, and the new words that are names by their capital letter");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Switch {
    Last,
    Facts,
    Carry,
    Absent(Shape),
    Refer(u32, u32),
    Start(u32),
    Must(u32),
}
because!(Switch, ChoiceByFit, "one choice of the reader: which part a rule gives; whether a sentence whose parts all name their things is a list of facts; whether a sentence with one open part is given by the parts of the sentence before; whether an opposite is the absence of its statement; whether a word stands for a named word of a kind; whether a single word of a kind starts a part of its own; whether a part with an open place of a kind is dropped");

#[derive(Clone, Debug, Default)]
pub struct ToldRules {
    pub on: bool,
    pub rules_on: bool,
    pub cls: CheapMap<u32, u32>,
    pub kmap: CheapMap<u32, u32>,
    pub alias: CheapMap<u32, u32>,
    pub pair: Vec<(Shape, Shape, Vec<usize>)>,
    pub pairs_of: CheapMap<Shape, Vec<Shape>>,
    pub neg_words: HashSet<u32>,
    pub before: CheapMap<(Shape, usize, Vec<u32>), Shape>,
    pub named: FirstSightTable<u32, FirstSightTable<u32, u32>>,
    pub open: HashSet<u32>,
    pub fixed: CheapMap<u32, u32>,
    pub single: CheapMap<u32, Shape>,
    pub capital: u32,
    pub after_capital: HashSet<u32>,
    pub bad: HashSet<Shape>,
    pub last: bool,
    pub facts_too: bool,
    pub carry: bool,
    pub refer: CheapMap<u32, u32>,
    pub start: HashSet<u32>,
    pub must: HashSet<u32>,
    pub pos: HashSet<Shape>,
    pub neg: CheapMap<Shape, (Shape, Vec<usize>)>,
    pub by_length: Vec<Shape>,
    pub table: FirstSightTable<TableRow, Sure>,
}
because!(ToldRules, ToldStatements, OppositeForms, ChoiceByFit, "the reader of told rules: the kinds of the words of the questions, the kinds of the learner that are read as these kinds, the words that are read as a fixed word, the opposites with their statements and the words that make them, the named words, the open words, the kinds with one word, the form for a single word of each kind, the kind of the words with a capital letter, the choices, the statement forms by length, and the table of answers. class_of gives the kind of a word in a story; shape_of, values and up give the forms and values of statements; flip turns one choice; describe gives the counted reading in words");

pub fn is_word(value: u32) -> bool {
    value & (KIND_MARK | PENDING) == 0
}
because!(is_word, ToldStatements, "a value that is a word, and not an open place or a place that nothing has filled");

pub fn embed(base: &[u32], shape: &[u32]) -> Option<Vec<usize>> {
    let mut keep: Vec<usize> = Vec::new();
    for (at, entry) in shape.iter().enumerate() {
        if keep.len() < base.len() && *entry == base[keep.len()] {
            keep.push(at);
        } else if entry & KIND_MARK != 0 {
            return None;
        }
    }
    (keep.len() == base.len()).then_some(keep)
}
because!(embed, OppositeForms, "the places of the entries of a statement form in a longer form, in their order, when every other entry of the longer form is a fixed word; nothing else");

pub fn shape_name<N: WordTexts>(names: &N, shape: &[u32]) -> String {
    let parts: Vec<String> = shape.iter().map(|entry| if entry & KIND_MARK != 0 { format!("<{}>", names.name(entry & !KIND_MARK)) } else { names.name(*entry).to_string() }).collect();
    parts.join(" ")
}
because!(shape_name, ToldStatements, "the text of a form: a fixed word as it is, a kind as the name of the kind between angle marks");

pub fn in_order<N: WordTexts>(names: &N, shapes: impl IntoIterator<Item = Shape>) -> Vec<Shape> {
    let mut out: Vec<(Shape, String)> = shapes.into_iter().map(|shape| {
        let name = shape_name(names, &shape);
        (shape, name)
    }).collect();
    out.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.1.cmp(&b.1)));
    out.into_iter().map(|(shape, _)| shape).collect()
}
because!(in_order, ToldStatements, "forms in one fixed order: the longest first, so that a longer form is tried before a form that it holds, and forms of one length by their text, so that the order is the same in every run");

pub fn class_text<N: WordTexts>(names: &N, class: u32) -> String {
    names.name(class).to_uppercase()
}
because!(class_text, ToldStatements, "the name of a kind as it is sorted: its most frequent word in capital letters");

pub fn as_written(text: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    let mut from: Option<usize> = None;
    for (at, sign) in text.char_indices() {
        if sign.is_ascii_alphanumeric() {
            from.get_or_insert(at);
            continue;
        }
        if let Some(start) = from.take() {
            out.push(&text[start..at]);
        }
        if WRITTEN_SIGNS.contains(sign) {
            out.push(&text[at..at + sign.len_utf8()]);
        }
    }
    if let Some(start) = from {
        out.push(&text[start..]);
    }
    out
}
because!(as_written, ToldStatements, "the words of a text as they are written, with their capital letters: a run of letters and digits, or one sign of arithmetic or one bracket");

pub const WRITTEN_SIGNS: &str = "-+*/=<>()[]{}";
because!(WRITTEN_SIGNS, ToldStatements, "the signs that are words of their own in a text as it is written, the same signs as in the cut of a text into words");

impl ToldRules {
    pub fn class_of(&self, words: &PatternWords, view: &View, word: u32) -> u32 {
        if view.new.contains(&word) {
            return self.capital;
        }
        if let Some(class) = self.cls.get(&word) {
            return *class;
        }
        if self.alias.contains_key(&word) {
            return ABSENT;
        }
        let kind = words.kind_with(word, &view.extra);
        if kind == ABSENT { ABSENT } else { self.kmap.get(&kind).copied().unwrap_or(ABSENT) }
    }

    pub fn shape_of(&self, words: &PatternWords, view: &View, tokens: &[u32]) -> Shape {
        tokens
            .iter()
            .map(|word| {
                let class = self.class_of(words, view, *word);
                if class == ABSENT { *word } else { KIND_MARK | class }
            })
            .collect()
    }

    pub fn tok(&self, tokens: &[u32]) -> Vec<u32> {
        tokens.iter().map(|word| self.alias.get(word).copied().unwrap_or(*word)).collect()
    }

    pub fn values(&self, shape: &[u32], tokens: &[u32]) -> Vec<u32> {
        shape
            .iter()
            .zip(tokens)
            .map(|(entry, word)| {
                if entry & KIND_MARK == 0 {
                    return *entry;
                }
                match self.fixed.get(&(entry & !KIND_MARK)) {
                    Some(fixed) => *fixed,
                    None if self.open.contains(word) => *entry,
                    None => *word,
                }
            })
            .collect()
    }

    pub fn pair_of(&self, shape: &[u32]) -> Option<&(Shape, Shape, Vec<usize>)> {
        self.pair.iter().find(|(held, _, _)| held == shape)
    }

    pub fn up(&self, shape: &[u32], values: &[u32]) -> Vec<u32> {
        let mut out = shape.to_vec();
        if let Some((_, _, keep)) = self.pair_of(shape) {
            for (at, value) in keep.iter().zip(values) {
                out[*at] = *value;
            }
        }
        out
    }

    pub fn setup<N: WordTexts>(&mut self, names: &N) {
        let shapes: Vec<Shape> = self.pos.iter().cloned().chain(self.neg.keys().filter(|shape| !self.pos.contains(*shape)).cloned()).collect();
        self.by_length = in_order(names, shapes);
    }

    pub fn flip<N: WordTexts>(&mut self, names: &N, switch: &Switch) {
        match switch {
            Switch::Last => self.last = !self.last,
            Switch::Facts => self.facts_too = !self.facts_too,
            Switch::Carry => self.carry = !self.carry,
            Switch::Absent(shape) => {
                if self.neg.remove(shape).is_some() {
                    self.pos.insert(shape.clone());
                } else if let Some((_, base, keep)) = self.pair_of(shape).cloned() {
                    self.neg.insert(shape.clone(), (base, keep));
                    self.pos.remove(shape);
                }
                self.setup(names);
            }
            Switch::Refer(word, class) => {
                if self.refer.get(word) == Some(class) {
                    self.refer.remove(word);
                } else {
                    self.refer.insert(*word, *class);
                }
            }
            Switch::Start(class) => {
                if !self.start.remove(class) {
                    self.start.insert(*class);
                }
            }
            Switch::Must(class) => {
                if !self.must.remove(class) {
                    self.must.insert(*class);
                }
            }
        }
    }

    pub fn describe<N: WordTexts>(&self, names: &N) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut forms: Vec<String> = self.pos.iter().filter(|shape| self.pair_of(shape).is_none()).map(|shape| shape_name(names, shape)).collect();
        forms.sort();
        out.push(format!("statement forms\t{}", forms.join("; ")));
        let mut opposites: Vec<String> = self.pair.iter().map(|(shape, _, _)| format!("{} {}", shape_name(names, shape), if self.neg.contains_key(shape) { "absence" } else if self.pos.contains(shape) { "own" } else { "unread" })).collect();
        opposites.sort();
        out.push(format!("opposite forms\t{}", opposites.join("; ")));
        let mut alias: Vec<(&str, &str)> = self.alias.iter().map(|(word, fixed)| (names.name(*word), names.name(*fixed))).collect();
        alias.sort();
        out.push(format!("alias\t{alias:?}"));
        let mut fixed: Vec<(String, &str)> = self.fixed.iter().map(|(class, word)| (class_text(names, *class), names.name(*word))).collect();
        fixed.sort();
        out.push(format!("fixed\t{fixed:?}"));
        out.push(format!("capital\t{}", if self.capital == ABSENT { String::new() } else { names.name(self.capital).to_string() }));
        let mut open: Vec<&str> = self.open.iter().filter(|word| !self.refer.contains_key(*word)).map(|word| names.name(*word)).collect();
        open.sort();
        out.push(format!("open\t{open:?}"));
        let mut refer: Vec<(&str, &str)> = self.refer.iter().map(|(word, class)| (names.name(*word), names.name(*class))).collect();
        refer.sort();
        out.push(format!("refer\t{refer:?}"));
        let classes = |set: &HashSet<u32>| {
            let mut found: Vec<String> = set.iter().map(|class| class_text(names, *class)).collect();
            found.sort();
            found.join(",")
        };
        out.push(format!("choices\tlast {} facts {} carry {} start {} must {} bad {}", self.last, self.facts_too, self.carry, classes(&self.start), classes(&self.must), self.bad.len()));
        let mut rows: Vec<String> = self.table.rows().iter().map(|((absent, asked, other), sure)| {
            let answer: Vec<&str> = sure.answer.iter().map(|word| names.name(*word)).collect();
            format!("row {absent} {asked} {other}\t{}\t{}\t{}\t{}", answer.join(" "), sure.lessons, sure.hit, sure.sure)
        }).collect();
        rows.sort();
        out.extend(rows);
        out.push(format!("on\t{} {}", self.on, self.rules_on));
        out
    }
}
