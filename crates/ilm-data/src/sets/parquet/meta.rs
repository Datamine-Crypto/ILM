use crate::sets::parquet::thrift::{THRIFT_BINARY, THRIFT_LIST, Reader, THRIFT_I64, THRIFT_STRUCT, THRIFT_I32};
use patterns::{because, source};

pub struct ParquetMeta;
source!(ParquetMeta, "the metadata of a parquet file as the format of Apache Parquet defines it in parquet.thrift: the schema as a flat list of elements in the order of a walk from the root, the row groups with a chunk for each column, and before each page a header with its type, its sizes and the encodings of its levels and values");

pub const BOOLEAN_TYPE: i32 = 0;
because!(BOOLEAN_TYPE, ParquetMeta, "the physical type of a column of truth values");

pub const INT32_TYPE: i32 = 1;
because!(INT32_TYPE, ParquetMeta, "the physical type of a column of whole numbers of four bytes");

pub const INT64_TYPE: i32 = 2;
because!(INT64_TYPE, ParquetMeta, "the physical type of a column of whole numbers of eight bytes");

pub const FLOAT_TYPE: i32 = 4;
because!(FLOAT_TYPE, ParquetMeta, "the physical type of a column of numbers with a point of the narrow width");

pub const DOUBLE_TYPE: i32 = 5;
because!(DOUBLE_TYPE, ParquetMeta, "the physical type of a column of numbers with a point in eight bytes");

pub const BYTES_TYPE: i32 = 6;
because!(BYTES_TYPE, ParquetMeta, "the physical type of a column of runs of bytes of any length, which holds texts");

pub const FIXED_TYPE: i32 = 7;
because!(FIXED_TYPE, ParquetMeta, "the physical type of a column of runs of bytes of one length");

pub const REQUIRED: i32 = 0;
because!(REQUIRED, ParquetMeta, "the repetition of a field that every record holds once");

pub const OPTIONAL: i32 = 1;
because!(OPTIONAL, ParquetMeta, "the repetition of a field that a record holds once or not");

pub const REPEATED: i32 = 2;
because!(REPEATED, ParquetMeta, "the repetition of a field that a record holds any number of times");

pub const CONVERTED_UTF8: i32 = 0;
because!(CONVERTED_UTF8, ParquetMeta, "the old annotation of bytes that are a text");

pub const CONVERTED_MAP: i32 = 1;
because!(CONVERTED_MAP, ParquetMeta, "the old annotation of a group that is a map");

pub const CONVERTED_MAP_KEY_VALUE: i32 = 2;
because!(CONVERTED_MAP_KEY_VALUE, ParquetMeta, "the old annotation of the repeated group of the pairs of a map");

pub const CONVERTED_LIST: i32 = 3;
because!(CONVERTED_LIST, ParquetMeta, "the old annotation of a group that is a list");

pub const CONVERTED_ENUM: i32 = 4;
because!(CONVERTED_ENUM, ParquetMeta, "the old annotation of bytes that are one of some named values, read as a text");

pub const CONVERTED_JSON: i32 = 19;
because!(CONVERTED_JSON, ParquetMeta, "the old annotation of bytes that are a text in the form of JSON");

pub const LOGICAL_STRING: i16 = 1;
because!(LOGICAL_STRING, ParquetMeta, "the field of the union of logical types that says the bytes are a text");

pub const LOGICAL_MAP: i16 = 2;
because!(LOGICAL_MAP, ParquetMeta, "the field of the union of logical types that says a group is a map");

pub const LOGICAL_LIST: i16 = 3;
because!(LOGICAL_LIST, ParquetMeta, "the field of the union of logical types that says a group is a list");

pub const LOGICAL_ENUM: i16 = 4;
because!(LOGICAL_ENUM, ParquetMeta, "the field of the union of logical types that says the bytes are one of some named values");

pub const LOGICAL_UNKNOWN: i16 = 11;
because!(LOGICAL_UNKNOWN, ParquetMeta, "the field of the union of logical types that says every value of the column is missing");

pub const LOGICAL_JSON: i16 = 12;
because!(LOGICAL_JSON, ParquetMeta, "the field of the union of logical types that says the bytes are a text in the form of JSON");

pub const ENCODING_PLAIN: i32 = 0;
because!(ENCODING_PLAIN, ParquetMeta, "the encoding of values one after the other in their plain form");

pub const ENCODING_PLAIN_DICTIONARY: i32 = 2;
because!(ENCODING_PLAIN_DICTIONARY, ParquetMeta, "the old name of values given as numbers into the dictionary of the chunk, and of the plain form of that dictionary");

pub const ENCODING_RLE: i32 = 3;
because!(ENCODING_RLE, ParquetMeta, "the encoding of runs of one value and packed bits mixed, for levels and truth values");

pub const ENCODING_RLE_DICTIONARY: i32 = 8;
because!(ENCODING_RLE_DICTIONARY, ParquetMeta, "the encoding of values as numbers into the dictionary of the chunk, written as runs and packed bits");

pub const UNCOMPRESSED: i32 = 0;
because!(UNCOMPRESSED, ParquetMeta, "the codec of pages that are written as they are");

pub const SNAPPY: i32 = 1;
because!(SNAPPY, ParquetMeta, "the codec of pages that are compressed with Snappy");

pub const DATA_PAGE: i32 = 0;
because!(DATA_PAGE, ParquetMeta, "the type of a page of values in the first form: levels and values compressed together");

pub const DICTIONARY_PAGE: i32 = 2;
because!(DICTIONARY_PAGE, ParquetMeta, "the type of the page that holds the dictionary of a chunk");

pub const DATA_PAGE_V2: i32 = 3;
because!(DATA_PAGE_V2, ParquetMeta, "the type of a page of values in the second form: the levels first and never compressed, then the values");

const FILE_SCHEMA: i16 = 2;
because!(FILE_SCHEMA, ParquetMeta, "the field of the file metadata with the list of schema elements");

const FILE_ROWS: i16 = 3;
because!(FILE_ROWS, ParquetMeta, "the field of the file metadata with the number of rows");

const FILE_GROUPS: i16 = 4;
because!(FILE_GROUPS, ParquetMeta, "the field of the file metadata with the list of row groups");

const ELEMENT_TYPE: i16 = 1;
because!(ELEMENT_TYPE, ParquetMeta, "the field of a schema element with its physical type, which a group has not");

const ELEMENT_LENGTH: i16 = 2;
because!(ELEMENT_LENGTH, ParquetMeta, "the field of a schema element with the length of its runs of bytes of one length");

const ELEMENT_REPETITION: i16 = 3;
because!(ELEMENT_REPETITION, ParquetMeta, "the field of a schema element that says whether a record holds the field once, may lack it, or holds it any number of times");

const ELEMENT_NAME: i16 = 4;
because!(ELEMENT_NAME, ParquetMeta, "the field of a schema element with the name by which a reader asks for a column");

const ELEMENT_CHILDREN: i16 = 5;
because!(ELEMENT_CHILDREN, ParquetMeta, "the field of a schema element with the number of its children, which only a group has");

const ELEMENT_CONVERTED: i16 = 6;
because!(ELEMENT_CONVERTED, ParquetMeta, "the field of a schema element with its old annotation");

const ELEMENT_LOGICAL: i16 = 10;
because!(ELEMENT_LOGICAL, ParquetMeta, "the field of a schema element with its logical type, a union of which one field is set");

const GROUP_COLUMNS: i16 = 1;
because!(GROUP_COLUMNS, ParquetMeta, "the field of a row group with its list of column chunks");

const GROUP_ROWS: i16 = 3;
because!(GROUP_ROWS, ParquetMeta, "the field of a row group with its number of rows");

const CHUNK_META: i16 = 3;
because!(CHUNK_META, ParquetMeta, "the field of a column chunk with its metadata");

const COLUMN_TYPE: i16 = 1;
because!(COLUMN_TYPE, ParquetMeta, "the field of the metadata of a column chunk with its physical type");

const COLUMN_PATH: i16 = 3;
because!(COLUMN_PATH, ParquetMeta, "the field of the metadata of a column chunk with the path of its column in the schema");

const COLUMN_CODEC: i16 = 4;
because!(COLUMN_CODEC, ParquetMeta, "the field of the metadata of a column chunk with its codec");

const COLUMN_VALUES: i16 = 5;
because!(COLUMN_VALUES, ParquetMeta, "the field of the metadata of a column chunk with its number of values, levels counted");

const COLUMN_COMPRESSED: i16 = 7;
because!(COLUMN_COMPRESSED, ParquetMeta, "the field of the metadata of a column chunk with the size of its pages as they are in the file");

const COLUMN_DATA_PAGE: i16 = 9;
because!(COLUMN_DATA_PAGE, ParquetMeta, "the field of the metadata of a column chunk with the place of its first page of values");

const COLUMN_DICTIONARY_PAGE: i16 = 11;
because!(COLUMN_DICTIONARY_PAGE, ParquetMeta, "the field of the metadata of a column chunk with the place of its dictionary page");

const PAGE_TYPE: i16 = 1;
because!(PAGE_TYPE, ParquetMeta, "the field of a page header that says whether the page holds a dictionary, values in the first form, or values in the second form");

const PAGE_UNCOMPRESSED: i16 = 2;
because!(PAGE_UNCOMPRESSED, ParquetMeta, "the field of a page header with the size of the page when it is not compressed");

const PAGE_COMPRESSED: i16 = 3;
because!(PAGE_COMPRESSED, ParquetMeta, "the field of a page header with the size of the page in the file");

const PAGE_DATA: i16 = 5;
because!(PAGE_DATA, ParquetMeta, "the field of a page header with the header of a page of values in the first form");

const PAGE_DICTIONARY: i16 = 7;
because!(PAGE_DICTIONARY, ParquetMeta, "the field of a page header with the header of a dictionary page");

const PAGE_DATA_V2: i16 = 8;
because!(PAGE_DATA_V2, ParquetMeta, "the field of a page header with the header of a page of values in the second form");

const DATA_VALUES: i16 = 1;
because!(DATA_VALUES, ParquetMeta, "the field of a page of values with its number of values, levels counted");

const DATA_ENCODING: i16 = 2;
because!(DATA_ENCODING, ParquetMeta, "the field of a page of values in the first form with the encoding of its values");

const V2_ENCODING: i16 = 4;
because!(V2_ENCODING, ParquetMeta, "the field of a page of values in the second form with the encoding of its values");

const V2_DEFINITION_BYTES: i16 = 5;
because!(V2_DEFINITION_BYTES, ParquetMeta, "the field of a page of values in the second form with the bytes of its levels of definition");

const V2_REPETITION_BYTES: i16 = 6;
because!(V2_REPETITION_BYTES, ParquetMeta, "the field of a page of values in the second form with the bytes of its levels of repetition");

const V2_COMPRESSED: i16 = 7;
because!(V2_COMPRESSED, ParquetMeta, "the field of a page of values in the second form that says whether its values are compressed; when it is missing they are");

const DICTIONARY_ENCODING: i16 = 2;
because!(DICTIONARY_ENCODING, ParquetMeta, "the field of a dictionary page with the encoding of its values");

#[derive(Clone, Debug, Default)]
pub struct Element {
    pub physical: Option<i32>,
    pub length: i32,
    pub repetition: i32,
    pub name: String,
    pub children: usize,
    pub converted: Option<i32>,
    pub logical: Option<i16>,
}
because!(Element, ParquetMeta, "one element of the schema: its physical type (a group has none), the length of fixed runs, its repetition, its name, the number of its children, and its annotations");

#[derive(Clone, Debug, Default)]
pub struct Chunk {
    pub physical: i32,
    pub path: Vec<String>,
    pub codec: i32,
    pub values: i64,
    pub compressed: i64,
    pub data_page: i64,
    pub dictionary_page: Option<i64>,
}
because!(Chunk, ParquetMeta, "the metadata of one column chunk of a row group: the physical type, the path of the column, the codec, the number of values, the size in the file and the places of its first pages");

#[derive(Clone, Debug, Default)]
pub struct Group {
    pub rows: i64,
    pub chunks: Vec<Chunk>,
}
because!(Group, ParquetMeta, "one row group: its number of rows and its column chunks in the order of the leaf columns");

#[derive(Clone, Debug, Default)]
pub struct FileMeta {
    pub schema: Vec<Element>,
    pub rows: i64,
    pub groups: Vec<Group>,
}
because!(FileMeta, ParquetMeta, "the metadata of a file: the schema, the number of rows and the row groups");

#[derive(Clone, Debug, Default)]
pub struct PageHeader {
    pub kind: i32,
    pub uncompressed: usize,
    pub compressed: usize,
    pub values: usize,
    pub encoding: i32,
    pub definition_bytes: usize,
    pub repetition_bytes: usize,
    pub values_compressed: bool,
}
because!(PageHeader, ParquetMeta, "what the header of a page says: its type, its two sizes, its number of values, the encoding of its values, and for a page of the second form the bytes of its levels and whether its values are compressed");

fn element(reader: &mut Reader) -> Result<Element, String> {
    let mut found = Element::default();
    reader.open()?;
    while let Some((number, kind)) = reader.field()? {
        match (number, kind) {
            (ELEMENT_TYPE, THRIFT_I32) => found.physical = Some(reader.signed()? as i32),
            (ELEMENT_LENGTH, THRIFT_I32) => found.length = reader.signed()? as i32,
            (ELEMENT_REPETITION, THRIFT_I32) => found.repetition = reader.signed()? as i32,
            (ELEMENT_NAME, THRIFT_BINARY) => found.name = reader.text()?,
            (ELEMENT_CHILDREN, THRIFT_I32) => found.children = reader.signed()? as usize,
            (ELEMENT_CONVERTED, THRIFT_I32) => found.converted = Some(reader.signed()? as i32),
            (ELEMENT_LOGICAL, THRIFT_STRUCT) => {
                reader.open()?;
                while let Some((which, inner)) = reader.field()? {
                    found.logical = Some(which);
                    reader.skip(inner)?;
                }
                reader.close();
            }
            _ => reader.skip(kind)?,
        }
    }
    reader.close();
    Ok(found)
}
because!(element, ParquetMeta, "one schema element from the metadata");

fn chunk(reader: &mut Reader) -> Result<Chunk, String> {
    let mut found = Chunk::default();
    reader.open()?;
    while let Some((number, kind)) = reader.field()? {
        if (number, kind) != (CHUNK_META, THRIFT_STRUCT) {
            reader.skip(kind)?;
            continue;
        }
        reader.open()?;
        while let Some((inner, inner_kind)) = reader.field()? {
            match (inner, inner_kind) {
                (COLUMN_TYPE, THRIFT_I32) => found.physical = reader.signed()? as i32,
                (COLUMN_PATH, THRIFT_LIST) => {
                    let (size, _) = reader.list()?;
                    for _ in 0..size {
                        found.path.push(reader.text()?);
                    }
                }
                (COLUMN_CODEC, THRIFT_I32) => found.codec = reader.signed()? as i32,
                (COLUMN_VALUES, THRIFT_I64) => found.values = reader.signed()?,
                (COLUMN_COMPRESSED, THRIFT_I64) => found.compressed = reader.signed()?,
                (COLUMN_DATA_PAGE, THRIFT_I64) => found.data_page = reader.signed()?,
                (COLUMN_DICTIONARY_PAGE, THRIFT_I64) => found.dictionary_page = Some(reader.signed()?),
                _ => reader.skip(inner_kind)?,
            }
        }
        reader.close();
    }
    reader.close();
    Ok(found)
}
because!(chunk, ParquetMeta, "one column chunk with its metadata");

fn group(reader: &mut Reader) -> Result<Group, String> {
    let mut found = Group::default();
    reader.open()?;
    while let Some((number, kind)) = reader.field()? {
        match (number, kind) {
            (GROUP_COLUMNS, THRIFT_LIST) => {
                let (size, _) = reader.list()?;
                for _ in 0..size {
                    found.chunks.push(chunk(reader)?);
                }
            }
            (GROUP_ROWS, THRIFT_I64) => found.rows = reader.signed()?,
            _ => reader.skip(kind)?,
        }
    }
    reader.close();
    Ok(found)
}
because!(group, ParquetMeta, "one row group from the metadata");

pub fn file_meta(bytes: &[u8]) -> Result<FileMeta, String> {
    let mut reader = Reader::new(bytes);
    let mut found = FileMeta::default();
    while let Some((number, kind)) = reader.field()? {
        match (number, kind) {
            (FILE_SCHEMA, THRIFT_LIST) => {
                let (size, _) = reader.list()?;
                for _ in 0..size {
                    found.schema.push(element(&mut reader)?);
                }
            }
            (FILE_ROWS, THRIFT_I64) => found.rows = reader.signed()?,
            (FILE_GROUPS, THRIFT_LIST) => {
                let (size, _) = reader.list()?;
                for _ in 0..size {
                    found.groups.push(group(&mut reader)?);
                }
            }
            _ => reader.skip(kind)?,
        }
    }
    Ok(found)
}
because!(file_meta, ParquetMeta, "the metadata of a file from the bytes of its footer");

pub fn page_header(bytes: &[u8]) -> Result<(PageHeader, usize), String> {
    let mut reader = Reader::new(bytes);
    let mut found = PageHeader { values_compressed: true, ..PageHeader::default() };
    while let Some((number, kind)) = reader.field()? {
        match (number, kind) {
            (PAGE_TYPE, THRIFT_I32) => found.kind = reader.signed()? as i32,
            (PAGE_UNCOMPRESSED, THRIFT_I32) => found.uncompressed = reader.signed()? as usize,
            (PAGE_COMPRESSED, THRIFT_I32) => found.compressed = reader.signed()? as usize,
            (PAGE_DATA | PAGE_DICTIONARY | PAGE_DATA_V2, THRIFT_STRUCT) => {
                reader.open()?;
                while let Some((inner, inner_kind)) = reader.field()? {
                    match (number, inner, inner_kind) {
                        (_, DATA_VALUES, THRIFT_I32) => found.values = reader.signed()? as usize,
                        (PAGE_DATA, DATA_ENCODING, THRIFT_I32) => found.encoding = reader.signed()? as i32,
                        (PAGE_DICTIONARY, DICTIONARY_ENCODING, THRIFT_I32) => found.encoding = reader.signed()? as i32,
                        (PAGE_DATA_V2, V2_ENCODING, THRIFT_I32) => found.encoding = reader.signed()? as i32,
                        (PAGE_DATA_V2, V2_DEFINITION_BYTES, THRIFT_I32) => found.definition_bytes = reader.signed()? as usize,
                        (PAGE_DATA_V2, V2_REPETITION_BYTES, THRIFT_I32) => found.repetition_bytes = reader.signed()? as usize,
                        (PAGE_DATA_V2, V2_COMPRESSED, _) => found.values_compressed = reader.truth(inner_kind)?,
                        _ => reader.skip(inner_kind)?,
                    }
                }
                reader.close();
            }
            _ => reader.skip(kind)?,
        }
    }
    Ok((found, reader.at))
}
because!(page_header, ParquetMeta, "the header of a page and the number of bytes that it takes, so that the page itself follows it; the values of a page of the second form are compressed unless its header says not");

pub fn is_text(element: &Element) -> bool {
    matches!(element.logical, Some(LOGICAL_STRING | LOGICAL_JSON | LOGICAL_ENUM)) || matches!(element.converted, Some(CONVERTED_UTF8 | CONVERTED_JSON | CONVERTED_ENUM))
}
because!(is_text, ParquetMeta, "whether the bytes of a column are a text: an annotation of a text, of JSON or of named values says so; other bytes stay bytes");

pub fn is_list(element: &Element) -> bool {
    element.logical == Some(LOGICAL_LIST) || element.converted == Some(CONVERTED_LIST)
}
because!(is_list, ParquetMeta, "whether a group is annotated as a list");

pub fn is_map(element: &Element) -> bool {
    element.logical == Some(LOGICAL_MAP) || matches!(element.converted, Some(CONVERTED_MAP | CONVERTED_MAP_KEY_VALUE))
}
because!(is_map, ParquetMeta, "whether a group is annotated as a map");

pub fn is_null_type(element: &Element) -> bool {
    element.logical == Some(LOGICAL_UNKNOWN)
}
because!(is_null_type, ParquetMeta, "whether every value of a column is missing by its logical type");
