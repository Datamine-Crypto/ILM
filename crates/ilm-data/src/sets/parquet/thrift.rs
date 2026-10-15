use crate::sets::parquet::wire::{wire_number};
use patterns::{because, source};

pub struct CompactThrift;
source!(CompactThrift, "the compact protocol of Apache Thrift, in which the metadata of a parquet file is written: a field is a byte with the step from the number of the last field and the type, numbers are variable-length with the zig-zag form for signed ones, a binary value is its length and its bytes, a list is a byte with its size and the type of its elements, and a struct ends with a byte of nought");

pub const THRIFT_STOP: u8 = 0;
because!(THRIFT_STOP, CompactThrift, "the type of the byte that ends a struct");

pub const THRIFT_TRUE: u8 = 1;
because!(THRIFT_TRUE, CompactThrift, "the type of a field that is a true value, and the byte of a true value inside a list");

pub const THRIFT_FALSE: u8 = 2;
because!(THRIFT_FALSE, CompactThrift, "the type of a field that is a false value");

pub const THRIFT_BYTE: u8 = 3;
because!(THRIFT_BYTE, CompactThrift, "the type of a field of one byte");

pub const THRIFT_I16: u8 = 4;
because!(THRIFT_I16, CompactThrift, "the type of a field of a small whole number, written as a variable-length number");

pub const THRIFT_I32: u8 = 5;
because!(THRIFT_I32, CompactThrift, "the type of a field of a whole number of the common width, written as a variable-length number");

pub const THRIFT_I64: u8 = 6;
because!(THRIFT_I64, CompactThrift, "the type of a field of a wide whole number, written as a variable-length number");

pub const THRIFT_DOUBLE: u8 = 7;
because!(THRIFT_DOUBLE, CompactThrift, "the type of a field of a number with a point, written in its eight bytes");

pub const THRIFT_BINARY: u8 = 8;
because!(THRIFT_BINARY, CompactThrift, "the type of a field of bytes or of a text: its length, then its bytes");

pub const THRIFT_LIST: u8 = 9;
because!(THRIFT_LIST, CompactThrift, "the type of a field that is a row of values of one type after a byte with their number and their type");

pub const THRIFT_SET: u8 = 10;
because!(THRIFT_SET, CompactThrift, "the type of a field that is a set, written as a list is");

pub const THRIFT_MAP: u8 = 11;
because!(THRIFT_MAP, CompactThrift, "the type of a field that is a map: its size, the types of its keys and values, then the pairs");

pub const THRIFT_STRUCT: u8 = 12;
because!(THRIFT_STRUCT, CompactThrift, "the type of a field that is a struct of fields");

const TYPE_BITS: u8 = 15;
because!(TYPE_BITS, CompactThrift, "the mask of the low half of a byte, which holds a type");

const HALF_BYTE: u32 = 4;
because!(HALF_BYTE, CompactThrift, "the bits of half a byte: the high half of a field byte holds the step of the field number, the high half of a list byte its size");

const LONG_LIST: u8 = 15;
because!(LONG_LIST, CompactThrift, "the size in a list byte that says the size follows as a variable-length number");

const DOUBLE_BYTES: usize = 8;
because!(DOUBLE_BYTES, CompactThrift, "the width of a number with a point in thrift, which writes it in the wide form of IEEE");

const THRIFT_DEEPEST: usize = 64;
because!(THRIFT_DEEPEST, CompactThrift, "the depth of nested structs and lists at most: metadata of a file is far less deep, and a bad file must not run without end");

pub struct Reader<'a> {
    bytes: &'a [u8],
    pub at: usize,
    last: Vec<i16>,
}
because!(Reader, CompactThrift, "a reader of compact thrift over some bytes: the place it has come to, and the number of the last field of each open struct. field gives the next field of the struct, or nothing at its end; the other calls read one value of a type; skip passes over a value of any type");

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { bytes, at: 0, last: vec![0] }
    }

    pub fn byte(&mut self) -> Result<u8, String> {
        let byte = *self.bytes.get(self.at).ok_or_else(|| format!("the metadata ends at byte {} in the middle of a value", self.at))?;
        self.at += 1;
        Ok(byte)
    }

    pub fn varint(&mut self) -> Result<u64, String> {
        wire_number(self.bytes, &mut self.at)
    }

    pub fn signed(&mut self) -> Result<i64, String> {
        let raw = self.varint()?;
        Ok((raw >> 1) as i64 ^ -((raw & 1) as i64))
    }

    pub fn double(&mut self) -> Result<f64, String> {
        let end = self.at + DOUBLE_BYTES;
        let bytes = self.bytes.get(self.at..end).ok_or("the metadata ends in a number with a point")?;
        self.at = end;
        let mut held = [0u8; DOUBLE_BYTES];
        held.copy_from_slice(bytes);
        Ok(f64::from_le_bytes(held))
    }

    pub fn binary(&mut self) -> Result<&'a [u8], String> {
        let size = self.varint()? as usize;
        let end = self.at.checked_add(size).filter(|end| *end <= self.bytes.len()).ok_or("the metadata ends in a binary value")?;
        let bytes = &self.bytes[self.at..end];
        self.at = end;
        Ok(bytes)
    }

    pub fn text(&mut self) -> Result<String, String> {
        let bytes = self.binary()?;
        String::from_utf8(bytes.to_vec()).map_err(|_| "a text of the metadata is no text".to_string())
    }

    pub fn field(&mut self) -> Result<Option<(i16, u8)>, String> {
        let byte = self.byte()?;
        let kind = byte & TYPE_BITS;
        if kind == THRIFT_STOP {
            return Ok(None);
        }
        let step = i16::from(byte >> HALF_BYTE);
        let last = self.last.last().copied().unwrap_or_default();
        let number = if step == 0 { self.signed()? as i16 } else { last + step };
        if let Some(held) = self.last.last_mut() {
            *held = number;
        }
        Ok(Some((number, kind)))
    }

    pub fn open(&mut self) -> Result<(), String> {
        if self.last.len() >= THRIFT_DEEPEST {
            return Err("the metadata nests too deep".to_string());
        }
        self.last.push(0);
        Ok(())
    }

    pub fn close(&mut self) {
        self.last.pop();
    }

    pub fn list(&mut self) -> Result<(usize, u8), String> {
        let byte = self.byte()?;
        let small = byte >> HALF_BYTE;
        let size = if small == LONG_LIST { self.varint()? as usize } else { usize::from(small) };
        Ok((size, byte & TYPE_BITS))
    }

    pub fn truth(&mut self, kind: u8) -> Result<bool, String> {
        match kind {
            THRIFT_TRUE => Ok(true),
            THRIFT_FALSE => Ok(false),
            _ => Ok(self.byte()? == THRIFT_TRUE),
        }
    }

    pub fn skip(&mut self, kind: u8) -> Result<(), String> {
        match kind {
            THRIFT_TRUE | THRIFT_FALSE => Ok(()),
            THRIFT_BYTE => self.byte().map(|_| ()),
            THRIFT_I16 | THRIFT_I32 | THRIFT_I64 => self.varint().map(|_| ()),
            THRIFT_DOUBLE => self.double().map(|_| ()),
            THRIFT_BINARY => self.binary().map(|_| ()),
            THRIFT_LIST | THRIFT_SET => {
                let (size, inner) = self.list()?;
                for _ in 0..size {
                    if inner == THRIFT_TRUE || inner == THRIFT_FALSE {
                        self.byte()?;
                    } else {
                        self.skip(inner)?;
                    }
                }
                Ok(())
            }
            THRIFT_MAP => {
                let size = self.varint()? as usize;
                if size > 0 {
                    let kinds = self.byte()?;
                    for _ in 0..size {
                        self.skip(kinds >> HALF_BYTE)?;
                        self.skip(kinds & TYPE_BITS)?;
                    }
                }
                Ok(())
            }
            THRIFT_STRUCT => {
                self.open()?;
                while let Some((_, inner)) = self.field()? {
                    self.skip(inner)?;
                }
                self.close();
                Ok(())
            }
            _ => Err(format!("an unknown type {kind} at byte {} of the metadata", self.at)),
        }
    }
}
