use crate::base::symbols::Symbols;
use patterns::{because, source};
use std::cmp::Ordering;

pub struct FrameKeys;
source!(FrameKeys, "what stands around a word of a request, as keys, each row from its widest key to its fullest: of the word itself its class in plain text, its class of values and the word; of each side of its frame the word before and the two words before, the word after and the two words after, the slot before; and the words a few places away. A word that the lessons hold once or never is written as a new word in a frame; a side is also counted with a hidden value in the place of a word of values or of a rare word, so that what follows the first word of a new name is counted from all the names of the lessons");

const FRAME_SECOND: usize = 2;
because!(FRAME_SECOND, FrameKeys, "the place of the second word before or after a word: the longer key of a side holds it, and the words from this place on are the marks a few places away");

const FRAME_FAR: usize = 5;
because!(FRAME_FAR, FrameKeys, "the words up to this many places before and after a word are marks of its place: further words seldom say what a word is");

const FRAME_START: &str = "<";
because!(FRAME_START, FrameKeys, "the word before the first word of a request, and the slot before its first word");

const FRAME_END: &str = ">";
because!(FRAME_END, FrameKeys, "the word after the last word of a request");

const FRAME_NEW: &str = "<new>";
because!(FRAME_NEW, FrameKeys, "a word of a frame that the lessons hold once or never; no word of a text can be this mark");

const FRAME_HIDDEN: &str = "<value>";
because!(FRAME_HIDDEN, FrameKeys, "a word beside a place that is a word of values or a rare word, written as a value whose word is hidden; no word of a text can be this mark");

pub(super) const FRAME_TABLE: u32 = 0;
because!(FRAME_TABLE, FrameKeys, "the place of the table of word classes in a key of a class: a run has one table");

pub(super) const SLOT_START: u32 = 0;
because!(SLOT_START, FrameKeys, "the slot before the first word of a request, in the key of the slot before");

pub(super) const SLOT_NONE: u32 = 1;
because!(SLOT_NONE, FrameKeys, "no slot before a word, in the key of the slot before; the slots of the kind have the numbers after it");

pub(super) const FRAME_SIDES: usize = 3;
because!(FRAME_SIDES, FrameKeys, "the sides of a frame: the words before, the words after, the slot before");

pub(super) const SIDE_LEFT: usize = 0;
because!(SIDE_LEFT, FrameKeys, "the place of the side of the words before");

pub(super) const SIDE_RIGHT: usize = 1;
because!(SIDE_RIGHT, FrameKeys, "the place of the side of the words after");

pub(super) const SIDE_SLOT: usize = 2;
because!(SIDE_SLOT, FrameKeys, "the place in the sides of a frame of the slot that the word before fills, which says whether a run goes on");

const FRAME_SHORT: usize = 3;
because!(FRAME_SHORT, FrameKeys, "a key of the word alone, or of the hidden word with one side of one word, is at most this long; such a key is kept with one reading, because it gives the shares of a word");

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub(super) enum FramePart {
    Class,
    Values,
    Word,
    Hidden,
    Left,
    TwoLeft,
    Right,
    TwoRight,
    Slot,
    ClassLeft,
    ClassRight,
    Behind,
    Ahead,
}
because!(FramePart, FrameKeys, "the sorts of the parts of a key; a key is a run of numbers whose parts each begin with their sort, so two different keys are two different runs");

pub(super) type FrameKey = Vec<u32>;
because!(FrameKey, FrameKeys, "one key: the run of its parts, each part its sort and its values");

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FrameMarks {
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) new: u32,
    pub(super) hidden: u32,
}
because!(FrameMarks, FrameKeys, "the numbers of the marks that a frame writes in the place of a word");

impl FrameMarks {
    pub(super) fn new(symbols: &mut Symbols) -> FrameMarks {
        let [start, end, new, hidden] = [FRAME_START, FRAME_END, FRAME_NEW, FRAME_HIDDEN].map(|mark| symbols.number(mark));
        FrameMarks { start, end, new, hidden }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FrameWord {
    pub(super) word: u32,
    pub(super) seen: u32,
    pub(super) hidden: u32,
    pub(super) new: bool,
    pub(super) values: Option<u32>,
    pub(super) class: Option<u32>,
}
because!(FrameWord, FrameKeys, "one word of a request as a frame needs it: its number; how a frame writes it as a neighbour (the word, or the mark of a new word) and with a hidden value (the mark of a hidden value for a word of values or a rare word); whether it is new; the class of values that it fills, for a word that is not new; and its class in plain text");

#[derive(Clone, Debug, Default)]
pub(super) struct PlaceKeys {
    pub(super) word: Vec<FrameKey>,
    pub(super) sides: [Vec<FrameKey>; FRAME_SIDES],
    pub(super) around: Vec<FrameKey>,
}
because!(PlaceKeys, FrameKeys, "the keys of one place: the row of the word, the rows of the three sides, and the words a few places away in their order");

fn frame_push(row: &mut Vec<FrameKey>, key: FrameKey) {
    if !row.contains(&key) {
        row.push(key);
    }
}
because!(frame_push, FrameKeys, "a key at the end of a row when the row does not hold it yet, so that a key stands in a row once, at its first place");

pub(super) fn frame_of(words: &[FrameWord], at: usize, before: u32, lesson: bool, marks: &FrameMarks, classes: bool, order: &dyn Fn(u32, u32) -> Ordering) -> PlaceKeys {
    let me = &words[at];
    let near = |back: usize, ahead: usize| if back == 0 { words.get(at + ahead).copied() } else { at.checked_sub(back).map(|place| words[place]) };
    let seen = |word: Option<FrameWord>, edge: u32| word.map_or(edge, |word| word.seen);
    let hidden = |word: Option<FrameWord>, edge: u32| word.map_or(edge, |word| word.hidden);
    let (one_before, two_before, one_after, two_after) = (near(1, 0), near(FRAME_SECOND, 0), near(0, 1), near(0, FRAME_SECOND));
    let mut frame = PlaceKeys::default();
    if me.new || (lesson && me.values.is_some()) {
        frame.word.push(vec![FramePart::Hidden as u32]);
    }
    if let Some(class) = me.class {
        frame.word.push(vec![FramePart::Class as u32, FRAME_TABLE, class]);
    }
    if !me.new {
        if let Some(values) = me.values {
            frame.word.push(vec![FramePart::Values as u32, values]);
        }
        frame.word.push(vec![FramePart::Word as u32, me.word]);
    }
    let sides = [(FramePart::Left, FramePart::TwoLeft, one_before, two_before, marks.start, true), (FramePart::Right, FramePart::TwoRight, one_after, two_after, marks.end, false)];
    for (side, (one, two, next, further, edge, behind)) in sides.into_iter().enumerate() {
        let (short, short_hidden) = (seen(next, edge), hidden(next, edge));
        let (long, long_hidden) = (seen(further, edge), hidden(further, edge));
        let pair = |first: u32, second: u32| if behind { vec![two as u32, second, first] } else { vec![two as u32, first, second] };
        let row = &mut frame.sides[side];
        if lesson || short == marks.new {
            frame_push(row, vec![one as u32, short_hidden]);
        }
        frame_push(row, vec![one as u32, short]);
        if lesson || short == marks.new || long == marks.new {
            frame_push(row, pair(short_hidden, long_hidden));
        }
        frame_push(row, pair(short, long));
        if classes && let Some(class) = next.and_then(|word| word.class) {
            let part = if behind { FramePart::ClassLeft } else { FramePart::ClassRight };
            row.insert(0, vec![part as u32, FRAME_TABLE, class]);
        }
    }
    frame.sides[SIDE_SLOT].push(vec![FramePart::Slot as u32, before]);
    let far: Vec<FrameKey> = (at.saturating_sub(FRAME_FAR)..at.saturating_sub(1)).map(|place| vec![FramePart::Behind as u32, words[place].seen]).chain((at + FRAME_SECOND..words.len().min(at + 1 + FRAME_FAR)).map(|place| vec![FramePart::Ahead as u32, words[place].seen])).collect();
    for key in far {
        frame_push(&mut frame.around, key);
    }
    frame.around.sort_by(|left, right| (left[0] == FramePart::Behind as u32).cmp(&(right[0] == FramePart::Behind as u32)).then_with(|| order(left[1], right[1])));
    frame
}
because!(frame_of, FrameKeys, "the keys of the word at a place, with the slot before it. The row of the word: a hidden word for a new word (and in a lesson for a word of values), the class in plain text, the class of values and the word of a word that is not new. A side of words: the side with hidden values first when the side holds a new word (in a lesson always), then the side as it stands, the word next to the place before the two words, and the class in plain text of the next word before all when the run has classes. The words a few places away come sorted, those after the place before those before it, each by its text");

pub(super) fn frame_near(word: &[FrameKey]) -> Option<&FrameKey> {
    word.iter().find(|key| key[0] == FramePart::Values as u32).or_else(|| word.iter().rev().find(|key| key[0] == FramePart::Class as u32))
}
because!(frame_near, FrameKeys, "the key of a word that is counted with the words a few places away: its class of values, or else its class in plain text");

pub(super) fn frame_short(key: &[u32]) -> bool {
    key.len() <= FRAME_SHORT && key.first().is_some_and(|part| *part <= FramePart::Hidden as u32)
}
because!(frame_short, FrameKeys, "is a key one of the word alone or of the hidden word with one side of one word: it is kept with one reading");

pub(super) fn frame_joined(pivot: &[u32], key: &[u32], into: &mut Vec<u32>) {
    into.clear();
    into.extend_from_slice(pivot);
    into.extend_from_slice(key);
}
because!(frame_joined, FrameKeys, "a key of the word with a key of a side: the row of the word that a side is counted with");
