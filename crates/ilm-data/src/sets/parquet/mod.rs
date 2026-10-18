pub mod column;
pub mod hybrid;
pub mod meta;
pub mod records;
pub mod show;
pub mod snappy;
pub mod thrift;
pub mod wire;

use ilm_core::base::symbols::byte_hash;
use crate::sets::parquet::column::read_chunk;
use crate::sets::parquet::meta::{FileMeta, file_meta};
use crate::sets::parquet::records::{Node, Value, assemble, joined, leaf_paths, schema_tree};
use patterns::{because, source};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub struct ParquetFile;
source!(ParquetFile, "a file of the Apache Parquet format: the bytes of the magic word at its start and end, the row groups with their column chunks, and at the end the metadata in compact thrift with its length of four bytes before the closing magic word");

const MAGIC: [u8; 4] = [80, 65, 82, 49];
because!(MAGIC, ParquetFile, "the magic word of the format, the four letters that open and close every file");

const LENGTH_BYTES: usize = 4;
because!(LENGTH_BYTES, ParquetFile, "the bytes of the length of the metadata before the closing magic word");

const MARK_NOTHING: u8 = b'n';
because!(MARK_NOTHING, ParquetFile, "the mark of a missing value in the fixed form of a value");

const MARK_TRUE: u8 = b't';
because!(MARK_TRUE, ParquetFile, "the mark of a true value in the fixed form of a value");

const MARK_FALSE: u8 = b'f';
because!(MARK_FALSE, ParquetFile, "the mark of a false value in the fixed form of a value");

const MARK_WHOLE: u8 = b'i';
because!(MARK_WHOLE, ParquetFile, "the mark before a whole number in the fixed form of a value");

const MARK_POINT: u8 = b'd';
because!(MARK_POINT, ParquetFile, "the mark before the bits of a number with a point in the fixed form of a value");

const MARK_TEXT: u8 = b's';
because!(MARK_TEXT, ParquetFile, "the mark before the length of a text in the fixed form of a value");

const MARK_BYTES: u8 = b'b';
because!(MARK_BYTES, ParquetFile, "the mark before the length of bytes in the fixed form of a value");

const MARK_END: u8 = b';';
because!(MARK_END, ParquetFile, "the mark after a number, so that two numbers in a row stay apart in the fixed form of a value");

const MARK_LENGTH: u8 = b':';
because!(MARK_LENGTH, ParquetFile, "the mark after a length, before the bytes that it counts in the fixed form of a value");

const MARK_LIST_OPEN: u8 = b'[';
because!(MARK_LIST_OPEN, ParquetFile, "the mark that opens a list in the fixed form of a value");

const MARK_LIST_CLOSE: u8 = b']';
because!(MARK_LIST_CLOSE, ParquetFile, "the mark that closes a list in the fixed form of a value");

const MARK_AFTER: u8 = b',';
because!(MARK_AFTER, ParquetFile, "the mark after each value of a list and each field of named values in the fixed form of a value");

const MARK_STRUCT_OPEN: u8 = b'{';
because!(MARK_STRUCT_OPEN, ParquetFile, "the mark that opens named values in the fixed form of a value");

const MARK_STRUCT_CLOSE: u8 = b'}';
because!(MARK_STRUCT_CLOSE, ParquetFile, "the mark that closes named values in the fixed form of a value");

const MARK_NAMED: u8 = b'=';
because!(MARK_NAMED, ParquetFile, "the mark between the name of a field and its value in the fixed form of a value");

const FOLD_TURN: u32 = 5;
because!(FOLD_TURN, ParquetFile, "the bits by which the digest of a column is turned before the hash of the next row is mixed in, so that the order of the rows counts");

pub struct ParquetTable {
    file: std::fs::File,
    pub meta: FileMeta,
    pub tree: Node,
}
because!(ParquetTable, ParquetFile, "an open parquet file with its metadata and its schema as a tree. names gives the top fields; groups the number of row groups; read_group gives the records of some top fields in one row group as values, one list for each field in the order asked; read gives them for the whole file, the row groups one after the other");

pub fn open(place: &Path) -> Result<ParquetTable, String> {
    let mut file = std::fs::File::open(place).map_err(|error| format!("{}: {error}", place.display()))?;
    let size = file.metadata().map_err(|error| error.to_string())?.len() as usize;
    let tail = LENGTH_BYTES + MAGIC.len();
    if size < tail + MAGIC.len() {
        return Err(format!("{}: too short for a parquet file", place.display()));
    }
    let mut head = [0u8; LENGTH_BYTES];
    file.read_exact(&mut head).map_err(|error| error.to_string())?;
    if head != MAGIC {
        return Err(format!("{}: no parquet file", place.display()));
    }
    let mut end = vec![0u8; tail];
    file.seek(SeekFrom::Start((size - tail) as u64)).map_err(|error| error.to_string())?;
    file.read_exact(&mut end).map_err(|error| error.to_string())?;
    if end[LENGTH_BYTES..] != MAGIC {
        return Err(format!("{}: a parquet file with no closing magic word", place.display()));
    }
    let mut length = [0u8; LENGTH_BYTES];
    length.copy_from_slice(&end[..LENGTH_BYTES]);
    let length = u32::from_le_bytes(length) as usize;
    let start = size.checked_sub(tail + length).ok_or("the metadata is longer than the file")?;
    let mut bytes = vec![0u8; length];
    file.seek(SeekFrom::Start(start as u64)).map_err(|error| error.to_string())?;
    file.read_exact(&mut bytes).map_err(|error| error.to_string())?;
    let meta = file_meta(&bytes)?;
    let tree = schema_tree(&meta.schema)?;
    Ok(ParquetTable { file, meta, tree })
}
because!(open, ParquetFile, "a parquet file opened by its footer: the magic word at both ends, then the metadata whose length stands before the closing word");

impl ParquetTable {
    pub fn names(&self) -> Vec<String> {
        self.tree.children.iter().map(|child| child.element.name.clone()).collect()
    }

    pub fn groups(&self) -> usize {
        self.meta.groups.len()
    }

    pub fn read(&mut self, names: &[&str]) -> Result<Vec<Vec<Value>>, String> {
        let mut out: Vec<Vec<Value>> = vec![Vec::new(); names.len()];
        for group in 0..self.groups() {
            for (slot, column) in self.read_group(group, names)?.into_iter().enumerate() {
                out[slot].extend(column);
            }
        }
        Ok(out)
    }

    pub fn read_group(&mut self, group: usize, names: &[&str]) -> Result<Vec<Vec<Value>>, String> {
        let mut leaf_of_top: Vec<(usize, Vec<crate::sets::parquet::records::PathStep>)> = Vec::new();
        let mut first_leaf = 0usize;
        let mut wanted: Vec<Vec<usize>> = vec![Vec::new(); names.len()];
        for child in &self.tree.children {
            let paths = leaf_paths(child);
            if let Some(slot) = names.iter().position(|name| *name == child.element.name) {
                for (number, place) in paths.iter().enumerate() {
                    wanted[slot].push(leaf_of_top.len());
                    leaf_of_top.push((first_leaf + number, place.clone()));
                }
            }
            first_leaf += paths.len();
        }
        if let Some(missing) = names.iter().zip(&wanted).find(|(_, leaves)| leaves.is_empty()) {
            return Err(format!("no column {} in the file", missing.0));
        }
        let group = self.meta.groups.get(group).cloned().ok_or("no such row group")?;
        let mut by_leaf: Vec<Vec<Value>> = Vec::with_capacity(leaf_of_top.len());
        for (leaf, place) in &leaf_of_top {
            let chunk = group.chunks.get(*leaf).ok_or("a row group has fewer chunks than the schema has leaves")?;
            let start = chunk.dictionary_page.filter(|place| *place > 0 && *place < chunk.data_page).unwrap_or(chunk.data_page) as u64;
            let mut bytes = vec![0u8; chunk.compressed as usize];
            self.file.seek(SeekFrom::Start(start)).map_err(|error| error.to_string())?;
            self.file.read_exact(&mut bytes).map_err(|error| error.to_string())?;
            let last = place.last().ok_or("a leaf with no place")?;
            let length = usize::try_from(last.element.length).unwrap_or_default();
            let mut data = read_chunk(&bytes, chunk, length, last.definition, last.repetition)?;
            by_leaf.push(assemble(place, &mut data, group.rows as usize)?);
        }
        let mut out: Vec<Vec<Value>> = Vec::with_capacity(names.len());
        for leaves in &wanted {
            let columns: Vec<Vec<Value>> = leaves.iter().map(|leaf| std::mem::take(&mut by_leaf[*leaf])).collect();
            out.push(joined(columns)?);
        }
        Ok(out)
    }
}

fn canonical(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Null => out.push(MARK_NOTHING),
        Value::Bool(truth) => out.push(if *truth { MARK_TRUE } else { MARK_FALSE }),
        Value::Int(number) => {
            out.push(MARK_WHOLE);
            out.extend_from_slice(number.to_string().as_bytes());
            out.push(MARK_END);
        }
        Value::Double(number) => {
            out.push(MARK_POINT);
            out.extend_from_slice(number.to_bits().to_string().as_bytes());
            out.push(MARK_END);
        }
        Value::Text(text) => {
            out.push(MARK_TEXT);
            out.extend_from_slice(text.len().to_string().as_bytes());
            out.push(MARK_LENGTH);
            out.extend_from_slice(text.as_bytes());
        }
        Value::Bytes(bytes) => {
            out.push(MARK_BYTES);
            out.extend_from_slice(bytes.len().to_string().as_bytes());
            out.push(MARK_LENGTH);
            out.extend_from_slice(bytes);
        }
        Value::List(items) => {
            out.push(MARK_LIST_OPEN);
            for item in items {
                canonical(item, out);
                out.push(MARK_AFTER);
            }
            out.push(MARK_LIST_CLOSE);
        }
        Value::Struct(fields) => {
            out.push(MARK_STRUCT_OPEN);
            for (name, item) in fields {
                out.extend_from_slice(name.len().to_string().as_bytes());
                out.push(MARK_LENGTH);
                out.extend_from_slice(name.as_bytes());
                out.push(MARK_NAMED);
                canonical(item, out);
                out.push(MARK_AFTER);
            }
            out.push(MARK_STRUCT_CLOSE);
        }
    }
}
because!(canonical, ParquetFile, "a value written as bytes in one fixed form: a letter for its kind, numbers in decimal (a number with a point by its bits), a text or bytes after their length, lists and named values with their marks; so two readers of one file can be compared by a hash of what they read");

pub fn digest(rows: &[Value]) -> u64 {
    let mut held: u64 = 0;
    let mut bytes: Vec<u8> = Vec::new();
    for row in rows {
        bytes.clear();
        canonical(row, &mut bytes);
        held = held.rotate_left(FOLD_TURN) ^ byte_hash(&bytes);
    }
    held
}
because!(digest, ParquetFile, "the digest of a column: the hash of the fixed form of each row, folded in the order of the rows");
