use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::parquet::open;
use crate::sets::readers::bigbench::{BenchPatterns, split_text};
use crate::sets::readers::common::{NEW_WORD_START, READ_FILES, rows_where, stripped, text_value};
use crate::sets::textmatch::TextPattern;
use patterns::{because, source};
use std::collections::BTreeSet;
use std::path::Path;

pub struct WordProblems;
source!(WordProblems, "word problems that people wrote, each set in the parquet files of its copy on the hub, read in the order of the files: the parts of MAWPS in the copy of allenai (a text that the rule of BIG-bench cuts into told sentences and a question, and its answer), ASDiv-A in the copy of MU-NLPC (the same with its result), ASDiv in the copy of EleutherAI and SVAMP in the copy of ChilleD (a body cut after the ends of sentences, a question, and an answer, from which ASDiv loses its unit in brackets); the new words of a kind are the words that stand with a capital letter inside a sentence and in no text of the kind in small letters, each with zq before it, in the order of the letters");

const PROBLEM_FILES: &str = "rows";
because!(PROBLEM_FILES, WordProblems, "the role in the description of the files of a kind, in the order in which they are read");

const MIX_KEY: &str = "mix";
because!(MIX_KEY, WordProblems, "the key of a line of the description that makes a kind from the lessons of other kinds, as the standard setting of SVAMP learns from MAWPS and ASDiv-A together and reads SVAMP");

const IN_BRACKETS: &str = r"\s*\([^()]*\)";
because!(IN_BRACKETS, WordProblems, "a unit in brackets after the number of an answer of ASDiv, with the white space before it, which the answer loses");

const WORD_OR_END: &str = r"[A-Za-z]+|[.?!]";
because!(WORD_OR_END, WordProblems, "a word of plain letters or an end mark of a sentence, the pieces by which a name is told from the first word of a sentence");

const PROBLEM_ENDS: &str = ".?!";
because!(PROBLEM_ENDS, WordProblems, "the end marks of a sentence, after which a word with a capital letter can be the first word of a sentence and no name");

const PROBLEM_LINE: char = '\n';
because!(PROBLEM_LINE, WordProblems, "the mark between the lines of a body");

fn body_parts(text: &str, cut: &TextPattern) -> Vec<String> {
    text.split(PROBLEM_LINE).flat_map(|line| cut.split(line)).map(|part| stripped(&part).to_string()).filter(|part| !part.is_empty()).collect()
}
because!(body_parts, WordProblems, "the told sentences of a body: each line cut after the ends of sentences, each part without white space at its ends, and no empty part");

fn problem_cases(place: &str, patterns: &BenchPatterns, units: &TextPattern) -> Result<Vec<Case>, String> {
    let names = open(Path::new(place))?.names();
    let has = |name: &str| names.iter().any(|held| held == name);
    let texts = |name: &str| -> Result<Vec<String>, String> { rows_where(place, [name], None)?.iter().map(|[value]| text_value(value, place)).collect() };
    let mut out: Vec<Case> = Vec::new();
    if has("input") || !has("body") && !has("Body") {
        let (text, answer) = if has("input") { ("input", "output_answer") } else { ("question", "result") };
        for (text, answer) in texts(text)?.into_iter().zip(texts(answer)?) {
            let (sentences, question) = split_text(&text, None, patterns);
            out.push(Case { sentences, question, answer: stripped(&answer).to_string() });
        }
        return Ok(out);
    }
    let (body, asked, answer) = if has("Body") { ("Body", "Question", "Answer") } else { ("body", "question", "answer") };
    for ((body, asked), answer) in texts(body)?.into_iter().zip(texts(asked)?).zip(texts(answer)?) {
        let answer = if has("Body") { answer } else { stripped(&units.replace_all(&answer, "")).to_string() };
        out.push(Case { sentences: body_parts(&body, &patterns.cut), question: stripped(&asked).to_string(), answer: stripped(&answer).to_string() });
    }
    Ok(out)
}
because!(problem_cases, WordProblems, "the cases of one file, by the columns that it has: a text with an answer cut by the rule of BIG-bench, or a body, a question and an answer");

fn problem_new_words(cases: &[Case], pieces: &TextPattern) -> Vec<(String, String)> {
    let mut inside: BTreeSet<String> = BTreeSet::new();
    let mut small: BTreeSet<String> = BTreeSet::new();
    let is_small = |word: &str| word.chars().any(|sign| sign.is_ascii_lowercase()) && !word.chars().any(|sign| sign.is_ascii_uppercase());
    for case in cases {
        for text in case.sentences.iter().chain([&case.question]) {
            let words: Vec<String> = pieces.find_all(text).into_iter().flatten().collect();
            for (place, word) in words.iter().enumerate() {
                let mut signs = word.chars();
                let first_capital = signs.next().is_some_and(|sign| sign.is_ascii_uppercase());
                if is_small(word) {
                    small.insert(word.clone());
                } else if place > 0 && !PROBLEM_ENDS.contains(words[place - 1].as_str()) && first_capital && is_small(signs.as_str()) {
                    inside.insert(word.to_lowercase());
                }
            }
        }
    }
    inside.into_iter().filter(|word| !small.contains(word)).map(|word| (word.clone(), format!("{NEW_WORD_START}{word}"))).collect()
}
because!(problem_new_words, WordProblems, "the table of new words of a kind: a word of more than one letter whose first letter alone is a capital, not first in its text and not after an end mark, becomes a new word when no text of the kind holds it in small letters");

fn files_cases(described: &Described, kind: &str, role: &str, patterns: &BenchPatterns, units: &TextPattern) -> Result<Vec<Case>, String> {
    let mut cases: Vec<Case> = Vec::new();
    for place in described.files(kind, role) {
        cases.extend(problem_cases(&place, patterns, units)?);
    }
    Ok(cases)
}
because!(files_cases, WordProblems, "the cases of the files of a kind in one role, in the order of the files");

pub fn problem_kind_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let patterns = BenchPatterns::new()?;
    let units = TextPattern::new(IN_BRACKETS)?;
    let pieces = TextPattern::new(WORD_OR_END)?;
    let Some(mixed) = described.values(MIX_KEY, kind) else {
        let cases = files_cases(described, kind, PROBLEM_FILES, &patterns, &units)?;
        let table = problem_new_words(&cases, &pieces);
        return Ok(KindCases { cases, new_words: Some(table), ..KindCases::default() });
    };
    let lists: Vec<Vec<Case>> = mixed.iter().map(|other| files_cases(described, other, PROBLEM_FILES, &patterns, &units)).collect::<Result<_, _>>()?;
    let longest = lists.iter().map(Vec::len).max().unwrap_or_default();
    let mut cases: Vec<Case> = Vec::new();
    for place in 0..longest {
        cases.extend(lists.iter().filter_map(|list| list.get(place)).cloned());
    }
    let read_from = Some(cases.len());
    cases.extend(files_cases(described, kind, READ_FILES, &patterns, &units)?);
    let table = problem_new_words(&cases, &pieces);
    Ok(KindCases { cases, read_from, new_words: Some(table), ..KindCases::default() })
}
because!(problem_kind_cases, WordProblems, "the cases of a kind in the order of its files, with its table of new words; a kind of a line mix learns from the cases of the kinds that the line names, one of each kind in turn, and reads the cases of its own files to read after them");
