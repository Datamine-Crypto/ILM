use crate::tools::parts::language::{WordRow, AnswerView, Word};
use patterns::{because, source};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub struct FoundPrograms;
source!(FoundPrograms, "the program of a form that is found from the lessons, written as a row of places in the way of the templates of program synthesis by example of Gulwani: a place holds a fixed answer word, the answer of a part, the answer of a part under the operations of the language, or one word that a table gives for one word of a part; the text of a form is a row of fixed words and kinds of parts");

pub type PartKind = i8;
because!(PartKind, FoundPrograms, "the kind of a part of a found form, as a small whole number: the kinds that the lessons show count up from the first, the whole text and a place that any one word can take have numbers of their own below them");

pub const WHOLE: PartKind = 0;
because!(WHOLE, FoundPrograms, "the kind of a form of a whole text, below every kind of a part");

pub const OPEN_KIND: PartKind = -1;
because!(OPEN_KIND, FoundPrograms, "the kind of an open place of a form: any one word stands there and says itself; it is below the kind of a whole text, and it counts as a part where the parts of a text are counted");

pub const RESERVED_FROM: Word = 2147483648;
because!(RESERVED_FROM, FoundPrograms, "the numbers from this one up are no words: they stand for the places of the parts of a form while the form is tested against the other forms, so they never equal a word, a word number never reaches them, and a test that asks whether a thing is a word says no for them");

pub const MARKS_FROM: Word = 3221225472;
because!(MARKS_FROM, FoundPrograms, "the numbers from this one up stand for the answer words that a place of a part says while a form is tested against the other forms: above the numbers of the places, so the two sorts never meet");

pub const MARK_WORDS: usize = 3;
because!(MARK_WORDS, FoundPrograms, "the answer words that the place of a part says while a form is tested: more than one, so that a program that turns or cuts the answer of the part is told from one that does not");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bit {
    Word(Word),
    Part(PartKind),
}
because!(Bit, FoundPrograms, "one place of the text of a found form: a fixed word, or a part of a kind (the open place is a kind too)");

impl Bit {
    pub fn word(&self) -> Option<Word> {
        match self {
            Bit::Word(word) => Some(*word),
            Bit::Part(_) => None,
        }
    }

    pub fn kind(&self) -> Option<PartKind> {
        if let Bit::Part(kind) = self { Some(*kind) } else { None }
    }
}

pub fn is_text_word(word: Word) -> bool {
    word < RESERVED_FROM
}
because!(is_text_word, FoundPrograms, "whether a number in a text is a word and not the place of a part of a form under test");

#[derive(Clone, Debug)]
pub struct LookTable {
    pub pairs: Vec<(Word, Word)>,
    map: HashMap<Word, Word>,
}
because!(LookTable, FoundPrograms, "the table of a looked-up word: its pairs of a key and the word for it, in the order of the texts of the keys, with an index; two tables are the same when their pairs are");

impl LookTable {
    pub fn of(pairs: Vec<(Word, Word)>) -> LookTable {
        let map = pairs.iter().copied().collect();
        LookTable { pairs, map }
    }

    pub fn get(&self, key: Word) -> Option<Word> {
        self.map.get(&key).copied()
    }
}

impl PartialEq for LookTable {
    fn eq(&self, other: &LookTable) -> bool {
        self.pairs == other.pairs
    }
}

impl Eq for LookTable {}

impl Hash for LookTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pairs.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    Word(Word),
    Part(usize),
    Seen(usize, AnswerView),
    Looked(usize, AnswerView),
    Look(usize, AnswerView, Arc<LookTable>),
}
because!(Place, FoundPrograms, "one place of a found program: a fixed answer word; the answer of a part; the answer of a part under a view; a looked-up word whose table is not counted yet; or a looked-up word with its table");

pub type Out = Vec<Place>;
because!(Out, FoundPrograms, "a found program: its places in their order");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormKey {
    pub pattern: Vec<Bit>,
    pub kind: PartKind,
    pub out: Out,
}
because!(FormKey, FoundPrograms, "a found form: the text of the form, the kind of part that it makes (or the whole text), and its program");

pub fn laid_out(out: &[Place], rows: &[WordRow]) -> Option<WordRow> {
    let mut row: WordRow = Vec::new();
    for place in out {
        match place {
            Place::Word(word) => row.push(*word),
            Place::Part(part) => row.extend(rows.get(*part)?),
            Place::Seen(part, view) => row.extend(view.on(rows.get(*part)?)),
            Place::Look(part, view, table) => {
                let key = view.on(rows.get(*part)?);
                let [key] = key.as_slice() else { return None };
                row.push(table.get(*key)?);
            }
            Place::Looked(_, _) => return None,
        }
    }
    Some(row)
}
because!(laid_out, FoundPrograms, "the answer that a found program gives with these answers of its parts, or nothing when a table does not hold the key, or the key is not one word");

pub struct Lexicon<'a> {
    pub names: &'a [String],
    pub extra: &'a [String],
    pub rank: &'a [u32],
}
because!(Lexicon, FoundPrograms, "the texts of the words and the place of each word in the order of the texts, for every choice among equals that compares words: such a choice is made by the texts, so it does not depend on the order in which the words were first seen. cmp_word and cmp_rows compare by that order, and by the texts themselves for a word that was not there when the order was made; repr writes a found form or program as the one text that orders two of them where nothing else does");

impl Lexicon<'_> {
    pub fn name(&self, word: Word) -> &str {
        let place = word as usize;
        match self.names.get(place) {
            Some(name) => name,
            None => self.extra.get(place - self.names.len()).map_or(NO_NAME, String::as_str),
        }
    }

    pub fn cmp_word(&self, one: Word, other: Word) -> Ordering {
        match (self.rank.get(one as usize), self.rank.get(other as usize)) {
            (Some(first), Some(second)) => first.cmp(second),
            _ => self.name(one).cmp(self.name(other)),
        }
    }

    pub fn cmp_rows(&self, one: &[Word], other: &[Word]) -> Ordering {
        for (first, second) in one.iter().zip(other) {
            let found = self.cmp_word(*first, *second);
            if found != Ordering::Equal {
                return found;
            }
        }
        one.len().cmp(&other.len())
    }

    fn quoted(&self, word: Word, out: &mut String) {
        let text = self.name(word);
        if text.contains(SINGLE_QUOTE) && !text.contains(DOUBLE_QUOTE) {
            out.push(DOUBLE_QUOTE);
            out.push_str(text);
            out.push(DOUBLE_QUOTE);
            return;
        }
        out.push(SINGLE_QUOTE);
        for sign in text.chars() {
            if sign == SINGLE_QUOTE || sign == QUOTE_ESCAPE {
                out.push(QUOTE_ESCAPE);
            }
            out.push(sign);
        }
        out.push(SINGLE_QUOTE);
    }

    fn view_text(view: &AnswerView, out: &mut String) {
        let operations: Vec<u8> = view.operations().collect();
        tuple_text(operations.len(), out, |at, out| out.push_str(&operations[at].to_string()));
    }

    fn place_text(&self, place: &Place, out: &mut String) {
        match place {
            Place::Word(word) => self.quoted(*word, out),
            Place::Part(part) => out.push_str(&part.to_string()),
            Place::Seen(part, view) | Place::Looked(part, view) => {
                tuple_text(PAIR_SIZE, out, |at, out| if at == 0 { out.push_str(&part.to_string()) } else { Lexicon::view_text(view, out) });
            }
            Place::Look(part, view, table) => {
                tuple_text(TRIPLE_SIZE, out, |at, out| match at {
                    0 => out.push_str(&part.to_string()),
                    1 => Lexicon::view_text(view, out),
                    _ => tuple_text(table.pairs.len(), out, |row, out| {
                        let (key, word) = table.pairs[row];
                        tuple_text(PAIR_SIZE, out, |side, out| self.quoted(if side == 0 { key } else { word }, out));
                    }),
                });
            }
        }
    }

    pub fn repr_out(&self, out_places: &[Place]) -> String {
        let mut out = String::new();
        tuple_text(out_places.len(), &mut out, |at, out| self.place_text(&out_places[at], out));
        out
    }

    pub fn repr_form(&self, form: &FormKey) -> String {
        let mut out = String::new();
        tuple_text(TRIPLE_SIZE, &mut out, |at, out| match at {
            0 => tuple_text(form.pattern.len(), out, |place, out| match form.pattern[place] {
                Bit::Word(word) => self.quoted(word, out),
                Bit::Part(kind) => out.push_str(&kind.to_string()),
            }),
            1 => out.push_str(&form.kind.to_string()),
            _ => out.push_str(&self.repr_out(&form.out)),
        });
        out
    }
}

const NO_NAME: &str = "";
because!(NO_NAME, FoundPrograms, "the text of a number that is no word: the empty text, which no word has");

const SINGLE_QUOTE: char = '\'';
because!(SINGLE_QUOTE, FoundPrograms, "the quote round the text of a word in the written form of a program, unless the word holds this quote itself");

const DOUBLE_QUOTE: char = '"';
because!(DOUBLE_QUOTE, FoundPrograms, "the quote round the text of a word that holds the other quote, in the written form of a program");

const QUOTE_ESCAPE: char = '\\';
because!(QUOTE_ESCAPE, FoundPrograms, "the sign before a quote or before itself inside a quoted word of the written form of a program");

const PAIR_SIZE: usize = 2;
because!(PAIR_SIZE, FoundPrograms, "a part with its view, and a key with its word, are written as rows of this many things");

const TRIPLE_SIZE: usize = 3;
because!(TRIPLE_SIZE, FoundPrograms, "a looked-up word (part, view, table) and a form (text, kind, program) are written as rows of this many things");

const OPEN_ROW: &str = "(";
because!(OPEN_ROW, FoundPrograms, "the sign that opens a row in the written form of a program");

const CLOSE_ROW: &str = ")";
because!(CLOSE_ROW, FoundPrograms, "the sign that closes a row in the written form of a program");

const BETWEEN_THINGS: &str = ", ";
because!(BETWEEN_THINGS, FoundPrograms, "the signs between two things of a row in the written form of a program");

const ONE_THING: &str = ",";
because!(ONE_THING, FoundPrograms, "the sign after the one thing of a row of one, so that it is told from the thing alone");

fn tuple_text(size: usize, out: &mut String, mut each: impl FnMut(usize, &mut String)) {
    out.push_str(OPEN_ROW);
    for at in 0..size {
        if at > 0 {
            out.push_str(BETWEEN_THINGS);
        }
        each(at, out);
    }
    if size == 1 {
        out.push_str(ONE_THING);
    }
    out.push_str(CLOSE_ROW);
}
because!(tuple_text, FoundPrograms, "a row of things in the written form of a program: the things with a comma and a space between them in round brackets, and a comma after the one thing of a row of one");

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Simple {
    uses_part: bool,
    fewer_words: isize,
    fewer_operations: isize,
    shorter: isize,
    text: String,
}
because!(Simple, FoundPrograms, "the order of two found programs that give the same runs, the simpler last: one that uses a part, then fewer fixed answer words, then fewer operations (a looked-up word counts as two), then fewer places, then the written form; programs are sorted with the largest first");

pub fn simple_rank(out: &[Place]) -> (bool, isize, isize, isize) {
    let words = out.iter().filter(|place| matches!(place, Place::Word(_))).count();
    let operations: usize = out.iter().map(|place| match place {
        Place::Seen(_, view) => view.size(),
        Place::Looked(_, view) | Place::Look(_, view, _) => view.size() + PAIR_SIZE,
        _ => 0,
    }).sum();
    (words < out.len(), -(words as isize), -(operations as isize), -(out.len() as isize))
}
because!(simple_rank, FoundPrograms, "the order of a found program without its written form: one that uses a part, then fewer fixed answer words, then fewer operations (a looked-up word counts as two), then fewer places; for a choice where the first of equals in the order of finding is taken");

pub fn simple(out: &[Place], lexicon: &Lexicon) -> Simple {
    let (uses_part, fewer_words, fewer_operations, shorter) = simple_rank(out);
    Simple { uses_part, fewer_words, fewer_operations, shorter, text: lexicon.repr_out(out) }
}
because!(simple, FoundPrograms, "the order of a found program among the programs that give the same runs, with its written form last");
