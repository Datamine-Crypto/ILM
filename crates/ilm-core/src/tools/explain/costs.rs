use patterns::{because, source};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::thread::ThreadId;
use std::time::Instant;

pub struct LearningCosts;
source!(LearningCosts, "where the explain tool spends the learning of a kind: the seconds of each way that it tries (the written texts, the bars of the last lessons, the label list, the ways to reckon, the meanings, the joining, the learned wording with its promises, forms, told side and rules), how often each ran, whether a way was taken, and the chains of rules that the learning made, by way and by text, with how many ended at a bound and their seconds, with notes of what the learning tried; kept for the thread that learns while a measure learns a kind, so that the board writes them beside the job, and shown to the progress of the board while the kind learns; a chain that a borrowed spare thread makes is not counted");

#[derive(Clone, Debug, Default)]
struct WaySpent {
    way: &'static str,
    seconds: f64,
    calls: usize,
    taken: bool,
}
because!(WaySpent, LearningCosts, "the seconds of one way of the learning of a kind, how often it ran, and whether it was taken once");

#[derive(Clone, Debug, Default)]
struct ChainsSpent {
    way: &'static str,
    text: String,
    chains: usize,
    ran_away: usize,
    seconds: f64,
}
because!(ChainsSpent, LearningCosts, "the chains of rules that one way made under one text, how many of them ended at a bound, and their seconds");

#[derive(Debug, Default)]
pub struct LearningNow {
    place: Mutex<(&'static str, String)>,
    chains: AtomicUsize,
}
because!(LearningNow, LearningCosts, "what a learning does now, as another thread can read it: the way and the text that run, and the chains made so far");

#[derive(Debug, Default)]
struct Ledger {
    open: bool,
    way: &'static str,
    text: String,
    ways: Vec<WaySpent>,
    chains: Vec<ChainsSpent>,
    inner: Vec<f64>,
    notes: Vec<String>,
    replies: Vec<(&'static str, usize, f64)>,
    searches: Vec<(&'static str, usize, usize)>,
    constants: usize,
    now: Option<Arc<LearningNow>>,
}
because!(Ledger, LearningCosts, "the costs of the learning that a thread runs: open while a measure learns a kind, the way and the text that run now, what each way and each text spent, for each way that runs now the seconds of the ways that ran inside it, so that a way counts only its own seconds, the notes of what the learning tried, the replies of the engine that each way asked for with their seconds, and what it does now for the progress of the board");

thread_local! {
    static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
}
because!(LEDGER, LearningCosts, "the costs of the learning of this thread");

static LEARNING: OnceLock<Mutex<HashMap<ThreadId, Arc<LearningNow>>>> = OnceLock::new();
because!(LEARNING, LearningCosts, "what each thread that learns a kind does now, by its thread, for the progress of the board");

#[derive(Clone, Debug, Default)]
pub struct LearningSpent {
    pub line: String,
    pub ways: Vec<(&'static str, f64)>,
    pub notes: Vec<String>,
}
because!(LearningSpent, LearningCosts, "the costs of the learning of a kind once it ended: the line of costs, each way with its own seconds, the most first, and the notes of what it tried");

const NO_COST_TEXT: &str = "-";
because!(NO_COST_TEXT, LearningCosts, "the text of chains that a way made under no text of its own");

const FIRST_WAY: &str = "learning";
because!(FIRST_WAY, LearningCosts, "the way of chains made before any way of the learning started");

const CHAINS_SHOWN: usize = 5;
because!(CHAINS_SHOWN, LearningCosts, "the ways and texts whose chains a line of costs names, those of the most seconds first: enough to see where the chains go, few enough for one line");

const COST_PLACES: usize = 2;
because!(COST_PLACES, LearningCosts, "the places after the point of the seconds in a line of costs");

const TAKEN_MARK: &str = " taken";
because!(TAKEN_MARK, LearningCosts, "the word after the seconds of a way that gave the model");

const COST_BETWEEN: &str = ", ";
because!(COST_BETWEEN, LearningCosts, "the marks between the ways and between the chains of a line of costs");

const SEARCHES_HEAD: &str = " | searches: ";
because!(SEARCHES_HEAD, LearningCosts, "the words before the searches for rules and readings of a line of costs, after the replies of the engine");

const CONSTANTS_HEAD: &str = " | rules with a constant left out: ";
because!(CONSTANTS_HEAD, LearningCosts, "the words before the count of the suggested rules that the constants judge left out, in a line of costs");

const NOTES_HEAD: &str = " | notes: ";
because!(NOTES_HEAD, LearningCosts, "the words before the notes of what the learning tried, at the end of a line of costs");

const NOTES_BETWEEN: &str = "; ";
because!(NOTES_BETWEEN, LearningCosts, "the marks between the notes of a line of costs");

pub const WAY_TEXTS: &str = "texts";
because!(WAY_TEXTS, LearningCosts, "the way that judges the written texts of the library on the lessons of a kind");

pub const WAY_BARS: &str = "bars";
because!(WAY_BARS, LearningCosts, "the way that works out the bars of the last lessons that the learned ways must pass");

pub const WAY_LABEL_LIST: &str = "label list";
because!(WAY_LABEL_LIST, LearningCosts, "the way of the list over the words of the told text");

pub const WAY_RECKONING: &str = "reckoning";
because!(WAY_RECKONING, LearningCosts, "the way of the ways to reckon over the numbers of a story");

pub const WAY_MEANINGS: &str = "meanings";
because!(WAY_MEANINGS, LearningCosts, "the way of the meanings of function words over lists");

pub const WAY_JOINING: &str = "joining";
because!(WAY_JOINING, LearningCosts, "the way of the ways of joining of a language of commands");

pub const WAY_REWRITE: &str = "rewrites";
because!(WAY_REWRITE, LearningCosts, "the way of the learned rewrites of sentences into another build (a sentence into the passive or a question)");

pub const WAY_BEYOND: &str = "wording beside texts";
because!(WAY_BEYOND, LearningCosts, "the way of the learned wording tried beside the written texts that read a kind in part");

pub const WAY_WORDING: &str = "wording";
because!(WAY_WORDING, LearningCosts, "the way of the learned wording of a kind that no written text reads");

pub const WAY_PROMISE: &str = "wording promise";
because!(WAY_PROMISE, LearningCosts, "the part of the learned wording that judges whether the rules of a text promise a wording");

pub const WAY_FORMS: &str = "wording forms";
because!(WAY_FORMS, LearningCosts, "the part of the learned wording that learns the forms of a text and judges them on the last lessons");

pub const WAY_TOLD: &str = "told side";
because!(WAY_TOLD, LearningCosts, "the part of the learned wording that learns how the told sentences read with the rules of a text");

pub const WAY_RULES: &str = "told rules";
because!(WAY_RULES, LearningCosts, "the part of the learned wording that learns the rules of a concept from the lessons");

pub const WAY_ALONE: &str = "rules from the lessons alone";
because!(WAY_ALONE, LearningCosts, "the way that learns the rules of a new concept and its wording from the lessons of a kind alone, with no text of the library under them, for a kind that nothing else reads");

pub const WAY_FINAL: &str = "wording on all lessons";

pub const WAY_BRIDGES: &str = "state bridges";

pub const WAY_ANSWERED: &str = "facts beside the wording";
because!(WAY_ANSWERED, LearningCosts, "the way that learns facts of knowledge beside the learned wording of a text for a kind that nothing reads, by the answers of its lessons");

pub const WAY_FACTS: &str = "written facts";

pub const WAY_TAUGHT: &str = "curriculum rules";
because!(WAY_TAUGHT, LearningCosts, "the way that learns again the rules that the kinds before a kind taught its text (the curriculum), apart from the search for the rules of the kind itself, so that a line of costs shows which of the two took the seconds");
because!(WAY_FACTS, LearningCosts, "the way that learns the facts of knowledge that a text with written question forms lacks from the lessons of a kind that no model reads");
because!(WAY_BRIDGES, LearningCosts, "the way that reads the told events of a kind that no rule reads as the events that start or end the same invented state and that a rule reads (bridges of a state)");
because!(WAY_FINAL, LearningCosts, "the part of the learned wording that learns the chosen wording again from all lessons");

fn learning_table() -> &'static Mutex<HashMap<ThreadId, Arc<LearningNow>>> {
    LEARNING.get_or_init(Mutex::default)
}
because!(learning_table, LearningCosts, "the table of what each learning thread does now, made once");

fn show_now(ledger: &Ledger) {
    if let Some(now) = &ledger.now {
        *now.place.lock().unwrap_or_else(PoisonError::into_inner) = (ledger.way, ledger.text.clone());
    }
}
because!(show_now, LearningCosts, "the way and the text that a learning runs now, given to the table that the progress of the board reads");

pub fn open_costs() {
    let now = Arc::new(LearningNow { place: Mutex::new((FIRST_WAY, NO_COST_TEXT.to_string())), chains: AtomicUsize::default() });
    learning_table().lock().unwrap_or_else(PoisonError::into_inner).insert(std::thread::current().id(), Arc::clone(&now));
    LEDGER.with(|ledger| *ledger.borrow_mut() = Ledger { open: true, way: FIRST_WAY, text: NO_COST_TEXT.to_string(), now: Some(now), ..Ledger::default() });
}
because!(open_costs, LearningCosts, "the costs of this thread opened afresh, before a measure learns a kind, and the learning shown to the progress of the board");

pub fn close_costs() -> LearningSpent {
    learning_table().lock().unwrap_or_else(PoisonError::into_inner).remove(&std::thread::current().id());
    LEDGER.with(|ledger| {
        let closed = std::mem::take(&mut *ledger.borrow_mut());
        let mut ways: Vec<(&'static str, f64)> = closed.ways.iter().map(|spent| (spent.way, spent.seconds)).collect();
        ways.sort_by(|one, other| other.1.total_cmp(&one.1));
        if closed.ways.is_empty() && closed.chains.is_empty() {
            return LearningSpent { line: String::new(), ways, notes: closed.notes };
        }
        let shown: Vec<String> = closed
            .ways
            .iter()
            .map(|spent| format!("{} {:.places$} s{}{}", spent.way, spent.seconds, if spent.calls > 1 { format!(" in {} runs", spent.calls) } else { String::new() }, if spent.taken { TAKEN_MARK } else { "" }, places = COST_PLACES))
            .collect();
        let mut chains: Vec<&ChainsSpent> = closed.chains.iter().collect();
        chains.sort_by(|one, other| other.seconds.total_cmp(&one.seconds));
        let made: usize = chains.iter().map(|spent| spent.chains).sum();
        let ran_away: usize = chains.iter().map(|spent| spent.ran_away).sum();
        let seconds: f64 = chains.iter().map(|spent| spent.seconds).sum();
        let most: Vec<String> = chains.iter().take(CHAINS_SHOWN).map(|spent| format!("{} on {}: {} made, {} ran away, {:.places$} s", spent.way, spent.text, spent.chains, spent.ran_away, spent.seconds, places = COST_PLACES)).collect();
        let notes = if closed.notes.is_empty() { String::new() } else { format!("{NOTES_HEAD}{}", closed.notes.join(NOTES_BETWEEN)) };
        let mut replies: Vec<&(&'static str, usize, f64)> = closed.replies.iter().collect();
        replies.sort_by(|one, other| other.2.total_cmp(&one.2));
        let asked: usize = replies.iter().map(|(_, count, _)| count).sum();
        let replied: f64 = replies.iter().map(|(_, _, seconds)| seconds).sum();
        let by_way: Vec<String> = replies.iter().map(|(way, count, seconds)| format!("{way} {count} in {seconds:.places$} s", places = COST_PLACES)).collect();
        let run: usize = closed.searches.iter().map(|(_, run, _)| run).sum();
        let empty: usize = closed.searches.iter().map(|(_, _, empty)| empty).sum();
        let searched = if closed.searches.is_empty() { String::new() } else { format!("{SEARCHES_HEAD}{run} run, {empty} found nothing; by way: {}", closed.searches.iter().map(|(way, run, empty)| format!("{way} {run} run, {empty} found nothing")).collect::<Vec<String>>().join(COST_BETWEEN)) };
        let constants = if closed.constants == usize::default() { String::new() } else { format!("{CONSTANTS_HEAD}{}", closed.constants) };
        let line = format!("ways: {} | chains: {made} made, {ran_away} ran away, {seconds:.places$} s; most: {} | replies: {asked} in {replied:.places$} s; by way: {}{searched}{constants}{notes}", shown.join(COST_BETWEEN), most.join(COST_BETWEEN), by_way.join(COST_BETWEEN), places = COST_PLACES);
        LearningSpent { line, ways, notes: closed.notes }
    })
}
because!(close_costs, LearningCosts, "the costs of the learning of a kind, and the costs of this thread closed: a line with each way, its own seconds, how often it ran and whether it was taken, then the chains made, how many ran away and their seconds, with the ways and texts of the most seconds (nothing when the learning spent nothing that was counted); each way with its seconds, the most first; and the notes of what the learning tried");

pub fn learning_now(thread: ThreadId) -> Option<String> {
    let now = learning_table().lock().unwrap_or_else(PoisonError::into_inner).get(&thread).cloned()?;
    let (way, text) = now.place.lock().unwrap_or_else(PoisonError::into_inner).clone();
    let chains = now.chains.load(Ordering::Relaxed);
    Some(if text == NO_COST_TEXT { format!("{way}, {chains} chains so far") } else { format!("{way} on {text}, {chains} chains so far") })
}
because!(learning_now, LearningCosts, "what the learning of a thread does now, as the progress of the board names it: the way, its text, and the chains made so far; nothing for a thread that does not learn now");

pub fn learning_note(note: String) {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        if ledger.open && !ledger.notes.contains(&note) {
            ledger.notes.push(note);
        }
    });
}
because!(learning_note, LearningCosts, "a note of something that the learning tried, kept once, while the costs of this thread are open");

pub fn learning_notes() -> Vec<String> {
    LEDGER.with(|ledger| ledger.borrow().notes.clone())
}
because!(learning_notes, LearningCosts, "the notes of what the learning of this thread tried so far");

pub struct CostScope {
    before: Option<(&'static str, String)>,
    way: Option<&'static str>,
    started: Instant,
    taken: bool,
}
because!(CostScope, LearningCosts, "a way or a text of the learning that runs until the scope ends: it puts back the way and the text that ran before it, and a way adds its own seconds, without those of the ways that ran inside it, which count for the way that holds it as a whole");

impl CostScope {
    pub fn taken(&mut self) {
        self.taken = true;
    }
}

impl Drop for CostScope {
    fn drop(&mut self) {
        let Some((way, text)) = self.before.take() else { return };
        let seconds = self.started.elapsed().as_secs_f64();
        LEDGER.with(|ledger| {
            let mut ledger = ledger.borrow_mut();
            ledger.way = way;
            ledger.text = text;
            show_now(&ledger);
            let Some(name) = self.way else { return };
            let own = seconds - ledger.inner.pop().unwrap_or_default();
            if let Some(outer) = ledger.inner.last_mut() {
                *outer += seconds;
            }
            match ledger.ways.iter_mut().find(|spent| spent.way == name) {
                Some(spent) => {
                    spent.seconds += own;
                    spent.calls += 1;
                    spent.taken |= self.taken;
                }
                None => ledger.ways.push(WaySpent { way: name, seconds: own, calls: 1, taken: self.taken }),
            }
        });
    }
}

pub fn cost_scope(way: Option<&'static str>, text: Option<&str>) -> CostScope {
    let before = LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        if !ledger.open {
            return None;
        }
        if way.is_some() {
            ledger.inner.push(f64::default());
        }
        let way_before = way.map_or(ledger.way, |name| std::mem::replace(&mut ledger.way, name));
        let text_before = match text {
            Some(text) => std::mem::replace(&mut ledger.text, text.to_string()),
            None => ledger.text.clone(),
        };
        show_now(&ledger);
        Some((way_before, text_before))
    });
    CostScope { before, way, started: Instant::now(), taken: false }
}
because!(cost_scope, LearningCosts, "a scope of a way, a text or both, which counts only while the costs of this thread are open");

pub fn spent_way<T>(way: &'static str, run: impl FnOnce() -> Option<T>) -> Option<T> {
    let mut scope = cost_scope(Some(way), None);
    let out = run();
    if out.is_some() {
        scope.taken();
    }
    out
}
because!(spent_way, LearningCosts, "one way of the learning of a kind, run in its scope and marked as taken when it gives a model");

pub fn reply_ended(started: Option<Instant>) {
    let Some(started) = started else { return };
    let seconds = started.elapsed().as_secs_f64();
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        let Ledger { way, replies, .. } = &mut *ledger;
        match replies.iter_mut().find(|(known, _, _)| *known == *way) {
            Some((_, count, spent)) => {
                *count += 1;
                *spent += seconds;
            }
            None => replies.push((*way, 1, seconds)),
        }
    });
}
because!(reply_ended, LearningCosts, "one reply of the engine counted for the way that runs now, with its seconds: a way that checks many candidates on many lessons asks for many replies, and their seconds show whether the checks are cheap or dear");

pub fn search_ended(found_nothing: bool) {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        if !ledger.open {
            return;
        }
        let Ledger { way, searches, .. } = &mut *ledger;
        match searches.iter_mut().find(|(known, _, _)| *known == *way) {
            Some((_, run, empty)) => {
                *run += 1;
                *empty += usize::from(found_nothing);
            }
            None => searches.push((*way, 1, usize::from(found_nothing))),
        }
    });
}
because!(search_ended, LearningCosts, "one search for the rules of a concept or the readings of told sentences that ran for the way that runs now (a search read back from the store or kept from earlier in the run does not run), and whether it found nothing: a search that finds nothing costs the board and gives nothing, so the line of costs shows how many there were");

pub fn constants_left_out(count: usize) {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        if ledger.open {
            ledger.constants += count;
        }
    });
}
because!(constants_left_out, LearningCosts, "suggested rules that enough lessons suggest and that the constants judge left out (a value of the answer kept as a fixed word, or a place of one value), counted for the learning of the kind, so the line of costs shows the kinds where the old judge would have judged a constant");

pub fn learning_clock() -> Option<Instant> {
    LEDGER.with(|ledger| ledger.borrow().open).then(Instant::now)
}
because!(learning_clock, LearningCosts, "the start of a timed part of the learning, a chain of rules or a reply of the engine, when the costs of this thread are open");

pub fn chain_ended(started: Option<Instant>, ran_away: bool) {
    let Some(started) = started else { return };
    let seconds = started.elapsed().as_secs_f64();
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        if let Some(now) = &ledger.now {
            now.chains.fetch_add(1, Ordering::Relaxed);
        }
        let Ledger { way, text, chains, .. } = &mut *ledger;
        match chains.iter_mut().find(|spent| spent.way == *way && spent.text == *text) {
            Some(spent) => {
                spent.chains += 1;
                spent.ran_away += usize::from(ran_away);
                spent.seconds += seconds;
            }
            None => chains.push(ChainsSpent { way: *way, text: text.clone(), chains: 1, ran_away: usize::from(ran_away), seconds }),
        }
    });
}
because!(chain_ended, LearningCosts, "one chain of rules counted for the way and the text that run now, with whether it ended at a bound and its seconds, and in the count that the progress of the board reads");
