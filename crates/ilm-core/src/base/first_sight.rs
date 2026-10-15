use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::ops::AddAssign;

pub struct NumberHash;
source!(NumberHash, "the multiplicative hash that Knuth describes: a key of whole numbers is mixed by one rotation, one exclusive or and one multiply by an odd constant near the golden section of the word for each number; cheap, and enough for tables whose keys are numbers of words and never texts");

pub struct FirstSightOrder;
source!(FirstSightOrder, "a table that keeps its keys in the order in which they were first put in. A learner by counting rests on that order where counts are equal: the most frequent of equal counts is the first put in, and a walk over a table goes in the order of the lessons; so such a table is a run of rows in the order of first sight with an index beside it");

const NUMBER_STEP: u64 = 0x9e37_79b9_7f4a_7c15;
because!(NUMBER_STEP, NumberHash, "the odd whole number nearest to the word size divided by the golden section, the multiplier that spreads keys which differ in few bits");

const NUMBER_TURN: u32 = 5;
because!(NUMBER_TURN, NumberHash, "the bits by which the hash is turned before the next number is mixed in, so that the order of the numbers of a key counts");

const NUMBER_BYTE_BITS: u32 = 8;
because!(NUMBER_BYTE_BITS, NumberHash, "the bits of one byte, for the rare key that reaches the hash as bytes");

#[derive(Clone, Copy, Default)]
pub struct NumberHasher {
    state: u64,
}
because!(NumberHasher, NumberHash, "the running hash of a key made of whole numbers; each number turns the state, is mixed in and is multiplied");

impl NumberHasher {
    fn mix(&mut self, number: u64) {
        self.state = (self.state.rotate_left(NUMBER_TURN) ^ number).wrapping_mul(NUMBER_STEP);
    }
}

impl Hasher for NumberHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(NUMBER_BYTE_BITS as usize) {
            let mut number = 0u64;
            for byte in chunk {
                number = (number << NUMBER_BYTE_BITS) | u64::from(*byte);
            }
            self.mix(number);
        }
    }

    fn write_u32(&mut self, number: u32) {
        self.mix(u64::from(number));
    }

    fn write_u64(&mut self, number: u64) {
        self.mix(number);
    }

    fn write_usize(&mut self, number: usize) {
        self.mix(number as u64);
    }
}

pub type NumberHashes = BuildHasherDefault<NumberHasher>;
because!(NumberHashes, NumberHash, "the maker of the hash of numbers for the open tables of the standard library");

pub type NumberTable<K, V> = HashMap<K, V, NumberHashes>;
because!(NumberTable, NumberHash, "an open table over keys of numbers, where the order of the keys is never asked for");

pub type NumberBag<K> = HashSet<K, NumberHashes>;
because!(NumberBag, NumberHash, "a set of keys of numbers that is only asked whether it holds a key");

#[derive(Clone, Debug)]
pub struct FirstSightTable<K, V> {
    index: NumberTable<K, u32>,
    rows: Vec<(K, V)>,
}
because!(FirstSightTable, FirstSightOrder, NumberHash, "a table that keeps its keys in the order of first sight: the rows lie in that order and an open table gives the row of a key. entry gives the value of a key for a change and puts the given value in at the end when the key is new, at does so with an empty value, put sets the value and leaves the key at its place, add adds to a count; first_most gives the first row with the highest count and by_count the rows from the highest count down with equals in their order; a walk over rows goes in the order of first sight; nothing is ever taken out, so the place of a row never changes");

impl<K, V> Default for FirstSightTable<K, V> {
    fn default() -> FirstSightTable<K, V> {
        FirstSightTable { index: NumberTable::default(), rows: Vec::new() }
    }
}

impl<K: Hash + Eq + Clone, V> FirstSightTable<K, V> {
    pub fn new() -> FirstSightTable<K, V> {
        FirstSightTable::default()
    }

    pub fn rows(&self) -> &[(K, V)] {
        &self.rows
    }

    pub fn rows_mut(&mut self) -> &mut [(K, V)] {
        &mut self.rows
    }

    pub fn place(&self, key: &K) -> Option<usize> {
        self.index.get(key).map(|row| *row as usize)
    }

    pub fn holds(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.place(key).map(|row| &self.rows[row].1)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.place(key) {
            Some(row) => Some(&mut self.rows[row].1),
            None => None,
        }
    }

    fn row_with(&mut self, key: K, fresh: V) -> usize {
        match self.place(&key) {
            Some(row) => row,
            None => {
                let row = self.rows.len();
                self.index.insert(key.clone(), row as u32);
                self.rows.push((key, fresh));
                row
            }
        }
    }

    pub fn put(&mut self, key: K, value: V) -> usize {
        match self.place(&key) {
            Some(row) => {
                self.rows[row].1 = value;
                row
            }
            None => self.row_with(key, value),
        }
    }

    pub fn entry(&mut self, key: K, fresh: V) -> &mut V {
        let row = self.row_with(key, fresh);
        &mut self.rows[row].1
    }

    pub fn row_of(&mut self, key: K) -> usize
    where
        V: Default,
    {
        self.row_with(key, V::default())
    }

    pub fn at(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        self.entry(key, V::default())
    }

    pub fn add(&mut self, key: K, count: V)
    where
        V: Default + AddAssign,
    {
        *self.at(key) += count;
    }

    pub fn first_most(&self) -> Option<&(K, V)>
    where
        V: PartialOrd,
    {
        let mut best: Option<&(K, V)> = None;
        for row in &self.rows {
            if best.is_none_or(|top| row.1 > top.1) {
                best = Some(row);
            }
        }
        best
    }

    pub fn by_count(&self) -> Vec<&(K, V)>
    where
        V: Ord,
    {
        let mut found: Vec<&(K, V)> = self.rows.iter().collect();
        found.sort_by(|one, other| other.1.cmp(&one.1));
        found
    }
}

pub type FirstSightCount<K> = FirstSightTable<K, i64>;
because!(FirstSightCount, FirstSightOrder, "how often each key was seen, with the keys in the order of first sight, so that the most frequent of equals is the one that the lessons showed first");

pub fn key_count<K: Hash + Eq + Clone>(counts: &FirstSightCount<K>, key: &K) -> i64 {
    counts.get(key).copied().unwrap_or_default()
}
because!(key_count, FirstSightOrder, "the count of a key, nought for a key that was never seen; asking changes nothing");

pub fn all_counts<K: Hash + Eq + Clone>(counts: &FirstSightCount<K>) -> i64 {
    counts.rows().iter().map(|(_, count)| *count).sum()
}
because!(all_counts, FirstSightOrder, "how many times any key of a table was seen: the whole against which the share of one key is taken");
