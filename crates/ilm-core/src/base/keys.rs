use patterns::{because, source};

pub struct KeysAsNumbers;
source!(KeysAsNumbers, "keys as numbers: a key is a short run of word numbers packed into one wide number, and each different key gets the next whole number once, so every count of a key is a place in a flat array; the table is open addressing with linear probing over a power of two, as Knuth describes it, and a place holds the number of its key beside a part of the hash, so a probe reads the wide keys only where that part is the same");

pub struct MultiplyHash;
source!(MultiplyHash, "hashing by one multiply, as Knuth gives it with the golden ratio: the key times a fixed odd number, and the high bits of the product are the place; keys that differ in a few low bits land far from each other");

const GOLDEN_STEP: u64 = 0x9e37_79b9_7f4a_7c15;
because!(GOLDEN_STEP, MultiplyHash, "the odd whole number nearest to the range of a machine word over the golden ratio, the factor that Knuth gives for this hash");

const WORD_BITS: u32 = 64;
because!(WORD_BITS, MultiplyHash, "the width of the product of the hash, from whose high end the place of a key is taken");

const HALF_BITS: u32 = 32;
because!(HALF_BITS, KeysAsNumbers, "the width of the number of a key inside a place of the table; the other part of the place holds the low end of the hash");

const KEY_HALF: u32 = 64;
because!(KEY_HALF, KeysAsNumbers, "the width of each of the parts in which a wide key goes through the hash");

const FIRST_ROOM: usize = 1024;
because!(FIRST_ROOM, KeysAsNumbers, "the places of an empty table: a power of the base of the machine, and large enough that the forms of a short text never grow it");

const ROOM_FOR_ONE: usize = 2;
because!(ROOM_FOR_ONE, KeysAsNumbers, "the table grows when the keys times this number pass its places, and it grows by this factor: with so much room a probe ends at once or after a short walk, and the number of places stays a power of the base of the machine");

pub const FIELD_BITS: u32 = 32;
because!(FIELD_BITS, KeysAsNumbers, "the width of one number inside a wide key: the width of a word number, so a key holds any word number in each of its fields");

pub const KEY_FIELDS: usize = 4;
because!(KEY_FIELDS, KeysAsNumbers, "the fields of a wide key: as many numbers of the full width as the key has room for");

pub const NO_NUMBER: u32 = u32::MAX;
because!(NO_NUMBER, KeysAsNumbers, "the mark of a field after the end of the numbers of a key, and of any place that holds no number; no word and no key has this number");

pub fn padded<const SIZE: usize>(numbers: &[u32]) -> [u32; SIZE] {
    let mut held = [NO_NUMBER; SIZE];
    held[..numbers.len()].copy_from_slice(numbers);
    held
}
because!(padded, KeysAsNumbers, "some word numbers as a row of a fixed size, the places after them empty: a key of a table that holds one to a few words has one size then, with no memory of its own");

pub fn key_field(value: u32, place: usize) -> u128 {
    (u128::from(value) + 1) << (FIELD_BITS * place as u32)
}
because!(key_field, KeysAsNumbers, "one number in its field of a wide key, raised by one so that a field with nothing in it says that the numbers of the key have ended");

pub fn packed(numbers: &[u32]) -> u128 {
    let mut key: u128 = 0;
    for (place, value) in numbers.iter().enumerate() {
        key |= key_field(*value, place);
    }
    key
}
because!(packed, KeysAsNumbers, "a few numbers as one wide key, each in a field of its own in their order; keys of different lengths are different keys");

pub fn fields_of(key: u128) -> [u32; KEY_FIELDS] {
    let mut numbers = [NO_NUMBER; KEY_FIELDS];
    for (place, value) in numbers.iter_mut().enumerate() {
        *value = ((key >> (FIELD_BITS * place as u32)) as u32).wrapping_sub(1);
    }
    numbers
}
because!(fields_of, KeysAsNumbers, "the numbers of a wide key in their order, with the mark of no number for each field after their end");

fn mixed(key: u128) -> u64 {
    let low = key as u64;
    let high = (key >> KEY_HALF) as u64;
    (low.wrapping_mul(GOLDEN_STEP) ^ high).wrapping_mul(GOLDEN_STEP)
}
because!(mixed, MultiplyHash, "the hash of a wide key: the low part goes through the multiply, the high part is mixed in, and one more multiply spreads both over the high bits");

#[derive(Clone, Debug)]
pub struct KeyNumbers {
    keys: Vec<u128>,
    slots: Vec<u64>,
    shift: u32,
}
because!(KeyNumbers, KeysAsNumbers, MultiplyHash, "the keys that were seen, each with its number, which is its place in the order of first sight. number gives the number of a key and a new number to a new key; find gives the number of a key that was seen and nothing for a new one, and changes nothing, so many threads read one table; forget takes every key out and keeps the room, at the cost of the keys that it held and not of its places, so one table serves the forms of one text after the other");

impl Default for KeyNumbers {
    fn default() -> KeyNumbers {
        KeyNumbers { keys: Vec::new(), slots: vec![0; FIRST_ROOM], shift: WORD_BITS - FIRST_ROOM.trailing_zeros() }
    }
}

impl KeyNumbers {
    pub fn key(&self, number: u32) -> u128 {
        self.keys[number as usize]
    }

    pub fn keys(&self) -> &[u128] {
        &self.keys
    }

    fn place(&self, key: u128, hash: u64) -> usize {
        let mask = self.slots.len() - 1;
        let mut at = (hash >> self.shift) as usize;
        loop {
            let slot = self.slots[at];
            if slot == 0 {
                return at;
            }
            if (slot >> HALF_BITS) as u32 == hash as u32 && self.keys[(slot as u32 - 1) as usize] == key {
                return at;
            }
            at = (at + 1) & mask;
        }
    }

    pub fn find(&self, key: u128) -> Option<u32> {
        let slot = self.slots[self.place(key, mixed(key))];
        (slot != 0).then(|| slot as u32 - 1)
    }

    fn grow(&mut self) {
        let wider = self.slots.len() * ROOM_FOR_ONE;
        self.slots = vec![0; wider];
        self.shift = WORD_BITS - wider.trailing_zeros();
        for number in 0..self.keys.len() {
            let hash = mixed(self.keys[number]);
            let at = self.place(self.keys[number], hash);
            self.slots[at] = (u64::from(hash as u32) << HALF_BITS) | (number as u64 + 1);
        }
    }

    pub fn number(&mut self, key: u128) -> u32 {
        let hash = mixed(key);
        let at = self.place(key, hash);
        let slot = self.slots[at];
        if slot != 0 {
            return slot as u32 - 1;
        }
        let number = self.keys.len() as u32;
        self.keys.push(key);
        self.slots[at] = (u64::from(hash as u32) << HALF_BITS) | (u64::from(number) + 1);
        if self.keys.len() * ROOM_FOR_ONE > self.slots.len() {
            self.grow();
        }
        number
    }

    pub fn forget(&mut self) {
        let mask = self.slots.len() - 1;
        while let Some(key) = self.keys.pop() {
            let held = self.keys.len() as u32 + 1;
            let mut at = (mixed(key) >> self.shift) as usize;
            while self.slots[at] as u32 != held {
                at = (at + 1) & mask;
            }
            self.slots[at] = 0;
        }
    }
}
