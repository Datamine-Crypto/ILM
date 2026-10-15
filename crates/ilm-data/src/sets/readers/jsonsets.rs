use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::json::parse_json;
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{LESSON_FILES, READ_FILES, lines_of, rows_where, stripped, text_of, text_value, told_sentences};
use patterns::{because, source};

pub struct JsonSets;
source!(JsonSets, "the sets whose raw files are JSON: the twenty tasks of bAbI of Weston and others (lines of JSON with a task number, a passage, a question and an answer), SpartQA (lines with a story, a question and an answer), and the one-step part of PrOntoQA (lines with a prompt that holds the context and the answer), with its five-step part in parquet");

const BABI_PREFIX: &str = "babi";
because!(BABI_PREFIX, JsonSets, "the start of the name of a kind of bAbI, before the number of its task");

const PASSAGE_LINE: char = '\n';
because!(PASSAGE_LINE, JsonSets, "the mark between the told sentences of a passage of bAbI");

const PROMPT_FILES: &str = "prompts";
because!(PROMPT_FILES, JsonSets, "the role in the description of a file of lines of JSON with prompts, which the one-step part of PrOntoQA has and the five-step part has not");

const CONTEXT_START: &str = "###Context:";
because!(CONTEXT_START, JsonSets, "the words before the context of a prompt of PrOntoQA");

const RESPONSE_START: &str = "###Response:";
because!(RESPONSE_START, JsonSets, "the words after the context of a prompt of PrOntoQA");

const PRONTO_ANSWER_START: &str = "###The answer is:";
because!(PRONTO_ANSWER_START, JsonSets, "the words before the answer at the end of a prompt of PrOntoQA");

const ASK: &str = "Is the following statement true or false?";
because!(ASK, JsonSets, "the question of every case of PrOntoQA, before its statement");

const ASK_GAP: &str = " ";
because!(ASK_GAP, JsonSets, "the mark between the question of PrOntoQA and its statement");

fn json_lines(place: &str) -> Result<Vec<Value>, String> {
    lines_of(&text_of(place)?).iter().map(|line| parse_json(line).map_err(|why| format!("{place}: {why}"))).collect()
}
because!(json_lines, JsonSets, "the values of a file with one JSON value on each line");

fn field<'a>(row: &'a Value, name: &str) -> Result<&'a Value, String> {
    row.get(name).ok_or_else(|| format!("a row with no field {name}"))
}
because!(field, JsonSets, "a named field of a row of JSON");

fn field_text(row: &Value, name: &str) -> Result<String, String> {
    text_value(field(row, name)?, name)
}
because!(field_text, JsonSets, "the text of a named field of a row of JSON");

pub fn babi_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let task = kind.strip_prefix(BABI_PREFIX).and_then(|number| number.parse::<i64>().ok()).ok_or_else(|| format!("the kind {kind} is no task of bAbI"))?;
    let mut cases: Vec<Case> = Vec::new();
    let mut read_from: Option<usize> = None;
    let read = described.files(kind, READ_FILES);
    for (part, files) in [described.files(kind, LESSON_FILES), read.clone()].into_iter().enumerate() {
        if part == 1 && !read.is_empty() {
            read_from = Some(cases.len());
        }
        for place in files {
            for row in json_lines(&place)? {
                if field(&row, "task")? != &Value::Int(task) {
                    continue;
                }
                let passage = field_text(&row, "passage")?;
                let sentences = passage.split(PASSAGE_LINE).filter(|sentence| !stripped(sentence).is_empty()).map(str::to_string).collect();
                cases.push(Case { sentences, question: field_text(&row, "question")?, answer: field_text(&row, "answer")? });
            }
        }
    }
    Ok(KindCases { cases, read_from, new_words: described.new_words(), ..KindCases::default() })
}
because!(babi_cases, JsonSets, "the cases of a task of bAbI from the train and the validation lines of the hub copy, then the test lines when the description names them, read from there: the lines of the passage that are not empty as told sentences, as they stand");

pub fn spartqa_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let mut cases: Vec<Case> = Vec::new();
    let mut train = 0usize;
    for (number, place) in described.files(kind, LESSON_FILES).into_iter().chain(described.files(kind, READ_FILES)).enumerate() {
        for row in json_lines(&place)? {
            cases.push(Case { sentences: told_sentences(&field_text(&row, "story")?), question: field_text(&row, "question")?, answer: field_text(&row, "answer")? });
        }
        if number == 0 {
            train = cases.len();
        }
    }
    Ok(KindCases { cases, read_from: Some(train), new_words: described.new_words(), ..KindCases::default() })
}
because!(spartqa_cases, JsonSets, "the cases of SpartQA: the story cut into sentences, the train lines first");

fn pronto_case(text: &str, answer: &str) -> Result<Case, String> {
    let mut pieces = told_sentences(stripped(text));
    let question = pieces.pop().ok_or("an empty question of PrOntoQA")?;
    Ok(Case { sentences: pieces, question, answer: stripped(answer).to_string() })
}
because!(pronto_case, JsonSets, "a case of the five-step part of PrOntoQA: the last sentence of the question as the question, the sentences before it as told");

pub fn prontoqa_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let mut cases: Vec<Case> = Vec::new();
    let prompts = described.files(kind, PROMPT_FILES);
    if let Some(place) = prompts.first() {
        for row in json_lines(place)? {
            let prompt = field_text(&row, "prompt")?;
            let context = prompt.split_once(CONTEXT_START).map(|(_, after)| after).ok_or("a prompt with no context")?;
            let context = context.split_once(RESPONSE_START).map_or(context, |(before, _)| before);
            let (told, statement) = context.split_once(ASK).ok_or("a prompt with no question")?;
            if statement.contains(ASK) {
                return Err("a prompt with the question twice".to_string());
            }
            let answer = prompt.rsplit_once(PRONTO_ANSWER_START).map(|(_, after)| after).ok_or("a prompt with no answer")?;
            cases.push(Case { sentences: told_sentences(stripped(told)), question: format!("{ASK}{ASK_GAP}{}", stripped(statement)), answer: stripped(answer).to_string() });
        }
        return Ok(KindCases { cases, read_from: None, new_words: described.new_words(), ..KindCases::default() });
    }
    let mut train = 0usize;
    for (number, place) in described.files(kind, LESSON_FILES).into_iter().chain(described.files(kind, READ_FILES)).enumerate() {
        for [question, answer] in rows_where(&place, ["question", "answer"], None)? {
            cases.push(pronto_case(&text_value(&question, kind)?, &text_value(&answer, kind)?)?);
        }
        if number == 0 {
            train = cases.len();
        }
    }
    Ok(KindCases { cases, read_from: Some(train), new_words: described.new_words(), ..KindCases::default() })
}
because!(prontoqa_cases, JsonSets, "the cases of a kind of PrOntoQA: the one-step part from the context, the question and the answer of each prompt, with no place where the cases to read start; the five-step part from its train file and then its other files");
