use patterns::{because, source};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

pub struct RotateMultiply;
source!(RotateMultiply, "the hash of the Firefox browser, which the Rust compiler uses for its own tables: for each machine word a rotation, an exclusive or and one multiply; far cheaper than a hash that resists an attacker, and the keys here are numbers that the program made itself");

pub const TURN: u32 = 5;
because!(TURN, RotateMultiply, "the bits by which the hash is rotated before each word, as the function is published");

pub const SPREAD: u64 = 0x517c_c1b7_2722_0a95;
because!(SPREAD, RotateMultiply, "the odd multiplier of the function as it is published, which spreads the bits of each word over the whole hash");

pub const BYTE_BITS: u32 = 8;
because!(BYTE_BITS, RotateMultiply, "the bits of one byte, to put the bytes of a key together into machine words");

pub const HIGH_HALF: u32 = 32;
because!(HIGH_HALF, RotateMultiply, "the bits by which the high half of a hash is moved down: the multiply puts the best mixed bits there, and a table cuts a hash to its low bits");

pub fn mix(hash: u64, word: u64) -> u64 {
    (hash.rotate_left(TURN) ^ word).wrapping_mul(SPREAD)
}
because!(mix, RotateMultiply, "one more machine word mixed into a hash");

#[derive(Clone, Copy, Default)]
pub struct CheapHasher {
    hash: u64,
}
because!(CheapHasher, RotateMultiply, "the state of the hash for the tables of the standard library: every number of a key is mixed in as one machine word, and bytes are put together into words first");

impl Hasher for CheapHasher {
    fn finish(&self) -> u64 {
        self.hash ^ (self.hash >> HIGH_HALF)
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut words = bytes.chunks_exact(std::mem::size_of::<u64>());
        for chunk in &mut words {
            self.hash = mix(self.hash, u64::from_le_bytes(chunk.try_into().unwrap_or_default()));
        }
        let rest = words.remainder();
        if !rest.is_empty() {
            let mut word: u64 = 0;
            let mut held: u32 = 0;
            for byte in rest {
                word |= u64::from(*byte) << held;
                held += BYTE_BITS;
            }
            self.hash = mix(self.hash, word);
        }
    }

    fn write_u32(&mut self, value: u32) {
        self.hash = mix(self.hash, u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        self.hash = mix(self.hash, value);
    }

    fn write_usize(&mut self, value: usize) {
        self.hash = mix(self.hash, value as u64);
    }
}

pub type CheapMap<K, V> = HashMap<K, V, BuildHasherDefault<CheapHasher>>;
because!(CheapMap, RotateMultiply, "a table of the standard library with the cheap hash, for keys that are numbers or rows of numbers");
