use patterns::{because, source};

pub struct SlotPatterns;
source!(SlotPatterns, "question answering by counted patterns over the told sentences of a story, on the twenty tasks of Weston and others: a pattern is the question with slots and up to a few told sentences with slots, each sentence found by a fixed way of pointing from a word that the question or an earlier sentence names, as in the relational pathfinding of Richards and Mooney; the count of a pattern over the lessons says where the answer stands when the pattern fits");

pub struct CodedNames;
source!(CodedNames, "a tagged machine word: the name of a slot, a free place and a literal word are one number whose high bits say which it is, so the form of a sentence is a flat row of numbers that is hashed and compared without a text");

pub const SLOT_LETTERS: &str = "ABCDEFGHIJKLMNOP";
because!(SLOT_LETTERS, SlotPatterns, "the names of the slots of a question, one letter for each; a question with more words of the story than letters keeps its later words as plain words");

pub const FREE: u32 = 0;
because!(FREE, CodedNames, "the code of a place of a form that any word of its kind fills, and of a name that is not there");

pub const LITERAL: u32 = 0x8000_0000;
because!(LITERAL, CodedNames, "the high bit of a code that is a word as it stands; the numbers of words stay below it");

pub const EARLIER: u32 = 0x4000_0000;
because!(EARLIER, CodedNames, "the bit of a code that names a free slot of a sentence that a pattern found: the number of the sentence and the label of the place follow in the lower bits");

pub const STEP_SHIFT: u32 = 24;
because!(STEP_SHIFT, CodedNames, "the bits below the number of the sentence in the name of a slot: room for every label of a place that a set of lessons can have");

pub const LABEL_MASK: u32 = 0x00FF_FFFF;
because!(LABEL_MASK, CodedNames, "the bits of the label in the name of a slot; the highest label is kept for the word that a pointing by kind finds");

pub const SPLIT_STEP: u32 = 63;
because!(SPLIT_STEP, CodedNames, "the number that stands in the place of the sentence in the name of a slot of a split: the highest that its bits hold, far above the sentences of any pattern");

pub const ASKED_SLOT: u32 = 0x8000_0000;
because!(ASKED_SLOT, CodedNames, "the high bit of a place of a question form that is a slot; a place without it is the word as it stands");

pub const DOTS: u32 = u32::MAX;
because!(DOTS, CodedNames, "the mark of a form that leaves the fixed words of its sentence out, in the place where the other forms hold the number of the fixed words");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    Whole,
    All,
    Before,
    After,
    Next,
    Prev,
}
because!(Scope, SlotPatterns, "where a sentence of a pattern is looked for: Whole is the first sentence, found in the whole story; All, Before and After find a later sentence in the whole story, or before or after the sentence that the pattern found last; Next and Prev are the sentence right after or right before that one");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Point {
    Any,
    Kind,
    Last,
    End,
    FreeKind,
    Near,
    NearKind,
    Split,
}
because!(Point, SlotPatterns, "how a sentence is pointed at: Any is the last sentence that holds the named slots, with its form; Kind is the last sentence that holds them and one more word of a kind; Last is the last sentence of a form; End is the last sentence of the story with its form; FreeKind is the last sentence with one word of a kind, for a question with no slot; Near and NearKind are the sentence next to the last one, with its form or with its one word of a kind; Split is no sentence but the answers of a part learner for the candidates of one kind");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    pub scope: Scope,
    pub point: Point,
    pub name: u32,
    pub second: u32,
    pub item: u32,
    pub value: u32,
}
because!(Step, SlotPatterns, CodedNames, "one sentence of a pattern as numbers: where it is looked for, how it is pointed at, the one or two names that it is found from, the form or the kind that it must have, and for a pointing by kind what the found word is (free, or a name that is bound already)");

pub fn asked_name(slot: usize) -> u32 {
    slot as u32 + 1
}
because!(asked_name, CodedNames, "the code of the name of a slot of the question: the slots are counted from one, so that no name is the code of a free place");

pub fn slot_name(step: u32, label: u32) -> u32 {
    EARLIER | (step << STEP_SHIFT) | label
}
because!(slot_name, CodedNames, "the code of the name of a free slot: the number of the sentence of the pattern and the label of the place in it");

pub fn split_slot(place: usize) -> u32 {
    slot_name(SPLIT_STEP, place as u32)
}
because!(split_slot, CodedNames, "the code of the name of a slot of a split: the candidates that are their own answer, in the order of the story");

pub const ALL_SPLIT: u32 = EARLIER | (SPLIT_STEP << STEP_SHIFT) | LABEL_MASK;
because!(ALL_SPLIT, CodedNames, "the part of a place of the answer that stands for every slot of a split together, however many candidates a story has");

pub fn name_step(code: u32) -> u32 {
    (code & !EARLIER) >> STEP_SHIFT
}
because!(name_step, CodedNames, "the number of the sentence in the name of a free slot");

pub const PAIR_CELLS: usize = 2;
because!(PAIR_CELLS, CodedNames, "the cells of one place in the row of a form: its label, and what stands there");
