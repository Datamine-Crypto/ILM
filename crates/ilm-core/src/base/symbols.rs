use patterns::{because, source};

pub struct WordNumbers;
source!(WordNumbers, "words as numbers: each different word of the lessons gets the next whole number once, and every table after that holds numbers; a text is compared or hashed one time, when it is read, and never in a loop that runs for each lesson; the table is open addressing with linear probing over a power of two, as Knuth describes it");

pub struct ByteHash;
source!(ByteHash, "the hash of Fowler, Noll and Vo in its form that mixes each byte in before the multiply: a start value and one multiply by a prime for each byte");

const HASH_START: u64 = 0xcbf2_9ce4_8422_2325;
because!(HASH_START, ByteHash, "the start value that Fowler, Noll and Vo give for a hash of this width");

const HASH_STEP: u64 = 0x0000_0100_0000_01b3;
because!(HASH_STEP, ByteHash, "the prime that Fowler, Noll and Vo give for a hash of this width");

const FIRST_SLOTS: usize = 1024;
because!(FIRST_SLOTS, WordNumbers, "the places of an empty table: a number that a mask cuts a hash to, and large enough that a small kind of task never grows it");

const FULL_PARTS: usize = 2;
because!(FULL_PARTS, WordNumbers, "the table grows when the words times this number pass its places: with so much room a probe ends at once or after a short walk, and the table stays small beside the texts of the words");

const GROWTH: usize = 2;
because!(GROWTH, WordNumbers, "the factor by which the places of a table grow: the smallest factor that keeps the number of places one that a mask cuts a hash to");

const VACANT: u32 = u32::MAX;
because!(VACANT, WordNumbers, "the mark of a place of the table that holds no word");

pub fn byte_hash(bytes: &[u8]) -> u64 {
    let mut hash = HASH_START;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(HASH_STEP);
    }
    hash
}
because!(byte_hash, ByteHash, "the hash of a run of bytes, for the one time that a word is looked up by its text");

#[derive(Clone, Debug)]
pub struct Symbols {
    names: Vec<String>,
    slots: Vec<u32>,
}
because!(Symbols, WordNumbers, ByteHash, "the words that were seen, each with its number, which is its place in the order of first sight; that order is the order of the lessons, so a table that is walked in the order of the numbers is walked as the lessons came. number gives the number of a word and a new number to a new word; find gives the number of a word that was seen and nothing for a new one, and changes nothing, so many threads read one table; name gives the text of a number for the one time that an answer is written; the table grows when it is too full for short probes");

impl Default for Symbols {
    fn default() -> Symbols {
        Symbols { names: Vec::new(), slots: vec![VACANT; FIRST_SLOTS] }
    }
}

impl Symbols {
    pub fn new() -> Symbols {
        Symbols::default()
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn name(&self, number: u32) -> &str {
        &self.names[number as usize]
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    fn place(&self, word: &str) -> usize {
        let mask = self.slots.len() - 1;
        let mut at = (byte_hash(word.as_bytes()) as usize) & mask;
        while self.slots[at] != VACANT && self.names[self.slots[at] as usize] != word {
            at = (at + 1) & mask;
        }
        at
    }

    pub fn find(&self, word: &str) -> Option<u32> {
        let found = self.slots[self.place(word)];
        (found != VACANT).then_some(found)
    }

    fn grow(&mut self) {
        let wider = self.slots.len() * GROWTH;
        self.slots = vec![VACANT; wider];
        for number in 0..self.names.len() {
            let at = self.place(&self.names[number]);
            self.slots[at] = number as u32;
        }
    }

    pub fn number(&mut self, word: &str) -> u32 {
        let at = self.place(word);
        if self.slots[at] != VACANT {
            return self.slots[at];
        }
        let number = self.names.len() as u32;
        self.names.push(word.to_string());
        self.slots[at] = number;
        if self.names.len() * FULL_PARTS > self.slots.len() {
            self.grow();
        }
        number
    }

    pub fn numbers(&mut self, words: &[String]) -> Vec<u32> {
        words.iter().map(|word| self.number(word)).collect()
    }
}
