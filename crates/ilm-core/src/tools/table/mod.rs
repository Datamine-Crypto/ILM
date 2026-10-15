pub mod program;
pub mod rows;

use crate::base::lanes::on_lanes;
use crate::base::words::answer_words;
use crate::tools::call::{Learned, Learner, Lessons, Said, Tool};
use crate::tools::table::program::{TableLesson, TableProgram, run, search};
use crate::tools::table::rows::{FORM_GAP, Reader, TABLE_SHARE};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct TableTool;
source!(TableTool, "a learner of questions over a told table: the stories of a kind are rows of named things with numbers (facts with their years), the questions name some of them, and a small program for each form of question is found from the lessons; no word of any set stands here, and no list of words");

pub const TABLE_TOOL: Tool = Tool { name: "table", needs_labels: false, probe: false, asks: false, over: None, learn: learn_table };
because!(TABLE_TOOL, TableTool, "the table tool in the list of the tools: it needs no label of a lesson sentence, it has no rule by the form of the question alone, and it asks for no answer");

const TABLE_OWN_SHARE: f64 = 0.9;
because!(TABLE_OWN_SHARE, TableTool, "the tool reads a kind only when its programs give back at least this share of the answers of its own lessons: a kind that is no table, or whose questions no program of the language answers, is left to the other tools");

const LEAST_PER_FORM: usize = 3;
because!(LEAST_PER_FORM, TableTool, "the lessons that a form of question needs on the mean: with fewer, a program would be chosen by too few answers, and the search is not made");

const NOT_TABLE: &str = "the table tool does not read this kind: its told sentences are no table of one form";
because!(NOT_TABLE, TableTool, "what the tool says when the lessons are no table");

const TABLE_NOT_READ: &str = "the table tool does not read this kind: no program of its language gives back the answers of the lessons";
because!(TABLE_NOT_READ, TableTool, "what the tool says when its programs do not read its own lessons");

const NOTE_START: &str = "table tool, the program of the form of the question:";
because!(NOTE_START, TableTool, "the start of the note of an answer, before the program that gave it");

type QuestionForm = (String, usize, usize);
because!(QuestionForm, TableTool, "a form of question with the number of fields of the rows of its story and the number of its values: the lessons that share it share one program");

struct TableModel {
    reader: Reader,
    programs: HashMap<QuestionForm, TableProgram>,
}
because!(TableModel, TableTool, "the model of the tool: the words of the form of the rows, and the program of each form of question. nearest gives a form that no lesson held the program of the known form with as many values that shares the most words with it");

impl TableModel {
    fn nearest(&self, key: &QuestionForm) -> Option<&TableProgram> {
        let mine: HashSet<&str> = key.0.split(FORM_GAP).collect();
        let mut best: Option<(usize, &QuestionForm)> = None;
        for known in self.programs.keys().filter(|known| known.1 == key.1 && known.2 == key.2) {
            let shared = known.0.split(FORM_GAP).collect::<HashSet<&str>>().intersection(&mine).count();
            if best.is_none_or(|(top, held)| shared > top || (shared == top && known < held)) {
                best = Some((shared, known));
            }
        }
        best.and_then(|(_, known)| self.programs.get(known))
    }
}

impl Learner for TableModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let table = self.reader.table(sentences);
        let asked = self.reader.asked(question, &table);
        let key: QuestionForm = (asked.form, table.width, asked.values.len());
        let Some(program) = self.programs.get(&key).or_else(|| self.nearest(&key)) else { return Said::default() };
        match run(program, &table, &asked.values) {
            Some(words) => Said { words, note: format!("{NOTE_START} {program:?}") },
            None => Said::default(),
        }
    }
}

pub fn learn_table(lessons: Lessons, lanes: usize) -> Learned {
    let reader = Reader::learn(lessons.cases);
    if reader.table_share(lessons.cases) < TABLE_SHARE {
        return Err(NOT_TABLE.to_string());
    }
    let mut groups: HashMap<QuestionForm, Vec<TableLesson>> = HashMap::new();
    for case in lessons.cases {
        let table = reader.table(&case.sentences);
        let asked = reader.asked(&case.question, &table);
        groups.entry((asked.form, table.width, asked.values.len())).or_default().push((table, asked.values, answer_words(&case.answer)));
    }
    if groups.len() * LEAST_PER_FORM > lessons.cases.len() {
        return Err(TABLE_NOT_READ.to_string());
    }
    let mut keys: Vec<QuestionForm> = groups.keys().cloned().collect();
    keys.sort();
    let found = on_lanes(keys.len(), lanes, &|place| search(&groups[&keys[place]], keys[place].1, keys[place].2));
    let mut support: Vec<(TableProgram, usize)> = Vec::new();
    for (program, count) in found.iter().flatten() {
        match support.iter_mut().find(|(held, _)| held == program) {
            Some(held) => held.1 += count,
            None => support.push((program.clone(), *count)),
        }
    }
    let mut right = 0usize;
    let mut programs: HashMap<QuestionForm, TableProgram> = HashMap::new();
    for (key, got) in keys.into_iter().zip(found) {
        let mut best: Option<(usize, usize, TableProgram)> = got.map(|(program, count)| (count, usize::default(), program));
        for (program, backing) in &support {
            let count = groups[&key].iter().filter(|(table, values, answer)| run(program, table, values).as_ref() == Some(answer)).count();
            if best.as_ref().is_none_or(|(top, top_backing, _)| count > *top || (count == *top && *backing > *top_backing)) {
                best = Some((count, *backing, program.clone()));
            }
        }
        if let Some((count, _, program)) = best {
            right += count;
            programs.insert(key, program);
        }
    }
    if (right as f64) < TABLE_OWN_SHARE * lessons.cases.len() as f64 {
        return Err(TABLE_NOT_READ.to_string());
    }
    Ok(Box::new(TableModel { reader, programs }))
}
because!(learn_table, TableTool, "the table tool as a tool of the one call: each form of question takes, of its own program and the programs of the other forms, the one that reads most of its lessons, and of equals the one that the most lessons of all forms took; a model when the lessons are a table of one form and these programs give back nearly all their answers, and else the words with which it does not read the kind");
