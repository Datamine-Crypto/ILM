use crate::sets::csv::{field_named, named_rows};
use crate::sets::export::Described;
use crate::sets::readers::common::{stripped, table_columns, text_of, text_value};
use crate::sets::textmatch::TextPattern;
use patterns::{because, source};
use std::path::Path;

pub struct WritingSets;
source!(WritingSets, "the sets of the writer as rows of facts and a text: each row of a part of a set, in the order of its file, is the list of facts that a person wrote from (each a slot and a value, in the order of the set) and the text as the person wrote it. One set writes its facts as slots with their values in brackets, one after the other (E2E of Novikova, Dusek and Rieser); another writes them inside the brackets of the sort of sentence that is wanted, which is then the first fact (ViGGO of Juraska, Bowden and Walker). The description of a set names its form, its file of each part and its two columns, and the program holds no name of a set");

const FACT_PATTERN: &str = r"\s*([^\[\],]+?)\s*\[([^\]]*)\]";
because!(FACT_PATTERN, WritingSets, "one fact as the sets write it: a slot of signs that are no brackets and no commas, then its value in square brackets, with white space round the slot");

const WHOLE_PATTERN: &str = r"(?s)\s*(\w+)\s*\((.*)\)\s*$";
because!(WHOLE_PATTERN, WritingSets, "a list of facts inside a sort of sentence: the sort as one word, then the facts in round brackets up to the end of the text, line breaks included");

const INSIDE_GROUP: usize = 2;
because!(INSIDE_GROUP, WritingSets, "the group of a list inside a sort of sentence that holds the facts, after the group of the sort");

const ACT_SLOT: &str = "act";
because!(ACT_SLOT, WritingSets, "the slot of the sort of sentence of a list inside a sort, the first fact of its row");

const ROWS_KEY: &str = "rows";
because!(ROWS_KEY, WritingSets, "the key of the line of a description that makes a set a set of the writer and names the form of its raw files");

const FROM_PARQUET: &str = "parquet";
because!(FROM_PARQUET, WritingSets, "the form of raw files that are tables of columns in parquet");

const FROM_FIELDS: &str = "fields";
because!(FROM_FIELDS, WritingSets, "the form of raw files that are rows of fields with a comma between them and a first row of names");

const FACTS_KEY: &str = "facts";
because!(FACTS_KEY, WritingSets, "the key of the line of a description that names how a set writes its facts");

const FACTS_INSIDE: &str = "inside";
because!(FACTS_INSIDE, WritingSets, "the facts of a row stand inside the brackets of the sort of sentence");

const COLUMNS_KEY: &str = "columns";
because!(COLUMNS_KEY, WritingSets, "the key of the line of a description with the column of the facts and the column of the text");

const CUT_KEY: &str = "cut";
because!(CUT_KEY, WritingSets, "the key of the line of a description with the codes of the signs by which the set writes a list inside one value");

const SET_KEY: &str = "set";
because!(SET_KEY, WritingSets, "the key of the line of the description with the short name of the set, which the first line of a file of rows names");

const ROW_ROLE: &str = "rows";
because!(ROW_ROLE, WritingSets, "the role in the description of a set of the raw file of a part, whose rows are the rows of the part");

const ROW_PARTS: &[&str] = &["train", "validation"];
because!(ROW_PARTS, WritingSets, "the parts of a set of the writer that are written: the train part, where the lessons and the development part come from, and the validation part, which is read");

const FINAL_ROWS: &str = "test";
because!(FINAL_ROWS, WritingSets, "the final part of a set of the writer, written only when the description names its file: the descriptions of the board do not, and the description of the one final read does");

const ROW_COMMA: char = ',';
because!(ROW_COMMA, WritingSets, "the sign between the fields of a row of a raw file of fields");

const BYTE_ORDER: char = '\u{feff}';
because!(BYTE_ORDER, WritingSets, "the sign of the order of bytes that a raw file of fields may start with; the reader takes it away, as a file opened with that sign in mind does");

const ROW_FIELD: &str = "\t";
because!(ROW_FIELD, WritingSets, "the mark between the fields of a line of a file of rows");

const ROW_LINE: &str = "\n";
because!(ROW_LINE, WritingSets, "the end of every line of a file of rows");

const HEAD_TEXT: &str = "#";
because!(HEAD_TEXT, WritingSets, "the mark of the first line of a file of rows");

const FACT_TEXT: &str = "F";
because!(FACT_TEXT, WritingSets, "the mark of a line with one fact: its slot and its value");

const WRITTEN_TEXT: &str = "T";
because!(WRITTEN_TEXT, WritingSets, "the mark of the line with the text of a row, which closes the row before an empty line");

const CODE_GAP: &str = ".";
because!(CODE_GAP, WritingSets, "the mark between the codes of the signs of the cut of a list, in a description and in the first line of a file of rows");

const ROWS_END: &str = ".txt";
because!(ROWS_END, WritingSets, "the end of the name of the file of rows of a part");

const ROW_ESCAPES: &[(char, &str)] = &[('\\', "\\\\"), ('\t', "\\t"), ('\n', "\\n"), ('\r', "\\r")];
because!(ROW_ESCAPES, WritingSets, "the signs that a slot, a value or a text may hold and a line may not, each written as a backslash and a letter, the backslash itself first, so that every line is one line and a reader can undo it");

pub type WritingRow = (Vec<(String, String)>, String);
because!(WritingRow, WritingSets, "one row of a set of the writer: its facts as slots and values, and its text");

fn fact_list(pattern: &TextPattern, text: &str) -> Vec<(String, String)> {
    pattern
        .find_all(text)
        .into_iter()
        .filter_map(|found| match found.as_slice() {
            [slot, value] => Some((slot.clone(), stripped(value).to_string())),
            _ => None,
        })
        .collect()
}
because!(fact_list, WritingSets, "the facts of a text in their order, each value without white space at its ends");

fn raw_rows(described: &Described, part: &str) -> Result<Vec<(String, String)>, String> {
    let place = described.file(part, ROW_ROLE)?;
    let line = described.lines.iter().find(|line| line.first().is_some_and(|key| key == COLUMNS_KEY)).ok_or_else(|| format!("the description names no {COLUMNS_KEY}"))?;
    let [_, facts_name, text_name] = line.as_slice() else { return Err(format!("the line {COLUMNS_KEY} of the description names no two columns")) };
    match described.value(ROWS_KEY) {
        Some(FROM_PARQUET) => {
            let [facts, texts] = table_columns(&place, [facts_name.as_str(), text_name.as_str()])?;
            facts.iter().zip(&texts).map(|(facts, text)| Ok((text_value(facts, &place)?, text_value(text, &place)?))).collect()
        }
        Some(FROM_FIELDS) => {
            let text = text_of(&place)?;
            let text = text.strip_prefix(BYTE_ORDER).unwrap_or(&text);
            named_rows(text, ROW_COMMA)
                .iter()
                .map(|row| {
                    let field = |name: &str| field_named(row, name).map(str::to_string).ok_or_else(|| format!("{place}: a row with no field {name}"));
                    Ok((field(facts_name)?, field(text_name)?))
                })
                .collect()
        }
        other => Err(format!("no raw form {other:?} of a set of the writer")),
    }
}
because!(raw_rows, WritingSets, "the text of the facts and the text of each row of the raw file of a part, in the order of the file, from the two columns that the description names");

fn rows_of(described: &Described, part: &str) -> Result<Vec<WritingRow>, String> {
    let pattern = TextPattern::new(FACT_PATTERN)?;
    let inside = described.value(FACTS_KEY) == Some(FACTS_INSIDE);
    let whole = TextPattern::new(WHOLE_PATTERN)?;
    let mut out: Vec<WritingRow> = Vec::new();
    for (facts_text, written) in raw_rows(described, part)? {
        let facts = if inside {
            let Some(groups) = whole.find(&facts_text, true) else { continue };
            let (Some(Some(act)), Some(Some(within))) = (groups.get(1), groups.get(INSIDE_GROUP)) else { continue };
            let mut facts = vec![(ACT_SLOT.to_string(), act.clone())];
            facts.extend(fact_list(&pattern, within));
            facts
        } else {
            fact_list(&pattern, &facts_text)
        };
        out.push((facts, stripped(&written).to_string()));
    }
    Ok(out)
}
because!(rows_of, WritingSets, "the rows of a part: the facts of each row in the form that the description names and its text without white space at its ends; a row whose facts inside a sort do not have that form is left out");

fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for sign in text.chars() {
        match ROW_ESCAPES.iter().find(|(held, _)| *held == sign) {
            Some((_, written)) => out.push_str(written),
            None => out.push(sign),
        }
    }
    out
}
because!(escaped, WritingSets, "a slot, a value or a text with each sign that a line may not hold written as its escape");

pub fn rows_file(set: &str, rows: &[WritingRow], cut: Option<&str>) -> String {
    let mut head: Vec<String> = vec![HEAD_TEXT.to_string(), format!("{SET_KEY}={set}"), format!("{ROWS_KEY}={}", rows.len())];
    if let Some(cut) = cut {
        head.push(format!("{CUT_KEY}={cut}"));
    }
    let mut out = head.join(ROW_FIELD) + ROW_LINE;
    for (facts, text) in rows {
        for (slot, value) in facts {
            out.push_str(&[FACT_TEXT, &escaped(slot), &escaped(value)].join(ROW_FIELD));
            out.push_str(ROW_LINE);
        }
        out.push_str(&[WRITTEN_TEXT, &escaped(text)].join(ROW_FIELD));
        out.push_str(ROW_LINE);
        out.push_str(ROW_LINE);
    }
    out
}
because!(rows_file, WritingSets, "the text of a file of rows: a first line with the name of the set, the number of rows and the codes of the cut of a list when the set has one, then for each row a line for each fact, a line for its text and an empty line");

pub fn write_rows(described: &Described, folder: &Path) -> Option<Result<Vec<String>, String>> {
    described.value(ROWS_KEY)?;
    Some((|| {
        let set = described.value(SET_KEY).ok_or_else(|| format!("the description names no {SET_KEY}"))?;
        let cut = described.value(CUT_KEY);
        if let Some(codes) = cut {
            codes.split(CODE_GAP).map(|code| code.parse::<u32>().ok().and_then(char::from_u32)).collect::<Option<String>>().ok_or_else(|| format!("the {CUT_KEY} of the description is no list of codes of signs"))?;
        }
        std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
        let mut said: Vec<String> = Vec::new();
        for part in ROW_PARTS {
            let rows = rows_of(described, part)?;
            let place = folder.join(format!("{part}{ROWS_END}"));
            std::fs::write(&place, rows_file(set, &rows, cut)).map_err(|error| format!("{}: {error}", place.display()))?;
            said.push(format!("{set} {part}: {} rows", rows.len()));
        }
        if !described.files(FINAL_ROWS, ROW_ROLE).is_empty() {
            let rows = rows_of(described, FINAL_ROWS)?;
            let place = folder.join(format!("{FINAL_ROWS}{ROWS_END}"));
            std::fs::write(&place, rows_file(set, &rows, cut)).map_err(|error| format!("{}: {error}", place.display()))?;
            said.push(format!("{set} {FINAL_ROWS}: {} rows", rows.len()));
        }
        Ok(said)
    })())
}
because!(write_rows, WritingSets, "the files of rows of a set of the writer, one for each part, written from the raw files that its description names, and the final part only when the description names it; nothing for a set whose description names no form of rows");
