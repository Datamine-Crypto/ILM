use ilm_core::know::signs::is_space;
use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases, NAMED_KEY, SETTING_KEY};
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{LESSON_FILES, NEW_WORD_START, rows_where, stripped, text_value};
use crate::sets::textmatch::TextPattern;
use patterns::{because, source};
use std::collections::BTreeSet;

pub struct BenchTasks;
source!(BenchTasks, "BIG-bench of Srivastava and others in the copy of tasksource, a train file and a validation file for each task where a block of a task starts at the row whose index is nought, and three tasks of BIG-Bench Hard of Suzgun and others in the copy of lukaemon, one test file each; a kind is one block of one task, read from both files (the train rows of the block, then its validation rows), from the validation file alone, or from the hard copy; each line of the text of a row loses its mark of a question or an answer and is cut after the ends of sentences, a line of choices staying whole; the question is the last part that ends with a question mark, or else the last part when it has no end mark, with the choices and the parts after it that have no end mark, the other parts are told; a kind of choices adds the choices of the row to the question in the order of their letters; the answer is the first target; the new words of a kind come from the list of its names in the order of the letters, each with zq and two letters by its place, a plural with the new word of its singular and an s");

const QA_MARK: &str = r"^\s*[QA]:\s*";
because!(QA_MARK, BenchTasks, "the mark of a question or an answer at the start of a line, with the white space round it, which a line loses");

const SENTENCE_CUT: &str = r#"(?:(?<=[.?!])|(?<=[.?!]["'”’)]))\s+"#;
because!(SENTENCE_CUT, BenchTasks, "the white space after the end of a sentence, also when a quote or a bracket closes the sentence, where a line is cut");

const LIST_MARK: &str = r"^\d+\.$";
because!(LIST_MARK, BenchTasks, "a number with a point that opens an item of a list, which stays with the part after it");

const CLOSERS: &str = "\"'”’)";
because!(CLOSERS, BenchTasks, "the quotes and the bracket that can stand after the end mark of a sentence");

const ASKED_END: char = '?';
because!(ASKED_END, BenchTasks, "the end mark of a part that asks");

const TOLD_END_MARKS: &str = ".!";
because!(TOLD_END_MARKS, BenchTasks, "the signs that close a part which says a thing and asks nothing");

const BENCH_LINE: char = '\n';
because!(BENCH_LINE, BenchTasks, "the mark between the lines of the text of a row");

const CHOICE_START: &str = "choice:";
because!(CHOICE_START, BenchTasks, "the start of a line of the text that gives a choice, which stays whole and goes to the question");

const CHOICE_PREFIX: &str = "choice: ";
because!(CHOICE_PREFIX, BenchTasks, "the words before each choice that a kind of choices adds to its question");

const CHOICES_END: &str = "+c";
because!(CHOICES_END, BenchTasks, "the end of the name of a kind that adds the choices of a row to its question");

const QUESTION_GAP: &str = " ";
because!(QUESTION_GAP, BenchTasks, "the mark between the parts of a question and between an item number and its part");

const HARD_COPY: &str = "bbh";
because!(HARD_COPY, BenchTasks, "the part of a kind read from the copy of BIG-Bench Hard, and the role of its file in the description");

const VALIDATION_PART: &str = "validation";
because!(VALIDATION_PART, BenchTasks, "the part of a kind read from the validation file alone, and the role of that file in the description");

const FIRST_LETTER: char = 'a';
because!(FIRST_LETTER, BenchTasks, "the letter of the place nought in the two letters of a new word");

const ALPHABET_SIZE: usize = 26;
because!(ALPHABET_SIZE, BenchTasks, "the letters of the plain alphabet, by which the place of a new word is cut into two letters");

const PLURAL_END: &str = "s";
because!(PLURAL_END, BenchTasks, "the end of a plural of a list of names, whose new word is that of its singular with this end");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PartEnd {
    Choice,
    Asked,
    Told,
    Open,
}
because!(PartEnd, BenchTasks, "what ends a part of a text: a line of choices, a question mark, the end mark of a told sentence, or no end mark");

struct BenchRow {
    inputs: String,
    targets: Vec<String>,
    choices: Vec<String>,
}
because!(BenchRow, BenchTasks, "one row of a task: its text, its targets, and the choices of a question of choices");

pub struct BenchPatterns {
    mark: TextPattern,
    pub cut: TextPattern,
    list: TextPattern,
}
because!(BenchPatterns, BenchTasks, "the patterns that cut the text of a row: the mark of a line, the cut after the end of a sentence, and the number of an item");

impl BenchPatterns {
    pub fn new() -> Result<BenchPatterns, String> {
        Ok(BenchPatterns { mark: TextPattern::new(QA_MARK)?, cut: TextPattern::new(SENTENCE_CUT)?, list: TextPattern::new(LIST_MARK)? })
    }
}

fn end_of(part: &str) -> PartEnd {
    let text = part.trim_end_matches(is_space).trim_end_matches(|sign: char| CLOSERS.contains(sign)).trim_end_matches(is_space);
    if text.ends_with(ASKED_END) {
        PartEnd::Asked
    } else if text.ends_with(|sign: char| TOLD_END_MARKS.contains(sign)) {
        PartEnd::Told
    } else {
        PartEnd::Open
    }
}
because!(end_of, BenchTasks, "the end mark of a part, looked for before the white space, the quotes and the bracket at its end");

pub fn split_text(text: &str, choices: Option<&[String]>, patterns: &BenchPatterns) -> (Vec<String>, String) {
    let mut parts: Vec<(PartEnd, String)> = Vec::new();
    for line in text.split(BENCH_LINE) {
        let line = patterns.mark.replace_all(line, "");
        let line = stripped(&line);
        if line.is_empty() {
            continue;
        }
        if line.starts_with(CHOICE_START) {
            parts.push((PartEnd::Choice, line.to_string()));
            continue;
        }
        let mut cut: Vec<String> = patterns.cut.split(line).iter().map(|piece| stripped(piece).to_string()).filter(|piece| !piece.is_empty()).collect();
        if cut.len() > 1 && patterns.list.find(&cut[0], false).is_some() {
            let second = cut.remove(1);
            cut[0] = [cut[0].as_str(), second.as_str()].join(QUESTION_GAP);
        }
        parts.extend(cut.into_iter().map(|piece| (end_of(&piece), piece)));
    }
    let plain: Vec<usize> = parts.iter().enumerate().filter(|(_, (end, _))| *end != PartEnd::Choice).map(|(place, _)| place).collect();
    let asked = plain.iter().copied().filter(|place| parts[*place].0 == PartEnd::Asked).last();
    let core = asked.or_else(|| plain.last().copied().filter(|place| parts[*place].0 == PartEnd::Open));
    let (mut question, mut sentences): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for (place, (end, piece)) in parts.into_iter().enumerate() {
        if Some(place) == core || end == PartEnd::Choice || (core.is_some_and(|core| place > core) && end == PartEnd::Open) {
            question.push(piece);
        } else {
            sentences.push(piece);
        }
    }
    if let Some(choices) = choices {
        let mut sorted = choices.to_vec();
        sorted.sort();
        question.extend(sorted.into_iter().map(|choice| format!("{CHOICE_PREFIX}{choice}")));
    }
    (sentences, question.join(QUESTION_GAP))
}
because!(split_text, BenchTasks, "the told sentences and the question of the text of a row, with the choices of the row in the question when they are given");

fn texts_of(value: &Value, place: &str) -> Result<Vec<String>, String> {
    let Value::List(items) = value else { return Err(format!("{place}: a list that is no list")) };
    items.iter().map(|item| text_value(item, place)).collect()
}
because!(texts_of, BenchTasks, "the texts of a column of lists of texts, such as the targets and the choices of a row");

fn bench_blocks(described: &Described, task: &str, part: &str) -> Result<Vec<Vec<BenchRow>>, String> {
    if part == HARD_COPY {
        let mut rows: Vec<BenchRow> = Vec::new();
        for place in described.files(task, HARD_COPY) {
            for [input, target] in rows_where(&place, ["input", "target"], None)? {
                rows.push(BenchRow { inputs: text_value(&input, &place)?, targets: vec![text_value(&target, &place)?], choices: Vec::new() });
            }
        }
        return Ok(vec![rows]);
    }
    let read = |role: &str| -> Result<Vec<Vec<BenchRow>>, String> {
        let mut blocks: Vec<Vec<BenchRow>> = Vec::new();
        for place in described.files(task, role) {
            for [inputs, targets, choices, number] in rows_where(&place, ["inputs", "targets", "multiple_choice_targets", "idx"], None)? {
                if number == Value::Int(0) || blocks.is_empty() {
                    blocks.push(Vec::new());
                }
                let row = BenchRow { inputs: text_value(&inputs, &place)?, targets: texts_of(&targets, &place)?, choices: texts_of(&choices, &place)? };
                if let Some(block) = blocks.last_mut() {
                    block.push(row);
                }
            }
        }
        Ok(blocks)
    };
    let valid = read(VALIDATION_PART)?;
    if part == VALIDATION_PART {
        return Ok(valid);
    }
    let train = read(LESSON_FILES)?;
    if train.len() != valid.len() {
        return Err(format!("{task}: {} blocks in the train part, {} in the validation part", train.len(), valid.len()));
    }
    Ok(train.into_iter().zip(valid).map(|(mut rows, more)| {
        rows.extend(more);
        rows
    }).collect())
}
because!(bench_blocks, BenchTasks, "the blocks of a task: of the hard copy one block of its rows; else the rows of the train file and of the validation file cut into blocks where the index of a row is nought, and for both files the train rows of each block before its validation rows");

fn bench_new_words<'a>(named: &'a str) -> Vec<(String, String)> {
    let words: BTreeSet<&'a str> = named.split(is_space).filter(|word| !word.is_empty()).collect();
    let plural = |word: &'a str| word.strip_suffix(PLURAL_END).filter(|single| words.contains(single));
    let mut table: Vec<(String, String)> = Vec::new();
    for word in words.iter().filter(|word| plural(word).is_none()) {
        let letter = |place: usize| char::from_u32(u32::from(FIRST_LETTER) + u32::try_from(place).unwrap_or_default()).unwrap_or(FIRST_LETTER);
        let new = format!("{NEW_WORD_START}{}{}", letter(table.len() / ALPHABET_SIZE), letter(table.len() % ALPHABET_SIZE));
        table.push((word.to_string(), new));
    }
    for word in words.iter() {
        if let Some(single) = plural(word) {
            let new = table.iter().find(|(held, _)| held == single).map(|(_, new)| format!("{new}{PLURAL_END}")).unwrap_or_default();
            table.push((word.to_string(), new));
        }
    }
    table
}
because!(bench_new_words, BenchTasks, "the table of new words of a kind from its list of names: each name that is no plural of another name of the list gets zq and two letters by its place, in the order of the letters; each plural then gets the new word of its singular with an s");

pub fn bench_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let setting = described.values(SETTING_KEY, kind).ok_or_else(|| format!("the description gives no task of the kind {kind}"))?;
    let [task, block, part] = setting else { return Err(format!("the setting of the kind {kind} is not a task, a block and a part")) };
    let block: usize = block.parse().map_err(|_| format!("the block {block} of the kind {kind} is no number"))?;
    let patterns = BenchPatterns::new()?;
    let with_choices = kind.ends_with(CHOICES_END);
    let mut cases: Vec<Case> = Vec::new();
    for row in bench_blocks(described, task, part)?.into_iter().nth(block).ok_or_else(|| format!("{task} has no block {block}"))? {
        let (sentences, question) = split_text(&row.inputs, with_choices.then_some(row.choices.as_slice()), &patterns);
        let answer = row.targets.into_iter().next().ok_or_else(|| format!("a row of {task} with no target"))?;
        cases.push(Case { sentences, question, answer });
    }
    let named = described.values(NAMED_KEY, kind).and_then(|values| values.first()).map_or_else(Vec::new, |named| bench_new_words(named));
    Ok(KindCases { cases, new_words: Some(named), ..KindCases::default() })
}
because!(bench_cases, BenchTasks, "the cases of a kind: the rows of its block of its task, each cut into told sentences and a question, the first target as the answer, with the table of new words of its list of names, which is empty for a kind with no list");
