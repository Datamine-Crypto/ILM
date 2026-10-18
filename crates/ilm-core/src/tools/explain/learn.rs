use crate::base::digest::Sha256;
use crate::base::lanes::on_lanes;
use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::call::{AnswerMark, FAULT_DETAIL, Learned, Learner, Lessons, Reading, Said};
use crate::tools::explain::known_good::stored_source;
use crate::tools::explain::marks::{SeenFrames, answer_mark};
use crate::tools::explain::story_disk::put_text;
use crate::tools::explain::text::LIBRARY_NOTE;
use crate::tools::explain::costs::{WAY_FACTS, WAY_TEXTS, WAY_WORDING, cost_scope, learning_notes, spent_way};
use crate::tools::explain::chain::{Facts, Filling, asks, chain_facts, rules_key, told_with_question, keeps_history, knowledge_base, reply, reply_filling};
use crate::tools::explain::derive::{BY_MARK, KNOWN_MARK, proof_lines};
use crate::tools::explain::provenance::{AnswerSteps, answer_steps, used_facts, used_rules};
use crate::tools::explain::why::missing_step;
use crate::tools::explain::lessons::{lesson_base, reads_lessons};
use crate::tools::explain::wording_reading::{learned_reading, reading_table};
use crate::tools::explain::wording::{worded_model, written_facts_model};
use crate::tools::explain::text::{LogicPiece, Topic, fact_numbers, logic_piece, read_topic, text_words, word_not_mark};
use crate::tools::explain::words::{WordId, word_text, word_texts};
use crate::base::words::first_capital;
use patterns::{because, source};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

pub struct ExplainedLogic;
source!(ExplainedLogic, "a tool whose logic is told in English in a library of texts, in place of code: it takes the topics of the library whose question forms fit the questions of the lessons, chains the rules of the topic of a question over the told sentences of the case, and answers with the first answer whose sentences hold; a new kind of reasoning is a new text of rules, which a reader of the library can check sentence by sentence");

pub const LIBRARY_FOLDER: &str = "data/explain";
because!(LIBRARY_FOLDER, ExplainedLogic, "the folder of the library from the root of the repository, where the program runs");

const LIBRARY_ENDING: &str = "txt";
because!(LIBRARY_ENDING, ExplainedLogic, "the texts of the library are plain text, so that any editor writes a new topic, and other files in the folder are left alone");

const NO_LIBRARY: &str = "the explain tool does not read this kind: the library of told logic cannot be read";
because!(NO_LIBRARY, ExplainedLogic, "what the tool says when the folder of the library is not there or a text of it cannot be read");

const NO_TOPIC: &str = "the explain tool does not read this kind: no topic of the library answers most of the lessons that it asks";
because!(NO_TOPIC, ExplainedLogic, "what the tool says for lessons whose questions no topic of the library asks, or answers right in most of a sample");

const TOPIC_SAMPLE: usize = 50;
because!(TOPIC_SAMPLE, ExplainedLogic, "the lessons on which a topic shows that it fits a kind: the first lessons whose questions it asks; enough to tell a topic written for the kind from one whose question form fits by chance, in little time");

const TOPIC_TRUST_PERCENT: usize = 50;
because!(TOPIC_TRUST_PERCENT, ExplainedLogic, "a topic is kept for a kind when it answers right at least this share of its sample: a topic written for the kind answers nearly all, and a topic whose question form fits a text that it was not written for answers few");

const WHOLE_PERCENT: usize = 100;
because!(WHOLE_PERCENT, ExplainedLogic, "shares are counted in hundredths, so that a share and its bar are compared in whole numbers");

const EXPLAIN_NOTE_NONE: &str = "no answer of the library holds";
because!(EXPLAIN_NOTE_NONE, ExplainedLogic, "the note of a case whose question no answer of the library answers");

const NO_SUCH_TOPIC: &str = "builds on a topic that the library does not hold";
because!(NO_SUCH_TOPIC, ExplainedLogic, "the fault of a topic that names a topic to build on that no text of the library is");

const TWO_TOPICS: &str = "two texts of the library have this name";
because!(TWO_TOPICS, ExplainedLogic, "the fault of a library whose folders hold two texts of one name, since a topic is named by its text");

fn library_texts(folder: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(folder).map_err(|fault| format!("{NO_LIBRARY}: {fault}"))? {
        let path = entry.map_err(|fault| format!("{NO_LIBRARY}: {fault}"))?.path();
        if path.is_dir() {
            library_texts(&path, out)?;
        } else if path.extension().is_some_and(|ending| ending == LIBRARY_ENDING) {
            out.push(path);
        }
    }
    Ok(())
}
because!(library_texts, ExplainedLogic, "the texts of the library in a folder and in the folders inside it, so that concepts, knowledge and wording can stand apart");

pub fn read_library(folder: &Path) -> Result<Vec<Topic>, String> {
    printed_library(folder).map(|(topics, _)| topics)
}
pub fn printed_library(folder: &Path) -> Result<(Vec<Topic>, String), String> {
    let mut paths = Vec::new();
    library_texts(folder, &mut paths)?;
    paths.sort_by_key(|path| path.file_stem().map(|stem| stem.to_string_lossy().to_string()).unwrap_or_default());
    let mut topics: Vec<Topic> = Vec::new();
    let mut print = Sha256::new();
    for path in paths {
        let text = std::fs::read_to_string(&path).map_err(|fault| format!("{NO_LIBRARY}: {fault}"))?;
        let name = path.file_stem().map(|stem| stem.to_string_lossy().to_string()).unwrap_or_default();
        if topics.iter().any(|topic| topic.name == name) {
            return Err(format!("{NO_LIBRARY}: {name}: {TWO_TOPICS}"));
        }
        if path.strip_prefix(folder).ok().and_then(|inside| inside.components().next()).is_some_and(|first| first.as_os_str() == LEARNED_FOLDER) && written_print(&text) != Some(lines_print(&text)) {
            return Err(format!("{NO_LIBRARY}: {name}: {LEARNED_EDITED}"));
        }
        let mut named = Vec::new();
        put_text(&mut named, &name);
        put_text(&mut named, &text);
        print.add(&named);
        topics.push(read_topic(&name, &text)?);
    }
    Ok((topics, print.hex().chars().take(LIBRARY_PRINT_SIGNS).collect()))
}
because!(printed_library, ExplainedLogic, "the topics of the library, refusing two texts of one name in any of its folders and a learned text whose lines do not match the print that its step wrote, and a fingerprint of the texts as they were read, each name and text with its length in the order of the names, so that two runs that print the same fingerprint read the same library and a run that differs from another with another fingerprint is no fault of the program");

pub const LINES_PRINT_MARK: &str = "lines print ";
because!(LINES_PRINT_MARK, ExplainedLogic, "the words in the first note of a learned text before the print of its lines that are not notes, which the step that wrote the text puts there and the reader of the library checks");

const LEARNED_EDITED: &str = "a learned text whose lines that are not notes do not match the print that the step which wrote it put in its first note: a person edited it, or the step did not write it; write it again with its step";
because!(LEARNED_EDITED, ExplainedLogic, "the fault of a learned text that a person edited: a line typed into it would count as discovered, so the library refuses it until its step writes it again");

pub fn lines_print(text: &str) -> String {
    let mut print = Sha256::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with(LIBRARY_NOTE)) {
        let mut bytes = Vec::new();
        put_text(&mut bytes, line);
        print.add(&bytes);
    }
    print.hex().chars().take(LIBRARY_PRINT_SIGNS).collect()
}
because!(lines_print, ExplainedLogic, "the print of the lines of a text that are not notes, each with its length, in their order: the step that writes a learned text puts it in the text's first note, and the reader of the library finds by it a learned text that a person changed");

fn written_print(text: &str) -> Option<String> {
    let note = text.lines().map(str::trim).find(|line| line.starts_with(LIBRARY_NOTE))?;
    let (_, after) = note.split_once(LINES_PRINT_MARK)?;
    Some(after.chars().take_while(char::is_ascii_hexdigit).collect())
}
because!(written_print, ExplainedLogic, "the print that the first note of a learned text holds after its mark, or nothing when its first note holds none");

const LIBRARY_PRINT_SIGNS: usize = 16;
because!(LIBRARY_PRINT_SIGNS, ExplainedLogic, "the signs of the fingerprint of the library that a run prints: enough that two libraries that differ never print the same, short enough to read and compare by eye");

because!(read_library, ExplainedLogic, "the topics of the library, one for each text of its folders in the order of the names of the texts, or the fault of the first text that cannot be read");

pub fn built_on(topics: &[Topic]) -> Result<Vec<Topic>, String> {
    let mut out: Vec<Topic> = Vec::new();
    for topic in topics {
        let mut whole = topic.clone();
        let mut done: Vec<String> = vec![topic.name.clone()];
        let mut waiting: Vec<String> = topic.builds_on.clone();
        while let Some(name) = waiting.pop() {
            if done.contains(&name) {
                continue;
            }
            let Some(base) = topics.iter().find(|other| other.name == name) else { return Err(format!("{}: {NO_SUCH_TOPIC}: {name}", topic.name)) };
            whole.rules.extend(base.rules.iter().cloned());
            whole.reply_lines.extend(base.reply_lines.iter().cloned());
            whole.facts.extend(base.facts.iter().cloned());
            waiting.extend(base.builds_on.iter().cloned());
            done.push(name);
        }
        whole.rules_key = rules_key(&whole.rules);
        out.push(whole);
    }
    Ok(out)
}
because!(built_on, ExplainedLogic, "each topic with the rules, reply lines and told facts of the topics that it builds on, and of the topics that they build on, each once, after its own; its question forms stay its own; the fault of a topic that builds on a name that the library does not hold");

pub struct Library {
    pub texts: Vec<Topic>,
    pub topics: Vec<Topic>,
    bases: Vec<OnceLock<Arc<Facts>>>,
    pub print: String,
}
because!(Library, ExplainedLogic, "the library as it is read: each text as it is written, each topic with the topics that it builds on, and for each topic the facts that it knows before any story, found the first time that a kind needs them and kept for every later kind, with a line on the error output when they stopped at their bound; knowledge_stopped says so for one topic; and for the library of the folder of the run, the fingerprint of the texts read");

const DEFAULT_MARK: &str = " otherwise";
because!(DEFAULT_MARK, ExplainedLogic, "the end of the answer of the library that a note names when the answer holds otherwise, as the library writes it, so that an answer by default is told from an answer that a reading proves");

const KNOWLEDGE_STOPPED: &str = "what a topic knows before any story stopped at the bound of its facts or reads, so some of it is missing";
because!(KNOWLEDGE_STOPPED, ExplainedLogic, "the line that names a topic whose knowledge was cut by a bound, so that a writer of the library sees it and the share of a kind does not fall with no word");

impl Library {
    pub fn of(texts: Vec<Topic>) -> Result<Library, String> {
        let topics = built_on(&texts)?;
        Ok(Library::built(texts, topics))
    }

    fn built(texts: Vec<Topic>, topics: Vec<Topic>) -> Library {
        let bases = topics.iter().map(|_| OnceLock::new()).collect();
        Library { texts, topics, bases, print: String::new() }
    }

    pub(super) fn base(&self, place: usize) -> &Arc<Facts> {
        let topic = &self.topics[place];
        self.bases[place].get_or_init(|| {
            let known = knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions));
            if known.stopped_at_bound() {
                eprintln!("{KNOWLEDGE_STOPPED}: {}", topic.name);
            }
            Arc::new(known)
        })
    }

    pub fn knowledge_stopped(&self, place: usize) -> bool {
        self.base(place).stopped_at_bound()
    }
}

static READ_ONCE: OnceLock<Result<Arc<Library>, String>> = OnceLock::new();
because!(READ_ONCE, ExplainedLogic, "the library of the folder, read and built the first time that a kind needs it and shared by every later kind of the run, since its texts do not change while the program runs");

pub fn library() -> Result<Arc<Library>, String> {
    READ_ONCE.get_or_init(|| printed_library(Path::new(LIBRARY_FOLDER)).and_then(|(topics, print)| Library::of(topics).map(|library| Library { print, ..library })).map(Arc::new)).clone()
}
because!(library, ExplainedLogic, "the library of the folder of the run, read once, or the fault of reading it");

pub struct ExplainModel {
    library: Arc<Library>,
    kept: Vec<usize>,
    taught: HashMap<usize, Arc<Facts>>,
    stops: AtomicUsize,
    reading: Reading,
    frames: Option<SeenFrames>,
}
because!(ExplainModel, ExplainedLogic, "how the model reads its kind (written topics, learned lines, or both), and the topics of the library whose question forms fit the questions of the lessons, by their places in the library, the one that answers the largest share of its sample first, and for a topic that reads the lessons as facts, the facts that it knows with the lessons of this learning; and how many chains of its answers ended at a bound; and for the model of a kind, the frames of the sentences of its lessons, by which it marks a guess");

impl ExplainModel {
    pub(super) fn first_topic(&self) -> Option<usize> {
        self.kept.first().copied()
    }
}

fn topic_answer(topic: &Topic, base: &Arc<Facts>, sentences: &[String], question: &str, asked: &[WordId], (stops, frames): (Option<&AtomicUsize>, Option<&SeenFrames>)) -> Option<Said> {
    if !topic.questions.iter().any(|form| asks(form, asked)) {
        return None;
    }
    let facts = chain_facts(&told_with_question(sentences, question, &topic.rules), base, &topic.rules, topic.rules_key);
    if facts.stopped_at_bound()
        && let Some(stops) = stops
    {
        stops.fetch_add(1, Ordering::Relaxed);
    }
    let answered = reply(&topic.questions, asked, &facts)?;
    let found = format!("{}: {}", topic.name, answered.reply.text);
    let note = match reply_filling(&topic.reply_lines, asked, &answered) {
        Some((line, filling)) => format!("{} ({found})", said_sentence(&line.sentence, &filling, &told_forms(sentences, question))),
        None => found,
    };
    let words = answer_words(&word_texts(&answered.words).join(" "));
    let mark = answer_mark(&words, answered.reply.text.ends_with(DEFAULT_MARK), (frames, sentences, question));
    Some(Said { words, note, mark })
}
because!(topic_answer, ExplainedLogic, "the answer of one topic to a question, with the facts that it knows before any story, a chain that ended at a bound counted when a count is given: nothing when none of its question forms fits the question or no answer of it holds; else the words of the answer and a note that names the topic and the answer of the library, after the reply sentence when a reply line says it");

fn topic_steps(topic: &Topic, base: &Arc<Facts>, sentences: &[String], question: &str, asked: &[WordId]) -> Option<AnswerSteps> {
    if !topic.questions.iter().any(|form| asks(form, asked)) {
        return None;
    }
    answer_steps(topic, base, &told_with_question(sentences, question, &topic.rules), asked, false)
}
because!(topic_steps, ExplainedLogic, "the answer of one topic with its proof, as topic_answer finds the answer: nothing for a topic none of whose question forms asks the question");

pub(super) fn proof_text(name: &str, answered: &AnswerSteps) -> Vec<String> {
    let mut lines = vec![format!("{PROOF_HEAD} {name}: {}", answered.text)];
    lines.extend(proof_lines(&answered.steps).into_iter().map(|line| format!("{PROOF_STEP}{line}")));
    if answered.by_default {
        lines.push(format!("{PROOF_STEP}{PROOF_DEFAULT}"));
    }
    lines
}
because!(proof_text, ExplainedLogic, "the proof of the answer of one topic as lines: the answer of the library that held, and the steps that it rests on, back to the told sentences; an answer that holds otherwise is said to rest on no step");

pub fn library_proof(library: &Library, sentences: &[String], question: &str) -> Option<(Vec<String>, Vec<String>)> {
    let asked = fact_numbers(question);
    (0..library.topics.len()).find_map(|place| {
        let topic = &library.topics[place];
        if !topic.questions.iter().any(|form| asks(form, &asked)) {
            return None;
        }
        let answered = answer_steps(topic, library.base(place), &told_with_question(sentences, question, &topic.rules), &asked, true)?;
        Some((answer_words(&word_texts(&answered.words).join(" ")), proof_text(&topic.name, &answered)))
    })
}
because!(library_proof, ExplainedLogic, "the answer of a whole library to a told text and a question that no kind of lessons stands behind, with its proof: the first topic, in the order of the library, whose question form fits and whose answer rests on told sentences, never an answer that holds otherwise; the words of the answer and the proof lines of that topic");

const DISCOVERED_ON: &str = " [discovered on ";
because!(DISCOVERED_ON, ExplainedLogic, "the words after a line of a proof whose rule or known fact the model learned from the lessons of a kind, before the name of that kind, as the report of discovery marks them");

const GIVEN_MARK: &str = " [given]";
because!(GIVEN_MARK, ExplainedLogic, "the words after a line of a proof whose rule a text of the library holds");

const MARK_CLOSE: &str = "]";
because!(MARK_CLOSE, ExplainedLogic, "the sign that closes the mark of a line of a proof");

const FACT_END: char = '.';
because!(FACT_END, ExplainedLogic, "the point that ends a learned line of a fact, which a line of a proof does not write");

pub fn marked_proof(proof: &[String], rules: &[(String, bool)], learned: &[String], kind: &str) -> Vec<String> {
    let found: HashSet<&str> = rules.iter().filter(|(_, held)| !*held).map(|(rule, _)| rule.as_str()).collect();
    let facts: HashSet<String> = learned.iter().map(|line| line.trim().trim_end_matches(FACT_END).to_lowercase()).collect();
    proof
        .iter()
        .map(|line| {
            if let Some(fact) = line.trim_start().strip_prefix(KNOWN_MARK) {
                return if facts.contains(&fact.trim().to_lowercase()) { format!("{line}{DISCOVERED_ON}{kind}{MARK_CLOSE}") } else { line.clone() };
            }
            match line.split_once(BY_MARK) {
                Some((_, rule)) if found.contains(rule.trim()) => match stored_source(rule.trim()) {
                    Some(source) => format!("{line}{DISCOVERED_ON}{source}{STORED_MARK}{MARK_CLOSE}"),
                    None => format!("{line}{DISCOVERED_ON}{kind}{MARK_CLOSE}"),
                },
                Some(_) => format!("{line}{GIVEN_MARK}"),
                None => line.clone(),
            }
        })
        .collect()
}
because!(marked_proof, ExplainedLogic, "the lines of a proof with each rule marked as discovered on a kind, when no text of the library holds it and the model learned it from the lessons of that kind, or as given; and each known fact that the model learned from the lessons marked as discovered, as the report of discovery marks them, so that one proof shows which parts of an answer were discovered and which were written by hand; a rule of the known-good store is marked as discovered on the kind that its note names, from the store");

const STORED_MARK: &str = ", from the store";
because!(STORED_MARK, ExplainedLogic, "the words after the source of a rule of a proof that came from the known-good store, so that a reader tells a rule taken from an earlier kind from one that the kind discovered itself");

const PROOF_HEAD: &str = "proof of the answer";
because!(PROOF_HEAD, ExplainedLogic, "the first line of a proof, before the topic and the answer of the library that held");

const PROOF_STEP: &str = "  ";
because!(PROOF_STEP, ExplainedLogic, "the indent of the steps of a proof under its first line");

const PROOF_DEFAULT: &str = "(an answer that holds otherwise: no step proves it)";
because!(PROOF_DEFAULT, ExplainedLogic, "what a proof says of an answer by default, so that a reader never takes it for a proved answer");

impl Learner for ExplainModel {
    fn proof(&self, sentences: &[String], question: &str) -> Vec<String> {
        let asked = fact_numbers(question);
        self.kept.iter().find_map(|place| topic_steps(&self.library.topics[*place], self.taught.get(place).unwrap_or_else(|| self.library.base(*place)), sentences, question, &asked).map(|answered| proof_text(&self.library.topics[*place].name, &answered))).unwrap_or_default()
    }

    fn rules_used(&self, sentences: &[String], question: &str) -> Vec<(String, bool)> {
        let asked = fact_numbers(question);
        self.kept.iter().find_map(|place| topic_steps(&self.library.topics[*place], self.taught.get(place).unwrap_or_else(|| self.library.base(*place)), sentences, question, &asked)).map(|answered| used_rules(&answered)).unwrap_or_default()
    }

    fn facts_used(&self, sentences: &[String], question: &str) -> Vec<(String, bool)> {
        let asked = fact_numbers(question);
        self.kept.iter().find_map(|place| topic_steps(&self.library.topics[*place], self.taught.get(place).unwrap_or_else(|| self.library.base(*place)), sentences, question, &asked)).map(|answered| used_facts(&answered)).unwrap_or_default()
    }

    fn missing(&self, sentences: &[String], question: &str, expected: &[String]) -> Vec<String> {
        self.kept.iter().find_map(|place| missing_step(&self.library.topics[*place], self.taught.get(place).unwrap_or_else(|| self.library.base(*place)), sentences, question, expected)).unwrap_or_default()
    }

    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let asked = fact_numbers(question);
        self.kept.iter().find_map(|place| topic_answer(&self.library.topics[*place], self.taught.get(place).unwrap_or_else(|| self.library.base(*place)), sentences, question, &asked, (Some(&self.stops), self.frames.as_ref()))).unwrap_or_else(|| Said { words: Vec::new(), note: EXPLAIN_NOTE_NONE.to_string(), mark: AnswerMark::NoAnswer })
    }

    fn bound_stops(&self) -> usize {
        self.stops.load(Ordering::Relaxed)
    }

    fn reading(&self) -> Option<Reading> {
        Some(self.reading)
    }
}

fn told_forms(sentences: &[String], question: &str) -> HashMap<String, String> {
    let mut seen: HashMap<String, Vec<(String, usize)>> = HashMap::new();
    for text in sentences.iter().map(String::as_str).chain([question]) {
        for word in text_words(text).into_iter().filter(|word| word_not_mark(word)) {
            let forms = seen.entry(word.to_lowercase()).or_default();
            match forms.iter_mut().find(|(form, _)| *form == word) {
                Some((_, uses)) => *uses += 1,
                None => forms.push((word, 1)),
            }
        }
    }
    seen.into_iter().filter_map(|(small, forms)| forms.into_iter().enumerate().max_by_key(|(at, (_, uses))| (*uses, Reverse(*at))).map(|(_, (form, _))| (small, form))).collect()
}
because!(told_forms, ExplainedLogic, "the form in which the story and the question write each word, by its small letters: the form that they use most, and of forms of equal use the first, so that a reply writes a name with its capital letter and a common word in small letters, as the story does");

fn said_word(run: &str, filling: &Filling, forms: &HashMap<String, String>) -> String {
    let form = |word: &WordId| forms.get(word_text(*word)).cloned().unwrap_or_else(|| word_text(*word).to_string());
    match logic_piece(run) {
        LogicPiece::Open(place) => filling.get(place).into_iter().flatten().map(form).collect::<Vec<String>>().join(" "),
        LogicPiece::OpenWithEnd(place, ending) => filling.get(place).into_iter().flatten().map(|stem| format!("{}{ending}", form(stem))).collect::<Vec<String>>().join(" "),
        LogicPiece::Word(_) => run.to_string(),
    }
}
because!(said_word, ExplainedLogic, "a word of a reply sentence as it is written: an open place as the words that fill it, in the form in which the story writes them, with the ending that follows the place, and any other word as the library writes it");

fn said_sentence(sentence: &str, filling: &Filling, forms: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let mut rest = sentence;
    for token in text_words(sentence) {
        let Some(at) = rest.find(token.as_str()) else { break };
        out.push_str(&rest[..at]);
        let next = at + token.len();
        out.push_str(&if word_not_mark(&token) { said_word(&token, filling, forms) } else { token });
        rest = &rest[next..];
    }
    out.push_str(rest);
    first_capital(&out)
}
because!(said_sentence, ExplainedLogic, "the sentence of a reply line with its open places filled: the marks and spaces of the library stay as it writes them, each open place becomes its words, and the reply opens with a capital letter, as each sentence of the library does, also when its first word fills an open place");

pub fn model_of(topics: Vec<Topic>) -> ExplainModel {
    let kept = (0..topics.len()).collect();
    ExplainModel { library: Arc::new(Library::built(topics.clone(), topics)), kept, taught: HashMap::new(), stops: AtomicUsize::default(), reading: Reading::Written, frames: None }
}
because!(model_of, ExplainedLogic, "a model with the given topics as they are, in their order, for a reader that has its texts in hand, read as written lines");

impl ExplainModel {
    pub(super) fn learned(self) -> ExplainModel {
        ExplainModel { reading: Reading::Learned, ..self }
    }

    pub(super) fn framed(self, cases: &[Case]) -> ExplainModel {
        let frames = SeenFrames::of(&self.kept.iter().map(|place| &self.library.topics[*place]).collect::<Vec<&Topic>>(), cases);
        ExplainModel { frames: Some(frames), ..self }
    }
}

const WORDING_FOLDER: &str = "wording";
because!(WORDING_FOLDER, ExplainedLogic, "the folder of the library that holds the wording of the kinds, apart from the concepts and the knowledge that the wording builds on");


fn folder_texts(folder: &str) -> HashSet<String> {
    let mut paths = Vec::new();
    let _ = library_texts(&Path::new(LIBRARY_FOLDER).join(folder), &mut paths);
    paths.iter().filter_map(|path| path.file_stem().map(|stem| stem.to_string_lossy().to_string())).collect()
}
because!(folder_texts, ExplainedLogic, "the names of the texts of one folder of the library of the run; none when the folder cannot be read");


const LEARNED_FOLDER: &str = "learned";
because!(LEARNED_FOLDER, ExplainedLogic, "the folder of the library that holds the texts that a step of the program wrote from what it learned, never a person: their facts are discovered, not given");

pub(crate) struct FolderTexts {
    pub(crate) wording: HashSet<String>,
    pub(crate) learned: HashSet<String>,
}
because!(FolderTexts, ExplainedLogic, "the names of the texts of two folders of the library: the wording folder, so that a learned reading knows whether the text that it learns over brings rules of a wording written for a kind, and the learned folder, so that a proof counts the facts of its texts as discovered and not as given");

static FOLDER_TEXTS: OnceLock<FolderTexts> = OnceLock::new();
because!(FOLDER_TEXTS, ExplainedLogic, "the names of the texts of the wording and learned folders of the library of the run, read once");

pub(crate) fn folder_names() -> &'static FolderTexts {
    FOLDER_TEXTS.get_or_init(|| FolderTexts { wording: folder_texts(WORDING_FOLDER), learned: folder_texts(LEARNED_FOLDER) })
}
because!(folder_names, ExplainedLogic, "the names of the texts of the wording and learned folders of the library of the run; none for a folder that cannot be read");

pub(crate) fn hand_rules(library: &Library, place: usize) -> bool {
    let wording = &folder_names().wording;
    let mut waiting: Vec<&str> = vec![library.texts[place].name.as_str()];
    let mut done: Vec<&str> = Vec::new();
    while let Some(name) = waiting.pop() {
        if done.contains(&name) {
            continue;
        }
        done.push(name);
        let Some(text) = library.texts.iter().find(|text| text.name == name) else { continue };
        if wording.contains(name) && !text.rules.is_empty() {
            return true;
        }
        waiting.extend(text.builds_on.iter().map(String::as_str));
    }
    false
}
because!(hand_rules, ExplainedLogic, "whether a text, or a text that it builds on, is a wording text of the library with rules of its own, written by a person for a kind: a learned wording over such a text reads by learned lines and written lines together, while the rules of concepts and knowledge are general knowledge");

struct TopicTrial {
    share: Option<(usize, usize)>,
    stops: usize,
    right: usize,
    read: usize,
    sample: usize,
    by_default: bool,
}
because!(TopicTrial, ExplainedLogic, "how a topic of the library did on the lessons of a kind: its share when it fits, the lessons whose chains ended at a bound, the lessons of its sample that it answered right, those that it read before it was judged and the size of its sample, and whether it was dropped because it answers only by an answer that holds otherwise; so a kind that no topic reads names the topics that came nearest");

fn topic_fit(library: &Library, place: usize, lessons: &Lessons, questions: &[Vec<WordId>]) -> TopicTrial {
    let topic = &library.topics[place];
    let sample: Vec<usize> = (0..lessons.cases.len()).filter(|at| topic.questions.iter().any(|form| asks(form, &questions[*at]))).take(TOPIC_SAMPLE).collect();
    if sample.is_empty() {
        return TopicTrial { share: None, stops: usize::default(), right: usize::default(), read: usize::default(), sample: usize::default(), by_default: false };
    }
    let _text = cost_scope(None, Some(&topic.name));
    let taught = reads_lessons(topic).then(|| lesson_base(topic, lessons.cases));
    let base = taught.as_ref().unwrap_or_else(|| library.base(place));
    let most_wrong = sample.len() - (0..=sample.len()).find(|right| right * WHOLE_PERCENT >= sample.len() * TOPIC_TRUST_PERCENT).unwrap_or(sample.len());
    let stops = AtomicUsize::default();
    let (mut right, mut wrong, mut proved) = (0usize, 0usize, 0usize);
    for at in &sample {
        let case = &lessons.cases[*at];
        let said = topic_answer(topic, base, &case.sentences, &case.question, &questions[*at], (Some(&stops), None));
        if let Some(said) = said.filter(|said| said.words == answer_words(&case.answer)) {
            right += 1;
            proved += usize::from(said.mark != AnswerMark::Default);
        } else {
            wrong += 1;
            if wrong > most_wrong {
                return TopicTrial { share: None, stops: stops.into_inner(), right, read: right + wrong, sample: sample.len(), by_default: false };
            }
        }
    }
    if proved == usize::default() {
        return TopicTrial { share: None, stops: stops.into_inner(), right, read: sample.len(), sample: sample.len(), by_default: true };
    }
    TopicTrial { share: Some((right, sample.len())), stops: stops.into_inner(), right, read: sample.len(), sample: sample.len(), by_default: false }
}
because!(topic_fit, ExplainedLogic, "the share of a topic of the library on a kind, when it fits: the lessons of its sample that it answers right and the size of the sample, the first lessons whose questions it asks; nothing when it asks none, answers right fewer than most of its sample, found as soon as it has answered wrong more lessons than the trust allows, so that a topic that does not fit costs only part of its sample, or answers every lesson that it answers right by an answer that holds otherwise, since such a topic reads nothing of the kind (a form that fits any question and gives the common answer of the kind by default); and in either case the lessons of the sample read so far whose chains ended at a bound");

fn topic_trials(library: &Library, lessons: &Lessons, lanes: usize) -> Vec<TopicTrial> {
    let questions: Vec<Vec<WordId>> = lessons.cases.iter().map(|case| fact_numbers(&case.question)).collect();
    on_lanes(library.topics.len(), lanes, &|place| topic_fit(library, place, lessons, &questions))
}
because!(topic_trials, ExplainedLogic, "the share of each topic of the library on a kind, as the lessons it answers right and the size of its sample, nothing for a topic that does not fit, and the lessons whose chains ended at a bound, the topics on several lanes at once");

pub fn topic_shares(library: &Library, lessons: &Lessons, lanes: usize) -> Vec<Option<(usize, usize)>> {
    topic_trials(library, lessons, lanes).into_iter().map(|trial| trial.share).collect()
}
because!(topic_shares, ExplainedLogic, "the share of each topic of the library on a kind, as the lessons it answers right and the size of its sample, the topics on several lanes at once; nothing for a topic that does not fit");

const LEARNING_STOPPED: &str = "the chains of some lessons ended at a bound of the engine while a topic was judged on them, so the topic may be dropped for a kind that it reads";
because!(LEARNING_STOPPED, ExplainedLogic, "the line on the error output that names a topic whose chains on the lessons of a kind ended at a bound, with how many lessons of its sample, so that a writer of the library sees a rule that runs away where no answer of the read part shows it");

const STOPPED_TOPICS: &str = "; topics whose chains ended at a bound on lessons: ";
because!(STOPPED_TOPICS, ExplainedLogic, "the words before the topics that ended at a bound in the fault of a kind that no topic reads, so that the line of the kind says why a topic written for it may be missing");

const TRIED_NOTE_BETWEEN: &str = "; ";
because!(TRIED_NOTE_BETWEEN, ExplainedLogic, "the marks before each note of what the learning tried, in the fault of a kind that no topic reads");

const STOPPED_BETWEEN: &str = ", ";
because!(STOPPED_BETWEEN, ExplainedLogic, "the marks between the names of the topics that ended at a bound");

static STOPS_TOLD: OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> = OnceLock::new();
because!(STOPS_TOLD, ExplainedLogic, "the topics whose stops in learning the run has named on the error output, so that each is named once and the output stays short");

fn learning_stops(library: &Library, trials: &[TopicTrial]) -> Vec<String> {
    let stopped: Vec<(String, usize)> = trials.iter().zip(&library.topics).filter(|(trial, _)| trial.stops > usize::default()).map(|(trial, topic)| (topic.name.clone(), trial.stops)).collect();
    if let Ok(mut told) = STOPS_TOLD.get_or_init(Default::default).lock() {
        for (name, stops) in &stopped {
            if told.insert(name.clone()) {
                eprintln!("{LEARNING_STOPPED}: {name} ({stops})");
            }
        }
    }
    stopped.into_iter().map(|(name, _)| name).collect()
}
because!(learning_stops, ExplainedLogic, "the names of the topics whose chains on the lessons of a kind ended at a bound while they were judged, each named on the error output the first time in the run with the number of such lessons of its sample");

fn later_share(topic: &Topic, base: &Arc<Facts>, lessons: &Lessons, questions: &[Vec<WordId>], from: usize) -> (usize, usize) {
    let window: Vec<usize> = (0..lessons.cases.len()).filter(|at| topic.questions.iter().any(|form| asks(form, &questions[*at]))).skip(from).take(TOPIC_SAMPLE).collect();
    let right = window.iter().filter(|at| topic_answer(topic, base, &lessons.cases[**at].sentences, &lessons.cases[**at].question, &questions[**at], (None, None)).is_some_and(|said| said.words == answer_words(&lessons.cases[**at].answer))).count();
    (right, window.len())
}
because!(later_share, ExplainedLogic, "the lessons that a topic answers right among the next lessons whose questions it asks after the given number, as many as a sample, and how many there are, with the words of the questions of the lessons and the base of the topic made once for all rounds");

fn broken_ties(library: &Library, lessons: &Lessons, tied: &mut [(usize, usize, usize)]) {
    let questions: Vec<Vec<WordId>> = lessons.cases.iter().map(|case| fact_numbers(&case.question)).collect();
    let bases: HashMap<usize, Arc<Facts>> = tied.iter().map(|(place, _, _)| (*place, if reads_lessons(&library.topics[*place]) { lesson_base(&library.topics[*place], lessons.cases) } else { Arc::clone(library.base(*place)) })).collect();
    let mut from = TOPIC_SAMPLE;
    loop {
        let more: Vec<(usize, usize)> = tied.iter().map(|(place, _, _)| later_share(&library.topics[*place], &bases[place], lessons, &questions, from)).collect();
        if more.iter().all(|(_, sample)| *sample == usize::default()) {
            return;
        }
        for ((_, right, sample), (more_right, more_sample)) in tied.iter_mut().zip(more) {
            *right += more_right;
            *sample += more_sample;
        }
        tied.sort_by(|(_, right, sample), (_, other_right, other_sample)| (other_right * sample).cmp(&(right * other_sample)));
        let parted = tied.windows(MIN_TIED).all(|pair| pair[0].1 * pair[1].2 != pair[1].1 * pair[0].2);
        if parted {
            return;
        }
        from += TOPIC_SAMPLE;
    }
}
because!(broken_ties, ExplainedLogic, "the order of topics that answer the same share of their samples: each is judged on the next lessons whose questions it asks, a sample more at a time, up to every lesson, until their shares differ; topics still equal keep the order of the library");

const MIN_TIED: usize = 2;
because!(MIN_TIED, ExplainedLogic, "a tie needs a topic and another of the same share, next to each other in the order of their shares, as a pair of the sorted list compares them");

pub fn reading_judgments(lessons: &Lessons, lanes: usize) -> Result<Vec<String>, String> {
    let library = library()?;
    let (plain, _, _) = model_and_stops(Arc::clone(&library), lessons, lanes);
    Ok(reading_table(&library, lessons, plain.as_ref(), lanes))
}
because!(reading_judgments, ExplainedLogic, "the table of the learned ways of a kind as the explain tool judges them when it learns from the given lessons, after the written topics that fit them");

pub fn model_for(library: Arc<Library>, lessons: &Lessons, lanes: usize) -> Option<ExplainModel> {
    model_and_stops(library, lessons, lanes).0
}
because!(model_for, ExplainedLogic, "the model of the tool for a kind from the topics of the library that fit its lessons, as model_and_stops makes it");

const TRIED_SHOWN: usize = 3;
because!(TRIED_SHOWN, ExplainedLogic, "the topics that the fault of a kind that no topic reads names, the nearest first: enough to see which text came nearest, few enough for one line");

const NEAREST_HEAD: &str = "nearest texts, right of read (sample): ";
because!(NEAREST_HEAD, ExplainedLogic, "the words before the topics that came nearest in the fault of a kind that no topic reads, each with the lessons of its sample that it answered right of those that it read before it was dropped, and the size of its sample");

const NONE_RIGHT_HEAD: &str = "the texts that ask its questions answered none right: ";
because!(NONE_RIGHT_HEAD, ExplainedLogic, "the words before the topics whose question forms fit the questions of a kind when none of them answered a lesson of its sample right, each with the size of its sample");

const TRIED_BETWEEN: &str = ", ";
because!(TRIED_BETWEEN, ExplainedLogic, "the marks between the topics that came nearest in the fault of a kind");

const BY_DEFAULT_ONLY: &str = ", by default only";
because!(BY_DEFAULT_ONLY, ExplainedLogic, "the words after a topic that answered most of its sample, and was dropped as every answer that it gave right holds otherwise and reads nothing of the kind");

const NO_TEXT_ASKS: &str = "no text of the library asks its questions";
because!(NO_TEXT_ASKS, ExplainedLogic, "what the fault of a kind says when no question form of any topic fits a question of its lessons");

fn tried_topics(library: &Library, trials: &[TopicTrial]) -> String {
    let mut asked: Vec<(&TopicTrial, &str)> = trials.iter().zip(&library.topics).filter(|(trial, _)| trial.sample > usize::default()).map(|(trial, topic)| (trial, topic.name.as_str())).collect();
    if asked.is_empty() {
        return NO_TEXT_ASKS.to_string();
    }
    asked.sort_by(|one, other| other.0.right.cmp(&one.0.right).then_with(|| one.1.cmp(other.1)));
    let (right, none): (Vec<&(&TopicTrial, &str)>, Vec<&(&TopicTrial, &str)>) = asked.iter().partition(|(trial, _)| trial.right > usize::default());
    if right.is_empty() {
        let shown: Vec<String> = none.iter().take(TRIED_SHOWN).map(|(trial, name)| format!("{name} ({})", trial.sample)).collect();
        let more = none.len().saturating_sub(TRIED_SHOWN);
        return format!("{NONE_RIGHT_HEAD}{}{}", shown.join(TRIED_BETWEEN), if more > usize::default() { format!(" and {more} more") } else { String::new() });
    }
    let shown: Vec<String> = right.iter().take(TRIED_SHOWN).map(|(trial, name)| format!("{name} {} of {} ({}){}", trial.right, trial.read, trial.sample, if trial.by_default { BY_DEFAULT_ONLY } else { "" })).collect();
    let rest = asked.len() - shown.len();
    format!("{NEAREST_HEAD}{}{}", shown.join(TRIED_BETWEEN), if rest > usize::default() { format!(" and {rest} more texts ask its questions") } else { String::new() })
}
because!(tried_topics, ExplainedLogic, "the topics whose question forms fit the questions of a kind that came nearest: those that answered the most lessons right first, each with the lessons that it answered right of those that it read before it was dropped and the size of its sample, and how many more ask its questions; when none answered a lesson right, the first of them by name; or that no text asks its questions");

fn model_and_stops(library: Arc<Library>, lessons: &Lessons, lanes: usize) -> (Option<ExplainModel>, Vec<String>, String) {
    let trials = topic_trials(&library, lessons, lanes);
    let stopped = learning_stops(&library, &trials);
    let mut kept: Vec<(usize, usize, usize)> = trials.iter().enumerate().filter_map(|(place, trial)| trial.share.map(|(right, sample)| (place, right, sample))).collect();
    if kept.is_empty() {
        return (None, stopped, tried_topics(&library, &trials));
    }
    kept.sort_by(|(_, right, sample), (_, other_right, other_sample)| (other_right * sample).cmp(&(right * other_sample)));
    let mut start = 0;
    while start < kept.len() {
        let (_, right, sample) = kept[start];
        let end = start + kept[start..].iter().take_while(|(_, other_right, other_sample)| other_right * sample == right * other_sample).count();
        if end - start >= MIN_TIED {
            broken_ties(&library, lessons, &mut kept[start..end]);
        }
        start = end;
    }
    let kept: Vec<usize> = kept.into_iter().map(|(place, _, _)| place).collect();
    let taught = kept.iter().filter(|place| reads_lessons(&library.topics[**place])).map(|place| (*place, lesson_base(&library.topics[*place], lessons.cases))).collect();
    (Some(ExplainModel { library, kept, taught, stops: AtomicUsize::default(), reading: Reading::Written, frames: None }), stopped, String::new())
}
because!(model_and_stops, ExplainedLogic, "the model of the tool for a kind, with the topics whose chains on its lessons ended at a bound: the topics of the library that fit the lessons, tried on several lanes at once, the one with the largest share of its sample first, topics of equal share judged on more of their lessons, and topics still equal in the order of the library, so that a topic that answers a question with \"otherwise\" cannot answer before a better topic of the same question; with no topic that fits, the topics that came nearest");

pub(super) fn learn_explain(lessons: Lessons, lanes: usize) -> Learned {
    learn_explain_with(library()?, lessons, lanes)
}

because!(learn_explain, ExplainedLogic, "the tool on lessons: the library of the run, read once, and the topics that fit the kind, best first; when no written question form fits the kind, the question forms learned from its lessons for a text of rules");

pub fn learn_explain_with(library: Arc<Library>, lessons: Lessons, lanes: usize) -> Learned {
    let (plain, stopped, tried) = {
        let _texts = cost_scope(Some(WAY_TEXTS), None);
        model_and_stops(Arc::clone(&library), &lessons, lanes)
    };
    let (taken, beyond) = learned_reading(&library, &lessons, plain.as_ref(), lanes);
    if let Some(model) = taken {
        return Ok(model);
    }
    if let Some(model) = plain {
        return Ok(match beyond {
            Some(worded) => Box::new(worded.framed(lessons.cases)),
            None => Box::new(model.framed(lessons.cases)),
        });
    }
    let chosen = spent_way(WAY_WORDING, || worded_model(&library, &lessons)).or_else(|| spent_way(WAY_FACTS, || written_facts_model(&library, &lessons)).map(|(model, _)| model));
    match chosen {
        Some(model) => Ok(Box::new(model.framed(lessons.cases))),
        None => {
            let notes: String = learning_notes().iter().map(|note| format!("{TRIED_NOTE_BETWEEN}{note}")).collect();
            if stopped.is_empty() { Err(format!("{NO_TOPIC}{FAULT_DETAIL}{tried}{notes}")) } else { Err(format!("{NO_TOPIC}{FAULT_DETAIL}{tried}{notes}{STOPPED_TOPICS}{}", stopped.join(STOPPED_BETWEEN))) }
        }
    }
}

because!(learn_explain_with, ExplainedLogic, "the tool on lessons with a given library: the topics that fit the kind, best first; when no written question form fits the kind, the question forms learned from its lessons for a text of rules; a step that judges facts in place of those of a text learns with a library that holds them");
