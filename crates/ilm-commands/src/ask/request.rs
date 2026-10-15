use crate::ask::facts::{FactSession, QuestionCase};
use crate::ask::route::{Destination, Feature, RouteKind, Router, count_cases, count_requests};
use ilm_core::base::lanes::{hardware_lanes, on_lanes};
use ilm_core::base::shares::share;
use ilm_core::base::symbols::Symbols;
use ilm_core::know::article::SENTENCE_ENDS;
use ilm_core::know::signs::is_space;
use ilm_core::choice::{Taken, picked, picked_labels, take};
use ilm_core::split::parts;
use ilm_core::cases::Case;
use ilm_core::classes::WordClasses;
use ilm_data::sets::folder::{Loaded, Source, load, read_source};
use ilm_core::tools::call::{Learner, Lessons};
use ilm_core::tools::{all_tools, named_tools};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct OneCommand;
source!(OneCommand, "one command for a request: the kind of the request is counted, and the request goes to the tools that reason over a told story, to the table of facts, or to the reader of requests to a service; every answer comes with a trace of each step, so a person sees how it came");

const SHOWN_KINDS: usize = 3;
because!(SHOWN_KINDS, OneCommand, "the kinds with the highest scores that the trace names: the chosen kind and the two nearest to it");

const SCORE_PLACES: usize = 2;
because!(SCORE_PLACES, OneCommand, "the places after the point of a score in the trace");

const SECONDS_ONE_PLACE: usize = 1;
because!(SECONDS_ONE_PLACE, OneCommand, "the places after the point of the seconds that the learning of a kind took");

const STEP_TOOL: usize = 2;
because!(STEP_TOOL, OneCommand, "the number of the step of the learning of a kind in the trace of a told story");

const STEP_NOTE: usize = 3;
because!(STEP_NOTE, OneCommand, "the number of the step of the tool and its note in the trace of a told story");

const TRACE_STEP_INDENT: &str = "      ";
because!(TRACE_STEP_INDENT, OneCommand, "the space before a line of the trace that says more about the step above it");

const ANSWER_INDENT: &str = "   ";
because!(ANSWER_INDENT, OneCommand, "the space before the line under an answer that names the article and the sentence that the answer stands in");

const FACTS_SOURCE: &str = "facts";
because!(FACTS_SOURCE, OneCommand, "the first part of the key of the kind of the questions about facts in the router; it sorts with the names of the sets");

const FACTS_KIND: &str = "a question about a fact";
because!(FACTS_KIND, OneCommand, "the second part of the key and the task of the kind of the questions about facts");

const REQUESTS_TOOL_NAME: &str = "requests";
because!(REQUESTS_TOOL_NAME, OneCommand, "the name of the tool that reads a request to a service into its facts");

const ROUTE_FILE: &str = "route.txt";
because!(ROUTE_FILE, OneCommand, "the file of the router in the folder of a table");

const COUNTED_FILE: &str = "counted.txt";
because!(COUNTED_FILE, OneCommand, "the file of the kinds of the counted tasks in the folder of a table, whose read cases the measure of the router sends");

const SETS_FILE: &str = "sets.txt";
because!(SETS_FILE, OneCommand, "the file in the folder of a table that names the folder of the sets whose kinds the router counted");

const BOARD_SPLIT: char = '|';
because!(BOARD_SPLIT, OneCommand, "the sign between the columns of a line of the table of a board");

const BOARD_TASK_COLUMN: usize = 5;
because!(BOARD_TASK_COLUMN, OneCommand, "the column of a line of the table of a board that says whether the task is learned and counted");

const COUNTED_MARK: &str = "YES";
because!(COUNTED_MARK, OneCommand, "the word in the column of a board for a task that is learned and counted");

const PERCENT_PLACES: usize = 1;
because!(PERCENT_PLACES, OneCommand, "the places after the point of a share in the measure of the router");

const FINE_PERCENT_PLACES: usize = 2;
because!(FINE_PERCENT_PLACES, OneCommand, "the places after the point of the share of the read cases that stay with the kinds of reasoning: it is near the whole, and a single case must show");

#[derive(Clone, Copy, Debug, Default)]
pub struct Went {
    pub facts: usize,
    pub requests: usize,
    pub stories: usize,
}
because!(Went, OneCommand, "how many requests went to the table of facts, to a reader of requests and to a kind of reasoning");

const COUNTED_FIELD: char = '\t';
because!(COUNTED_FIELD, OneCommand, "the sign between the fields of a line of the file of the counted kinds");

#[derive(Clone, Debug)]
pub struct CountedKind {
    pub source: String,
    pub kind: String,
    pub task: String,
}
because!(CountedKind, OneCommand, "a kind of a task that a board counted: its set, its name and its task");

pub fn sentences_of(request: &str) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut before: Option<char> = None;
    let mut signs = request.trim_matches(is_space).chars().peekable();
    while let Some(sign) = signs.next() {
        if is_space(sign) && before.is_some_and(|last| SENTENCE_ENDS.contains(&last)) {
            pieces.push(std::mem::take(&mut current));
            while signs.next_if(|next| is_space(*next)).is_some() {}
            before = None;
            continue;
        }
        current.push(sign);
        before = Some(sign);
    }
    pieces.push(current);
    pieces.into_iter().map(|piece| piece.trim_matches(is_space).to_string()).filter(|piece| !piece.is_empty()).collect()
}
because!(sentences_of, OneCommand, "the sentences of a request: the text is cut where space follows the end of a sentence, and each piece is taken without the space around it");

fn counted_tasks(board_file: &Path) -> std::io::Result<HashSet<(String, String)>> {
    let mut yes: HashSet<(String, String)> = HashSet::new();
    for line in BufReader::new(std::fs::File::open(board_file)?).lines() {
        let line = line?;
        let columns: Vec<&str> = line.split(BOARD_SPLIT).map(str::trim).collect();
        if columns.len() > BOARD_TASK_COLUMN + 1 && columns[BOARD_TASK_COLUMN] == COUNTED_MARK {
            yes.insert((columns[0].to_string(), columns[1].to_string()));
        }
    }
    Ok(yes)
}
because!(counted_tasks, OneCommand, "the tasks that a board counts, as their set and their name, read from the lines of its table");

fn sets_of(folder: &Path) -> std::io::Result<Vec<Source>> {
    let mut names: Vec<String> = std::fs::read_dir(folder)?.filter_map(Result::ok).filter(|entry| entry.path().is_dir()).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    Ok(names.iter().filter_map(|name| read_source(&folder.join(name)).ok()).collect())
}
because!(sets_of, OneCommand, "the sets of a folder in the order of their names; a folder that says nothing of a set is passed over");

fn learned_part(loaded: &Loaded, fit: usize, held: usize) -> (Vec<Case>, Vec<Vec<String>>) {
    let cut = parts(&loaded.kind.cases, fit, held, loaded.start, loaded.stories);
    (picked(&loaded.kind.cases, &cut.learn), picked_labels(&loaded.kind.labels, &cut.learn))
}
because!(learned_part, OneCommand, "the lessons of a kind with their label lines, cut as the measure cuts them");

fn read_part(loaded: &Loaded, fit: usize, held: usize) -> Vec<Case> {
    let cut = parts(&loaded.kind.cases, fit, held, loaded.start, loaded.stories);
    picked(&loaded.kind.cases, &cut.read)
}
because!(read_part, OneCommand, "the read cases of a kind, cut as the measure cuts them");

type KindCounts = (Symbols, HashMap<Feature, u32>, usize);
because!(KindCounts, OneCommand, "the counts of one kind of the router with its words numbered in a table of the kind alone: the table, the counts, and the number of lessons or different requests that were counted");

pub fn build_router(sets: &Path, board_file: &Path, questions: &[QuestionCase], folder: &Path) -> std::io::Result<(usize, usize)> {
    let yes = counted_tasks(board_file)?;
    let sources = sets_of(sets)?;
    let mut stories: Vec<(&Source, &String, &String)> = Vec::new();
    for from in &sources {
        for (task, task_kinds) in &from.tasks {
            if yes.contains(&(from.set.clone(), task.clone())) {
                stories.extend(task_kinds.iter().map(|kind| (from, task, kind)));
            }
        }
    }
    let requests: Vec<(&Source, &String, &String)> = sources.iter().filter_map(|from| from.requests.as_ref().map(|(slots, intent)| (from, slots, intent))).collect();
    let asked: Vec<Case> = questions.iter().filter(|case| !case.held).map(|case| Case { sentences: Vec::new(), question: case.question.clone(), answer: case.answer.clone() }).collect();
    let facts_place = stories.len();
    let made: Vec<std::io::Result<KindCounts>> = on_lanes(facts_place + 1 + requests.len(), hardware_lanes(), &|place| {
        let mut table = Symbols::new();
        if place == facts_place {
            let counts = count_cases(&mut table, &asked, true);
            return Ok((table, counts, asked.len()));
        }
        let (from, kind, request) = if place < facts_place { (stories[place].0, stories[place].2, false) } else { (requests[place - facts_place - 1].0, requests[place - facts_place - 1].1, true) };
        let loaded = load(from, kind)?;
        let (lessons, _) = learned_part(&loaded, from.fit, from.held);
        if request {
            let (counts, texts) = count_requests(&mut table, &lessons);
            return Ok((table, counts, texts));
        }
        let counts = count_cases(&mut table, &lessons, false);
        Ok((table, counts, lessons.len()))
    });
    let mut words = Symbols::new();
    let mut counted_kinds: Vec<(HashMap<Feature, u32>, usize)> = Vec::with_capacity(made.len());
    for one in made {
        let (table, counts, size) = one?;
        let numbers: Vec<u32> = (0..table.len() as u32).map(|place| words.number(table.name(place))).collect();
        counted_kinds.push((counts.into_iter().map(|(feature, count)| (feature.renumbered(&numbers), count)).collect(), size));
    }
    let mut counted_kinds = counted_kinds.into_iter();
    let mut kinds: Vec<RouteKind> = Vec::new();
    let mut counted: Vec<CountedKind> = Vec::new();
    for ((from, task, kind), (counts, lessons)) in stories.iter().zip(counted_kinds.by_ref()) {
        let label = format!("{}: {task}", from.set);
        counted.push(CountedKind { source: from.name.clone(), kind: kind.to_string(), task: label.clone() });
        kinds.push(RouteKind { source: from.name.clone(), kind: kind.to_string(), task: label, fit: from.fit, held: from.held, lessons, way: Destination::Story, counts });
    }
    if let Some((counts, lessons)) = counted_kinds.next() {
        kinds.push(RouteKind { source: FACTS_SOURCE.to_string(), kind: FACTS_KIND.to_string(), task: FACTS_KIND.to_string(), fit: 0, held: 0, lessons, way: Destination::Fact, counts });
    }
    for ((from, slots, intent), (counts, texts)) in requests.iter().zip(counted_kinds) {
        kinds.retain(|kind| !(kind.source == from.name && kind.kind == **slots));
        kinds.push(RouteKind { source: from.name.clone(), kind: slots.to_string(), task: format!("{}: a request read into its facts", from.set), fit: from.fit, held: from.held, lessons: texts, way: Destination::Request(intent.to_string()), counts });
    }
    let router = Router::new(words, kinds);
    std::fs::create_dir_all(folder)?;
    router.write(&folder.join(ROUTE_FILE))?;
    let mut out = BufWriter::new(std::fs::File::create(folder.join(COUNTED_FILE))?);
    for kind in &counted {
        writeln!(out, "{}{COUNTED_FIELD}{}{COUNTED_FIELD}{}", kind.source, kind.kind, kind.task)?;
    }
    out.flush()?;
    std::fs::write(folder.join(SETS_FILE), sets.to_string_lossy().as_bytes())?;
    Ok((router.kinds.len(), counted.len()))
}
because!(build_router, OneCommand, "the router of the command and the list of the counted kinds: every kind of a task that the board counts, with the lessons of the kind; the kind of the questions about facts, with the lesson questions; and for each set of requests the kind of its slots, with its different lesson requests read as questions, in place of the same kind as a story. The kinds are loaded and counted on all threads, each with a table of words of its own, and the words get their numbers of the whole in the order of the kinds, which is the order of one pass; the first kind that does not load ends the building, as in one pass");

pub struct SessionStory {
    pub taken: Taken,
    pub lessons: usize,
    pub seconds: f64,
}
because!(SessionStory, OneCommand, "the model that the tools learned for a kind of story at the first request of that kind, with its lessons and the seconds of its learning");

pub struct SessionReader {
    pub model: Box<dyn Learner>,
    pub requests: usize,
    pub seconds: f64,
}
because!(SessionReader, OneCommand, "the reader of requests that was learned for a set at the first request to it, with its different lesson requests and the seconds of its learning");

pub struct Whole {
    pub router: Router,
    pub facts: FactSession,
    pub sets: PathBuf,
    pub counted: Vec<CountedKind>,
    pub classes: Option<WordClasses>,
    pub lanes: usize,
    stories: Mutex<HashMap<usize, Arc<SessionStory>>>,
    requests: Mutex<HashMap<usize, Arc<SessionReader>>>,
}
because!(Whole, OneCommand, "the parts of the command: the router, the session of the facts, the folder of the sets whose kinds a story goes to, the counted kinds, the table of word classes for the tools, the threads, and the models of the kinds that the session learned, kept for its later requests because they do not change. open reads them from the folder of a table, the router on a thread of its own beside the facts; answer gives the answer of one request with its trace; kind gives the kind that a request is sent to");

fn kinds_line(router: &Router, order: &[(f64, usize)], from: usize, most: usize) -> String {
    order.iter().skip(from).take(most).map(|(score, place)| format!("{} {:.places$}", router.kinds[*place].kind, score, places = SCORE_PLACES)).collect::<Vec<String>>().join("; ")
}
because!(kinds_line, OneCommand, "the names and scores of some kinds of the order of a request, for the trace");

impl Whole {
    pub fn open(folder: &Path, classes: Option<WordClasses>, lanes: usize) -> std::io::Result<Whole> {
        let (router, facts) = std::thread::scope(|scope| {
            let router = scope.spawn(|| Router::read(&folder.join(ROUTE_FILE)));
            let facts = FactSession::open(folder, lanes);
            (router.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault)), facts)
        });
        let (router, facts) = (router?, facts?);
        let sets = PathBuf::from(std::fs::read_to_string(folder.join(SETS_FILE))?.trim_matches(is_space));
        let mut counted: Vec<CountedKind> = Vec::new();
        for line in BufReader::new(std::fs::File::open(folder.join(COUNTED_FILE))?).lines() {
            let line = line?;
            let fields: Vec<&str> = line.split(COUNTED_FIELD).collect();
            if let [source, kind, task] = fields.as_slice() {
                counted.push(CountedKind { source: source.to_string(), kind: kind.to_string(), task: task.to_string() });
            }
        }
        Ok(Whole { router, facts, sets, counted, classes, lanes, stories: Mutex::new(HashMap::new()), requests: Mutex::new(HashMap::new()) })
    }

    pub fn kind(&self, sentences: &[String], question: &str) -> usize {
        self.router.route(sentences, question)[0].1
    }

    fn source_of(&self, name: &str) -> std::io::Result<Source> {
        read_source(&self.sets.join(name))
    }

    fn story(&self, place: usize, order: &[(f64, usize)], told: &[String], question: &str) -> std::io::Result<(Option<String>, Vec<String>)> {
        let kind = &self.router.kinds[place];
        let shown: Vec<String> = order.iter().take(SHOWN_KINDS).map(|(score, at)| format!("{} ({}) {:.places$}", self.router.kinds[*at].kind, self.router.kinds[*at].task, score, places = SCORE_PLACES)).collect();
        let mut trace = vec![format!("{}. the kind of the request, counted from the lessons of {} kinds: {}", 1, self.router.kinds.len(), shown.join("; "))];
        let why = self.router.why(place, told, question);
        if !why.is_empty() {
            trace.push(format!("{TRACE_STEP_INDENT}pairs of words that this kind holds: {why}"));
        }
        let kept = self.stories.lock().ok().and_then(|stories| stories.get(&place).cloned());
        let learned = match kept {
            Some(learned) => {
                trace.push(format!("{STEP_TOOL}. learned before in this session: {} lessons of this kind in {:.places$} s", learned.lessons, learned.seconds, places = SECONDS_ONE_PLACE));
                learned
            }
            None => {
                let from = self.source_of(&kind.source)?;
                let loaded = load(&from, &kind.kind)?;
                let started = Instant::now();
                let (lessons, labels) = learned_part(&loaded, kind.fit, kind.held);
                let taken = take(&all_tools(), Lessons { cases: &lessons, labels: &labels, ask: None, classes: self.classes.as_ref() }, self.lanes, false);
                let seconds = started.elapsed().as_secs_f64();
                let taken = match taken {
                    Ok(taken) => taken,
                    Err(why) => {
                        trace.push(format!("{STEP_TOOL}. no tool learns this kind from its {} lessons: {why}", lessons.len()));
                        return Ok((None, trace));
                    }
                };
                trace.push(format!("{STEP_TOOL}. learned now: {} lessons of this kind in {:.places$} s", lessons.len(), seconds, places = SECONDS_ONE_PLACE));
                let learned = Arc::new(SessionStory { taken, lessons: lessons.len(), seconds });
                if let Ok(mut stories) = self.stories.lock() {
                    stories.insert(place, Arc::clone(&learned));
                }
                learned
            }
        };
        let said = learned.taken.model.answer(told, question);
        trace.push(format!("{STEP_NOTE}. the tool that the entry point took for this kind: {}; its note: {}", learned.taken.named, said.note));
        Ok(((!said.words.is_empty()).then(|| said.words.join(" ")), trace))
    }

    fn request(&self, place: usize, order: &[(f64, usize)], intent: &str, told: &[String], question: &str) -> std::io::Result<(Option<String>, Vec<String>)> {
        let kind = &self.router.kinds[place];
        let mut trace = vec![format!("{}. the kind of the request, counted from the lessons: {} (next: {})", 0, kind.task, kinds_line(&self.router, order, 1, SHOWN_KINDS - 1))];
        let Ok(tools) = named_tools(REQUESTS_TOOL_NAME) else {
            trace.push(format!("{}. the reader of requests is not a tool of this program yet", 1));
            return Ok((None, trace));
        };
        let kept = self.requests.lock().ok().and_then(|requests| requests.get(&place).cloned());
        let (reader, when) = match kept {
            Some(reader) => (reader, "made before in this session"),
            None => {
                let from = self.source_of(&kind.source)?;
                let started = Instant::now();
                let mut lessons: Vec<Case> = Vec::new();
                let mut labels: Vec<Vec<String>> = Vec::new();
                for name in [kind.kind.as_str(), intent] {
                    let loaded = load(&from, name)?;
                    let (cases, lines) = learned_part(&loaded, from.fit, from.held);
                    labels.extend(if lines.is_empty() { vec![Vec::new(); cases.len()] } else { lines });
                    lessons.extend(cases);
                }
                let distinct: HashSet<&[String]> = lessons.iter().map(|case| case.sentences.as_slice()).collect();
                let learned = (tools[0].learn)(Lessons { cases: &lessons, labels: &labels, ask: None, classes: self.classes.as_ref() }, self.lanes);
                let seconds = started.elapsed().as_secs_f64();
                let model = match learned {
                    Ok(model) => model,
                    Err(why) => {
                        trace.push(format!("{}. the reader of requests does not read this set: {why}", 1));
                        return Ok((None, trace));
                    }
                };
                let reader = Arc::new(SessionReader { model, requests: distinct.len(), seconds });
                if let Ok(mut requests) = self.requests.lock() {
                    requests.insert(place, Arc::clone(&reader));
                }
                (reader, "made")
            }
        };
        let model = &reader.model;
        let text = told.iter().map(String::as_str).chain(std::iter::once(question)).collect::<Vec<&str>>().join(" ");
        trace.push(format!("{}. the reader of requests of this set, {when} from {} lesson requests in {:.places$} s; how each word was read:", 1, reader.requests, reader.seconds, places = SECONDS_ONE_PLACE));
        trace.extend(model.explain(&text).into_iter().map(|line| format!("{TRACE_STEP_INDENT}{line}")));
        let facts = model.facts(&text).unwrap_or_default();
        let said = facts.iter().map(|(slot, value)| format!("{slot}: {value}")).collect::<Vec<String>>().join("; ");
        Ok(((!facts.is_empty()).then_some(said), trace))
    }

    fn fact(&self, order: &[(f64, usize)], question: &str) -> std::io::Result<(Option<String>, Vec<String>)> {
        let mut trace = vec![format!("{}. the kind of the request, counted from the lessons: {} (next: {})", 0, FACTS_KIND, kinds_line(&self.router, order, 1, SHOWN_KINDS - 1))];
        let got = self.facts.answer(question)?;
        trace.extend(got.trace);
        Ok((got.row.map(|row| format!("{}\n{ANSWER_INDENT}in the article \"{}\": {}", row.word, row.title, row.sentence)), trace))
    }

    pub fn answer(&self, request: &str) -> std::io::Result<(Option<String>, Vec<String>)> {
        let sentences = sentences_of(request);
        let (question, told) = match sentences.split_last() {
            Some((last, before)) => (last.clone(), before.to_vec()),
            None => (String::new(), Vec::new()),
        };
        let order = self.router.route(&told, &question);
        let place = order[0].1;
        match &self.router.kinds[place].way {
            Destination::Request(intent) => self.request(place, &order, intent, &told, &question),
            Destination::Fact => self.fact(&order, &question),
            Destination::Story => self.story(place, &order, &told, &question),
        }
    }

    fn went(&self, place: usize, went: &mut Went) {
        match self.router.kinds[place].way {
            Destination::Fact => went.facts += 1,
            Destination::Request(_) => went.requests += 1,
            Destination::Story => went.stories += 1,
        }
    }

    pub fn routes(&self, cases: &[QuestionCase]) -> std::io::Result<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();
        let held: Vec<&QuestionCase> = cases.iter().filter(|case| case.held).collect();
        let mut went = Went::default();
        for case in &held {
            self.went(self.kind(&[], &case.question), &mut went);
        }
        lines.push(format!("held questions about facts: {}; to the table of facts {} ({:.places$}%), to a reader of requests {}, to a kind of reasoning {}", held.len(), went.facts, share(went.facts, held.len().max(1)), went.requests, went.stories, places = PERCENT_PLACES));
        let mut went = Went::default();
        let mut total = 0usize;
        let mut sources: HashMap<String, Source> = HashMap::new();
        for counted in &self.counted {
            if !sources.contains_key(&counted.source) {
                sources.insert(counted.source.clone(), self.source_of(&counted.source)?);
            }
            let from = &sources[&counted.source];
            let read = read_part(&load(from, &counted.kind)?, from.fit, from.held);
            total += read.len();
            for case in &read {
                self.went(self.kind(&case.sentences, &case.question), &mut went);
            }
        }
        lines.push(format!("read cases of the kinds of reasoning: {total}; to a kind of reasoning {} ({:.places$}%), to the table of facts {}, to a reader of requests {}", went.stories, share(went.stories, total.max(1)), went.facts, went.requests, places = FINE_PERCENT_PLACES));
        for (place, kind) in self.router.kinds.iter().enumerate() {
            if !matches!(kind.way, Destination::Request(_)) {
                continue;
            }
            let from = self.source_of(&kind.source)?;
            let read = read_part(&load(&from, &kind.kind)?, from.fit, from.held);
            let mut seen: HashSet<&str> = HashSet::new();
            let texts: Vec<&str> = read.iter().flat_map(|case| case.sentences.iter()).map(String::as_str).filter(|text| seen.insert(text)).collect();
            let (mut own, mut other) = (0usize, 0usize);
            let mut went = Went::default();
            for text in &texts {
                let to = self.kind(&[], text);
                if to == place {
                    own += 1;
                } else if matches!(self.router.kinds[to].way, Destination::Request(_)) {
                    other += 1;
                } else {
                    self.went(to, &mut went);
                }
            }
            lines.push(format!("held requests of {}: {}; to their reader {own} ({:.places$}%), to the other reader of requests {other}, to the table of facts {}, to a kind of reasoning {}", from.set, texts.len(), share(own, texts.len().max(1)), went.facts, went.stories, places = PERCENT_PLACES));
        }
        Ok(lines)
    }
}
