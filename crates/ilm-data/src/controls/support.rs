use crate::controls::babi::babi_exact;
use crate::controls::file::ChangedCase;
use crate::sets::readers::common::stripped;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct SupportChanges;
source!(SupportChanges, "changed stories of the twenty tasks: the files of the first form of the set name the supporting sentences of each question, which no learner gets; they are read from the training file of a task and joined to the lessons by story and question. Each supporting sentence is taken out in turn, with the answer that the exact reading of the task gives for the changed story, or no answer where the story no longer gives one; and the first told sentence that supports nothing is taken out, with the old answer");

const SUPPORT_KIND: &str = "babi";
because!(SUPPORT_KIND, SupportChanges, "the start of the name of a kind of the twenty tasks, before the number of its task");

const SUPPORT_FILE_START: &str = "qa";
because!(SUPPORT_FILE_START, SupportChanges, "the start of the name of a file of the first form of the set, before the number of its task");

const SUPPORT_FILE_NUMBER_END: &str = "_";
because!(SUPPORT_FILE_NUMBER_END, SupportChanges, "the mark after the number of the task in the name of a file of the set");

const SUPPORT_FILE_END: &str = "_train.txt";
because!(SUPPORT_FILE_END, SupportChanges, "the end of the name of the training file of a task");

const SUPPORT_FIELD: char = '\t';
because!(SUPPORT_FIELD, SupportChanges, "the mark between the question, the answer and the numbers of the supporting sentences of a line of a question");

const SUPPORT_NUMBER_GAP: char = ' ';
because!(SUPPORT_NUMBER_GAP, SupportChanges, "the mark between the number of a line and its text");

const SUPPORT_CARRIAGE: &str = "\r";
because!(SUPPORT_CARRIAGE, SupportChanges, "the sign that ends a line in some files, alone or before the line feed, and is read as the end of a line");

const SUPPORT_LINE: &str = "\n";
because!(SUPPORT_LINE, SupportChanges, "the end of a line of a file of the set");

const SUPPORT_TRUTH_GAP: &str = " ";
because!(SUPPORT_TRUTH_GAP, SupportChanges, "the mark between the words of a new truth");

const SUPPORT_FIELDS: usize = 3;
because!(SUPPORT_FIELDS, SupportChanges, "the fields of a line of a question: the question, the answer and the numbers of the supporting sentences");

const SORT_SUPPORT_OUT: &str = "a supporting sentence taken out";
because!(SORT_SUPPORT_OUT, SupportChanges, "the sort of the change where a sentence that supports the answer is taken out");

const SORT_UNUSED_OUT: &str = "another sentence taken out";
because!(SORT_UNUSED_OUT, SupportChanges, "the sort of the change where the first sentence that supports nothing is taken out");

type SupportTable = HashMap<(Vec<String>, String), Vec<usize>>;
because!(SupportTable, SupportChanges, "the places of the supporting sentences of each question in its story, by the story so far and the question");

fn support_file(folder: &Path, task: usize) -> Result<PathBuf, String> {
    let start = format!("{SUPPORT_FILE_START}{task}{SUPPORT_FILE_NUMBER_END}");
    let mut names: Vec<String> = std::fs::read_dir(folder).map_err(|error| format!("{}: {error}", folder.display()))?.filter_map(|entry| entry.ok().map(|entry| entry.file_name().to_string_lossy().into_owned())).collect();
    names.sort();
    let name = names.into_iter().find(|name| name.starts_with(&start) && name.ends_with(SUPPORT_FILE_END)).ok_or_else(|| format!("{}: no training file of task {task}", folder.display()))?;
    Ok(folder.join(name))
}
because!(support_file, SupportChanges, "the training file of a task in the folder of the first form of the set");

fn support_table(file: &Path) -> Result<SupportTable, String> {
    let text = std::fs::read_to_string(file).map_err(|error| format!("{}: {error}", file.display()))?;
    let text = text.replace(&format!("{SUPPORT_CARRIAGE}{SUPPORT_LINE}"), SUPPORT_LINE).replace(SUPPORT_CARRIAGE, SUPPORT_LINE);
    let mut lines: Vec<&str> = text.split(SUPPORT_LINE).collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut out: SupportTable = HashMap::new();
    let mut story: Vec<String> = Vec::new();
    let mut place: HashMap<u64, usize> = HashMap::new();
    for line in lines {
        let (number, rest) = line.split_once(SUPPORT_NUMBER_GAP).unwrap_or((line, ""));
        let number: u64 = stripped(number).parse().map_err(|_| format!("{}: a line with no number", file.display()))?;
        if number == 1 {
            story.clear();
            place.clear();
        }
        if rest.contains(SUPPORT_FIELD) {
            let fields: Vec<&str> = rest.split(SUPPORT_FIELD).collect();
            if fields.len() < SUPPORT_FIELDS {
                return Err(format!("{}: a question with no numbers", file.display()));
            }
            let mut marked: Vec<usize> = Vec::new();
            for found in fields[SUPPORT_FIELDS - 1].split_whitespace() {
                let found: u64 = found.parse().map_err(|_| format!("{}: a number of a supporting sentence that is no number", file.display()))?;
                marked.extend(place.get(&found).copied());
            }
            out.insert((story.clone(), stripped(fields[0]).to_string()), marked);
        } else {
            place.insert(number, story.len());
            story.push(stripped(rest).to_string());
        }
    }
    Ok(out)
}
because!(support_table, SupportChanges, "the supporting sentences of each question of a training file: a line with a question names the numbers of its supporting lines, and a line with the number one starts a new story");

pub fn support_changes(lessons: &[Case], whole: bool, kind: &str, folder: Option<&Path>) -> Result<Vec<ChangedCase>, String> {
    let task: usize = kind.strip_prefix(SUPPORT_KIND).and_then(|number| number.parse().ok()).ok_or("no kind of the twenty tasks")?;
    let folder = folder.ok_or("the folder of the training files of the first form of the set is not named")?;
    let table = support_table(&support_file(folder, task)?)?;
    let part: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in part {
        let case = &lessons[place];
        let Some(marked) = table.get(&(case.sentences.clone(), case.question.clone())).filter(|marked| !marked.is_empty()) else { continue };
        let mut supporting = marked.clone();
        supporting.sort_unstable();
        supporting.dedup();
        let without = |at: usize| -> Vec<String> { case.sentences.iter().enumerate().filter(|(other, _)| *other != at).map(|(_, sentence)| sentence.clone()).collect() };
        for at in supporting {
            let cut = without(at);
            let truth = babi_exact(task, &cut, &case.question).map(|words| words.join(SUPPORT_TRUTH_GAP)).unwrap_or_default();
            out.push(ChangedCase { place: Some(place), sort: SORT_SUPPORT_OUT.to_string(), sentences: cut, question: case.question.clone(), truth });
        }
        if let Some(free) = (0..case.sentences.len()).find(|at| !marked.contains(at)) {
            out.push(ChangedCase { place: Some(place), sort: SORT_UNUSED_OUT.to_string(), sentences: without(free), question: case.question.clone(), truth: case.answer.clone() });
        }
    }
    Ok(out)
}
because!(support_changes, SupportChanges, "the changed stories of the lessons that the model does not learn, or of all lessons for a model of all lessons, whose question the training file names with its supporting sentences");
