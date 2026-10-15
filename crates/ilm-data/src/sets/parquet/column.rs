use crate::sets::parquet::hybrid::Hybrid;
use crate::sets::parquet::meta::{BOOLEAN_TYPE, BYTES_TYPE, Chunk, DATA_PAGE, DATA_PAGE_V2, DICTIONARY_PAGE, DOUBLE_TYPE, FIXED_TYPE, FLOAT_TYPE, INT32_TYPE, INT64_TYPE, ENCODING_PLAIN, ENCODING_PLAIN_DICTIONARY, ENCODING_RLE, ENCODING_RLE_DICTIONARY, SNAPPY, UNCOMPRESSED, page_header};
use crate::sets::parquet::snappy::decompress;
use crate::sets::parquet::wire::WIRE_BYTE_BITS;
use patterns::{because, source};

pub struct ColumnPages;
source!(ColumnPages, "the pages of a column chunk of parquet: an optional dictionary page, then pages of values; a page of values holds the levels of repetition and of definition of each value and the values that are there, in the plain form or as numbers into the dictionary; the levels say where a value starts a new list and how deep it is defined");

const FOUR_BYTES: usize = 4;
because!(FOUR_BYTES, ColumnPages, "the bytes of a number of the narrow width of the format: the length before the levels of a page of the first form, the length of a run of bytes, a narrow whole number, a narrow number with a point");

const EIGHT_BYTES: usize = 8;
because!(EIGHT_BYTES, ColumnPages, "the bytes of a wide whole number and of a wide number with a point");

#[derive(Clone, Debug, PartialEq)]
pub enum Leaf {
    Bool(bool),
    Int(i64),
    Double(f64),
    Bytes(Vec<u8>),
}
because!(Leaf, ColumnPages, "one value of a leaf column as the file holds it: a truth value, a whole number, a number with a point (the narrow kind widened), or a run of bytes");

#[derive(Clone, Debug, Default)]
pub struct ColumnData {
    pub repetitions: Vec<u32>,
    pub definitions: Vec<u32>,
    pub values: Vec<Leaf>,
}
because!(ColumnData, ColumnPages, "the levels of every value of a leaf column and the values that are there, in the order of the file");

fn bytes_at<const N: usize>(bytes: &[u8], at: usize) -> Result<[u8; N], String> {
    let mut held = [0u8; N];
    held.copy_from_slice(bytes.get(at..at + N).ok_or("a page ends in a number")?);
    Ok(held)
}
because!(bytes_at, ColumnPages, "the bytes of a number of a fixed width at a place of a page");

pub fn plain_values(bytes: &[u8], count: usize, physical: i32, length: usize) -> Result<Vec<Leaf>, String> {
    let mut out: Vec<Leaf> = Vec::with_capacity(count);
    let mut at = 0usize;
    for place in 0..count {
        let leaf = match physical {
            BOOLEAN_TYPE => {
                let byte = *bytes.get(place / WIRE_BYTE_BITS).ok_or("a page ends in truth values")?;
                Leaf::Bool((byte >> (place % WIRE_BYTE_BITS)) & 1 == 1)
            }
            INT32_TYPE => {
                let value = i32::from_le_bytes(bytes_at::<FOUR_BYTES>(bytes, at)?);
                at += FOUR_BYTES;
                Leaf::Int(i64::from(value))
            }
            INT64_TYPE => {
                let value = i64::from_le_bytes(bytes_at::<EIGHT_BYTES>(bytes, at)?);
                at += EIGHT_BYTES;
                Leaf::Int(value)
            }
            FLOAT_TYPE => {
                let value = f32::from_le_bytes(bytes_at::<FOUR_BYTES>(bytes, at)?);
                at += FOUR_BYTES;
                Leaf::Double(f64::from(value))
            }
            DOUBLE_TYPE => {
                let value = f64::from_le_bytes(bytes_at::<EIGHT_BYTES>(bytes, at)?);
                at += EIGHT_BYTES;
                Leaf::Double(value)
            }
            BYTES_TYPE => {
                let size = u32::from_le_bytes(bytes_at::<FOUR_BYTES>(bytes, at)?) as usize;
                at += FOUR_BYTES;
                let piece = bytes.get(at..at + size).ok_or("a page ends in a run of bytes")?;
                at += size;
                Leaf::Bytes(piece.to_vec())
            }
            FIXED_TYPE => {
                let piece = bytes.get(at..at + length).ok_or("a page ends in a run of bytes of one length")?;
                at += length;
                Leaf::Bytes(piece.to_vec())
            }
            _ => return Err(format!("the physical type {physical} is not read")),
        };
        out.push(leaf);
    }
    Ok(out)
}
because!(plain_values, ColumnPages, "some values in the plain form of their physical type: truth values one bit each, the least bit first; numbers in the order of the least byte first; runs of bytes each after its length of four bytes");

fn levels(bytes: &[u8], width: u32, count: usize) -> Result<Vec<u32>, String> {
    let mut out: Vec<u32> = Vec::with_capacity(count);
    if width == 0 {
        out.resize(count, 0);
        return Ok(out);
    }
    Hybrid::new(bytes, width)?.take(count, &mut out)?;
    Ok(out)
}
because!(levels, ColumnPages, "the levels of the values of a page from runs and packed bits; a column whose largest level is nought has none written and every level is nought");

fn page_values(bytes: &[u8], encoding: i32, count: usize, physical: i32, length: usize, dictionary: &[Leaf]) -> Result<Vec<Leaf>, String> {
    match encoding {
        ENCODING_PLAIN => plain_values(bytes, count, physical, length),
        ENCODING_PLAIN_DICTIONARY | ENCODING_RLE_DICTIONARY => {
            let width = u32::from(*bytes.first().unwrap_or(&0));
            let mut numbers: Vec<u32> = Vec::with_capacity(count);
            if count > 0 {
                Hybrid::new(&bytes[1..], width)?.take(count, &mut numbers)?;
            }
            numbers.iter().map(|number| dictionary.get(*number as usize).cloned().ok_or_else(|| format!("a number {number} past the dictionary of {} values", dictionary.len()))).collect()
        }
        ENCODING_RLE if physical == BOOLEAN_TYPE => {
            let size = u32::from_le_bytes(bytes_at::<FOUR_BYTES>(bytes, 0)?) as usize;
            let mut numbers: Vec<u32> = Vec::with_capacity(count);
            Hybrid::new(bytes.get(FOUR_BYTES..FOUR_BYTES + size).ok_or("truth values in runs end early")?, 1)?.take(count, &mut numbers)?;
            Ok(numbers.into_iter().map(|number| Leaf::Bool(number == 1)).collect())
        }
        _ => Err(format!("the encoding {encoding} of values is not read")),
    }
}
because!(page_values, ColumnPages, "the values that are there in a page, by the encoding of the page: the plain form, numbers into the dictionary after one byte with their width, or truth values in runs after the length of the runs");

fn unpacked(body: &[u8], codec: i32, size: usize) -> Result<Vec<u8>, String> {
    match codec {
        UNCOMPRESSED => Ok(body.to_vec()),
        SNAPPY => decompress(body, size),
        _ => Err(format!("the codec {codec} is not read")),
    }
}
because!(unpacked, ColumnPages, "the bytes of a page as they were before the codec of its chunk");

pub fn read_chunk(bytes: &[u8], chunk: &Chunk, length: usize, most_definition: u32, most_repetition: u32) -> Result<ColumnData, String> {
    let definition_width = crate::sets::parquet::hybrid::width_of(most_definition);
    let repetition_width = crate::sets::parquet::hybrid::width_of(most_repetition);
    let mut data = ColumnData::default();
    let mut dictionary: Vec<Leaf> = Vec::new();
    let mut at = 0usize;
    let goal = chunk.values as usize;
    while data.definitions.len() < goal {
        let (header, used) = page_header(bytes.get(at..).ok_or("a column chunk ends before its last page")?)?;
        at += used;
        let body = bytes.get(at..at + header.compressed).ok_or("a page ends after its column chunk")?;
        at += header.compressed;
        match header.kind {
            DICTIONARY_PAGE => {
                let page = unpacked(body, chunk.codec, header.uncompressed)?;
                dictionary = plain_values(&page, header.values, chunk.physical, length)?;
            }
            DATA_PAGE => {
                let page = unpacked(body, chunk.codec, header.uncompressed)?;
                let mut place = 0usize;
                let mut leveled = |width: u32| -> Result<Vec<u32>, String> {
                    if width == 0 {
                        return levels(&[], 0, header.values);
                    }
                    let size = u32::from_le_bytes(bytes_at::<FOUR_BYTES>(&page, place)?) as usize;
                    let start = place + FOUR_BYTES;
                    place = start + size;
                    levels(page.get(start..start + size).ok_or("the levels of a page end early")?, width, header.values)
                };
                let repetitions = leveled(repetition_width)?;
                let definitions = leveled(definition_width)?;
                let present = definitions.iter().filter(|level| **level == most_definition).count();
                let values = page_values(&page[place..], header.encoding, present, chunk.physical, length, &dictionary)?;
                data.repetitions.extend(repetitions);
                data.definitions.extend(definitions);
                data.values.extend(values);
            }
            DATA_PAGE_V2 => {
                let levels_end = header.repetition_bytes + header.definition_bytes;
                let repetitions = levels(body.get(..header.repetition_bytes).ok_or("a page ends in its levels")?, repetition_width, header.values)?;
                let definitions = levels(body.get(header.repetition_bytes..levels_end).ok_or("a page ends in its levels")?, definition_width, header.values)?;
                let rest = &body[levels_end..];
                let page = if header.values_compressed { unpacked(rest, chunk.codec, header.uncompressed - levels_end)? } else { rest.to_vec() };
                let present = definitions.iter().filter(|level| **level == most_definition).count();
                let values = page_values(&page, header.encoding, present, chunk.physical, length, &dictionary)?;
                data.repetitions.extend(repetitions);
                data.definitions.extend(definitions);
                data.values.extend(values);
            }
            _ => {}
        }
    }
    Ok(data)
}
because!(read_chunk, ColumnPages, "the levels and the values of one column chunk from its bytes: the pages one after the other until the chunk has given as many levels as its metadata says; a page of the first form is unpacked whole and holds a length before each run of levels, a page of the second form holds its levels first, as they are, and opens only its values");
