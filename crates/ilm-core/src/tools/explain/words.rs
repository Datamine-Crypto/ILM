use crate::tools::explain::text::{not_bracket, word_not_mark};
use patterns::{because, source};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{OnceLock, PoisonError, RwLock};

pub struct RunWords;
source!(RunWords, "words as numbers (string interning, as the symbols of Lisp are kept): each different word of a run has one number in one table, so that the engine compares, hashes and copies numbers in place of letters, and turns a number back into its word only where letters count, at an ending, in a number, and in the words it says; a number tells a few facts of its word in its lowest bits, so that the tests of the open places need no letters");

pub type WordId = u32;
because!(WordId, RunWords, "the number of a word: its place in the table of the run, above the bits that tell whether it is a word and not a mark, whether it is the bracket of a name, and whether it is written in digits");

const FLAG_BITS: u32 = 3;
because!(FLAG_BITS, RunWords, "the lowest bits of the number of a word that tell facts of its letters: one each for a word that is no mark, for a bracket of a name, and for digits");

pub const WORD_IS_TEXT: WordId = 1;
because!(WORD_IS_TEXT, RunWords, "the bit of a word that starts with a letter or a digit, which an open place may take");

pub const WORD_IS_BRACKET: WordId = 2;
because!(WORD_IS_BRACKET, RunWords, "the bit of a bracket of a name, which no open place takes");

pub const WORD_IS_DIGITS: WordId = 4;
because!(WORD_IS_DIGITS, RunWords, "the bit of a word whose signs are all digits, which a comparison or a step of arithmetic reads as a number");

const SHARDS: usize = 64;
because!(SHARDS, RunWords, "the parts of the table of words, each with its own lock, chosen by a hash of the word, so that the lanes that cut stories into words at the same time seldom wait for each other");

const CHUNK_BITS: u32 = 12;
because!(CHUNK_BITS, RunWords, "the words of the table are kept in runs of this many bits of places, each run made when its first word comes, so that a number turns back into its word with no lock");

const CHUNK_SIZE: usize = 4096;
because!(CHUNK_SIZE, RunWords, "the places of one run of the table of words: two to the bits of a run");

const CHUNKS: usize = 65536;
because!(CHUNKS, RunWords, "the runs of the table of words: room for more words than all the sets of the board hold");

struct WordTable {
    shards: Vec<RwLock<HashMap<&'static str, WordId>>>,
    next: AtomicU32,
    texts: Vec<OnceLock<Box<[OnceLock<&'static str>]>>>,
}
because!(WordTable, RunWords, "the table of the words of the run: the number of each word in the part that a hash of the word picks, the next free place, and the word of each place in runs of places that are made once and read with no lock");

static TABLE: OnceLock<WordTable> = OnceLock::new();
because!(TABLE, RunWords, "the one table of words of the run, shared by every lane, so that a word has the same number in the library, in each story and in each answer; the numbers come in the order in which the lanes meet the words, which changes from run to run, so no result may depend on the order of the numbers: they are only compared and hashed, and any sort or tie that reads words turns them into letters first");

fn table() -> &'static WordTable {
    TABLE.get_or_init(|| WordTable { shards: (0..SHARDS).map(|_| RwLock::default()).collect(), next: AtomicU32::new(0), texts: (0..CHUNKS).map(|_| OnceLock::new()).collect() })
}
because!(table, RunWords, "the table of words, made empty the first time that a word needs a number");

fn shard_of(text: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    (hasher.finish() as usize) % SHARDS
}
because!(shard_of, RunWords, "the part of the table that holds a word, by a hash of its letters");

fn flags_of(text: &str) -> WordId {
    let mut flags = WordId::default();
    if word_not_mark(text) {
        flags |= WORD_IS_TEXT;
    }
    if !not_bracket(text) {
        flags |= WORD_IS_BRACKET;
    }
    if text.bytes().all(|sign| sign.is_ascii_digit()) {
        flags |= WORD_IS_DIGITS;
    }
    flags
}
because!(flags_of, RunWords, "the bits that tell the facts of the letters of a word");

pub fn word_number(text: &str) -> WordId {
    let table = table();
    let shard = &table.shards[shard_of(text)];
    if let Some(found) = shard.read().unwrap_or_else(PoisonError::into_inner).get(text) {
        return *found;
    }
    let mut held = shard.write().unwrap_or_else(PoisonError::into_inner);
    if let Some(found) = held.get(text) {
        return *found;
    }
    let place = table.next.fetch_add(1, Ordering::Relaxed);
    let kept: &'static str = Box::leak(text.to_string().into_boxed_str());
    let run = table.texts[(place >> CHUNK_BITS) as usize].get_or_init(|| (0..CHUNK_SIZE).map(|_| OnceLock::new()).collect());
    let _ = run[place as usize % CHUNK_SIZE].set(kept);
    let number = (place << FLAG_BITS) | flags_of(text);
    held.insert(kept, number);
    number
}
because!(word_number, RunWords, "the number of a word: the number that the table holds for it, or a new place at the end of the table, its letters kept for the run");

pub fn word_text(number: WordId) -> &'static str {
    let place = number >> FLAG_BITS;
    table().texts[(place >> CHUNK_BITS) as usize].get().and_then(|run| run[place as usize % CHUNK_SIZE].get()).copied().unwrap_or_default()
}
because!(word_text, RunWords, "the letters of the word of a number, read with no lock");

pub fn word_has(number: WordId, fact: WordId) -> bool {
    number & fact == fact
}
because!(word_has, RunWords, "whether the word of a number has a fact of its letters that its lowest bits tell: a word and not a mark, as an open place needs; a bracket of a name, which an open place that may hold marks does not take; or digits only");

pub fn word_numbers(texts: &[String]) -> Vec<WordId> {
    texts.iter().map(|text| word_number(text)).collect()
}
because!(word_numbers, RunWords, "the numbers of some words in their order");

pub fn word_texts(numbers: &[WordId]) -> Vec<String> {
    numbers.iter().map(|number| word_text(*number).to_string()).collect()
}
because!(word_texts, RunWords, "the words of some numbers in their order");
