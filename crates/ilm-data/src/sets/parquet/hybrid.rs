use crate::sets::parquet::wire::{WIRE_BYTE_BITS, wire_number};
use patterns::{because, source};

pub struct RunsAndBits;
source!(RunsAndBits, "the hybrid encoding of parquet for levels and for numbers into a dictionary: runs, each with a header that is a variable-length number; a header with its low bit clear is a run of one value repeated, the value in the bytes that its width needs; a header with its low bit set is groups of eight values packed in their width of bits, the least bit first");

const WIDEST: u32 = 32;
because!(WIDEST, RunsAndBits, "the width of a level or of a number into a dictionary at most");

pub struct Hybrid<'a> {
    bytes: &'a [u8],
    at: usize,
    width: u32,
}
because!(Hybrid, RunsAndBits, "a reader of runs and packed bits over some bytes with a width of bits for each value; take gives the next values");

impl<'a> Hybrid<'a> {
    pub fn new(bytes: &'a [u8], width: u32) -> Result<Hybrid<'a>, String> {
        if width > WIDEST {
            return Err(format!("a width of {width} bits is too wide"));
        }
        Ok(Hybrid { bytes, at: 0, width })
    }

    pub fn take(&mut self, count: usize, out: &mut Vec<u32>) -> Result<(), String> {
        let goal = out.len() + count;
        let value_bytes = (self.width as usize).div_ceil(WIRE_BYTE_BITS);
        while out.len() < goal {
            let header = wire_number(self.bytes, &mut self.at)?;
            if header & 1 == 0 {
                let run = (header >> 1) as usize;
                let piece = self.bytes.get(self.at..self.at + value_bytes).ok_or("a run ends in its value")?;
                self.at += value_bytes;
                let value = piece.iter().rev().fold(0u32, |held, byte| (held << WIRE_BYTE_BITS) | u32::from(*byte));
                let size = run.min(goal - out.len());
                out.extend(std::iter::repeat_n(value, size));
                continue;
            }
            let groups = (header >> 1) as usize;
            let size = groups * self.width as usize;
            let packed = self.bytes.get(self.at..self.at + size).ok_or("packed bits end early")?;
            self.at += size;
            let mask = if self.width == 0 { 0 } else { u32::MAX >> (WIDEST - self.width) };
            let mut bit = 0usize;
            for _ in 0..groups * WIRE_BYTE_BITS {
                let mut value = 0u32;
                let mut done = 0u32;
                while done < self.width {
                    let byte = packed[bit / WIRE_BYTE_BITS];
                    let inside = bit % WIRE_BYTE_BITS;
                    let take = (WIRE_BYTE_BITS - inside).min((self.width - done) as usize);
                    let piece = (u32::from(byte) >> inside) & (u32::MAX >> (WIDEST - take as u32));
                    value |= piece << done;
                    done += take as u32;
                    bit += take;
                }
                if out.len() < goal {
                    out.push(value & mask);
                }
            }
        }
        Ok(())
    }
}

pub fn width_of(most: u32) -> u32 {
    u32::BITS - most.leading_zeros()
}
because!(width_of, RunsAndBits, "the bits that the largest value of a level needs; a column whose levels are all nought needs none");
