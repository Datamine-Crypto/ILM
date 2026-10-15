use crate::base::keys::{NO_NUMBER, padded};
use crate::base::lanes::{hardware_lanes, on_lanes, shards};
use crate::base::symbols::Symbols;
use crate::know::article::{lowered, word_runs};
use crate::know::letters::letters;
use crate::know::reader::{MORE_HITS, MORE_SEEN};
use patterns::{because, source};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct QuestionFrames;
source!(QuestionFrames, "a reader of questions by counts: from a question that a person wrote to its subject and its relation. From the lessons (a question, the name of its subject, the relation) it counts the frames at the two ends of the subject, how often each word stands inside a subject, the form of the question with one mark in the place of its subject for each relation, and the words and pairs of words of the forms of each relation; no relation and no word is named in the program");

pub struct KnownNames;
source!(KnownNames, "with a list of known names (the titles of an encyclopedia), a run of words of a question is of one of a few sorts: a known name that lies in no longer known name of the question, a known name inside a longer one, no known name; the lessons give how much more often a subject is of each sort than another run of words is");

pub struct MeasuredWays;
source!(MeasuredWays, "how sure a relation is, is measured and not assumed: a model of the first part of the lessons reads the rest, and for each way of finding the relation (the form of the question; the words of the form, by how far the first relation is ahead of the second, and apart for a form with a word that no lesson of the chosen relation holds) the questions and the right relations are counted");

const ASKED_LEAST: u32 = 3;
because!(ASKED_LEAST, QuestionFrames, "a frame, a form of a question and a usual word are counted from this many lessons or more: fewer are the chance of one question");

const SUBJECT_MOST: usize = 8;
because!(SUBJECT_MOST, QuestionFrames, "the most words of the subject of a question: the titles of films and books are longer than the names of people, and longer runs are nearly never a subject");

const EDGE_WORDS: usize = 3;
because!(EDGE_WORDS, QuestionFrames, "the words of the longest frame at an end of a subject: the word at the end and the words next to it");

const SUBJECT_MARK: &str = "<x>";
because!(SUBJECT_MARK, QuestionFrames, "the mark in the place of the subject in the form of a question; no word of a question holds these signs");

const START_MARK: &str = "<s>";
because!(START_MARK, QuestionFrames, "the mark before the first word of a question, so that the start of a question is a place of a frame");

const END_MARK: &str = "</s>";
because!(END_MARK, QuestionFrames, "the mark after the last word of a question");

const NEW_MARK: &str = "<new>";
because!(NEW_MARK, QuestionFrames, "the one class of all words that few lesson questions hold: the names of subjects are mostly such words, the words of a question are not");

const WAYS_FROM: usize = 30;
because!(WAYS_FROM, MeasuredWays, "the ways are measured only from this many lessons on: with fewer, the part that is read is too small for a count");

const EARLY_WAYS: usize = 2;
because!(EARLY_WAYS, MeasuredWays, "of the parts of the lessons, this many are learned from before the ways are measured");

const WAYS_OF: usize = 3;
because!(WAYS_OF, MeasuredWays, "the parts that the lessons are cut in for the measure of the ways");

const AHEAD_MOST: u32 = 5;
because!(AHEAD_MOST, MeasuredWays, "the highest step of how far the first relation is ahead of the second: beyond it the first relation is nearly always right");

const AHEAD_ALONE: f64 = 9.0;
because!(AHEAD_ALONE, MeasuredWays, "how far ahead a relation is that has no second relation beside it: more than the highest step");

const NAME_END: char = '\n';
because!(NAME_END, KnownNames, "the sign after each name of a file of known names");

const UTF_BITS: u32 = 8;
because!(UTF_BITS, KnownNames, "the width in bits of the units of the Unicode form that a file of names is written in, named in the error for a file that is not in that form with the words that the reader of a text file of the standard library gives");

const HALVES: usize = 2;
because!(HALVES, KnownNames, "a search by halving cuts its range in this many parts at each step");

const PAIR: usize = 2;
because!(PAIR, QuestionFrames, "the words of a pair of words next to each other");

const FIRST_END: usize = 2;
because!(FIRST_END, QuestionFrames, "the first place before which a run of words can end: after the start mark and one word");

const SORTS: usize = 3;
because!(SORTS, KnownNames, "the sorts of a run of words when known names are given");

const STRONG_PAIRS: usize = 3;
because!(STRONG_PAIRS, QuestionFrames, "the pairs of words of the form of a question that the trace names for a relation found by the words: enough to see why the relation was chosen");

pub const SORT_NAMES: &[&str] = &["a known name that lies in no longer known name of the question", "a known name inside a longer known name", "no known name"];
because!(SORT_NAMES, KnownNames, "the sorts of a run of words in their order, as the trace of a question names them");

const LONGEST: usize = 0;
because!(LONGEST, KnownNames, "the place of the sort of a known name that lies in no longer known name");

const INSIDE: usize = 1;
because!(INSIDE, KnownNames, "the place of the sort of a known name inside a longer known name");

const UNKNOWN: usize = 2;
because!(UNKNOWN, KnownNames, "the place of the sort of a run of words that is no known name");

pub trait Names: Sync {
    fn holds(&self, name: &str) -> bool;
}
because!(Names, KnownNames, "a list of known names that can say whether it holds a name, given as its letters");

#[derive(Default)]
pub struct SortedNames {
    text: Vec<u8>,
    starts: Vec<usize>,
    pub count: usize,
}
because!(SortedNames, KnownNames, "the known names as the bytes of one text with a name on each line, in rising order of their bytes, the start of each line and the number of the names: a name is found by halving; the text is read as bytes in runs on all threads and its lines are found on all threads, each run of the text checked as Unicode on its own, which it can be because a run ends at a line end");

impl SortedNames {
    pub fn read(path: &std::path::Path) -> std::io::Result<SortedNames> {
        let size = std::fs::metadata(path)?.len() as usize;
        let mut text = vec![0u8; size];
        let parts = shards(size, hardware_lanes());
        let mut rest: &mut [u8] = &mut text;
        let mut runs: Vec<(u64, &mut [u8])> = Vec::with_capacity(parts.len());
        for part in &parts {
            let (run, after) = rest.split_at_mut(part.len());
            runs.push((part.start as u64, run));
            rest = after;
        }
        let read: Vec<std::io::Result<()>> = std::thread::scope(|scope| {
            let hands: Vec<_> = runs
                .into_iter()
                .map(|(from, run)| {
                    scope.spawn(move || -> std::io::Result<()> {
                        let mut file = std::fs::File::open(path)?;
                        std::io::Seek::seek(&mut file, std::io::SeekFrom::Start(from))?;
                        std::io::Read::read_exact(&mut file, run)
                    })
                })
                .collect();
            hands.into_iter().map(|hand| hand.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))).collect()
        });
        read.into_iter().collect::<std::io::Result<()>>()?;
        let pieces = shards(text.len(), hardware_lanes());
        let found: Vec<Option<Vec<usize>>> = on_lanes(pieces.len(), hardware_lanes(), &|piece| {
            let range = pieces[piece].clone();
            let ends: Vec<usize> = text[range.clone()].iter().enumerate().filter(|(_, sign)| char::from(**sign) == NAME_END).map(|(at, _)| range.start + at + 1).collect();
            let from = if piece == 0 { 0 } else { text[..range.start].iter().rposition(|sign| char::from(*sign) == NAME_END).map_or(0, |at| at + 1) };
            let to = ends.last().copied().unwrap_or(from).max(from);
            let to = if piece + 1 == pieces.len() { text.len() } else { to };
            std::str::from_utf8(&text[from.min(to)..to]).is_ok().then_some(ends)
        });
        let mut starts = vec![0];
        for ends in found {
            let Some(ends) = ends else { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("stream did not contain valid UTF-{UTF_BITS}"))) };
            starts.extend(ends);
        }
        Ok(SortedNames { count: starts.len(), text, starts })
    }

    fn line(&self, number: usize) -> &[u8] {
        let end = self.starts.get(number + 1).map_or(self.text.len(), |next| next - 1);
        &self.text[self.starts[number]..end]
    }
}

impl Names for SortedNames {
    fn holds(&self, name: &str) -> bool {
        let (mut low, mut high) = (0usize, self.starts.len());
        while low < high {
            let middle = low + (high - low) / HALVES;
            match self.line(middle).cmp(name.as_bytes()) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return true,
            }
        }
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Edge {
    pub left: bool,
    pub words: [u32; EDGE_WORDS],
}
because!(Edge, QuestionFrames, "a frame at one end of a run of words: the side, and its words in the order of the question");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Way {
    Form,
    Words { ahead: u32, strange: bool },
}
because!(Way, MeasuredWays, "the way by which the relation of a question was found: the form of the question as the lessons hold it, or the words of the form, with how far the first relation is ahead of the second and whether the form holds a word that no lesson of that relation holds");

#[derive(Clone, Copy, Debug)]
pub struct Asking<'a> {
    pub question: &'a str,
    pub subject: &'a str,
    pub relation: &'a str,
}
because!(Asking, QuestionFrames, "a lesson of the reader of questions: the question as a person wrote it, the name of its subject, and its relation");

#[derive(Clone, Debug, Default)]
pub struct Read {
    pub subject: String,
    pub first: usize,
    pub end: usize,
    pub relations: Vec<u32>,
    pub form: Vec<u32>,
    pub by_form: usize,
    pub way: Option<Way>,
    pub sure: (u32, u32),
    pub left: Option<Edge>,
    pub right: Option<Edge>,
    pub sort: Option<usize>,
}
because!(Read, QuestionFrames, "what the reader found in a question: the subject and its places; all relations in the order of the reader, with the number of them that the form of the question gave, and the form of the question with the mark of the subject; the way of the first relation and the two counts of how sure that way is; the frames at the two ends of the subject and its sort of name");

#[derive(Default)]
pub struct Questions {
    pub words: Symbols,
    pub relations: Symbols,
    pub usual: Vec<bool>,
    pub edges: HashMap<Edge, (u32, u32)>,
    pub stands: HashMap<u32, (u32, u32)>,
    pub forms: HashMap<Vec<u32>, Vec<(u32, u32)>>,
    pub features: Vec<HashMap<(u32, u32), u32>>,
    pub of_relation: Vec<u32>,
    pub total: Vec<u32>,
    pub vocabulary: usize,
    pub base_left: f64,
    pub base_right: f64,
    pub sort_weight: Vec<f64>,
    pub sort_counts: Vec<(u32, u32, u32, u32)>,
    pub ways: HashMap<Way, (u32, u32)>,
    pub used: usize,
    marks: Marks,
}
because!(Questions, QuestionFrames, KnownNames, MeasuredWays, "what the reader of questions counted: the words and the relations as numbers; the usual words; for each kept frame its hits and readings; for each class of a word its places and its places inside a subject; for each kept form the lessons of each relation; for each relation its words and pairs, its lessons and the sum of its counts; the share of all places that start and that end a subject; the weight and the counts of each sort of name; the measured ways. save writes it to a file and load reads it back as it was; learn counts it from lessons in runs on all threads, with the measured ways counted on a thread of their own beside it; count_run counts one run of lessons; ways_of reads the later lessons with a model of the early ones on all threads and sums the counts, so they do not depend on the number of threads; read gives the subject and the relations of a question");

pub struct RunCounts;
source!(RunCounts, "the lessons are counted in runs on all threads: each run of lessons, in their order, counts what one pass counts, with its relations in the order of their first lesson of the run; the runs are then summed in their order, so a relation gets the number that one pass gives it and the relations of a form stand in the order of their first lesson, and the counts are the same on every number of threads");

#[derive(Default)]
struct RunCount<'a> {
    used: usize,
    stands: HashMap<u32, (u32, u32)>,
    edges: HashMap<Edge, (u32, u32)>,
    subject_sorts: [u32; SORTS],
    other_sorts: [u32; SORTS],
    relations: Vec<&'a str>,
    of_relation: Vec<u32>,
    features: Vec<HashMap<(u32, u32), u32>>,
    forms: HashMap<Vec<u32>, Vec<(u32, u32)>>,
}
because!(RunCount, RunCounts, "what one run of lessons counts: the lessons with their subject in the question, the places and the frames of the ends, the sorts of names, the relations of the run in the order of first sight with their lessons and their words, and the forms with the relations of the run in their order");

pub struct SavedReader;
source!(SavedReader, "a reader of questions that was learned once is kept as a plain file beside the table that it serves, every count and share as it was learned: the words and the relations in the order of their numbers, so each number is the same after reading, the relations of each form in their order, and each share as the bits of its double; read back, it reads every question as the learned reader does");

const SAVED_FIELD: char = '\t';
because!(SAVED_FIELD, SavedReader, "the sign between the fields of a line of a saved reader");

const SAVED_ITEM: char = ' ';
because!(SAVED_ITEM, SavedReader, "the sign between the items of a list inside one field of a saved reader");

const MARKS_TAG: &str = "marks";
because!(MARKS_TAG, SavedReader, "the line with the numbers of the four marks");

const WORD_TAG: &str = "word";
because!(WORD_TAG, SavedReader, "a line with one word, in the order of the numbers of the words");

const RELATION_TAG: &str = "relation";
because!(RELATION_TAG, SavedReader, "a line with one relation, its lessons and the sum of its counts, in the order of the numbers of the relations");

const USUAL_TAG: &str = "usual";
because!(USUAL_TAG, SavedReader, "the line with the number of words that the usual words are counted over and the numbers of the usual words");

const EDGE_TAG: &str = "edge";
because!(EDGE_TAG, SavedReader, "a line with a kept frame of an end of a subject and its hits and readings");

const STAND_TAG: &str = "stand";
because!(STAND_TAG, SavedReader, "a line with a class of a word and its places and its places inside a subject");

const FORM_TAG: &str = "form";
because!(FORM_TAG, SavedReader, "a line with a kept form of a question and its relations with their lessons, in their order");

const FEATURE_TAG: &str = "feature";
because!(FEATURE_TAG, SavedReader, "a line with a word or a pair of words of the forms of a relation and its count");

const WAY_TAG: &str = "way";
because!(WAY_TAG, SavedReader, "a line with a measured way and its right and all questions");

const SIZES_TAG: &str = "sizes";
because!(SIZES_TAG, SavedReader, "the line with the vocabulary and the lessons used, and the bits of the shares of the frames that start and end a subject");

const SORT_TAG: &str = "sort";
because!(SORT_TAG, SavedReader, "a line with the bits of the weight of a sort of name and its four counts");

const FORM_WAY: &str = "form";
because!(FORM_WAY, SavedReader, "the way of the form of the question in a line of a measured way");

const WORDS_WAY: &str = "words";
because!(WORDS_WAY, SavedReader, "the way of the words of the form in a line of a measured way");

fn saved_number<T: std::str::FromStr>(field: Option<&str>) -> std::io::Result<T> {
    field.and_then(|text| text.parse::<T>().ok()).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "a saved reader with a field that is no number"))
}
because!(saved_number, SavedReader, "a number of a field of a saved reader, or the fault of a file that is not whole");

fn saved_list(field: Option<&str>) -> std::io::Result<Vec<u32>> {
    field.unwrap_or_default().split(SAVED_ITEM).filter(|item| !item.is_empty()).map(|item| saved_number::<u32>(Some(item))).collect()
}
because!(saved_list, SavedReader, "the numbers of a list inside one field of a saved reader");

#[derive(Clone, Copy, Debug, Default)]
struct Marks {
    subject: u32,
    start: u32,
    end: u32,
    new: u32,
}
because!(Marks, QuestionFrames, "the numbers of the four marks among the words");

pub fn words_of(text: &str) -> Vec<String> {
    let mut low = String::new();
    word_runs(text)
        .into_iter()
        .map(|(start, end)| {
            lowered(&text[start..end], &mut low);
            low.clone()
        })
        .collect()
}
because!(words_of, QuestionFrames, "the words of a question in small letters");

fn find(words: &[String], name: &str) -> Option<(usize, usize)> {
    let size = name.chars().count();
    for first in 0..words.len() {
        let mut run = String::new();
        for (end, word) in words.iter().enumerate().skip(first) {
            run.push_str(&letters(word));
            if run == name {
                return Some((first, end + 1));
            }
            if run.chars().count() >= size {
                break;
            }
        }
    }
    None
}
because!(find, QuestionFrames, "the place of a name in the words of a question, compared by letters: the first run of words whose letters are the letters of the name");

fn left_edges(tokens: &[u32], at: usize) -> Vec<Edge> {
    let mut found: Vec<Edge> = Vec::new();
    for before in (1..EDGE_WORDS).rev() {
        if at >= before {
            found.push(Edge { left: true, words: padded(&tokens[at - before..=at]) });
        }
    }
    found.push(Edge { left: true, words: padded(&tokens[at - 1..at]) });
    found
}
because!(left_edges, QuestionFrames, "the frames of the start of a run at a place, the longest first: the first word of the run with the two words or the one word before it, and the word before the run alone");

fn right_edges(tokens: &[u32], at: usize) -> Vec<Edge> {
    let mut found: Vec<Edge> = Vec::new();
    for after in (1..EDGE_WORDS).rev() {
        if at + after <= tokens.len() {
            found.push(Edge { left: false, words: padded(&tokens[at - 1..at + after]) });
        }
    }
    found.push(Edge { left: false, words: padded(&tokens[at..=at]) });
    found
}
because!(right_edges, QuestionFrames, "the frames of the end of a run before a place, the longest first: the last word of the run with the two words or the one word after it, and the word after the run alone");

fn features(form: &[u32]) -> Vec<(u32, u32)> {
    form.iter().map(|word| (*word, NO_NUMBER)).chain(form.windows(PAIR).map(|pair| (pair[0], pair[1]))).collect()
}
because!(features, QuestionFrames, "the words of a form and its pairs of words next to each other");

fn sorts(words: &[String], names: &dyn Names) -> HashMap<(usize, usize), usize> {
    let size = words.len();
    let mut known: HashSet<(usize, usize)> = HashSet::new();
    for first in 0..size {
        let mut run = String::new();
        for last in first..(first + SUBJECT_MOST).min(size) {
            run.push_str(&letters(&words[last]));
            if names.holds(&run) {
                known.insert((first + 1, last + 1 + 1));
            }
        }
    }
    let mut found: HashMap<(usize, usize), usize> = HashMap::new();
    for first in 1..=size {
        for end in first + 1..=(first + SUBJECT_MOST).min(size + 1) {
            let sort = if !known.contains(&(first, end)) {
                UNKNOWN
            } else if known.iter().any(|(other, last)| *other <= first && end <= *last && (*other, *last) != (first, end)) {
                INSIDE
            } else {
                LONGEST
            };
            found.insert((first, end), sort);
        }
    }
    found
}
because!(sorts, KnownNames, "the sort of every run of words of a question up to the most words of a subject, by the places of its first word and of the word after its last");

impl Questions {
    fn classes(&self, words: &[String]) -> Vec<u32> {
        let mut tokens = vec![self.marks.start];
        tokens.extend(words.iter().map(|word| self.words.find(word).filter(|number| self.usual.get(*number as usize).copied().unwrap_or(false)).unwrap_or(self.marks.new)));
        tokens.push(self.marks.end);
        tokens
    }

    fn ways_of(lessons: &[Asking], names: Option<&dyn Names>, lanes: usize) -> HashMap<Way, (u32, u32)> {
        let cut = lessons.len() * EARLY_WAYS / WAYS_OF;
        let inner = Questions::learn(&lessons[..cut], names, false, lanes);
        let later = &lessons[cut..];
        let pieces = shards(later.len(), lanes);
        let counted: Vec<HashMap<Way, (u32, u32)>> = on_lanes(pieces.len(), lanes, &|piece| {
            let mut mine: HashMap<Way, (u32, u32)> = HashMap::new();
            for lesson in &later[pieces[piece].clone()] {
                let Some(got) = inner.read(lesson.question, names) else { continue };
                let (Some(way), Some(first)) = (got.way, got.relations.first()) else { continue };
                let count = mine.entry(way).or_insert((0, 0));
                count.0 += u32::from(inner.relations.name(*first) == lesson.relation);
                count.1 += 1;
            }
            mine
        });
        let mut ways: HashMap<Way, (u32, u32)> = HashMap::new();
        for mine in counted {
            for (way, (right, all)) in mine {
                let count = ways.entry(way).or_insert((0, 0));
                count.0 += right;
                count.1 += all;
            }
        }
        ways
    }

    fn count_run<'a>(&self, lessons: &[Asking<'a>], asked: &[Vec<String>], names: Option<&dyn Names>) -> RunCount<'a> {
        let mut run = RunCount::default();
        for (lesson, words) in lessons.iter().zip(asked) {
            let name = letters(lesson.subject);
            let Some(span) = (if name.is_empty() { None } else { find(words, &name) }) else { continue };
            run.used += 1;
            let tokens = self.classes(words);
            let (first, end) = (span.0 + 1, span.1 + 1);
            for place in 1..tokens.len() - 1 {
                let count = run.stands.entry(tokens[place]).or_insert((0, 0));
                count.0 += 1;
                count.1 += u32::from(first <= place && place < end);
                for one in left_edges(&tokens, place) {
                    let count = run.edges.entry(one).or_insert((0, 0));
                    count.0 += u32::from(place == first);
                    count.1 += 1;
                }
            }
            for place in FIRST_END..tokens.len() {
                for one in right_edges(&tokens, place) {
                    let count = run.edges.entry(one).or_insert((0, 0));
                    count.0 += u32::from(place == end);
                    count.1 += 1;
                }
            }
            if let Some(names) = names {
                for (span_of, sort) in sorts(words, names) {
                    if span_of == (first, end) {
                        run.subject_sorts[sort] += 1;
                    } else {
                        run.other_sorts[sort] += 1;
                    }
                }
            }
            let relation = match run.relations.iter().position(|known| *known == lesson.relation) {
                Some(place) => place,
                None => {
                    run.relations.push(lesson.relation);
                    run.of_relation.push(0);
                    run.features.push(HashMap::new());
                    run.relations.len() - 1
                }
            };
            run.of_relation[relation] += 1;
            let form: Vec<u32> = tokens[..first].iter().chain(std::iter::once(&self.marks.subject)).chain(tokens[end..].iter()).copied().collect();
            for feature in features(&form) {
                *run.features[relation].entry(feature).or_insert(0) += 1;
            }
            let by = run.forms.entry(form).or_default();
            match by.iter_mut().find(|(known, _)| *known as usize == relation) {
                Some(count) => count.1 += 1,
                None => by.push((relation as u32, 1)),
            }
        }
        run
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let mut out = BufWriter::new(std::fs::File::create(path)?);
        let field = SAVED_FIELD;
        let joined = |numbers: &mut dyn Iterator<Item = u32>| numbers.map(|number| number.to_string()).collect::<Vec<String>>().join(&SAVED_ITEM.to_string());
        writeln!(out, "{MARKS_TAG}{field}{}{field}{}{field}{}{field}{}", self.marks.subject, self.marks.start, self.marks.end, self.marks.new)?;
        for number in 0..self.words.len() as u32 {
            writeln!(out, "{WORD_TAG}{field}{}", self.words.name(number))?;
        }
        for number in 0..self.relations.len() {
            writeln!(out, "{RELATION_TAG}{field}{}{field}{}{field}{}", self.relations.name(number as u32), self.of_relation[number], self.total[number])?;
        }
        let usual = joined(&mut (0..self.usual.len() as u32).filter(|number| self.usual[*number as usize]));
        writeln!(out, "{USUAL_TAG}{field}{}{field}{usual}", self.usual.len())?;
        writeln!(out, "{SIZES_TAG}{field}{}{field}{}{field}{}{field}{}", self.vocabulary, self.used, self.base_left.to_bits(), self.base_right.to_bits())?;
        for (weight, (mine, all_mine, other, all_other)) in self.sort_weight.iter().zip(&self.sort_counts) {
            writeln!(out, "{SORT_TAG}{field}{}{field}{mine}{field}{all_mine}{field}{other}{field}{all_other}", weight.to_bits())?;
        }
        let mut edges: Vec<(&Edge, &(u32, u32))> = self.edges.iter().collect();
        edges.sort_unstable();
        for (edge, (hit, seen)) in edges {
            writeln!(out, "{EDGE_TAG}{field}{}{field}{}{field}{hit}{field}{seen}", u32::from(edge.left), joined(&mut edge.words.iter().copied()))?;
        }
        let mut stands: Vec<(&u32, &(u32, u32))> = self.stands.iter().collect();
        stands.sort_unstable();
        for (class, (all, inside)) in stands {
            writeln!(out, "{STAND_TAG}{field}{class}{field}{all}{field}{inside}")?;
        }
        let mut forms: Vec<_> = self.forms.iter().collect();
        forms.sort_unstable();
        for (form, by) in forms {
            writeln!(out, "{FORM_TAG}{field}{}{field}{}", joined(&mut form.iter().copied()), joined(&mut by.iter().flat_map(|(relation, count)| [*relation, *count])))?;
        }
        for (relation, by) in self.features.iter().enumerate() {
            let mut counts: Vec<(&(u32, u32), &u32)> = by.iter().collect();
            counts.sort_unstable();
            for ((first, second), count) in counts {
                writeln!(out, "{FEATURE_TAG}{field}{relation}{field}{first}{field}{second}{field}{count}")?;
            }
        }
        let mut ways: Vec<(&Way, &(u32, u32))> = self.ways.iter().collect();
        ways.sort_unstable();
        for (way, (right, all)) in ways {
            let (name, ahead, strange) = match way {
                Way::Form => (FORM_WAY, 0, false),
                Way::Words { ahead, strange } => (WORDS_WAY, *ahead, *strange),
            };
            writeln!(out, "{WAY_TAG}{field}{name}{field}{ahead}{field}{}{field}{right}{field}{all}", u32::from(strange))?;
        }
        out.flush()
    }

    pub fn load(path: &Path) -> std::io::Result<Questions> {
        let text = std::fs::read_to_string(path)?;
        let mut model = Questions::default();
        for line in text.lines() {
            let mut fields = line.split(SAVED_FIELD);
            let tag = fields.next().unwrap_or_default();
            match tag {
                MARKS_TAG => model.marks = Marks { subject: saved_number(fields.next())?, start: saved_number(fields.next())?, end: saved_number(fields.next())?, new: saved_number(fields.next())? },
                WORD_TAG => {
                    model.words.number(fields.next().unwrap_or_default());
                }
                RELATION_TAG => {
                    model.relations.number(fields.next().unwrap_or_default());
                    model.of_relation.push(saved_number(fields.next())?);
                    model.total.push(saved_number(fields.next())?);
                    model.features.push(HashMap::new());
                }
                USUAL_TAG => {
                    model.usual = vec![false; saved_number(fields.next())?];
                    for number in saved_list(fields.next())? {
                        model.usual[number as usize] = true;
                    }
                }
                SIZES_TAG => {
                    model.vocabulary = saved_number(fields.next())?;
                    model.used = saved_number(fields.next())?;
                    model.base_left = f64::from_bits(saved_number(fields.next())?);
                    model.base_right = f64::from_bits(saved_number(fields.next())?);
                }
                SORT_TAG => {
                    model.sort_weight.push(f64::from_bits(saved_number(fields.next())?));
                    model.sort_counts.push((saved_number(fields.next())?, saved_number(fields.next())?, saved_number(fields.next())?, saved_number(fields.next())?));
                }
                EDGE_TAG => {
                    let left = saved_number::<u32>(fields.next())? != 0;
                    let words = saved_list(fields.next())?;
                    model.edges.insert(Edge { left, words: padded(&words) }, (saved_number(fields.next())?, saved_number(fields.next())?));
                }
                STAND_TAG => {
                    model.stands.insert(saved_number(fields.next())?, (saved_number(fields.next())?, saved_number(fields.next())?));
                }
                FORM_TAG => {
                    let form = saved_list(fields.next())?;
                    let by: Vec<(u32, u32)> = saved_list(fields.next())?.chunks(PAIR).map(|pair| (pair[0], pair[pair.len() - 1])).collect();
                    model.forms.insert(form, by);
                }
                FEATURE_TAG => {
                    let relation: usize = saved_number(fields.next())?;
                    let key = (saved_number(fields.next())?, saved_number(fields.next())?);
                    let count = saved_number(fields.next())?;
                    model.features.get_mut(relation).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "a saved reader with a count of a relation that it does not name"))?.insert(key, count);
                }
                WAY_TAG => {
                    let name = fields.next().unwrap_or_default();
                    let ahead: u32 = saved_number(fields.next())?;
                    let strange = saved_number::<u32>(fields.next())? != 0;
                    let way = if name == FORM_WAY { Way::Form } else { Way::Words { ahead, strange } };
                    model.ways.insert(way, (saved_number(fields.next())?, saved_number(fields.next())?));
                }
                _ => {}
            }
        }
        Ok(model)
    }

    pub fn learn(lessons: &[Asking], names: Option<&dyn Names>, measure_ways: bool, lanes: usize) -> Questions {
        if measure_ways && lessons.len() >= WAYS_FROM {
            let (ways, mut model) = std::thread::scope(|scope| {
                let ways = scope.spawn(|| Questions::ways_of(lessons, names, lanes));
                let model = Questions::learn(lessons, names, false, lanes);
                (ways.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault)), model)
            });
            model.ways = ways;
            return model;
        }
        let mut model = Questions::default();
        model.marks = Marks { subject: model.words.number(SUBJECT_MARK), start: model.words.number(START_MARK), end: model.words.number(END_MARK), new: model.words.number(NEW_MARK) };
        let asked: Vec<Vec<String>> = lessons.iter().map(|lesson| words_of(lesson.question)).collect();
        let mut frequency: Vec<u32> = Vec::new();
        for words in &asked {
            let mut mine: Vec<u32> = words.iter().map(|word| model.words.number(word)).collect();
            mine.sort_unstable();
            mine.dedup();
            frequency.resize(model.words.len(), 0);
            mine.iter().for_each(|number| frequency[*number as usize] += 1);
        }
        model.usual = frequency.iter().map(|count| *count >= ASKED_LEAST).collect();
        let pieces = shards(lessons.len(), lanes);
        let runs: Vec<RunCount> = on_lanes(pieces.len(), lanes, &|piece| model.count_run(&lessons[pieces[piece].clone()], &asked[pieces[piece].clone()], names));
        let mut edges: HashMap<Edge, (u32, u32)> = HashMap::new();
        let mut forms: HashMap<Vec<u32>, Vec<(u32, u32)>> = HashMap::new();
        let mut subject_sorts = [0u32; SORTS];
        let mut other_sorts = [0u32; SORTS];
        for run in runs {
            model.used += run.used;
            for (token, (all, inside)) in run.stands {
                let count = model.stands.entry(token).or_insert((0, 0));
                count.0 += all;
                count.1 += inside;
            }
            for (one, (hit, seen)) in run.edges {
                let count = edges.entry(one).or_insert((0, 0));
                count.0 += hit;
                count.1 += seen;
            }
            for sort in 0..SORTS {
                subject_sorts[sort] += run.subject_sorts[sort];
                other_sorts[sort] += run.other_sorts[sort];
            }
            let mut global: Vec<u32> = Vec::with_capacity(run.relations.len());
            for (name, (lessons_of, features_of)) in run.relations.iter().zip(run.of_relation.iter().zip(run.features)) {
                let relation = model.relations.number(name) as usize;
                if relation == model.of_relation.len() {
                    model.of_relation.push(0);
                    model.features.push(HashMap::new());
                }
                model.of_relation[relation] += lessons_of;
                for (feature, count) in features_of {
                    *model.features[relation].entry(feature).or_insert(0) += count;
                }
                global.push(relation as u32);
            }
            let mut run_forms: Vec<(Vec<u32>, Vec<(u32, u32)>)> = run.forms.into_iter().collect();
            run_forms.sort_unstable();
            for (form, mine) in run_forms {
                let by = forms.entry(form).or_default();
                for (local, count) in mine {
                    let relation = global[local as usize];
                    match by.iter_mut().find(|(known, _)| *known == relation) {
                        Some(held) => held.1 += count,
                        None => by.push((relation, count)),
                    }
                }
            }
        }
        let base = |left: bool| {
            let single = edges.iter().filter(|(one, _)| one.left == left && one.words[1] == NO_NUMBER);
            let (hit, seen) = single.fold((0u32, 0u32), |sum, (_, count)| (sum.0 + count.0, sum.1 + count.1));
            f64::from(hit) / f64::from(seen.max(1))
        };
        model.base_left = base(true);
        model.base_right = base(false);
        model.forms = forms.into_iter().filter(|(_, by)| by.iter().map(|(_, count)| *count).sum::<u32>() >= ASKED_LEAST).collect();
        let (mine, other): (u32, u32) = (subject_sorts.iter().sum(), other_sorts.iter().sum());
        if names.is_some() && mine > 0 && other > 0 {
            for sort in [LONGEST, INSIDE, UNKNOWN] {
                model.sort_weight.push(((f64::from(subject_sorts[sort]) + MORE_HITS) / (f64::from(mine) + MORE_SEEN)) / ((f64::from(other_sorts[sort]) + MORE_HITS) / (f64::from(other) + MORE_SEEN)));
                model.sort_counts.push((subject_sorts[sort], mine, other_sorts[sort], other));
            }
        }
        model.edges = edges.into_iter().filter(|(_, count)| count.1 >= ASKED_LEAST).collect();
        let all: HashSet<(u32, u32)> = model.features.iter().flat_map(|by| by.keys().copied()).collect();
        model.vocabulary = all.len() + 1;
        model.total = model.features.iter().map(|by| by.values().sum()).collect();
        model
    }

    fn side(&self, edges: &[Edge], left: bool) -> (f64, Option<Edge>) {
        for one in edges {
            if let Some((hit, seen)) = self.edges.get(one) {
                return ((f64::from(*hit) + MORE_HITS) / (f64::from(*seen) + MORE_SEEN), Some(*one));
            }
        }
        (if left { self.base_left } else { self.base_right }, None)
    }

    fn subject(&self, tokens: &[u32], words: &[String], names: Option<&dyn Names>) -> Option<(usize, usize, Option<Edge>, Option<Edge>, Option<usize>)> {
        let size = tokens.len();
        let lefts: Vec<(f64, Option<Edge>)> = (0..size).map(|place| if (1..size - 1).contains(&place) { self.side(&left_edges(tokens, place), true) } else { (f64::default(), None) }).collect();
        let rights: Vec<(f64, Option<Edge>)> = (0..size).map(|place| if (FIRST_END..size).contains(&place) { self.side(&right_edges(tokens, place), false) } else { (f64::default(), None) }).collect();
        let odds: Vec<f64> = (0..size)
            .map(|place| {
                if !(1..size - 1).contains(&place) {
                    return f64::default();
                }
                let (stands, inside) = self.stands.get(&tokens[place]).copied().unwrap_or((0, 0));
                ((f64::from(inside) + MORE_HITS) / (f64::from(stands - inside) + MORE_HITS)).ln()
            })
            .collect();
        let sort_of = match names {
            Some(names) if !self.sort_weight.is_empty() => Some(sorts(words, names)),
            _ => None,
        };
        let mut best: Option<(f64, usize, usize)> = None;
        for first in 1..size - 1 {
            let mut inner = f64::default();
            for end in first + 1..=(first + SUBJECT_MOST).min(size - 1) {
                inner += odds[end - 1];
                let mut score = lefts[first].0.ln() + rights[end].0.ln() + inner;
                if let Some(sort_of) = &sort_of {
                    score += self.sort_weight[sort_of[&(first, end)]].ln();
                }
                if best.is_none_or(|(top, _, _)| score > top) {
                    best = Some((score, first, end));
                }
            }
        }
        best.map(|(_, first, end)| (first, end, lefts[first].1, rights[end].1, sort_of.map(|sort_of| sort_of[&(first, end)])))
    }

    fn ordered(&self, tokens: &[u32], first: usize, end: usize) -> (Vec<u32>, usize, Option<Way>, Vec<u32>) {
        let form: Vec<u32> = tokens[..first].iter().chain(std::iter::once(&self.marks.subject)).chain(tokens[end..].iter()).copied().collect();
        let mut order: Vec<u32> = Vec::new();
        let mut way: Option<Way> = None;
        if let Some(held) = self.forms.get(&form) {
            let mut by = held.clone();
            by.sort_by(|one, other| other.1.cmp(&one.1));
            order.extend(by.iter().map(|(relation, _)| *relation));
            way = Some(Way::Form);
        }
        let by_form = order.len();
        let all: u32 = self.of_relation.iter().sum();
        let found = features(&form);
        let mut scored: Vec<(f64, u32)> = Vec::new();
        for (relation, lessons) in self.of_relation.iter().enumerate() {
            let total = f64::from(self.total[relation]) + (self.vocabulary as f64);
            let mut score = (f64::from(*lessons) / f64::from(all)).ln();
            for feature in &found {
                score += ((f64::from(self.features[relation].get(feature).copied().unwrap_or(0)) + MORE_HITS) / total).ln();
            }
            scored.push((score, relation as u32));
        }
        scored.sort_by(|one, other| other.0.total_cmp(&one.0).then_with(|| self.relations.name(other.1).cmp(self.relations.name(one.1))));
        if way.is_none() && !scored.is_empty() {
            let ahead = if scored.len() > 1 { scored[0].0 - scored[1].0 } else { AHEAD_ALONE };
            let mine = &self.features[scored[0].1 as usize];
            let marks = [self.marks.subject, self.marks.start, self.marks.end, self.marks.new];
            let strange = form.iter().any(|word| !marks.contains(word) && mine.get(&(*word, NO_NUMBER)).copied().unwrap_or(0) == 0);
            way = Some(Way::Words { ahead: (ahead as u32).min(AHEAD_MOST), strange });
        }
        for (_, relation) in scored {
            if !order[..by_form].contains(&relation) {
                order.push(relation);
            }
        }
        (order, by_form, way, form)
    }

    pub fn read(&self, question: &str, names: Option<&dyn Names>) -> Option<Read> {
        let words = words_of(question);
        if words.is_empty() {
            return None;
        }
        let tokens = self.classes(&words);
        let (first, end, left, right, sort) = self.subject(&tokens, &words, names)?;
        let (relations, by_form, way, form) = self.ordered(&tokens, first, end);
        let sure = way.and_then(|way| self.ways.get(&way)).copied().unwrap_or((0, 0));
        Some(Read { subject: words[first - 1..end - 1].join(" "), first, end, relations, form, by_form, way, sure, left, right, sort })
    }

    pub fn relation_of<'a>(&'a self, read: &Read) -> Option<&'a str> {
        read.relations.first().map(|relation| self.relations.name(*relation))
    }

    fn edge_text(&self, edge: &Edge) -> String {
        edge.words.iter().filter(|word| **word != NO_NUMBER).map(|word| self.words.name(*word)).collect::<Vec<&str>>().join(" ")
    }

    pub fn relation_note(&self, read: &Read) -> String {
        let Some(relation) = read.relations.first() else { return String::new() };
        if read.by_form > 0 {
            let held = self.forms.get(&read.form).map_or(&[][..], Vec::as_slice);
            let count = held.iter().find(|(known, _)| known == relation).map_or(0, |(_, count)| *count);
            let all: u32 = held.iter().map(|(_, count)| *count).sum();
            return format!("the form of the question, {count} of {all} lessons");
        }
        let mine = &self.features[*relation as usize];
        let mut pairs: Vec<(u32, &str, &str)> = read.form.windows(PAIR).map(|pair| (mine.get(&(pair[0], pair[1])).copied().unwrap_or(0), self.words.name(pair[0]), self.words.name(pair[1]))).collect();
        pairs.sort_by(|one, other| other.cmp(one));
        let strong: Vec<String> = pairs.iter().take(STRONG_PAIRS).map(|(count, first, second)| format!("{first} {second} ({count})")).collect();
        format!("the words of the form; its pairs with the most lessons of this relation: {}", strong.join(", "))
    }

    pub fn read_trace(&self, read: &Read) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        let before = match &read.left {
            Some(edge) => {
                let (hit, seen) = self.edges.get(edge).copied().unwrap_or((0, 0));
                format!("; before it: {} ({hit} of {seen} such places of the lessons start a subject)", self.edge_text(edge))
            }
            None => "; no frame before it".to_string(),
        };
        lines.push(format!("subject: the words {} to {} of the question{before}", read.first, read.end - 1));
        lines.push(match &read.right {
            Some(edge) => {
                let (hit, seen) = self.edges.get(edge).copied().unwrap_or((0, 0));
                format!("after it: {} ({hit} of {seen} such places of the lessons end a subject)", self.edge_text(edge))
            }
            None => "no frame after it".to_string(),
        });
        if let Some(sort) = read.sort {
            let (mine, all_mine, other, all_other) = self.sort_counts[sort];
            lines.push(format!("{} ({mine} of {all_mine} subjects of the lessons are of this sort, {other} of {all_other} other runs of words)", SORT_NAMES[sort]));
        }
        if let Some(relation) = read.relations.first() {
            lines.push(format!("relation {}: {}", self.relations.name(*relation), self.relation_note(read)));
            if read.sure.1 > 0 {
                lines.push(format!("how sure the relation is: of {} held lesson questions whose relation was found this way, {} were right ({:.none$}%)", read.sure.1, read.sure.0, crate::base::shares::share(read.sure.0 as usize, read.sure.1 as usize), none = 0));
            }
        }
        lines
    }

    pub fn way_text(way: Option<Way>) -> String {
        match way {
            None => String::new(),
            Some(Way::Form) => "the form of the question".to_string(),
            Some(Way::Words { ahead, strange }) => format!("the words of the form|{ahead}|{}", if strange { "with a word that no lesson of this relation holds" } else { "every word stands in lessons of this relation" }),
        }
    }
}

pub struct Shares {
    pub subject: f64,
    pub relation: f64,
    pub both: f64,
}
because!(Shares, QuestionFrames, "what the reader gets right of some held questions, in hundredths: the subject (compared by letters), the relation, and both");

pub fn held_shares(model: &Questions, held: &[Asking], names: Option<&dyn Names>) -> Shares {
    let (mut subject, mut relation, mut both) = (0usize, 0usize, 0usize);
    for case in held {
        let got = model.read(case.question, names);
        let same_subject = got.as_ref().is_some_and(|got| letters(&got.subject) == letters(case.subject));
        let same_relation = got.as_ref().is_some_and(|got| model.relation_of(got) == Some(case.relation));
        subject += usize::from(same_subject);
        relation += usize::from(same_relation);
        both += usize::from(same_subject && same_relation);
    }
    let share = |count: usize| crate::base::shares::share(count, held.len().max(1));
    Shares { subject: share(subject), relation: share(relation), both: share(both) }
}
because!(held_shares, QuestionFrames, "the held questions read by a model: how often the subject, the relation and both are right");
