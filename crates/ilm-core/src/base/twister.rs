use crate::base::digest::sha512;
use patterns::{because, source};

pub struct Twister;
source!(Twister, "the Mersenne Twister of Matsumoto and Nishimura, with its seeding by an array of words, and the draws that the library of the language Python makes from it: a number below one from two words, a whole number below a bound by taking just enough bits and drawing again when the number is too large, a choice by weights through the running sums of the weights, and a shuffle from the back; a text as a seed is its bytes followed by their hash by the Secure Hash Standard, read as one whole number. With these, a program that draws from a named seed gives the same draws in every run and in every language that follows the same steps");

const MT_SIZE: usize = 624;
because!(MT_SIZE, Twister, "the words of the state of the twister");

const MT_SHIFT: usize = 397;
because!(MT_SHIFT, Twister, "the distance in the state between a word and the word that it is mixed with");

const MT_MATRIX: u32 = 0x9908_b0df;
because!(MT_MATRIX, Twister, "the twist of the twister, given by its authors");

const MT_UPPER: u32 = 0x8000_0000;
because!(MT_UPPER, Twister, "the highest bit of a word, taken from one word of the state in the twist");

const MT_LOWER: u32 = 0x7fff_ffff;
because!(MT_LOWER, Twister, "the lower bits of a word, taken from the next word of the state in the twist");

const MT_TEMPER_ONE: u32 = 11;
because!(MT_TEMPER_ONE, Twister, "the first shift of the tempering of an output word");

const MT_TEMPER_TWO: u32 = 7;
because!(MT_TEMPER_TWO, Twister, "the second shift of the tempering of an output word");

const MT_TEMPER_MASK_TWO: u32 = 0x9d2c_5680;
because!(MT_TEMPER_MASK_TWO, Twister, "the mask of the second step of the tempering");

const MT_TEMPER_THREE: u32 = 15;
because!(MT_TEMPER_THREE, Twister, "the third shift of the tempering of an output word");

const MT_TEMPER_MASK_THREE: u32 = 0xefc6_0000;
because!(MT_TEMPER_MASK_THREE, Twister, "the mask of the third step of the tempering");

const MT_TEMPER_FOUR: u32 = 18;
because!(MT_TEMPER_FOUR, Twister, "the last shift of the tempering of an output word");

const MT_INIT: u32 = 1_812_433_253;
because!(MT_INIT, Twister, "the factor with which the state grows from one word, given by the authors");

const MT_INIT_SHIFT: u32 = 30;
because!(MT_INIT_SHIFT, Twister, "the shift with which a word of the state is mixed into the next when the state grows");

const MT_ARRAY_SEED: u32 = 19_650_218;
because!(MT_ARRAY_SEED, Twister, "the word from which the state grows before an array of words is mixed in, given by the authors");

const MT_ARRAY_ONE: u32 = 1_664_525;
because!(MT_ARRAY_ONE, Twister, "the factor of the first pass that mixes an array of words into the state");

const MT_ARRAY_TWO: u32 = 1_566_083_941;
because!(MT_ARRAY_TWO, Twister, "the factor of the second pass that mixes an array of words into the state");

const MT_WORD_BITS: u32 = 32;
because!(MT_WORD_BITS, Twister, "the bits that one draw of the twister gives; a draw of fewer bits keeps the high ones, which are the best mixed");

const MT_HIGH_DROP: u32 = 5;
because!(MT_HIGH_DROP, Twister, "the bits that the first word of a number below one leaves out, so that it gives the high bits of a mantissa");

const MT_LOW_DROP: u32 = 6;
because!(MT_LOW_DROP, Twister, "the bits that the second word of a number below one leaves out, so that the two give the bits of a mantissa");

const MT_HIGH_SCALE: f64 = 67_108_864.0;
because!(MT_HIGH_SCALE, Twister, "the weight of the high bits of a number below one: the number of the low bits that follow them");

const MT_FRACTION: f64 = 9_007_199_254_740_992.0;
because!(MT_FRACTION, Twister, "the number of steps of a number below one: one more than the largest whole number that the mantissa of a double holds");

const MT_BYTES: usize = 4;
because!(MT_BYTES, Twister, "the bytes of one word of the array that a seed gives");

#[derive(Clone, Debug)]
pub struct TwisterDraws {
    state: Vec<u32>,
    next: usize,
}
because!(TwisterDraws, Twister, "the state of the twister and the place of its next word. seeded makes it from a text; word gives the next tempered word; bits gives a whole number of up to one word of bits; below gives a whole number below a bound; unit gives a number below one; choice gives a place in a run of a length; weighted gives a place by whole weights; between gives a whole number between two bounds; shuffle turns a run about; pool_sample takes some places of a run without taking one twice");

impl TwisterDraws {
    fn grown(seed: u32) -> Vec<u32> {
        let mut state = vec![0u32; MT_SIZE];
        state[0] = seed;
        for place in 1..MT_SIZE {
            state[place] = MT_INIT.wrapping_mul(state[place - 1] ^ (state[place - 1] >> MT_INIT_SHIFT)).wrapping_add(place as u32);
        }
        state
    }

    fn with_array(key: &[u32]) -> TwisterDraws {
        let mut state = TwisterDraws::grown(MT_ARRAY_SEED);
        let (mut place, mut at) = (1usize, 0usize);
        for _ in 0..MT_SIZE.max(key.len()) {
            let mixed = (state[place - 1] ^ (state[place - 1] >> MT_INIT_SHIFT)).wrapping_mul(MT_ARRAY_ONE);
            state[place] = (state[place] ^ mixed).wrapping_add(key[at]).wrapping_add(at as u32);
            place += 1;
            at += 1;
            if place >= MT_SIZE {
                state[0] = state[MT_SIZE - 1];
                place = 1;
            }
            if at >= key.len() {
                at = 0;
            }
        }
        for _ in 0..MT_SIZE - 1 {
            let mixed = (state[place - 1] ^ (state[place - 1] >> MT_INIT_SHIFT)).wrapping_mul(MT_ARRAY_TWO);
            state[place] = (state[place] ^ mixed).wrapping_sub(place as u32);
            place += 1;
            if place >= MT_SIZE {
                state[0] = state[MT_SIZE - 1];
                place = 1;
            }
        }
        state[0] = MT_UPPER;
        TwisterDraws { state, next: MT_SIZE }
    }

    pub fn seeded(text: &str) -> TwisterDraws {
        let mut bytes = text.as_bytes().to_vec();
        bytes.extend(sha512(text.as_bytes()));
        let start = bytes.iter().position(|byte| *byte != 0).unwrap_or(bytes.len());
        let mut little: Vec<u8> = bytes[start..].iter().rev().copied().collect();
        if little.is_empty() {
            little.push(0);
        }
        while little.len() % MT_BYTES != 0 {
            little.push(0);
        }
        let key: Vec<u32> = little
            .chunks(MT_BYTES)
            .map(|chunk| {
                let mut word = [0u8; MT_BYTES];
                word.copy_from_slice(chunk);
                u32::from_le_bytes(word)
            })
            .collect();
        TwisterDraws::with_array(&key)
    }

    fn twist(&mut self) {
        for place in 0..MT_SIZE {
            let joined = (self.state[place] & MT_UPPER) | (self.state[(place + 1) % MT_SIZE] & MT_LOWER);
            let mut word = self.state[(place + MT_SHIFT) % MT_SIZE] ^ (joined >> 1);
            if joined & 1 != 0 {
                word ^= MT_MATRIX;
            }
            self.state[place] = word;
        }
        self.next = 0;
    }

    pub fn word(&mut self) -> u32 {
        if self.next >= MT_SIZE {
            self.twist();
        }
        let mut word = self.state[self.next];
        self.next += 1;
        word ^= word >> MT_TEMPER_ONE;
        word ^= (word << MT_TEMPER_TWO) & MT_TEMPER_MASK_TWO;
        word ^= (word << MT_TEMPER_THREE) & MT_TEMPER_MASK_THREE;
        word ^ (word >> MT_TEMPER_FOUR)
    }

    pub fn bits(&mut self, count: u32) -> u32 {
        if count == 0 {
            return 0;
        }
        self.word() >> (MT_WORD_BITS - count)
    }

    pub fn below(&mut self, bound: usize) -> usize {
        let count = usize::BITS - bound.leading_zeros();
        loop {
            let drawn = self.bits(count) as usize;
            if drawn < bound {
                return drawn;
            }
        }
    }

    pub fn unit(&mut self) -> f64 {
        let high = f64::from(self.word() >> MT_HIGH_DROP);
        let low = f64::from(self.word() >> MT_LOW_DROP);
        (high * MT_HIGH_SCALE + low) * (f64::from(1u8) / MT_FRACTION)
    }

    pub fn choice(&mut self, length: usize) -> usize {
        self.below(length)
    }

    pub fn weighted(&mut self, weights: &[u32]) -> usize {
        let mut running: Vec<u32> = Vec::with_capacity(weights.len());
        let mut sum = 0u32;
        for weight in weights {
            sum += weight;
            running.push(sum);
        }
        let total = f64::from(sum);
        let point = self.unit() * total;
        let last = weights.len() - 1;
        running[..last].partition_point(|held| f64::from(*held) <= point)
    }

    pub fn between(&mut self, low: usize, high: usize) -> usize {
        low + self.below(high - low + 1)
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for place in (1..items.len()).rev() {
            let other = self.below(place + 1);
            items.swap(place, other);
        }
    }

    pub fn pool_sample(&mut self, length: usize, count: usize) -> Vec<usize> {
        let mut pool: Vec<usize> = (0..length).collect();
        let mut out: Vec<usize> = Vec::with_capacity(count);
        for taken in 0..count {
            let place = self.below(length - taken);
            out.push(pool[place]);
            pool[place] = pool[length - taken - 1];
        }
        out
    }
}
