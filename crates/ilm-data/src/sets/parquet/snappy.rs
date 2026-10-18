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

const WIDE_MOVE: usize = 16;
because!(WIDE_MOVE, SnappyFormat, "the bytes that one move takes at once for a short element: a literal of at most this length, or a copy of at most this length that reaches back at least as far, moves this many bytes, and the bytes after its end are written again by the next elements");

fn number_at(bytes: &[u8], at: usize, size: usize) -> Result<usize, String> {
    let piece = bytes.get(at..at + size).ok_or("a Snappy element ends early")?;
    Ok(piece.iter().rev().fold(0usize, |value, byte| (value << WIRE_BYTE_BITS) | usize::from(*byte)))
}
because!(number_at, SnappyFormat, "a number of some bytes at a place, the least byte first");

fn element_of(bytes: &[u8], at: &mut usize) -> Option<(bool, usize, usize)> {
    let tag = *bytes.get(*at)?;
    *at += 1;
    let kind = tag & ELEMENT_BITS;
    if kind == SNAPPY_LITERAL {
        let small = usize::from(tag >> KIND_SHIFT);
        if small < SHORT_LITERAL {
            return Some((true, small + 1, 0));
        }
        let extra = small - SHORT_LITERAL + 1;
        let value = number_at(bytes, *at, extra).ok()?;
        *at += extra;
        return Some((true, value + 1, 0));
    }
    if kind == COPY_SHORT {
        let low = usize::from(*bytes.get(*at)?);
        *at += 1;
        return Some((false, usize::from((tag >> KIND_SHIFT) & SHORT_LENGTH_BITS) + SHORT_LENGTH_FROM, (usize::from(tag >> SHORT_OFFSET_SHIFT) << WIRE_BYTE_BITS) | low));
    }
    let wide = if kind == COPY_MIDDLE { MIDDLE_OFFSET_BYTES } else { LONG_OFFSET_BYTES };
    let offset = number_at(bytes, *at, wide).ok()?;
    *at += wide;
    Some((false, usize::from(tag >> KIND_SHIFT) + 1, offset))
}
because!(element_of, SnappyFormat, "one element at a place of a block, and the place after it: whether it is a literal, its length, and for a copy how far it reaches back; nothing for an element that ends early");

fn quick_decompress(bytes: &[u8], expected: usize) -> Option<Vec<u8>> {
    let mut at = 0usize;
    let length = wire_number(bytes, &mut at).ok()? as usize;
    if length != expected {
        return None;
    }
    let mut out = vec![0u8; length + WIDE_MOVE];
    let mut pos = 0usize;
    while at < bytes.len() {
        let (literal, size, offset) = element_of(bytes, &mut at)?;
        if pos + size > length {
            return None;
        }
        if literal {
            if at + size > bytes.len() {
                return None;
            }
            if size <= WIDE_MOVE && at + WIDE_MOVE <= bytes.len() {
                out[pos..pos + WIDE_MOVE].copy_from_slice(&bytes[at..at + WIDE_MOVE]);
            } else {
                out[pos..pos + size].copy_from_slice(&bytes[at..at + size]);
            }
            at += size;
            pos += size;
            continue;
        }
        if offset == 0 || offset > pos {
            return None;
        }
        let from = pos - offset;
        if offset >= WIDE_MOVE && size <= WIDE_MOVE {
            out.copy_within(from..from + WIDE_MOVE, pos);
        } else if offset >= size {
            out.copy_within(from..from + size, pos);
        } else {
            for place in 0..size {
                out[pos + place] = out[from + place];
            }
        }
        pos += size;
    }
    if pos != length {
        return None;
    }
    out.truncate(length);
    Some(out)
}
because!(quick_decompress, SnappyFormat, "the bytes of a well formed Snappy block, decoded in place in a buffer of its whole length: a short element moves the bytes of one wide move, which the next elements write again where it went too far; a copy that overlaps the bytes that it writes goes byte by byte, so that it repeats them; nothing for a block that is not well formed");

pub fn decompress(bytes: &[u8], expected: usize) -> Result<Vec<u8>, String> {
    if let Some(out) = quick_decompress(bytes, expected) {
        return Ok(out);
    }
    plain_decompress(bytes, expected)
}
because!(decompress, SnappyFormat, "the bytes of a Snappy block: decoded in place when the block is well formed, else by the reading element by element, which says what is wrong with it");

fn plain_decompress(bytes: &[u8], expected: usize) -> Result<Vec<u8>, String> {
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
because!(plain_decompress, SnappyFormat, "the bytes of a Snappy block, element by element; a copy may overlap the bytes that it writes, so it goes byte by byte; a block whose length is not the length that the page says is refused");
