use ilm_core::base::signs::is_space;
use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{LESSON_FILES, READ_AT_LEAST, READ_FILES, TOLD_ENDS, python_words, rows_where, stripped, text_value, told_sentences};
use patterns::{because, source};

pub struct GraphsAndBeliefs;
source!(GraphsAndBeliefs, "two sets of the hub in parquet files with a train file and files to read: NLGraph of Wang and others, graphs told as lines of edges with a question line, its kinds the rows of one task; and the ToM-QA tasks of Natural Instructions, stories with a question about where a thing is or where a person looks, in three parts, its kinds the stories of a part whose question holds one word");

const GRAPH_PREFIX: &str = "nlgraph_";
because!(GRAPH_PREFIX, GraphsAndBeliefs, "the start of the name of a kind of NLGraph, before its task");

const QUESTION_START: &str = "Q:";
because!(QUESTION_START, GraphsAndBeliefs, "the start of a line of a text of NLGraph that asks, whose last one is the question of the case");

const ANSWER_START: &str = "A:";
because!(ANSWER_START, GraphsAndBeliefs, "a line of NLGraph that only opens the answer and is no told line");

const GRAPH_LINE: char = '\n';
because!(GRAPH_LINE, GraphsAndBeliefs, "the mark between the lines of a text of NLGraph");

const BEFORE_EDGE: &str = "):";
because!(BEFORE_EDGE, GraphsAndBeliefs, "the signs after which a run of white space before an opening bracket parts two edges of a graph");

const EDGE_OPEN: char = '(';
because!(EDGE_OPEN, GraphsAndBeliefs, "the sign that opens an edge of a graph");

const BELIEF_PARTS: char = '_';
because!(BELIEF_PARTS, GraphsAndBeliefs, "the mark between the set, the part and the question in the name of a kind of ToM-QA");

const BELIEF_MARKS: &[(&str, &str)] = &[("real", "really"), ("begin", "beginning"), ("look", "look"), ("think", "think")];
because!(BELIEF_MARKS, GraphsAndBeliefs, "the word of each sort of question of ToM-QA that its question holds: where the thing really is, where it was at the beginning, where a person will look, where one person thinks that another searches");

const ASKED_MARK: &str = "?";
because!(ASKED_MARK, GraphsAndBeliefs, "the question mark that may stand right after the word of a question of ToM-QA");

const INPUT_START: &str = "Input: ";
because!(INPUT_START, GraphsAndBeliefs, "the words before the story of a row of ToM-QA, after the instruction of the task");

const OUTPUT_START: &str = "Output:";
because!(OUTPUT_START, GraphsAndBeliefs, "the word after the story of a row of ToM-QA");

fn graph_pieces(text: &str) -> Vec<String> {
    let signs: Vec<(usize, char)> = text.char_indices().collect();
    let run = |from: usize, keep: &dyn Fn(char) -> bool| signs[from..].iter().take_while(|(_, sign)| keep(*sign)).count();
    let byte_at = |place: usize| signs.get(place).map_or(text.len(), |sign| sign.0);
    let mut out: Vec<String> = Vec::new();
    let (mut start, mut at) = (0usize, 0usize);
    while at < signs.len() {
        let before = at.checked_sub(1).map(|place| signs[place].1);
        let spaces = run(at, &is_space);
        let found = if spaces > 0 && before.is_some_and(|sign| TOLD_ENDS.contains(sign)) {
            spaces
        } else if signs[at].1 == GRAPH_LINE {
            run(at, &|sign| sign == GRAPH_LINE)
        } else if spaces > 0 && before.is_some_and(|sign| BEFORE_EDGE.contains(sign)) && signs.get(at + spaces).is_some_and(|sign| sign.1 == EDGE_OPEN) {
            spaces
        } else {
            0
        };
        if found == 0 {
            at += 1;
            continue;
        }
        out.push(text[start..byte_at(at)].to_string());
        start = byte_at(at + found);
        at += found;
    }
    out.push(text[start..].to_string());
    out.into_iter().map(|piece| stripped(&piece).to_string()).filter(|piece| !piece.is_empty()).collect()
}
because!(graph_pieces, GraphsAndBeliefs, "the told pieces of a graph: the text cut at each run of white space after the end of a sentence, at each run of line breaks, and at each run of white space between the end of an edge and the opening of the next, each piece without white space at its ends");

fn graph_case(text: &str, answer: &str) -> Case {
    let lines: Vec<&str> = text.split(GRAPH_LINE).map(stripped).filter(|line| !line.is_empty()).collect();
    let asked: Vec<&str> = lines.iter().copied().filter(|line| line.starts_with(QUESTION_START)).collect();
    let told: Vec<&str> = lines.iter().copied().filter(|line| !line.starts_with(QUESTION_START) && *line != ANSWER_START).collect();
    let joined = told.join(&GRAPH_LINE.to_string());
    Case { sentences: graph_pieces(&joined), question: asked.last().copied().unwrap_or_default().to_string(), answer: stripped(answer).to_string() }
}
because!(graph_case, GraphsAndBeliefs, "a case of NLGraph: the told lines cut into pieces, the last question line as the question, the answer without white space at its ends");

pub fn nlgraph_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let task = Value::Text(kind.strip_prefix(GRAPH_PREFIX).ok_or_else(|| format!("the kind {kind} is no kind of this set"))?.to_string());
    let mut cases: Vec<Case> = Vec::new();
    let mut train = 0usize;
    for (number, place) in described.files(kind, LESSON_FILES).into_iter().chain(described.files(kind, READ_FILES)).enumerate() {
        for [question, answer, _] in rows_where(&place, ["question", "answer", "difficulty"], Some(("task", &task)))? {
            cases.push(graph_case(&text_value(&question, kind)?, &text_value(&answer, kind)?));
        }
        if number == 0 {
            train = cases.len();
        }
    }
    let read_from = (cases.len() - train >= READ_AT_LEAST).then_some(train);
    Ok(KindCases { cases, read_from, new_words: described.new_words(), ..KindCases::default() })
}
because!(nlgraph_cases, GraphsAndBeliefs, "the cases of a task of NLGraph in the order of its files; a kind with too few cases outside the train file keeps the cut by the order of its cases");

fn belief_case(text: &str, answers: &Value, mark: &str) -> Result<Option<Case>, String> {
    let story = text.rsplit_once(INPUT_START).map(|(_, after)| after).ok_or("a row of ToM-QA with no input")?;
    let story = story.rsplit_once(OUTPUT_START).map_or(story, |(before, _)| before);
    let mut pieces = told_sentences(story);
    let Some(question) = pieces.pop() else { return Ok(None) };
    let words = python_words(&question);
    let asked = format!("{mark}{ASKED_MARK}");
    if !words.iter().any(|word| *word == mark || *word == asked) {
        return Ok(None);
    }
    let Value::List(answers) = answers else { return Err("the outputs of a row of ToM-QA are no list".to_string()) };
    let answer = answers.first().and_then(Value::text).ok_or("a row of ToM-QA with no output")?.to_string();
    Ok(Some(Case { sentences: pieces, question, answer }))
}
because!(belief_case, GraphsAndBeliefs, "a case of ToM-QA from a row: the story between the input and the output cut into sentences, the last one the question when it holds the word of the sort of question, and the first output as the answer");

pub fn tomqa_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let mut parts = kind.split(BELIEF_PARTS).skip(1);
    let (Some(part), Some(sort)) = (parts.next(), parts.next()) else { return Err(format!("the kind {kind} is no kind of this set")) };
    let mark = BELIEF_MARKS.iter().find(|(held, _)| *held == sort).map(|(_, word)| *word).ok_or_else(|| format!("no sort of question {sort}"))?;
    let mut cases: Vec<Case> = Vec::new();
    let mut train = 0usize;
    for (number, place) in described.files(part, LESSON_FILES).into_iter().chain(described.files(part, READ_FILES)).enumerate() {
        for [input, output] in rows_where(&place, ["input", "output"], None)? {
            if let Some(case) = belief_case(&text_value(&input, kind)?, &output, mark)? {
                cases.push(case);
            }
        }
        if number == 0 {
            train = cases.len();
        }
    }
    Ok(KindCases { cases, read_from: Some(train), new_words: described.new_words(), ..KindCases::default() })
}
because!(tomqa_cases, GraphsAndBeliefs, "the cases of a kind of ToM-QA: the stories of its part whose question holds the word of its sort, the train file first");
