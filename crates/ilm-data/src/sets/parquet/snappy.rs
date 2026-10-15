use crate::sets::parquet::wire::{WIRE_BYTE_BITS, wire_number};
use patterns::{because, source};

pub struct SnappyFormat;
source!(SnappyFormat, "the raw form of Snappy, the compression of Google that parquet pages use: the length of the whole as a variable-length number, then elements, each a literal run of bytes or a copy of bytes that came before, told apart by the low two bits of the first byte of the element");

const ELEMENT_BITS: u8 = 3;
because!(ELEMENT_BITS, SnappyFormat, "the mask of the low two bits of the first byte of an element, which say its kind");

const SNAPPY_LITERAL: u8 = 0;
because!(SNAPPY_LITERAL, SnappyFormat, "the kind of an element that is a run of bytes as they are");

const COPY_SHORT: u8 = 1;
because!(COPY_SHORT, SnappyFormat, "the kind of a copy with an offset of eleven bits, the high ones in the first byte of the element and the low ones in the byte after it");

const COPY_MIDDLE: u8 = 2;
because!(COPY_MIDDLE, SnappyFormat, "the kind of a copy whose offset is a number of the middle width after the first byte of the element");

const KIND_SHIFT: u32 = 2;
because!(KIND_SHIFT, SnappyFormat, "the bits of the kind of an element, below the rest of its first byte");

const SHORT_LITERAL: usize = 60;
because!(SHORT_LITERAL, SnappyFormat, "a literal whose length less one is below this number has it in its first byte; from it on, the first byte says how many bytes of length follow");

const SHORT_LENGTH_BITS: u8 = 7;
because!(SHORT_LENGTH_BITS, SnappyFormat, "the mask of the three bits of the length of a copy with an offset of eleven bits");

const SHORT_LENGTH_FROM: usize = 4;
because!(SHORT_LENGTH_FROM, SnappyFormat, "the least length of a copy with an offset of eleven bits; its three bits of length count from it");

const SHORT_OFFSET_SHIFT: u32 = 5;
because!(SHORT_OFFSET_SHIFT, SnappyFormat, "where the high bits of the offset of a copy with an offset of eleven bits begin in its first byte");

const MIDDLE_OFFSET_BYTES: usize = 2;
because!(MIDDLE_OFFSET_BYTES, SnappyFormat, "the width of the offset of a copy of the middle kind: enough to point back inside a block of the size that Snappy writes");

const LONG_OFFSET_BYTES: usize = 4;
because!(LONG_OFFSET_BYTES, SnappyFormat, "the width of the offset of a copy of the long kind, for a block longer than an offset of the middle width can reach");

fn number_at(bytes: &[u8], at: usize, size: usize) -> Result<usize, String> {
    let piece = bytes.get(at..at + size).ok_or("a Snappy element ends early")?;
    Ok(piece.iter().rev().fold(0usize, |value, byte| (value << WIRE_BYTE_BITS) | usize::from(*byte)))
}
because!(number_at, SnappyFormat, "a number of some bytes at a place, the least byte first");

pub fn decompress(bytes: &[u8], expected: usize) -> Result<Vec<u8>, String> {
    let mut at = 0usize;
    let length = wire_number(bytes, &mut at)? as usize;
    if length != expected {
        return Err(format!("a Snappy block says {length} bytes and the page says {expected}"));
    }
    let mut out: Vec<u8> = Vec::with_capacity(length);
    while at < bytes.len() {
        let tag = bytes[at];
        at += 1;
        let kind = tag & ELEMENT_BITS;
        if kind == SNAPPY_LITERAL {
            let small = usize::from(tag >> KIND_SHIFT);
            let size = if small < SHORT_LITERAL {
                small + 1
            } else {
                let extra = small - SHORT_LITERAL + 1;
                let value = number_at(bytes, at, extra)?;
                at += extra;
                value + 1
            };
            let piece = bytes.get(at..at + size).ok_or("a Snappy literal ends early")?;
            out.extend_from_slice(piece);
            at += size;
            continue;
        }
        let (size, offset) = if kind == COPY_SHORT {
            let low = usize::from(*bytes.get(at).ok_or("a Snappy copy ends early")?);
            at += 1;
            (usize::from((tag >> KIND_SHIFT) & SHORT_LENGTH_BITS) + SHORT_LENGTH_FROM, (usize::from(tag >> SHORT_OFFSET_SHIFT) << WIRE_BYTE_BITS) | low)
        } else {
            let wide = if kind == COPY_MIDDLE { MIDDLE_OFFSET_BYTES } else { LONG_OFFSET_BYTES };
            let offset = number_at(bytes, at, wide)?;
            at += wide;
            (usize::from(tag >> KIND_SHIFT) + 1, offset)
        };
        if offset == 0 || offset > out.len() {
            return Err("a Snappy copy points before the start".to_string());
        }
        let from = out.len() - offset;
        for place in 0..size {
            let byte = out[from + place];
            out.push(byte);
        }
    }
    if out.len() != length {
        return Err(format!("a Snappy block gave {} bytes and says {length}", out.len()));
    }
    Ok(out)
}
because!(decompress, SnappyFormat, "the bytes of a Snappy block; a copy may overlap the bytes that it writes, so it goes byte by byte; a block whose length is not the length that the page says is refused");
