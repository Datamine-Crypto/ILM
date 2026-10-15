use ilm_core::know::signs::is_space;
use ilm_core::cases::Case;
use crate::sets::parquet::open;
use crate::sets::parquet::records::Value;
use patterns::{because, source};
use std::path::Path;

pub struct PythonText;
source!(PythonText, "the reading of a plain text file as Python reads it in text mode: the bytes are the Unicode form of files, and a line ends at a line break, at a return with a line break, or at a return alone; the strip of a line takes away the white space of Python at both ends, which is the white space of Unicode and the four separators of files, groups, records and units");

pub struct EvenLessons;
source!(EvenLessons, "lessons spread over the whole train part of a set: every k-th train case from the first is taken first, k being the train cases divided by the lessons, then the other train cases in their order, then the cases to read; a train file in an order of length or of kind then gives lessons of every sort, since the measure takes the first cases of a kind as its lessons");

pub const TOLD_ENDS: &str = ".!?";
because!(TOLD_ENDS, PythonText, "the signs that end a sentence, after which a run of white space parts two told sentences");

const TEXT_RETURN: char = '\r';
because!(TEXT_RETURN, PythonText, "the return, which ends a line alone or together with a line break");

const TEXT_BREAK: char = '\n';
because!(TEXT_BREAK, PythonText, "the line break, which ends a line");

pub const LESSON_FILES: &str = "train";
because!(LESSON_FILES, EvenLessons, "the role in a description of the files whose cases come first, the lessons");

pub const READ_FILES: &str = "read";
because!(READ_FILES, EvenLessons, "the role in a description of the files whose cases come after the lessons, in their order, and are read");

pub const READ_AT_LEAST: usize = 100;
because!(READ_AT_LEAST, EvenLessons, "the cases outside the lessons that the measure needs to read a kind; a kind with fewer keeps the cut by the order of its cases");

pub const NEW_WORD_START: &str = "zq";
because!(NEW_WORD_START, EvenLessons, "the start of a new word, which no text of a set holds, before a word of the set");

pub const EVEN_LESSONS: usize = 1000;
because!(EVEN_LESSONS, EvenLessons, "the lessons that the measure takes from a kind of these sets, as the board fits it");

pub fn text_of(place: &str) -> Result<String, String> {
    let bytes = std::fs::read(place).map_err(|error| format!("{place}: {error}"))?;
    String::from_utf8(bytes).map_err(|_| format!("{place}: no UTF-8 text"))
}
because!(text_of, PythonText, "the text of a file in the Unicode form of files; a file that is not is refused, as Python refuses it");

pub fn lines_of(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut signs = text.chars().peekable();
    while let Some(sign) = signs.next() {
        line.push(sign);
        if sign == TEXT_BREAK || sign == TEXT_RETURN {
            if sign == TEXT_RETURN {
                line.pop();
                line.push(TEXT_BREAK);
                if signs.peek() == Some(&TEXT_BREAK) {
                    signs.next();
                }
            }
            out.push(std::mem::take(&mut line));
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}
because!(lines_of, PythonText, "the lines of a text as a loop over a file in text mode gives them: each with its end as one line break, the last one without an end when the text has none");

pub fn stripped(text: &str) -> &str {
    text.trim_matches(is_space)
}
because!(stripped, PythonText, "a text with the white space of Python taken away at both ends");

pub fn even_lessons(train: Vec<Case>, rest: Vec<Case>) -> Vec<Case> {
    let step = (train.len() / EVEN_LESSONS).max(1);
    let first: Vec<usize> = (0..train.len()).step_by(step).take(EVEN_LESSONS).collect();
    let mut taken = vec![false; train.len()];
    first.iter().for_each(|place| taken[*place] = true);
    let mut cases: Vec<Case> = first.iter().map(|place| train[*place].clone()).collect();
    cases.extend(train.into_iter().enumerate().filter(|(place, _)| !taken[*place]).map(|(_, case)| case));
    cases.extend(rest);
    cases
}
because!(even_lessons, EvenLessons, "the cases of a kind with its lessons spread evenly over the train part, then the cases to read");

pub fn table_columns<const N: usize>(place: &str, names: [&str; N]) -> Result<[Vec<Value>; N], String> {
    let mut table = open(Path::new(place))?;
    <[Vec<Value>; N]>::try_from(table.read(&names)?).map_err(|_| format!("{place}: other columns than asked"))
}
because!(table_columns, PythonText, "some columns of a parquet file as values, one list for each column in the order asked");

pub fn told_sentences(text: &str) -> Vec<String> {
    let signs: Vec<(usize, char)> = text.char_indices().collect();
    let mut out: Vec<String> = Vec::new();
    let mut start = 0usize;
    let mut at = 1usize;
    while at < signs.len() {
        if TOLD_ENDS.contains(signs[at - 1].1) && is_space(signs[at].1) {
            let mut end = at;
            while end < signs.len() && is_space(signs[end].1) {
                end += 1;
            }
            out.push(text[start..signs[at].0].to_string());
            start = signs.get(end).map_or(text.len(), |sign| sign.0);
            at = end + 1;
            continue;
        }
        at += 1;
    }
    out.push(text[start..].to_string());
    out.into_iter().map(|piece| stripped(&piece).to_string()).filter(|piece| !piece.is_empty()).collect()
}
because!(told_sentences, PythonText, "the sentences of a told text as the readers of the sets cut it: at each run of white space after the end of a sentence, each piece without white space at its ends, and no empty piece");

pub fn in_one_case(word: &str, capitals: bool) -> bool {
    let (wanted, other): (fn(char) -> bool, fn(char) -> bool) = if capitals { (char::is_uppercase, char::is_lowercase) } else { (char::is_lowercase, char::is_uppercase) };
    word.chars().any(wanted) && !word.chars().any(other)
}
because!(in_one_case, PythonText, "whether a word is written in one case: it holds a letter of that case and no letter of the other");

pub fn python_words(text: &str) -> Vec<&str> {
    text.split(is_space).filter(|piece| !piece.is_empty()).collect()
}
because!(python_words, PythonText, "the pieces of a text between runs of white space, as the split of Python with no argument gives them");

pub fn rows_where<const N: usize>(place: &str, names: [&str; N], wanted: Option<(&str, &Value)>) -> Result<Vec<[Value; N]>, String> {
    let mut table = open(Path::new(place))?;
    let mut asked: Vec<&str> = names.to_vec();
    if let Some((name, _)) = wanted {
        asked.push(name);
    }
    let mut columns = table.read(&asked)?;
    let keep: Vec<bool> = match wanted {
        Some((_, value)) => columns.pop().unwrap_or_default().iter().map(|held| held == value).collect(),
        None => vec![true; columns.first().map_or(0, Vec::len)],
    };
    let mut iterators: Vec<std::vec::IntoIter<Value>> = columns.into_iter().map(Vec::into_iter).collect();
    let mut out: Vec<[Value; N]> = Vec::new();
    for kept in keep {
        let row: Vec<Value> = iterators.iter_mut().map(|column| column.next().unwrap_or(Value::Null)).collect();
        if kept {
            out.push(<[Value; N]>::try_from(row).map_err(|_| format!("{place}: a row with other columns than asked"))?);
        }
    }
    Ok(out)
}
because!(rows_where, PythonText, "the rows of some columns of a parquet file, in the order of the file, where one column has a value when a value is asked, as a filtered read of pyarrow gives them");

pub fn text_value(value: &Value, place: &str) -> Result<String, String> {
    value.text().map(str::to_string).ok_or_else(|| format!("{place}: a value that is no text"))
}
because!(text_value, PythonText, "the text of a value of a column that holds texts");
