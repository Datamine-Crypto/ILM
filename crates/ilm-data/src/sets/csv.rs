use patterns::{because, source};

pub struct CsvText;
source!(CsvText, "a text of rows with a delimiter as the csv module of Python reads it in its first dialect: a field in quotes may hold delimiters and line ends, two quotes inside it stand for one, a sign after the closing quote stays in the field, a row ends at a line break, a return, or both, and an empty line is no row");

const CSV_QUOTE: char = '"';
because!(CSV_QUOTE, CsvText, "the sign round a field that holds a delimiter, a quote or a line end");

const CSV_RETURN: char = '\r';
because!(CSV_RETURN, CsvText, "the return, which ends a row alone or with a line break");

const CSV_BREAK: char = '\n';
because!(CSV_BREAK, CsvText, "the line break, which ends a row");

#[derive(Clone, Copy, PartialEq, Eq)]
enum CsvState {
    StartRecord,
    StartField,
    InField,
    InQuoted,
    QuoteInQuoted,
}
because!(CsvState, CsvText, "where the reader is: before a row, before a field, inside a plain field, inside a field in quotes, or right after a quote inside a field in quotes");

pub fn csv_rows(text: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut state = CsvState::StartRecord;
    let ends = |sign: char| sign == CSV_BREAK || sign == CSV_RETURN;
    for sign in text.chars() {
        if state == CsvState::StartRecord {
            if ends(sign) {
                continue;
            }
            state = CsvState::StartField;
        }
        match state {
            CsvState::StartField => {
                if ends(sign) {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    state = CsvState::StartRecord;
                } else if sign == CSV_QUOTE {
                    state = CsvState::InQuoted;
                } else if sign == delimiter {
                    row.push(std::mem::take(&mut field));
                } else {
                    field.push(sign);
                    state = CsvState::InField;
                }
            }
            CsvState::InField => {
                if ends(sign) {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    state = CsvState::StartRecord;
                } else if sign == delimiter {
                    row.push(std::mem::take(&mut field));
                    state = CsvState::StartField;
                } else {
                    field.push(sign);
                }
            }
            CsvState::InQuoted => {
                if sign == CSV_QUOTE {
                    state = CsvState::QuoteInQuoted;
                } else {
                    field.push(sign);
                }
            }
            CsvState::QuoteInQuoted => {
                if sign == CSV_QUOTE {
                    field.push(CSV_QUOTE);
                    state = CsvState::InQuoted;
                } else if sign == delimiter {
                    row.push(std::mem::take(&mut field));
                    state = CsvState::StartField;
                } else if ends(sign) {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    state = CsvState::StartRecord;
                } else {
                    field.push(sign);
                    state = CsvState::InField;
                }
            }
            CsvState::StartRecord => {}
        }
    }
    if state != CsvState::StartRecord && (!field.is_empty() || !row.is_empty() || state != CsvState::StartField) {
        row.push(field);
        rows.push(row);
    }
    rows
}
because!(csv_rows, CsvText, "the rows of a text, each a list of its fields; a row cut by the end of the text keeps what it has");

pub fn named_rows(text: &str, delimiter: char) -> Vec<Vec<(String, String)>> {
    let mut rows = csv_rows(text, delimiter).into_iter();
    let names = rows.next().unwrap_or_default();
    rows.map(|row| names.iter().cloned().zip(row).collect()).collect()
}
because!(named_rows, CsvText, "the rows after the first one, each field with the name that the first row gives its place, as a reader of rows into field_named fields gives them");

pub fn field_named<'a>(row: &'a [(String, String)], name: &str) -> Option<&'a str> {
    row.iter().find(|(held, _)| held == name).map(|(_, value)| value.as_str())
}
because!(field_named, CsvText, "the value of a row in the column that the first row of the file names so, or nothing for a row too short to have it");
