use crate::ask::table::{FactLookup, TableFact, write_table};
use ilm_core::base::lines::one_line;
use ilm_core::base::shares::share;
use ilm_core::know::letters::letters;
use ilm_core::know::probe::{Numbers, lessons_of, read_files};
use ilm_core::know::questions::{Asking, Names, Questions, Read, SortedNames};
use ilm_core::know::signs::is_space;
use ilm_core::know::store::{Fact, OTHERS_FILE, QUESTIONS_FILE, SUBJECTS_FILE, Texts, read_facts};
use ilm_core::know::sure::{Knowledge, OFTEN_NAMES};
use ilm_core::know::titles::head;
use ilm_core::classes::WordClasses;
use patterns::{because, source};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

pub struct FactQuestions;
source!(FactQuestions, "a question about a fact goes through three steps, each a table of counts and none a network: the question is read into a subject and a relation; the fact is looked up in the table of what was read; the answer is given with the sentence of the encyclopedia that states it and, where the probe set gives a sentence form for the relation, the fact in that form. A relation that the reader is not sure of is not looked up, so the command says that it does not know in place of a wrong answer");

const QUESTION_FIT: usize = 1000;
because!(QUESTION_FIT, FactQuestions, "the facts of the lesson questions that the reader of facts learns from, at most this many for a relation (the first ones), so that the relations that people ask about most do not outweigh the frames of the others");

const SURE_PARTS: u32 = 2;
because!(SURE_PARTS, FactQuestions, "a relation is not sure when half or fewer of the held lesson questions that were read in the same way got it right: the right ones taken this many times are then not more than all");

const NEAREST_RELATIONS: usize = 5;
because!(NEAREST_RELATIONS, FactQuestions, "the relations that the command names when it is not sure of the relation of a question: enough for a person to see what was near");

const STEP_LOOK: usize = 2;
because!(STEP_LOOK, FactQuestions, "the number of the step of the look up in the trace, after the step of the reading");

const STEP_SAY: usize = 3;
because!(STEP_SAY, FactQuestions, "the number of the step of the sentence in the trace, after the look up");

const TRACE_INDENT: &str = "      ";
because!(TRACE_INDENT, FactQuestions, "the space before a line of the trace that says more about the step above it");

const CHANCE_PLACES: usize = 2;
because!(CHANCE_PLACES, FactQuestions, "the places after the point of a chance or a share of votes in the trace: a hundredth is as fine as a joined chance means anything");

const SUBJECT_PLACE: &str = "[X]";
because!(SUBJECT_PLACE, FactQuestions, "the place of the subject in the sentence form of a relation, as the probe set writes it");

const OBJECT_PLACE: &str = "[Y]";
because!(OBJECT_PLACE, FactQuestions, "the place of the object in the sentence form of a relation, as the probe set writes it");

const SPACE_POINT: &str = " .";
because!(SPACE_POINT, FactQuestions, "the end of a sentence form of the probe set, a space before the point, which a written sentence does not have");

const SENTENCE_POINT: &str = ".";
because!(SENTENCE_POINT, FactQuestions, "what ends a written sentence, set in place of the end of a sentence form, which has a space before it");

const FRAME_BETWEEN: char = '|';
because!(FRAME_BETWEEN, FactQuestions, "the sign between the parts of a frame as the table writes it");

const TABLE_ABOUT: &str = "about.txt";
because!(TABLE_ABOUT, FactQuestions, "the file of a table of the command that names its inputs and gives the sentence forms and the labels of the relations");

const QUESTIONS_COPY: &str = "questions.txt";
because!(QUESTIONS_COPY, FactQuestions, "the copy of the lesson questions in the folder of a table, so that the reader of questions is learned from what the table was built with");

const READER_SAVED: &str = "reader.txt";
because!(READER_SAVED, FactQuestions, "the file of a table with the reader of questions as it was learned from the lesson questions of the table, so that the command does not learn it again at each opening");

const ABOUT_FIELD: char = '\t';
because!(ABOUT_FIELD, FactQuestions, "the sign between the fields of a line of the file that names the inputs of a table");

const NAMES_LINE: &str = "names";
because!(NAMES_LINE, FactQuestions, "the line of the file of a table with the file of known names that the reader of questions uses");

const TEMPLATE_LINE: &str = "template";
because!(TEMPLATE_LINE, FactQuestions, "a line of the file of a table with the sentence form of a relation");

const LABEL_LINE: &str = "label";
because!(LABEL_LINE, FactQuestions, "a line of the file of a table with the label of a relation as the question set gives it");

const HELD_MARK: &str = "H";
because!(HELD_MARK, FactQuestions, "the mark of a held question in a file of questions");

#[derive(Clone, Debug)]
pub struct QuestionCase {
    pub held: bool,
    pub question: String,
    pub subject: String,
    pub relation: String,
    pub label: String,
    pub answer: String,
}
because!(QuestionCase, FactQuestions, "one question that a person wrote about a fact: whether it is held, the question, the name of its subject, the relation, the label of the relation and the answer");

pub fn read_questions(path: &Path) -> std::io::Result<Vec<QuestionCase>> {
    let mut found: Vec<QuestionCase> = Vec::new();
    for line in BufReader::new(std::fs::File::open(path)?).lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(ABOUT_FIELD).collect();
        if let [part, question, subject, relation, label, answer] = fields.as_slice() {
            found.push(QuestionCase { held: *part == HELD_MARK, question: question.to_string(), subject: subject.to_string(), relation: relation.to_string(), label: label.to_string(), answer: answer.to_string() });
        }
    }
    Ok(found)
}
because!(read_questions, FactQuestions, "the questions of a file in their order; a line that is no question is passed over");

pub fn asking(cases: &[QuestionCase]) -> Vec<Asking<'_>> {
    cases.iter().map(|case| Asking { question: &case.question, subject: &case.subject, relation: &case.relation }).collect()
}
because!(asking, FactQuestions, "questions as the reader of questions learns from them");

pub struct Built {
    pub articles: usize,
    pub facts: usize,
    pub numbers: Numbers,
}
because!(Built, FactQuestions, "what the building of a table gave: the articles and facts of the table, and the numbers of the reading");

pub fn build_facts(store: &Path, facts_file: &Path, questions: &[QuestionCase], names_file: &Path, classes: &WordClasses, lanes: usize, folder: &Path) -> std::io::Result<Built> {
    let facts = read_facts(facts_file)?;
    let mut taken_of: HashMap<&str, usize> = HashMap::new();
    let mut more: Vec<Fact> = Vec::new();
    for case in questions.iter().filter(|case| !case.held) {
        let count = taken_of.entry(case.relation.as_str()).or_insert(0);
        if *count < QUESTION_FIT {
            *count += 1;
            more.push(Fact { lesson: true, relation: case.relation.clone(), subject: case.subject.clone(), object: case.answer.clone(), template: String::new() });
        }
    }
    let texts = Texts::read_files(store, &[SUBJECTS_FILE, QUESTIONS_FILE])?;
    let all: Vec<&Fact> = facts.iter().filter(|fact| fact.lesson).chain(more.iter()).collect();
    let taken = lessons_of(&all, 0, &texts);
    let mut model = Knowledge::learn(&taken, classes, lanes);
    drop(taken);
    drop(texts);
    let mut numbers = Numbers { lessons: all.len(), ..Numbers::default() };
    read_files(&mut model, store, &[SUBJECTS_FILE, OTHERS_FILE, QUESTIONS_FILE], lanes, &mut numbers)?;
    let (articles, count) = write_table(folder, &model, lanes)?;
    let mut about = BufWriter::new(std::fs::File::create(folder.join(TABLE_ABOUT))?);
    writeln!(about, "{NAMES_LINE}{ABOUT_FIELD}{}", names_file.to_string_lossy())?;
    let mut templates: Vec<(&str, &str)> = Vec::new();
    for fact in facts.iter().filter(|fact| fact.lesson) {
        match templates.iter_mut().find(|(relation, _)| *relation == fact.relation) {
            Some(held) => held.1 = &fact.template,
            None => templates.push((&fact.relation, &fact.template)),
        }
    }
    for (relation, template) in templates {
        writeln!(about, "{TEMPLATE_LINE}{ABOUT_FIELD}{relation}{ABOUT_FIELD}{}", one_line(template))?;
    }
    let mut labels: Vec<(&str, &str)> = Vec::new();
    for case in questions.iter().filter(|case| !case.held) {
        match labels.iter_mut().find(|(relation, _)| *relation == case.relation) {
            Some(held) => held.1 = &case.label,
            None => labels.push((&case.relation, &case.label)),
        }
    }
    for (relation, label) in labels {
        writeln!(about, "{LABEL_LINE}{ABOUT_FIELD}{relation}{ABOUT_FIELD}{}", one_line(label))?;
    }
    about.flush()?;
    let mut copy = BufWriter::new(std::fs::File::create(folder.join(QUESTIONS_COPY))?);
    for case in questions.iter().filter(|case| !case.held) {
        let fields = [String::new(), one_line(&case.question), one_line(&case.subject), one_line(&case.relation), one_line(&case.label), one_line(&case.answer)];
        writeln!(copy, "{}", fields.join(&ABOUT_FIELD.to_string()))?;
    }
    copy.flush()?;
    drop(copy);
    let names = SortedNames::read(names_file)?;
    let cases = read_questions(&folder.join(QUESTIONS_COPY))?;
    Questions::learn(&asking(&cases), Some(&names as &dyn Names), true, lanes).save(&folder.join(READER_SAVED))?;
    Ok(Built { articles, facts: count, numbers })
}
because!(build_facts, FactQuestions, "the table of facts of the command: the reader of facts learns from the lesson facts of the probe set and from the facts of the lesson questions, reads every article of the store, and its table is written with the sentence forms and labels of the relations, a copy of the lesson questions and the reader of questions learned from that copy");

pub struct FactSession {
    pub reader: Questions,
    pub names: SortedNames,
    pub lookup: FactLookup,
    pub templates: HashMap<String, String>,
    pub labels: HashMap<String, String>,
}
because!(FactSession, FactQuestions, "the parts of the command that answer a question about a fact: the reader of questions with its known names, the table of facts, and the sentence forms and labels of the relations. open reads the known names and the reader that the table saved (or learns the reader from the lesson questions of the table when it saved none) while the names of the table are read on another thread; answer gives the answer of one question with its trace");

#[derive(Clone, Debug, Default)]
pub struct FactAnswer {
    pub read: Option<Read>,
    pub relation: Option<String>,
    pub row: Option<TableFact>,
    pub said: Option<String>,
    pub trace: Vec<String>,
}
because!(FactAnswer, FactQuestions, "the answer of one question about a fact: what was read, the relation, the fact that was found, the fact as a sentence, and the trace of every step");

fn said_as_sentence(templates: &HashMap<String, String>, labels: &HashMap<String, String>, relation: &str, title: &str, word: &str) -> (String, &'static str) {
    match templates.get(relation).filter(|form| !form.is_empty()) {
        Some(form) => (form.replace(SUBJECT_PLACE, head(title)).replace(OBJECT_PLACE, word).replace(SPACE_POINT, SENTENCE_POINT), "the sentence form of the relation in the probe set"),
        None => (format!("{}: {}: {word}.", head(title), labels.get(relation).map_or(relation, String::as_str)), "the label of the relation in the question set (the sets give no sentence form for it)"),
    }
}
because!(said_as_sentence, FactQuestions, "a fact as a sentence: the sentence form of the relation with the name of the article and the answer in their places; with no form, the name, the label of the relation and the answer");

fn chosen_row(rows: Vec<TableFact>, subject: &str) -> Option<TableFact> {
    let exact: Vec<&TableFact> = rows.iter().filter(|row| letters(&row.title) == subject).collect();
    let pool: Vec<&TableFact> = if exact.is_empty() { rows.iter().collect() } else { exact };
    let mut best: Option<&TableFact> = None;
    for row in pool {
        if best.is_none_or(|top| row.chance > top.chance) {
            best = Some(row);
        }
    }
    best.cloned()
}
because!(chosen_row, FactQuestions, "the fact of a subject as a question writes it: from the articles whose title has the letters of the subject, and with no such article from every article of that name; the one with the highest chance, the first of equals in the order of the titles");

impl FactSession {
    pub fn open(folder: &Path, lanes: usize) -> std::io::Result<FactSession> {
        let mut names_file = PathBuf::new();
        let mut templates: HashMap<String, String> = HashMap::new();
        let mut labels: HashMap<String, String> = HashMap::new();
        for line in BufReader::new(std::fs::File::open(folder.join(TABLE_ABOUT))?).lines() {
            let line = line?;
            let fields: Vec<&str> = line.split(ABOUT_FIELD).collect();
            match fields.as_slice() {
                [NAMES_LINE, path] => names_file = PathBuf::from(path),
                [TEMPLATE_LINE, relation, template] => {
                    templates.insert(relation.to_string(), template.to_string());
                }
                [LABEL_LINE, relation, label] => {
                    labels.insert(relation.to_string(), label.to_string());
                }
                _ => {}
            }
        }
        let saved = folder.join(READER_SAVED);
        let (learned, lookup) = std::thread::scope(|scope| {
            let lookup = scope.spawn(|| FactLookup::open(folder));
            let loaded = saved.exists().then(|| scope.spawn(|| Questions::load(&saved)));
            let cases = loaded.is_none().then(|| scope.spawn(|| read_questions(&folder.join(QUESTIONS_COPY))));
            let learned = SortedNames::read(&names_file).and_then(|names| {
                let reader = match (loaded, cases) {
                    (Some(loaded), _) => loaded.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))?,
                    (None, Some(cases)) => {
                        let cases = cases.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))?;
                        Questions::learn(&asking(&cases), Some(&names as &dyn Names), true, lanes)
                    }
                    (None, None) => Questions::default(),
                };
                Ok((names, reader))
            });
            (learned, lookup.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault)))
        });
        let (names, reader) = learned?;
        Ok(FactSession { reader, names, lookup: lookup?, templates, labels })
    }

    fn label<'a>(&'a self, relation: &'a str) -> &'a str {
        self.labels.get(relation).map_or(relation, String::as_str)
    }

    pub fn answer(&self, question: &str) -> std::io::Result<FactAnswer> {
        let Some(read) = self.reader.read(question, Some(&self.names as &dyn Names)) else {
            return Ok(FactAnswer { trace: vec!["the question has no word".to_string()], ..FactAnswer::default() });
        };
        let relation = self.reader.relation_of(&read).map(str::to_string);
        let mut trace = vec![format!("{}. read: subject \"{}\", relation {} ({})", 1, read.subject, relation.as_deref().unwrap_or("None"), relation.as_deref().and_then(|relation| self.labels.get(relation)).map_or("no label", String::as_str))];
        trace.extend(self.reader.read_trace(&read).into_iter().map(|line| format!("{TRACE_INDENT}{line}")));
        let (right, questions) = read.sure;
        if questions > 0 && right * SURE_PARTS <= questions {
            let nearest: Vec<&str> = read.relations.iter().take(NEAREST_RELATIONS).map(|relation| self.label(self.reader.relations.name(*relation))).collect();
            trace.push(format!("{STEP_LOOK}. no look up: the relation of this question is not sure. The lessons hold questions about these relations: {} (the nearest five)", nearest.join("; ")));
            return Ok(FactAnswer { read: Some(read), relation, trace, ..FactAnswer::default() });
        }
        let subject = letters(&read.subject);
        let rows = match &relation {
            Some(relation) => self.lookup.rows(&subject, relation)?,
            None => Vec::new(),
        };
        let Some(row) = chosen_row(rows, &subject) else {
            let articles = self.lookup.under(&subject).len();
            let none = if articles > 0 { "; none of them keeps a word for this relation" } else { "" };
            trace.push(format!("{STEP_LOOK}. look up: no fact. Articles read under this name: {articles}{none}"));
            return Ok(FactAnswer { read: Some(read), relation, trace, ..FactAnswer::default() });
        };
        let sure = if row.answers > 0 {
            format!("{TRACE_INDENT}how sure: of {} held lesson facts with an answer at this level, {} were right ({:.none$}%)", row.answers, row.right, share(row.right as usize, row.answers as usize), none = 0)
        } else {
            format!("{TRACE_INDENT}how sure: no held lesson fact had an answer at this level")
        };
        let frame: Vec<&str> = row.frame.split(FRAME_BETWEEN).collect();
        if row.sure {
            trace.push(format!("{STEP_LOOK}. look up: no sentence of the article \"{}\" states this fact; its sure words give \"{}\" with {:.places$} of their votes", row.title, row.word, row.chance, places = CHANCE_PLACES));
            let word = frame.get(1).copied().unwrap_or("");
            let often = frame.get(STEP_LOOK).and_then(|step| step.parse::<usize>().ok()).and_then(|step| OFTEN_NAMES.get(step)).copied().unwrap_or("");
            trace.push(format!("{TRACE_INDENT}the sure word with the most votes: \"{word}\", written {often} (in the lessons {} of {} articles that write it so have this answer)", row.hit, row.seen));
            trace.push(format!("{TRACE_INDENT}its first sentence: {}", row.sentence));
        } else if row.listed {
            trace.push(format!("{STEP_LOOK}. look up: the article \"{}\" writes every word of \"{}\", an object of the lessons for this relation", row.title, row.word));
            trace.push(format!("{TRACE_INDENT}in the lessons {} of {} articles that write every word of it have this object", row.hit, row.seen));
            trace.push(format!("{TRACE_INDENT}the sentence of its first word: {}", row.sentence));
        } else {
            trace.push(format!("{STEP_LOOK}. look up: the article \"{}\" keeps \"{}\" for this relation, joined chance {:.places$}", row.title, row.word, row.chance, places = CHANCE_PLACES));
            let shown = frame.join(" ");
            trace.push(if row.seen > 0 { format!("{TRACE_INDENT}its best frame: {shown} (in the lessons the word in this frame is the object {} of {} times)", row.hit, row.seen) } else { format!("{TRACE_INDENT}its best frame: {shown}") });
            trace.push(format!("{TRACE_INDENT}the sentence: {}", row.sentence));
        }
        trace.push(sure);
        let (text, how) = said_as_sentence(&self.templates, &self.labels, &row.relation, &row.title, &row.word);
        trace.push(format!("{STEP_SAY}. as a sentence ({how}): {text}"));
        Ok(FactAnswer { read: Some(read), relation, row: Some(row), said: Some(text), trace })
    }
}

#[derive(Clone, Debug, Default)]
pub struct FactCounts {
    pub held: usize,
    pub read_right: usize,
    pub given: usize,
    pub right: usize,
    pub one_word: usize,
    pub one_read_right: usize,
    pub one_given: usize,
    pub one_right: usize,
    pub one_both: usize,
    pub seconds: f64,
}
because!(FactCounts, FactQuestions, "the numbers of the command on held questions: the questions, those read right (subject and relation), those with an answer, those whose answer is the answer of the set, and the same for the questions whose answer is one word, with the seconds of all answers");

pub fn one_word(answer: &str) -> bool {
    answer.split(is_space).filter(|part| !part.is_empty()).count() == 1
}
because!(one_word, FactQuestions, "whether an answer of the set is one word: one run of signs between spaces");

pub fn held_answers(session: &FactSession, cases: &[QuestionCase]) -> std::io::Result<(FactCounts, Vec<String>)> {
    let started = std::time::Instant::now();
    let mut counts = FactCounts::default();
    let mut lines: Vec<String> = Vec::new();
    for case in cases.iter().filter(|case| case.held) {
        let got = session.answer(&case.question)?;
        let single = one_word(&case.answer);
        let read_right = got.read.as_ref().is_some_and(|read| letters(&read.subject) == letters(&case.subject)) && got.relation.as_deref() == Some(case.relation.as_str());
        let given = got.row.is_some();
        let right = got.row.as_ref().is_some_and(|row| letters(&row.word) == letters(&case.answer));
        counts.held += 1;
        counts.read_right += usize::from(read_right);
        counts.given += usize::from(given);
        counts.right += usize::from(right);
        counts.one_word += usize::from(single);
        counts.one_read_right += usize::from(single && read_right);
        counts.one_given += usize::from(single && given);
        counts.one_right += usize::from(single && right);
        counts.one_both += usize::from(single && read_right && right);
        let fields = [
            got.read.as_ref().map_or(String::new(), |read| one_line(&read.subject)),
            got.relation.clone().unwrap_or_default(),
            got.row.as_ref().map_or(String::new(), |row| row.word.clone()),
            got.row.as_ref().map_or(String::new(), |row| row.title.clone()),
            got.said.clone().map(|said| one_line(&said)).unwrap_or_default(),
        ];
        lines.push(fields.join(&ABOUT_FIELD.to_string()));
    }
    counts.seconds = started.elapsed().as_secs_f64();
    Ok((counts, lines))
}
because!(held_answers, FactQuestions, "the held questions of a file answered by the command: the numbers, and for each question the subject that was read, the relation, the answer, the title of its article and the answer as a sentence");
