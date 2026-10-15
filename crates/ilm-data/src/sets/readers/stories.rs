use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{LESSON_FILES, READ_AT_LEAST, READ_FILES, rows_where, stripped, text_value, told_sentences};
use patterns::{because, source};

pub struct ToldStories;
source!(ToldStories, "the sets whose case is a told story, a question and a label, each in a column of a parquet file of the hub, with a train file and the files to read after it: StepGame, FLD, BoardgameQA, MindGames, ProsQA and RuleTaker; the kinds of a set are rows where one column has one value (a configuration, a depth, a setup), or the cases of a number of steps; a story is cut into sentences at the end of each sentence");

const STORY_COLUMNS: usize = 3;
because!(STORY_COLUMNS, ToldStories, "the columns of a row that a case of a story set takes: the story, the question and the label");

const GAME_PREFIX: &str = "bgqa_d";
because!(GAME_PREFIX, ToldStories, "the start of the name of a kind of BoardgameQA, before the depth of its proofs");

const GAME_CONFIG: &str = "Main-depth";
because!(GAME_CONFIG, ToldStories, "the start of the configuration of the main part of BoardgameQA for a depth");

const PROS_PREFIX: &str = "prosqa_";
because!(PROS_PREFIX, ToldStories, "the start of the name of a kind of ProsQA, before its number of rule uses");

const PROS_END: &str = "hop";
because!(PROS_END, ToldStories, "the end of the name of a kind of ProsQA, after its number of rule uses");

fn tail<'a>(kind: &'a str, prefix: &str) -> Result<&'a str, String> {
    kind.strip_prefix(prefix).ok_or_else(|| format!("the kind {kind} is no kind of this set"))
}
because!(tail, ToldStories, "what the name of a kind holds after the start that its set gives every kind");

fn whole_number(text: &str) -> Result<i64, String> {
    text.parse::<i64>().map_err(|_| format!("{text} is no whole number"))
}
because!(whole_number, ToldStories, "the number that the name of a kind ends with, which a column of the set holds as a whole number");

fn of_files<const N: usize>(described: &Described, kind: &str, columns: [&str; N], wanted: Option<(&str, &Value)>) -> Result<(Vec<[Value; N]>, usize), String> {
    let mut rows: Vec<[Value; N]> = Vec::new();
    let mut train = 0usize;
    for (number, place) in described.files(kind, LESSON_FILES).into_iter().chain(described.files(kind, READ_FILES)).enumerate() {
        rows.extend(rows_where(&place, columns, wanted)?);
        if number == 0 {
            train = rows.len();
        }
    }
    Ok((rows, train))
}
because!(of_files, ToldStories, "the rows of a kind from the train file and then from each file to read, with the number of rows of the train file");

fn told_case(story: &Value, question: &Value, answer: &Value, place: &str) -> Result<Case, String> {
    Ok(Case { sentences: told_sentences(&text_value(story, place)?), question: text_value(question, place)?, answer: text_value(answer, place)? })
}
because!(told_case, ToldStories, "a case from a story, a question and an answer");

fn last_is_question(text: &Value, answer: &Value, place: &str) -> Result<Case, String> {
    let mut pieces = told_sentences(stripped(&text_value(text, place)?));
    let question = pieces.pop().ok_or_else(|| format!("{place}: an empty text"))?;
    Ok(Case { sentences: pieces, question, answer: text_value(answer, place)? })
}
because!(last_is_question, ToldStories, "a case from a text whose last sentence is the question");

struct ByValue {
    prefix: &'static str,
    start: &'static str,
    column: &'static str,
    whole: bool,
    columns: [&'static str; STORY_COLUMNS],
}
because!(ByValue, ToldStories, "a set whose kinds are the rows of one value of a column: the start of the names of its kinds, the start of the value before what the name of a kind ends with, the column, whether the value is a whole number, and the columns of the story, the question and the label");

const BY_VALUE: &[ByValue] = &[
    ByValue { prefix: "step_k", start: "qa", column: "config", whole: false, columns: ["story", "question", "label"] },
    ByValue { prefix: "fld_d", start: "", column: "original_tree_depth", whole: true, columns: ["context", "hypothesis", "proof_label"] },
    ByValue { prefix: "mind_", start: "", column: "setup", whole: false, columns: ["premise", "hypothesis", "label"] },
    ByValue { prefix: "rt_d", start: "depth-", column: "config", whole: false, columns: ["context", "question", "label"] },
];
because!(BY_VALUE, ToldStories, "the sets whose kinds are the rows of one value of a column: StepGame by the configuration of a number of steps, FLD by the depth of the proof tree, MindGames by the setup, RuleTaker by the configuration of a depth of theories");

pub fn by_value_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let set = BY_VALUE.iter().find(|set| kind.starts_with(set.prefix)).ok_or_else(|| format!("the kind {kind} is no kind of a set read by the value of a column"))?;
    let end = tail(kind, set.prefix)?;
    let wanted = if set.whole { Value::Int(whole_number(end)?) } else { Value::Text(format!("{}{end}", set.start)) };
    let (rows, train) = of_files(described, kind, set.columns, Some((set.column, &wanted)))?;
    let cases = rows.iter().map(|[story, question, label]| told_case(story, question, label, kind)).collect::<Result<Vec<Case>, String>>()?;
    Ok(KindCases { cases, read_from: Some(train), new_words: described.new_words(), ..KindCases::default() })
}
because!(by_value_cases, ToldStories, "the cases of a kind of a set read by the value of a column: the rows of that value in the train file and then in the files to read, the story cut into sentences");

pub fn boardgame_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let config = Value::Text(format!("{GAME_CONFIG}{}", tail(kind, GAME_PREFIX)?));
    let (rows, train) = of_files(described, kind, ["example", "label"], Some(("config", &config)))?;
    let cases = rows.iter().map(|[text, label]| last_is_question(text, label, kind)).collect::<Result<Vec<Case>, String>>()?;
    Ok(KindCases { cases, read_from: Some(train), new_words: described.new_words(), ..KindCases::default() })
}
because!(boardgame_cases, ToldStories, "the cases of a kind of BoardgameQA: the rows of its depth in the main part, the last sentence of the example as the question");

pub fn prosqa_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let hops = whole_number(tail(kind, PROS_PREFIX)?.strip_suffix(PROS_END).unwrap_or_default())? as usize;
    let (rows, train) = of_files(described, kind, ["question", "answer", "steps"], None)?;
    let mut cases: Vec<Case> = Vec::new();
    let mut in_train = 0usize;
    for (number, [text, answer, steps]) in rows.iter().enumerate() {
        let Value::List(steps) = steps else { return Err(format!("{kind}: the steps of a row are no list")) };
        if steps.len() == hops + 1 {
            cases.push(last_is_question(text, answer, kind)?);
            if number < train {
                in_train += 1;
            }
        }
    }
    let read_from = (cases.len() - in_train >= READ_AT_LEAST).then_some(in_train);
    Ok(KindCases { cases, read_from, new_words: described.new_words(), ..KindCases::default() })
}
because!(prosqa_cases, ToldStories, "the cases of a kind of ProsQA: the rows whose steps are one more than its rule uses, the last sentence of the text as the question; a kind with too few cases outside the train file keeps the cut by the order of its cases");

