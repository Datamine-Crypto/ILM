use ilm_core::cases::Case;
use ilm_core::choice::picked;
use ilm_core::base::exact::steps_beyond_range;
use ilm_core::base::lanes::on_lanes_then;
use ilm_core::base::process::process_use;
use ilm_core::tools::explain::costs::learning_now;
use ilm_core::tools::call::{FAULT_DETAIL, Reading, Tool};
use crate::measure::kind::{ABOVE, Asked, DEFAULT_RISE_POINTS, Ground, LEARNED, MarkCounts, READ_UNLIKE_POINTS, Row, SHARE_PLACES, measure, share_that_counts};
use ilm_core::classes::WordClasses;
use ilm_data::sets::folder::{Loaded, Source, final_file, kind_weight, load, load_pair, new_words};
use ilm_core::split::{SEALED_AFTER, parts};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Mutex, PoisonError};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

pub struct TaskBoard;
source!(TaskBoard, "the board of the tasks of the public sets: every kind of every set through the choice among all tools, and for each task of a set whether all its kinds are learned, also with new words, also from few lessons, and from a small form to a large one; a task counts once with all its sizes and depths, as the authors of its set made it, so that a set with many sizes does not count many times");

const BAR_PEOPLE: f64 = 90.0;
because!(BAR_PEOPLE, TaskBoard, "the bar of a kind whose text people wrote, in hundredths of the read cases: people write one thing in many ways and with faults, so the bar is lower than for text that a program wrote");

const FEW: usize = 200;
because!(FEW, TaskBoard, "the lessons of the pass that asks whether a learned kind is learned from few lessons: a third of the usual lessons");

const NO_CASES: &str = "the cases of the kind could not be read";
because!(NO_CASES, TaskBoard, "the note of a kind whose case file is not there or cannot be read");

struct Job {
    from: usize,
    kind: String,
    pair: Option<(String, String)>,
    fit: usize,
    new_words: bool,
}
because!(Job, TaskBoard, "one run of the measure for the board: the set, the kind (or the task of a pair with its small and its large kind), the most lessons, and whether the read cases are read again with new words");

fn cross_key(case: &Case) -> u64 {
    let mut hasher = DefaultHasher::new();
    if case.sentences.is_empty() {
        (false, &case.question).hash(&mut hasher);
    } else {
        (true, &case.sentences).hash(&mut hasher);
    }
    hasher.finish()
}
because!(cross_key, TaskBoard, "the story of a case for a comparison between kinds, as one number: its told sentences, or its question when it has none");

fn touched(loaded: &Loaded, fit: usize, held: usize) -> HashSet<u64> {
    let cut = parts(&loaded.kind.cases, fit, held.max(SEALED_AFTER), loaded.start, loaded.stories);
    cut.learn.iter().chain(cut.read.iter()).map(|place| cross_key(&loaded.kind.cases[*place])).collect()
}
because!(touched, TaskBoard, "the stories of a kind that a run of development may have learned from or read");

pub fn final_part(from: &Source, kind: &str, fit: usize, held: usize) -> Vec<Case> {
    if let Some(given) = final_file(from, kind) {
        return given;
    }
    let Ok(loaded) = load(from, kind) else { return Vec::new() };
    let cut = parts(&loaded.kind.cases, fit, held, loaded.start, loaded.stories);
    if cut.sealed.is_empty() {
        return Vec::new();
    }
    let mut seen: HashSet<u64> = HashSet::new();
    for other in from.all.iter().filter(|other| other.as_str() != kind) {
        if let Ok(theirs) = load(from, other) {
            seen.extend(touched(&theirs, fit, held));
        }
    }
    picked(&loaded.kind.cases, &cut.sealed).into_iter().filter(|case| !seen.contains(&cross_key(case))).collect()
}
because!(final_part, TaskBoard, "the cases of a kind for the one final run: the file that the set keeps for it when there is one; else the sealed part of the cut without every case whose story another kind of the set has learned from or read, because kinds of one set can share stories and a run of one kind must not have read the story of a final case of another");

pub fn read_stories(cases: &[&Case]) -> HashSet<u64> {
    cases.iter().map(|case| cross_key(case)).collect()
}
because!(read_stories, TaskBoard, "the stories of some cases that a kind reads, as numbers, which the lessons that it borrows must not hold");

pub fn kept_apart(cases: Vec<Case>, read: &HashSet<u64>) -> Vec<Case> {
    cases.into_iter().filter(|case| !read.contains(&cross_key(case))).collect()
}
because!(kept_apart, TaskBoard, "the lessons of another kind without each case whose story the kind that borrows them reads, so that no read case of a kind reaches its lessons through the kinds before it, as kinds of one set can share stories");

pub fn earlier_lessons(from: &Source, kind: &str, fit: usize, held: usize) -> Vec<Vec<Case>> {
    let Ok(own) = load(from, kind) else { return Vec::new() };
    let cut = parts(&own.kind.cases, fit, held.max(SEALED_AFTER), own.start, own.stories);
    let read = read_stories(&cut.read.iter().chain(cut.sealed.iter()).map(|place| &own.kind.cases[*place]).collect::<Vec<&Case>>());
    from.kinds
        .iter()
        .take_while(|other| other.as_str() != kind)
        .map(|other| {
            let Ok(loaded) = load(from, other) else { return Vec::new() };
            let cut = parts(&loaded.kind.cases, fit, held, loaded.start, loaded.stories);
            kept_apart(picked(&loaded.kind.cases, &cut.learn), &read)
        })
        .collect()
}
because!(earlier_lessons, TaskBoard, "the lessons of the kinds that come before a kind in the order of its set, each cut as its own run of the measure cuts it, without the cases whose story the kind reads in its read or sealed part, so that a learner may learn a concept step by step over the kinds of a set from their lessons, never from a read case; none for a kind that the set does not list; a kind before it that does not load gives no lesson and keeps its place, so that a model names the kinds before it by their places in the order of the set");

pub fn earlier_kinds_named(kinds: &[String], kind: &str, places: &[usize]) -> Vec<String> {
    let before: Vec<&String> = kinds.iter().take_while(|other| other.as_str() != kind).collect();
    places.iter().filter_map(|place| before.get(*place).map(|name| (*name).clone())).collect()
}
because!(earlier_kinds_named, TaskBoard, "the names of the kinds that a model of a kind names by their places among the kinds before it in the order of its set, as the board gives them in the lessons of a task: the kinds whose lessons gave lines of the model");

fn job_name(job: &Job, sources: &[Source]) -> String {
    match &job.pair {
        Some((small, large)) => format!("{}:{small}>{large}", sources[job.from].name),
        None => job.kind.clone(),
    }
}
because!(job_name, TaskBoard, "the name of a job as its row and the file of the times of the jobs name it: the kind, or the set with the small and the large kind of a pair");

fn run(job: &Job, sources: &[Source], tools: &[Tool], asked: &Asked, (classes, few): (Option<&WordClasses>, bool)) -> Row {
    let from = &sources[job.from];
    let name = job_name(job, sources);
    let (loaded, last) = match &job.pair {
        Some((small, large)) => (load_pair(from, small, large), large.as_str()),
        None => (load(from, &job.kind), job.kind.as_str()),
    };
    let Ok(loaded) = loaded else { return Row { kind: name, note: NO_CASES.to_string(), ..Row::default() } };
    let words = if job.new_words { new_words(from, &job.kind) } else { None };
    let asked = Asked { fit: job.fit, held: from.held, new_words: job.new_words, control_fit: (job.fit != from.fit).then_some(from.fit), ..*asked };
    let last_part = if asked.sealed { Some(final_part(from, last, asked.fit, asked.held)) } else { None };
    let earlier = || earlier_lessons(from, &job.kind, asked.fit, asked.held);
    measure(&Ground { name: &name, cases: &loaded.kind.cases, labels: &loaded.kind.labels, start: loaded.start, stories: loaded.stories, final_part: last_part.as_deref(), sealed: None, new_words: words.as_deref(), answerer: None, classes, before: if job.pair.is_some() || few { None } else { Some(&earlier) }, stored: job.pair.is_none() && !few }, tools, &asked)
}
because!(run, TaskBoard, "one job of the board through the measure, with the lessons of the kinds before its kind in its set, but not in the run with few lessons, so that a kind that is learned from few lessons is learned from those alone and no claim grows by borrowing; a pair is the cases of its small kind and then of its large kind, and its final part is the one of its large kind; when a job learns from another number of lessons than the set gives (the run with few lessons, or every job of a run that names its number of lessons), the controls are still learned from the lessons of the set, so that a control weakened by few lessons cannot make a kind look learned");

fn job_weight(job: &Job, sources: &[Source]) -> u64 {
    let from = &sources[job.from];
    match &job.pair {
        Some((small, large)) => kind_weight(from, small, job.fit) + kind_weight(from, large, job.fit),
        None => kind_weight(from, &job.kind, job.fit),
    }
}
because!(job_weight, TaskBoard, "the guessed work of a job: of its kind, or of both kinds of a pair");

fn few_job(job: &Job) -> Job {
    Job { from: job.from, kind: job.kind.clone(), pair: None, fit: FEW, new_words: false }
}
because!(few_job, TaskBoard, "the job of the pass with few lessons for a kind of the first pass");

fn kind_passed(job: &Job, row: &Row, sources: &[Source]) -> bool {
    let from = &sources[job.from];
    passed(row, bar_of(from), !from.bar_only.contains(&job.kind))
}
because!(kind_passed, TaskBoard, "whether a kind of the first pass is learned, with the bar of its set and with the controls unless its set marks the kind as one where no model can be above them");

fn few_counts(job: &Job, sources: &[Source]) -> bool {
    let from = &sources[job.from];
    from.tasks.iter().any(|(task, kinds)| kinds.contains(&job.kind) && !from.not_counted.iter().any(|(name, _)| name == task))
}
because!(few_counts, TaskBoard, "whether the run with few lessons of a kind can change the board: only a task that is counted can be learned from few lessons, so a kind whose tasks the set all marks as not counted needs no such run");

pub struct JobTime {
    pub kind: String,
    pub few: bool,
    pub guess: u64,
    pub start: f64,
    pub end: f64,
    pub cpu: Option<f64>,
    pub tool: String,
    pub marks: MarkCounts,
    pub costs: String,
    pub seen: Vec<String>,
}
because!(JobTime, TaskBoard, "when one job of the board ran: its kind, whether it is the run with few lessons, the guess of its work that set its place in the queue, its start and end in seconds from the start of the board, its CPU seconds when the board runs on one thread, the tool that was taken, the answer marks of its read part (counts), where its learning spent its time, and what the progress lines saw it learn while it ran; the jobs that end last show what holds the whole run");

struct Passes {
    rows: Vec<Row>,
    few: Vec<Option<Row>>,
    first_seconds: f64,
    timed: Vec<JobTime>,
}
because!(Passes, TaskBoard, "the rows of the jobs of the first pass and of the pairs in the order of the jobs, the row with few lessons of each learned kind of a counted task at the place of its job, and the seconds until the last job of the first pass ended, and when each job ran");

const FIRST_PASS: &str = "first";
because!(FIRST_PASS, TaskBoard, "the word of the pass of a job of the first pass or of a pair in the file of the times of the jobs");

pub fn known_times(text: &str) -> HashMap<String, f64> {
    let mut known: HashMap<String, f64> = HashMap::new();
    for line in text.lines().skip(1) {
        let cells: Vec<&str> = line.split(JOB_BETWEEN).collect();
        if let [kind, _, _, _, _, seconds, ..] = cells.as_slice() {
            if let Ok(seconds) = seconds.trim().parse::<f64>() {
                *known.entry(kind.to_string()).or_default() += seconds;
            }
        }
    }
    known
}
because!(known_times, TaskBoard, "the seconds of each job in a file of the times of the jobs that an earlier board wrote, by the name of the job, with the seconds of its run with few lessons added: that run is asked when the job ends, so a job with a long run with few lessons must start early too");

fn measured_weights(jobs: &[Job], sources: &[Source], known: &HashMap<String, f64>) -> Vec<u64> {
    let guesses: Vec<u64> = jobs.iter().map(|job| job_weight(job, sources)).collect();
    let seconds: Vec<Option<f64>> = jobs.iter().map(|job| known.get(&job_name(job, sources)).copied()).collect();
    let (guessed, timed) = guesses.iter().zip(&seconds).filter_map(|(guess, seconds)| seconds.map(|seconds| (*guess as f64, seconds))).fold((f64::default(), f64::default()), |sum, one| (sum.0 + one.0, sum.1 + one.1));
    if timed <= f64::default() {
        return guesses;
    }
    guesses.iter().zip(&seconds).map(|(guess, seconds)| seconds.map_or(*guess, |seconds| (seconds * guessed / timed) as u64)).collect()
}
because!(measured_weights, TaskBoard, "the work of each job for its place in the queue: the seconds that an earlier board measured for it, in the units of the guesses, so that the jobs that the file does not hold keep their guess beside them; with no times, the guesses alone. A guess by bytes can be far from the work of a tool that is slow on few bytes, and a long job that starts late holds the whole run; the order of the queue changes no result");

const PROGRESS_SECONDS: u64 = 10;
because!(PROGRESS_SECONDS, TaskBoard, "the seconds between two progress lines of the board on the error output, so that a person sees which jobs hold the run while it runs");

const PROGRESS_SHOWN: usize = 5;
because!(PROGRESS_SHOWN, TaskBoard, "the running jobs that a progress line names, the longest running first: the jobs that hold the run");

const FEW_MARK: &str = " (few lessons)";
because!(FEW_MARK, TaskBoard, "the words after the name of a running job that is the run with few lessons of its kind, in a progress line");

const STORE_LESSONS: &str = ", the store ";
because!(STORE_LESSONS, TaskBoard, "the words in the lessons of a task before the kinds whose stored lines its learning took, so that 'text, answer' alone keeps meaning the lessons of the task's own kinds");

const BETWEEN_EARLIER: &str = " ";
because!(BETWEEN_EARLIER, TaskBoard, "the mark between the names of the kinds before a kind whose lessons its model learned over, in the lessons of a task");

fn progress_line(seconds: f64, ended: usize, running: &[(usize, String, f64, ThreadId)]) -> String {
    let mut longest: Vec<(&str, f64, ThreadId)> = running.iter().map(|(_, name, start, thread)| (name.as_str(), seconds - start, *thread)).collect();
    longest.sort_by(|one, other| other.1.total_cmp(&one.1));
    let shown: Vec<String> = longest.iter().take(PROGRESS_SHOWN).map(|(name, so_far, thread)| format!("{name} {so_far:.none$} s{}", learning_now(*thread).map_or(String::new(), |now| format!(" ({now})")), none = 0)).collect();
    let more = longest.len().saturating_sub(PROGRESS_SHOWN);
    let rest = if more > usize::default() { format!(" and {more} more") } else { String::new() };
    format!("board at {seconds:.none$} s: {ended} runs ended, {} running: {}{rest}", longest.len(), shown.join(", "), none = 0)
}
because!(progress_line, TaskBoard, "a progress line of the board: its seconds so far, the runs that ended, and the running jobs with their seconds so far, the longest first, each that learns now with the way and the text of its learning and the chains made so far");

fn run_all(jobs: &[Job], first_count: usize, sources: &[Source], tools: &[Tool], asked: &Asked, classes: Option<&WordClasses>, known: &HashMap<String, f64>) -> Passes {
    let started = Instant::now();
    let alone = Asked { lanes: 1, ..*asked };
    let weights: Vec<u64> = measured_weights(jobs, sources, known);
    let mut order: Vec<usize> = (0..jobs.len()).collect();
    order.sort_by_key(|at| std::cmp::Reverse(weights[*at]));
    let last_end = AtomicU64::new(0);
    let timed: Mutex<Vec<JobTime>> = Mutex::new(Vec::new());
    let running: Mutex<Vec<(usize, String, f64, ThreadId)>> = Mutex::new(Vec::new());
    let seen: Mutex<HashMap<usize, Vec<String>>> = Mutex::new(HashMap::new());
    let (ended, marks) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let timed_run = |at: usize, few: bool| {
        let few_one = few.then(|| few_job(&jobs[order[at]]));
        let job = few_one.as_ref().unwrap_or(&jobs[order[at]]);
        let start = started.elapsed().as_secs_f64();
        let mark = marks.fetch_add(1, Ordering::Relaxed);
        let name = if few { format!("{}{FEW_MARK}", job_name(job, sources)) } else { job_name(job, sources) };
        running.lock().unwrap_or_else(PoisonError::into_inner).push((mark, name, start, std::thread::current().id()));
        let cpu_start = if asked.lanes == 1 { process_use() } else { None };
        let row = run(job, sources, tools, &alone, (classes, few));
        let cpu = cpu_start.zip(process_use()).map(|(before, after)| after.cpu_seconds - before.cpu_seconds);
        running.lock().unwrap_or_else(PoisonError::into_inner).retain(|(other, _, _, _)| *other != mark);
        ended.fetch_add(1, Ordering::Relaxed);
        let end = started.elapsed().as_secs_f64();
        let seen_running = seen.lock().unwrap_or_else(PoisonError::into_inner).remove(&mark).unwrap_or_default();
        timed.lock().unwrap_or_else(PoisonError::into_inner).push(JobTime { kind: row.kind.clone(), few, guess: weights[order[at]], start, end, cpu, tool: row.tool.clone(), marks: row.marks, costs: row.costs.clone(), seen: seen_running });
        row
    };
    let (stop, stopped) = channel::<()>();
    let (running_now, ended_now, seen_now) = (&running, &ended, &seen);
    let (done, few_done) = std::thread::scope(|scope| {
        scope.spawn(move || {
            while stopped.recv_timeout(Duration::from_secs(PROGRESS_SECONDS)) == Err(RecvTimeoutError::Timeout) {
                let now = started.elapsed().as_secs_f64();
                let jobs_now = running_now.lock().unwrap_or_else(PoisonError::into_inner).clone();
                eprintln!("{}", progress_line(now, ended_now.load(Ordering::Relaxed), &jobs_now));
                let mut seen_jobs = seen_now.lock().unwrap_or_else(PoisonError::into_inner);
                for (mark, _, start, thread) in &jobs_now {
                    if let Some(learning) = learning_now(*thread) {
                        seen_jobs.entry(*mark).or_default().push(format!("{:.none$} s {learning}", now - start, none = 0));
                    }
                }
            }
        });
        let found = on_lanes_then(
            order.len(),
            asked.lanes,
            &|at| {
                let row = timed_run(at, false);
                last_end.fetch_max(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
                row
            },
            &|at, row| order[at] < first_count && few_counts(&jobs[order[at]], sources) && kind_passed(&jobs[order[at]], row, sources),
            &|at| timed_run(at, true),
        );
        drop(stop);
        found
    });
    let mut rows: Vec<Row> = (0..jobs.len()).map(|_| Row::default()).collect();
    let mut few: Vec<Option<Row>> = (0..jobs.len()).map(|_| None).collect();
    for ((at, row), few_row) in order.iter().zip(done).zip(few_done) {
        rows[*at] = row;
        few[*at] = few_row;
    }
    let mut timed = timed.into_inner().unwrap_or_else(PoisonError::into_inner);
    timed.sort_by(|one, other| one.start.total_cmp(&other.start));
    Passes { rows, few, first_seconds: Duration::from_nanos(last_end.into_inner()).as_secs_f64(), timed }
}
because!(run_all, TaskBoard, "the jobs of the first pass and the pairs side by side on the threads, each job on one thread, the largest guess first so that no thread starts a large job when the others are near their end; a kind of the first pass that is learned, in a task that is counted, asks for its job with few lessons at once, and the next free thread runs it before the next job of the first pass, so that a job and its run with few lessons end early when the job starts early; the rows come back in the order of the jobs, and the times of the jobs in the order of their starts; while the jobs run, a progress line on the error output names the running jobs at times");

fn passed(row: &Row, bar: f64, controls: bool) -> bool {
    if row.learner.is_none() {
        return false;
    }
    let Some(counts) = share_that_counts(row) else { return false };
    counts >= bar && (!controls || counts >= row.one_answer.max(row.question) + ABOVE)
}
because!(passed, TaskBoard, "a kind passes when its share that counts is at the bar of its set and clearly above the controls; a kind that its set marks as one where no model can be above the controls passes at the bar alone");

fn bar_of(from: &Source) -> f64 {
    if from.people { BAR_PEOPLE } else { LEARNED }
}
because!(bar_of, TaskBoard, "the bar of the kinds of a set, by who wrote its text");

fn lowest(shares: impl Iterator<Item = f64>) -> Option<f64> {
    shares.fold(None, |low: Option<f64>, one| Some(low.map_or(one, |low| low.min(one))))
}
because!(lowest, TaskBoard, "the lowest of some shares, and nothing for no share");

#[derive(Clone, Debug, Default)]
pub struct Counts {
    pub kinds: usize,
    pub tasks: usize,
    pub kinds_learned: usize,
    pub tasks_learned: usize,
    pub sets: usize,
    pub labelled: usize,
    pub word_tasks: usize,
    pub word_passed: usize,
    pub no_words: usize,
    pub few: usize,
    pub small_large: usize,
    pub people: usize,
    pub people_labelled: usize,
    pub read_learned: usize,
    pub read_mixed: usize,
    pub read_written: usize,
    pub rules_discovered: usize,
    pub rules_mixed: usize,
    pub rules_given: usize,
    pub rules_none: usize,
    pub facts_discovered: usize,
    pub facts_mixed: usize,
    pub facts_given: usize,
    pub all_discovered: usize,
    pub ladder_found: usize,
    pub ladder_no_rule: usize,
    pub ladder_learned: usize,
}
because!(Counts, TaskBoard, "the numbers of a board: kinds and tasks, how many are learned and in how many sets, how many learned tasks have stronger lessons, are measured and pass with new words or hold no name at all, are learned from few lessons, hold from a small form to a large one, are text that people wrote, how the learned tasks read (learned, mixed, written), how their rules read (discovered, mixed, given, or no rule), and how many tasks of all rest on discovered rules alone; and the ladder of the learned tasks that read by learned lines: with discovered rules, with no rule, or with rules of the library");

pub struct Board {
    pub text: Vec<String>,
    pub counts: Counts,
    pub jobs: Vec<String>,
}
because!(Board, TaskBoard, "a board as the lines that are printed and as its numbers, and the lines of the times of its jobs");

const GIGABYTE: f64 = 1_073_741_824.0;
because!(GIGABYTE, TaskBoard, "the bytes of a gigabyte, as the peak memory of the board is told, the same unit as the cap on private memory");

const PEAK_PLACES: usize = 2;
because!(PEAK_PLACES, TaskBoard, "the places after the point of the peak memory of the board: a hundredth of a gigabyte tells runs apart");

const SLOW_SECONDS: f64 = 3.0;
because!(SLOW_SECONDS, TaskBoard, "the seconds of a kind with no result from which the board names where its time went: a few seconds of a search that found nothing are worth a look");

const SLOW_WAY_LEAST: f64 = 0.05;
because!(SLOW_WAY_LEAST, TaskBoard, "the least seconds of a way of a slow kind with no result that the board names: a way of less is no part of where the time went");

const SLOW_PLACES: usize = 1;
because!(SLOW_PLACES, TaskBoard, "the places after the point of the seconds of a way of a slow kind with no result");

const SLOW_WAYS_SHOWN: usize = 3;
because!(SLOW_WAYS_SHOWN, TaskBoard, "the ways of the learning of a slow kind with no result that the board names, those of the most seconds first");

const BUDGET_HEAD: &str = "STOPPED AT THE BUDGET of the learning (kinds whose learners stopped at the bound of the work of one learning, with the work of their chains): ";
because!(BUDGET_HEAD, TaskBoard, "the words before the kinds of the board whose learners stopped at the bound of the work of one learning, a line after the table only when a kind stopped there, so that a search that ran away shows on the board and the time that it would have taken does not");

const BUDGET_BETWEEN: &str = "; ";
because!(BUDGET_BETWEEN, TaskBoard, "the mark between the kinds that stopped at the budget of the learning");

const JOB_WAYS_BETWEEN: &str = ", ";
because!(JOB_WAYS_BETWEEN, TaskBoard, "the marks between the ways of a slow kind with no result");

pub const SLOW_HEAD: &str = "where the time of the slow kinds with no result went: ";
because!(SLOW_HEAD, TaskBoard, "the words before the slow kinds with no result, each with its seconds and the ways of its learning that took the most; a line of seconds, which differ from run to run, so that a comparison of two boards reads it apart");

const JOB_BETWEEN: &str = " | ";
because!(JOB_BETWEEN, TaskBoard, "the mark between the fields of a line of the times of the jobs");

const JOB_PLACES: usize = 1;
because!(JOB_PLACES, TaskBoard, "the places after the point of the seconds of a job: a tenth of a second tells the jobs apart and keeps the lines short");

const JOB_CPU_HEAD: &str = "CPU ";
because!(JOB_CPU_HEAD, TaskBoard, "the word before the CPU seconds of a job in its line of the times of the jobs, which a board on one thread gives: no other job runs beside it then, so the CPU of the process in the while of the job is the job's own, and other programs on the machine do not reach it as they reach the seconds of the clock");

const JOB_MARKS_HEAD: &str = "marks: ";
because!(JOB_MARKS_HEAD, TaskBoard, "the word before the answer marks of a job's read part in its line of the times of the jobs, the counts of answers by proof, by default and guesses, with the wrong ones of each, and of no answer");

fn job_lines(timed: &[JobTime]) -> Vec<String> {
    let mut lines = vec!["kind | pass | guess of work | start s | end s | seconds | tool | CPU seconds on one thread | answer marks of the read part | ways of the learning | chains of the learning | seen while it ran".to_string()];
    lines.extend(timed.iter().map(|job| {
        let pass = if job.few { "few lessons" } else { FIRST_PASS };
        let mut fields = vec![job.kind.clone(), pass.to_string(), job.guess.to_string(), format!("{:.*}", JOB_PLACES, job.start), format!("{:.*}", JOB_PLACES, job.end), format!("{:.*}", JOB_PLACES, job.end - job.start), job.tool.clone()];
        if let Some(cpu) = job.cpu {
            fields.push(format!("{JOB_CPU_HEAD}{:.*} s", JOB_PLACES, cpu));
        }
        if job.marks.read_cases() > usize::default() {
            fields.push(format!("{JOB_MARKS_HEAD}{}", job.marks.counts()));
        }
        if !job.costs.is_empty() {
            fields.push(job.costs.clone());
        }
        if !job.seen.is_empty() {
            fields.push(format!("{SEEN_HEAD}{}", job.seen.join(SEEN_BETWEEN)));
        }
        fields.join(JOB_BETWEEN)
    }));
    lines
}
because!(job_lines, TaskBoard, "the times of the jobs as lines in the order of their starts, one line for each job, with its CPU seconds on a board of one thread, the ways of its learning and the chains that they made, and what the progress lines saw it learn while it ran");

pub fn grown_ways(jobs: &[String], order: &str) -> Vec<String> {
    let before: HashMap<(String, String), Vec<(String, f64)>> = order
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cells: Vec<&str> = line.split(JOB_BETWEEN).collect();
            Some(((cells.first()?.to_string(), cells.get(1)?.to_string()), way_seconds(line)))
        })
        .collect();
    jobs.iter()
        .enumerate()
        .map(|(at, line)| {
            let cells: Vec<&str> = line.split(JOB_BETWEEN).collect();
            let (Some(kind), Some(pass)) = (cells.first(), cells.get(1)) else { return line.clone() };
            let Some(was) = before.get(&(kind.to_string(), pass.to_string())).filter(|_| at > usize::default()) else { return line.clone() };
            let grown: Vec<String> = way_seconds(line)
                .iter()
                .filter_map(|(way, now)| {
                    let then = was.iter().find(|(known, _)| known == way)?.1;
                    (*now >= then * SLOWER_TIMES && now - then >= SLOWER_SECONDS).then(|| format!("{way} {then:.places$} s to {now:.places$} s", places = JOB_PLACES))
                })
                .collect();
            if grown.is_empty() { line.clone() } else { format!("{line}{JOB_BETWEEN}{GROWN_HEAD}{}", grown.join(SEEN_BETWEEN)) }
        })
        .collect()
}
because!(grown_ways, TaskBoard, "the lines of the jobs with the ways of each job whose seconds grew against an earlier file of the times of the jobs (the order file) by as many times and as many seconds as a slower task of a compare, so that a change that makes one way of one kind dearer is seen in the run that brings it (the told amounts once made the wording forms of logic_grid_puzzle four times dearer); a job that the earlier file does not hold, and the line of the names of the columns, stay as they are");

pub fn grown_jobs(jobs: &[String]) -> Vec<String> {
    jobs.iter()
        .filter_map(|line| {
            let (_, ways) = line.split_once(GROWN_HEAD)?;
            let cells: Vec<&str> = line.split(JOB_BETWEEN).collect();
            Some(format!("{} ({}): {ways}", cells.first()?, cells.get(1)?))
        })
        .collect()
}
because!(grown_jobs, TaskBoard, "the jobs whose ways grew against the order file, each with its pass and the ways, for a line after the board");

fn way_seconds(line: &str) -> Vec<(String, f64)> {
    let Some(ways) = line.split(JOB_BETWEEN).find_map(|cell| cell.trim().strip_prefix(WAYS_HEAD)) else { return Vec::new() };
    ways.split(WAYS_BETWEEN)
        .filter_map(|spent| {
            let words: Vec<&str> = spent.split_whitespace().collect();
            let at = words.windows(WORD_PAIR).position(|pair| pair[1] == SECONDS_WORD && pair[0].parse::<f64>().is_ok())?;
            Some((words[..at].join(WAY_WORD_GAP), words[at].parse().ok()?))
        })
        .collect()
}
because!(way_seconds, TaskBoard, "the seconds of each way of the learning of a job, from its line in a file of the times of the jobs: the ways as the ledger of costs writes them, each a name, its seconds and the word of seconds, and then what else it says (in so many runs, taken)");

const WAYS_HEAD: &str = "ways: ";
because!(WAYS_HEAD, TaskBoard, "the words that open the ways of the learning of a job in its line, as the ledger of costs writes them");

const WAYS_BETWEEN: &str = ", ";
because!(WAYS_BETWEEN, TaskBoard, "the marks between the ways of the learning of a job in its line, as the ledger of costs writes them");

const SECONDS_WORD: &str = "s";
because!(SECONDS_WORD, TaskBoard, "the word after the seconds of a way of the learning in the line of a job");

const WORD_PAIR: usize = 2;
because!(WORD_PAIR, TaskBoard, "a word and the one after it: the seconds of a way and the word of seconds");

const WAY_WORD_GAP: &str = " ";
because!(WAY_WORD_GAP, TaskBoard, "the space between the words of the name of a way, as the ledger of costs writes it");

const GROWN_HEAD: &str = "grew against the order file: ";
because!(GROWN_HEAD, TaskBoard, "the words before the ways of a job whose seconds grew against the order file, at the end of its line, so that a person or a check finds them by these words");

const SEEN_HEAD: &str = "seen while it ran: ";
because!(SEEN_HEAD, TaskBoard, "the words before what the progress lines saw a job learn, at its seconds so far, in the line of the job");

const SEEN_BETWEEN: &str = "; ";
because!(SEEN_BETWEEN, TaskBoard, "the marks between the sightings of a job in its line");

const NO_READING: &str = "-";
because!(NO_READING, TaskBoard, "what the column of the reading says for a task whose kinds have no model that says how it reads");

fn reading_word(reading: Reading) -> &'static str {
    match reading {
        Reading::Learned => "learned",
        Reading::Written => "written",
        Reading::Mixed => "mixed",
    }
}
because!(reading_word, TaskBoard, "what the board prints for a way of reading, in the words of its summary line, so that a person reads the column with no key");

fn task_rules(rows: &[&Row], counted: fn(&Row) -> (usize, usize)) -> &'static str {
    let given: usize = rows.iter().filter(|row| row.learner.is_some()).map(|row| counted(row).0).sum();
    let found: usize = rows.iter().filter(|row| row.learner.is_some()).map(|row| counted(row).1).sum();
    match (given > usize::default(), found > usize::default()) {
        (false, false) => NO_READING,
        (false, true) => RULES_DISCOVERED,
        (true, false) => RULES_GIVEN,
        (true, true) => RULES_MIXED,
    }
}
because!(task_rules, TaskBoard, "where the rules of a task come from, or its known facts with the counts of facts: discovered when every rule that the proofs of the answers of its kinds rest on is held by no text of the library, as the models learned them from the lessons; given when every such rule is a rule of the library; mixed when both; nothing when the proofs rest on no rule");

const RULES_DISCOVERED: &str = "discovered";
because!(RULES_DISCOVERED, TaskBoard, "the word of the column of the rules for a task whose rules were all learned from the lessons");

const RULES_GIVEN: &str = "given";
because!(RULES_GIVEN, TaskBoard, "the word of the column of the rules for a task whose rules are all rules of the library");

const RULES_MIXED: &str = "mixed";
because!(RULES_MIXED, TaskBoard, "the word of the column of the rules for a task that rests on rules of both kinds");

fn task_reading(rows: &[&Row]) -> Option<Reading> {
    let readings: Vec<Reading> = rows.iter().filter(|row| row.learner.is_some()).filter_map(|row| row.reading).collect();
    let first = *readings.first()?;
    Some(if readings.iter().all(|reading| *reading == first) { first } else { Reading::Mixed })
}
because!(task_reading, TaskBoard, "how a task reads: the way that the models of its kinds with a result share, and mixed when they differ; nothing when no kind has a model that says how it reads");

fn lessons_of(source: &Source, asked: &Asked) -> usize {
    if asked.fit > usize::default() { asked.fit } else { source.fit }
}
because!(lessons_of, TaskBoard, "the most lessons of the kinds of a set on the board: the number that the run names for every set, else the number that the set gives");

const MARKS_HEAD: &str = "answer marks of all read cases (by proof: steps prove the answer, and the told sentences and the question hold no two pieces in a row that the lessons did not show, a piece being a word that a line names or a run of other words with its length; guess: they hold such a pair, or plain counting): ";
because!(MARKS_HEAD, TaskBoard, "the start of the line of the marks of all answers of the board, which says what each mark means");

const MARKS_KINDS: &str = "; kinds whose answers by proof are all right, of the kinds with an answer by proof: ";
because!(MARKS_KINDS, TaskBoard, "the words before the count of the kinds where no answer that claims a proof is wrong, the measure of an answer mark that can be trusted");

const MARKS_MOVED: &str = "; right answers of steps marked as guesses, of the right answers of steps: ";
because!(MARKS_MOVED, TaskBoard, "the words before the count of the right answers that the guess mark takes from the proofs, the cost of the mark: a mark that takes many right proofs says little");

pub fn board(sources: &[Source], tools: &[Tool], asked: &Asked, classes: Option<&WordClasses>, known: &HashMap<String, f64>) -> Board {
    let started = Instant::now();
    let first_jobs: Vec<Job> = sources
        .iter()
        .enumerate()
        .flat_map(|(from, source)| source.tasks.iter().flat_map(move |(_, kinds)| kinds.iter().map(move |kind| Job { from, kind: kind.clone(), pair: None, fit: lessons_of(source, asked), new_words: true })))
        .collect();
    let pair_jobs: Vec<Job> = sources
        .iter()
        .enumerate()
        .flat_map(|(from, source)| source.pairs.iter().map(move |(task, small, large)| Job { from, kind: task.clone(), pair: Some((small.clone(), large.clone())), fit: lessons_of(source, asked), new_words: false }))
        .collect();
    let first_count = first_jobs.len();
    let mut together = first_jobs;
    together.extend(pair_jobs);
    let Passes { mut rows, few: mut few_by_job, first_seconds, timed } = run_all(&together, first_count, sources, tools, asked, classes, known);
    let pair_rows = rows.split_off(first_count);
    let pair_jobs = together.split_off(first_count);
    let first_jobs = together;
    let is_learned: Vec<bool> = first_jobs.iter().zip(&rows).map(|(job, row)| kind_passed(job, row, sources)).collect();
    let wants_few: Vec<bool> = first_jobs.iter().zip(&is_learned).map(|(job, learned)| *learned && few_counts(job, sources)).collect();
    let few_jobs: Vec<Job> = first_jobs.iter().zip(&wants_few).filter(|(_, wanted)| **wanted).map(|(job, _)| few_job(job)).collect();
    let alone = Asked { lanes: 1, ..*asked };
    let few_rows: Vec<Row> = first_jobs
        .iter()
        .zip(&wants_few)
        .enumerate()
        .filter(|(_, (_, wanted))| **wanted)
        .map(|(at, (job, _))| few_by_job[at].take().unwrap_or_else(|| run(&few_job(job), sources, tools, &alone, (classes, true))))
        .collect();
    let seconds = started.elapsed().as_secs_f64();

    let row_of = |from: usize, kind: &str| first_jobs.iter().position(|job| job.from == from && job.kind == kind);
    let few_of = |from: usize, kind: &str| few_jobs.iter().position(|job| job.from == from && job.kind == kind).is_some_and(|at| kind_passed(&few_jobs[at], &few_rows[at], sources));
    let mut counts = Counts { kinds: first_jobs.len(), kinds_learned: is_learned.iter().filter(|learned| **learned).count(), ..Counts::default() };
    let mut sets: Vec<&str> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("set | task | text by | lessons | kinds learned | task learned | lowest share | control: question alone, highest | new words, lowest share | learned from {FEW} lessons | small to large | read by | rules by | facts by | answer marks: proof (share right), default, guess, none | seconds"));
    for (from, source) in sources.iter().enumerate() {
        let bar = bar_of(source);
        let by = if source.people { "people" } else { "program" };
        for (task, kinds) in &source.tasks {
            counts.tasks += 1;
            let places: Vec<usize> = kinds.iter().filter_map(|kind| row_of(from, kind)).collect();
            let mine: Vec<&Row> = places.iter().map(|place| &rows[*place]).collect();
            let got = places.iter().filter(|place| is_learned[**place]).count();
            let passes = got == kinds.len();
            let why_not = if passes { source.not_counted.iter().find(|(name, _)| name == task).map(|(_, why)| why.as_str()) } else { None };
            let whole = passes && why_not.is_none();
            let words: Vec<(f64, f64, usize)> = mine.iter().filter_map(|row| row.new_words).filter(|(_, changed, _)| *changed > f64::default()).collect();
            let idle = source.no_names.contains(task);
            let words_pass = !words.is_empty() && whole && words.iter().all(|(right, _, _)| *right >= bar);
            let is_few = whole && kinds.iter().all(|kind| few_of(from, kind));
            let pair = pair_jobs.iter().position(|job| job.from == from && job.kind == *task).map(|at| &pair_rows[at]);
            let pair_pass = pair.is_some_and(|row| passed(row, bar, true)) && !source.not_counted.iter().any(|(name, _)| name == task);
            let with_labels = mine.iter().any(|row| row.labels);
            let labels = with_labels || !source.stronger.is_empty();
            let borrowed = mine.iter().any(|row| row.curriculum);
            let mut earlier: Vec<String> = Vec::new();
            for name in mine.iter().flat_map(|row| earlier_kinds_named(&source.kinds, &row.kind, &row.curriculum_from)) {
                if !earlier.contains(&name) {
                    earlier.push(name);
                }
            }
            let named = if earlier.is_empty() { String::new() } else { format!(" ({})", earlier.join(BETWEEN_EARLIER)) };
            let mut stored: Vec<&str> = mine.iter().flat_map(|row| row.stored_from.iter().map(String::as_str)).collect();
            stored.sort_unstable();
            stored.dedup();
            let from_store = if stored.is_empty() { String::new() } else { format!("{STORE_LESSONS}({})", stored.join(BETWEEN_EARLIER)) };
            let lessons = format!("text, answer{}{}{}{}", if with_labels { ", labels" } else { "" }, if borrowed { format!(", earlier kinds{named}") } else { String::new() }, from_store, if source.stronger.is_empty() { String::new() } else { format!(", {}", source.stronger) });
            let read_by = task_reading(&mine);
            let rules_by = task_rules(&mine, |row| row.rules);
            let facts_by = task_rules(&mine, |row| row.facts);
            counts.all_discovered += usize::from(rules_by == RULES_DISCOVERED);
            if whole {
                counts.rules_discovered += usize::from(rules_by == RULES_DISCOVERED);
                counts.rules_mixed += usize::from(rules_by == RULES_MIXED);
                counts.rules_given += usize::from(rules_by == RULES_GIVEN);
                counts.rules_none += usize::from(rules_by == NO_READING);
                counts.facts_discovered += usize::from(facts_by == RULES_DISCOVERED);
                counts.facts_mixed += usize::from(facts_by == RULES_MIXED);
                counts.facts_given += usize::from(facts_by == RULES_GIVEN);
                counts.read_learned += usize::from(read_by == Some(Reading::Learned));
                let learned_lines = read_by == Some(Reading::Learned);
                counts.ladder_found += usize::from(learned_lines && rules_by == RULES_DISCOVERED);
                counts.ladder_no_rule += usize::from(learned_lines && rules_by == NO_READING);
                counts.ladder_learned += usize::from(learned_lines && (rules_by == RULES_GIVEN || rules_by == RULES_MIXED));
                counts.read_mixed += usize::from(read_by == Some(Reading::Mixed));
                counts.read_written += usize::from(read_by == Some(Reading::Written));
                counts.tasks_learned += 1;
                counts.labelled += usize::from(labels);
                counts.people_labelled += usize::from(labels && source.people);
                if !sets.contains(&source.set.as_str()) {
                    sets.push(source.set.as_str());
                }
                counts.word_tasks += usize::from(!words.is_empty() && !idle);
                counts.no_words += usize::from(idle);
                counts.word_passed += usize::from(words_pass && !idle);
                counts.few += usize::from(is_few);
                counts.people += usize::from(source.people);
            }
            counts.small_large += usize::from(pair_pass);
            let Some(low) = lowest(mine.iter().filter_map(|row| row.learner)) else {
                let mut notes: Vec<&str> = mine.iter().map(|row| row.note.split(FAULT_DETAIL).next().unwrap_or_default()).collect();
                notes.sort_unstable();
                notes.dedup();
                lines.push(format!("{} | {task} | {by} | {lessons} | 0 of {} | no | no result: {}", source.set, kinds.len(), notes.join("; ")));
                continue;
            };
            let task_learned = if whole { "YES".to_string() } else { why_not.map_or("no".to_string(), |why| format!("passes, not counted: {why}")) };
            let question = mine.iter().map(|row| row.question).fold(f64::default(), f64::max);
            let with_words = lowest(words.iter().map(|(right, _, _)| *right)).map_or("-".to_string(), |low| format!("{low:.places$}{}", if words_pass { " pass" } else { "" }, places = SHARE_PLACES));
            let from_few = if is_few {
                "yes"
            } else if whole {
                "no"
            } else {
                "-"
            };
            let large = pair.and_then(|row| row.learner).map_or("-".to_string(), |share| format!("{share:.places$}{}", if pair_pass { " pass" } else { "" }, places = SHARE_PLACES));
            let all_seconds: f64 = mine.iter().map(|row| row.all_seconds).sum();
            let marks = mine.iter().fold(MarkCounts::default(), |sum, row| sum.added(row.marks)).shares();
            lines.push(format!("{} | {task} | {by} | {lessons} | {got} of {} | {task_learned} | {low:.places$} | {question:.places$} | {with_words} | {from_few} | {large} | {} | {rules_by} | {facts_by} | {marks} | {all_seconds:.none$}", source.set, kinds.len(), read_by.map_or(NO_READING, reading_word), places = SHARE_PLACES, none = 0));
        }
    }
    counts.sets = sets.len();
    let mut stopped: Vec<String> = rows.iter().filter_map(|row| row.stopped.as_ref().map(|(names, tool)| format!("{} ({names} stopped, {tool} taken)", row.kind))).collect();
    stopped.sort_unstable();
    let names: Vec<&str> = sources.iter().map(|source| source.name.as_str()).collect();
    let part = if asked.sealed { "the SEALED part" } else { "the read part" };
    let apply = counts.tasks_learned - counts.no_words;
    let mut text = vec![
        format!("The board of the tasks ({part}), sets: {}", names.join(", ")),
        format!(
            "{} kinds in {} tasks; {} kinds learned; {} tasks learned in {} sets; whole run {seconds:.none$} s on {} threads (first pass {first_seconds:.none$} s){}",
            counts.kinds,
            counts.tasks,
            counts.kinds_learned,
            counts.tasks_learned,
            counts.sets,
            asked.lanes,
            process_use().map_or(String::new(), |used| format!(", CPU {:.none$} s, peak {:.places$} GB", used.cpu_seconds, used.peak_bytes as f64 / GIGABYTE, none = 0, places = PEAK_PLACES)),
            none = 0
        ),
        String::new(),
    ];
    text.extend(lines);
    text.push(String::new());
    text.push(format!(
        "kinds where the tool that was taken from the first part of the lessons stopped on all lessons, so that the next tool was taken: {}{}",
        stopped.len(),
        if stopped.is_empty() { String::new() } else { format!(": {}", stopped.join("; ")) }
    ));
    let mut bounded: Vec<String> = rows.iter().filter(|row| row.bound_stops > usize::default()).map(|row| format!("{} ({})", row.kind, row.bound_stops)).collect();
    bounded.sort_unstable();
    if !bounded.is_empty() {
        text.push(format!("stopped at a bound (readings of the read part that ended at a bound of the model's work, so an answer may miss a fact): {}", bounded.join("; ")));
    }
    let mut budgeted: Vec<String> = rows.iter().filter(|row| row.budget_stopped).map(|row| format!("{} ({})", row.kind, row.learning_work)).collect();
    budgeted.sort_unstable();
    if !budgeted.is_empty() {
        text.push(format!("{BUDGET_HEAD}{}", budgeted.join(BUDGET_BETWEEN)));
    }
    let beyond = steps_beyond_range();
    if beyond > u64::default() {
        text.push(format!("steps of exact amounts that left the width of the whole numbers, so that their answers were lost: {beyond}"));
    }
    let mut tried: Vec<(&str, Vec<&str>)> = Vec::new();
    for row in rows.iter().filter(|row| row.learner.is_none()) {
        let Some((_, detail)) = row.note.split_once(FAULT_DETAIL) else { continue };
        match tried.iter_mut().find(|(seen, _)| *seen == detail) {
            Some((_, kinds)) => kinds.push(row.kind.as_str()),
            None => tried.push((detail, vec![row.kind.as_str()])),
        }
    }
    if !tried.is_empty() {
        let groups: Vec<String> = tried.iter().map(|(detail, kinds)| format!("{} ({detail})", kinds.join(", "))).collect();
        text.push(format!("what was tried for the kinds with no result: {}", groups.join("; ")));
    }
    let mut slow: Vec<&Row> = rows.iter().filter(|row| row.learner.is_none() && row.all_seconds >= SLOW_SECONDS).collect();
    slow.sort_by(|one, other| other.all_seconds.total_cmp(&one.all_seconds));
    if !slow.is_empty() {
        let kinds: Vec<String> = slow
            .iter()
            .map(|row| format!("{} {:.none$} s ({})", row.kind, row.all_seconds, row.spent_ways.iter().filter(|(_, seconds)| *seconds >= SLOW_WAY_LEAST).take(SLOW_WAYS_SHOWN).map(|(way, seconds)| format!("{way} {seconds:.places$} s", places = SLOW_PLACES)).collect::<Vec<String>>().join(JOB_WAYS_BETWEEN), none = 0))
            .collect();
        text.push(format!("{SLOW_HEAD}{}", kinds.join("; ")));
    }
    let unlike: Vec<String> = rows.iter().filter_map(|row| row.one_case.filter(|(lessons, read)| (lessons - read).abs() >= READ_UNLIKE_POINTS).map(|(lessons, read)| format!("{} {lessons:.none$} and {read:.none$}", row.kind, none = 0))).collect();
    if !unlike.is_empty() {
        text.push(format!("read parts unlike their lessons (the share of cases whose told text the kind tells once, in the lessons and in the read part, where the two differ by {READ_UNLIKE_POINTS} points or more: the line of such a kind reads mostly one kind of story): {}", unlike.join(", ")));
    }
    let guessed: Vec<String> = rows.iter().filter_map(|row| row.default_shares.filter(|(without, with)| with - without >= DEFAULT_RISE_POINTS).map(|(without, with)| format!("{} {without:.none$} and {with:.none$}", row.kind, none = 0))).collect();
    if !guessed.is_empty() {
        text.push(format!("answers by default with new words (the share of the read cases that the model answers by a default, without and with new words, where new words raise it by {DEFAULT_RISE_POINTS} points or more: a reading that needs the words of its lessons, such as names that a list knows): {}", guessed.join(", ")));
    }
    let named: Vec<String> = rows.iter().filter_map(|row| row.named.filter(|(named, _)| *named > usize::default()).map(|(named, forms)| format!("{} {named} of {forms}", row.kind))).collect();
    if !named.is_empty() {
        text.push(format!("learned forms by named words (forms that name a word that a learned form with fewer fixed words reads too, of all the learned forms of a kind: values learned one by one): {}", named.join(", ")));
    }
    let proved: Vec<&Row> = rows.iter().filter(|row| row.marks.proof != (usize::default(), usize::default())).collect();
    let all_marks = rows.iter().fold(MarkCounts::default(), |sum, row| sum.added(row.marks));
    text.push(format!("{MARKS_HEAD}{}{MARKS_KINDS}{} of {}{MARKS_MOVED}{} of {}", all_marks.shares(), proved.iter().filter(|row| row.marks.proof.1 == usize::default()).count(), proved.len(), all_marks.guess.0, all_marks.proof.0 + all_marks.guess.0));
    text.push(format!("breadth: {} tasks learned in {} sets ({} of the tasks are learned with stronger lessons: labels of the lesson sentences that the set gives)", counts.tasks_learned, counts.sets, counts.labelled));
    text.push(format!(
        "new words: {} of {apply} learned tasks pass where new words apply ({} are measured, {} have no table that changes a read case and count as not passed; {} more tasks hold no name, place or thing, as their set says)",
        counts.word_passed,
        counts.word_tasks,
        apply - counts.word_tasks,
        counts.no_words
    ));
    text.push(format!("how the learned tasks read: {} learned, {} mixed, {} written (learned: every line that reads the told text or answers was learned from the lessons of the kind; written: lines of the library alone; mixed: both)", counts.read_learned, counts.read_mixed, counts.read_written));
    text.push(format!("how the rules of the learned tasks read: {} discovered, {} mixed, {} given, {} with no rule (discovered: every rule that the proofs of the answers rest on was learned from the lessons, as no text of the library holds it; given: rules of the library alone; mixed: both); tasks whose rules are all discovered: {} of {} learned tasks, {} of {} tasks", counts.rules_discovered, counts.rules_mixed, counts.rules_given, counts.rules_none, counts.rules_discovered, counts.tasks_learned, counts.all_discovered, counts.tasks));
    text.push(format!("how the known facts of the learned tasks read: {} discovered, {} mixed, {} given, {} rest on no known fact (discovered: every fact known before the story that the proofs of the answers rest on was learned from the lessons, as no text of the library holds it; given: facts of the library alone; mixed: both)", counts.facts_discovered, counts.facts_mixed, counts.facts_given, counts.tasks_learned - counts.facts_discovered - counts.facts_mixed - counts.facts_given));
    text.push(format!("the ladder of the learned tasks: {} discovered ({} with discovered rules, {} that need no rule), {} learned, {} mixed, {} written (discovered: read by learned lines, and no rule of the library is used; learned: read by learned lines with rules of the library; mixed: read by learned and written lines; written: read by written lines)", counts.ladder_found + counts.ladder_no_rule, counts.ladder_found, counts.ladder_no_rule, counts.ladder_learned, counts.read_mixed, counts.read_written));
    text.push(format!("learn small, read large: {} tasks", counts.small_large));
    text.push(format!("few lessons: {} of {} learned tasks are learned from {FEW} lessons", counts.few, counts.tasks_learned));
    text.push(format!("text that people wrote: {} learned tasks ({} with stronger lessons)", counts.people, counts.people_labelled));
    Board { text, counts, jobs: job_lines(&timed) }
}
because!(board, TaskBoard, "the board, with its whole run in seconds of the clock and, when the program can read them, its own CPU seconds and peak memory, so that the load of other programs is not read as its cost; the board in three passes: every kind with the lessons that its set names, read also with new words; every learned kind of a counted task again from few lessons; the pairs of a small and a large kind. A task is learned when all its kinds pass, and a task that passes by a regularity of its set is printed and not counted; a task with no result names the fault in its line, and what was tried for each such kind follows the table");

pub struct BoardCompare;
source!(BoardCompare, "two boards of the same tasks side by side: the line of each task compared field by field, apart from its seconds, which are compared on their own because other runs on the machine change them; a task is slower when its seconds grow by both a factor and a number of seconds; the lines after the table compared by their names, and the count line apart from its whole run; a board of fewer sets is compared on its own sets, as a run of one set is compared with a reference of all");

const SLOWER_TIMES: f64 = 2.0;
because!(SLOWER_TIMES, BoardCompare, "how many times its seconds must grow before a task is named as slower: a smaller rise is the noise of a machine that other runs share");

const SLOWER_SECONDS: f64 = 3.0;
because!(SLOWER_SECONDS, BoardCompare, "the seconds that a task must grow by before it is named as slower, so that a task of a second or two is not named for a small rise");

const TABLE_HEAD: &str = "set | task | ";
because!(TABLE_HEAD, BoardCompare, "the start of the head line of the table of a board, which names its fields");

const KEY_FIELDS: usize = 2;
because!(KEY_FIELDS, BoardCompare, "the fields that name a task in its line: its set and its task");

const SECONDS_FIELD: &str = "seconds";
because!(SECONDS_FIELD, BoardCompare, "the name of the last field of the line of a task with a result, its seconds");

const WHOLE_RUN: &str = "; whole run ";
because!(WHOLE_RUN, BoardCompare, "the words before the seconds of the whole run in the count line of a board, which change from run to run and are shown apart");

const NAME_END: &str = ": ";
because!(NAME_END, BoardCompare, "the marks after the name of a line after the table, so that the lines of two boards are paired by their names");

struct BoardFile<'a> {
    names: Vec<&'a str>,
    tasks: Vec<Vec<&'a str>>,
    summary: Vec<&'a str>,
    whole: Option<&'a str>,
    slow: Option<&'a str>,
}
because!(BoardFile, BoardCompare, "a board as text read back: the names of the fields of its table, the fields of each line of a task, the other lines, the whole run of its count line, and the line of the slow kinds with no result, whose seconds are read apart");

fn board_file(text: &str) -> BoardFile<'_> {
    let mut file = BoardFile { names: Vec::new(), tasks: Vec::new(), summary: Vec::new(), whole: None, slow: None };
    let mut in_table = false;
    for line in text.lines() {
        if line.starts_with(TABLE_HEAD) {
            file.names = line.split(JOB_BETWEEN).collect();
            in_table = true;
        } else if line.is_empty() {
            in_table = false;
        } else if in_table {
            file.tasks.push(line.split(JOB_BETWEEN).collect());
        } else if line.starts_with(SLOW_HEAD) {
            file.slow = Some(line);
        } else if let Some((counts, whole)) = line.split_once(WHOLE_RUN) {
            file.summary.push(counts);
            file.whole = Some(whole);
        } else {
            file.summary.push(line);
        }
    }
    file
}
because!(board_file, BoardCompare, "the parts of a board read from its text: the head line of the table, the lines of the tasks up to the first empty line after it, and every other line that is not empty, the count line without its whole run");

fn task_key(fields: &[&str]) -> String {
    fields.iter().take(KEY_FIELDS).copied().collect::<Vec<&str>>().join(JOB_BETWEEN)
}
because!(task_key, BoardCompare, "the name of a task in a board: its set and its task");

fn without_seconds<'a, 'b>(fields: &'b [&'a str], names: &[&str]) -> (&'b [&'a str], Option<f64>) {
    match (names.last(), fields.split_last()) {
        (Some(last), Some((seconds, rest))) if *last == SECONDS_FIELD && fields.len() == names.len() => (rest, seconds.trim().parse::<f64>().ok()),
        _ => (fields, None),
    }
}
because!(without_seconds, BoardCompare, "the fields of the line of a task without its seconds, and the seconds, when the line has every field of the head line; a line with no result has no seconds");

fn field_changes(was: &[&str], now: &[&str], names: &[&str]) -> Vec<String> {
    if was.len() != now.len() {
        return vec![format!("{} -> {}", was[KEY_FIELDS.min(was.len())..].join(JOB_BETWEEN), now[KEY_FIELDS.min(now.len())..].join(JOB_BETWEEN))];
    }
    (KEY_FIELDS..now.len()).filter(|at| was[*at] != now[*at]).map(|at| format!("{}: {} -> {}", names.get(at).copied().unwrap_or(SECONDS_FIELD), was[at], now[at])).collect()
}
because!(field_changes, BoardCompare, "what changed in the line of a task: each field that differs, with its name, its value before and its value now; the whole line after the name of the task when the two lines have a different number of fields");

fn summary_key(line: &str) -> &str {
    line.split_once(NAME_END).map_or("", |(name, _)| name)
}
because!(summary_key, BoardCompare, "the name of a line after the table: its words before the first colon, or nothing for the count line");

fn named_changes((was, was_names): (&[&str], &[&str]), (now, now_names): (&[&str], &[&str])) -> Vec<String> {
    (KEY_FIELDS..now.len())
        .filter_map(|at| {
            let name = now_names.get(at)?;
            let before = was_names.iter().position(|known| known == name).and_then(|place| was.get(place))?;
            (*before != now[at]).then(|| format!("{name}: {before} -> {}", now[at]))
        })
        .collect()
}
because!(named_changes, BoardCompare, "what changed in the line of a task with a result in both boards, field by field under the same name in the two head lines, so that a column that one board has and the other lacks (facts by, beside rules by) changes no line");

pub fn compare_boards(reference: &str, board: &str) -> Vec<String> {
    let was = board_file(reference);
    let now = board_file(board);
    let before: HashMap<String, &Vec<&str>> = was.tasks.iter().map(|fields| (task_key(fields), fields)).collect();
    let here: HashSet<String> = now.tasks.iter().map(|fields| task_key(fields)).collect();
    let sets: HashSet<&str> = now.tasks.iter().filter_map(|fields| fields.first().copied()).collect();
    let (ours, others): (Vec<&Vec<&str>>, Vec<&Vec<&str>>) = was.tasks.iter().partition(|fields| fields.first().is_some_and(|set| sets.contains(set)));
    let (mut same, mut changed, mut slower, mut only_here) = (usize::default(), Vec::new(), Vec::new(), Vec::new());
    for fields in &now.tasks {
        let name = task_key(fields);
        let Some(old) = before.get(&name) else {
            only_here.push(format!("only here | {name}"));
            continue;
        };
        let (old_fields, old_seconds) = without_seconds(old, &was.names);
        let (new_fields, new_seconds) = without_seconds(fields, &now.names);
        let differences = match (old_seconds, new_seconds) {
            (Some(_), Some(_)) => named_changes((old_fields, &was.names), (new_fields, &now.names)),
            _ if old_fields == new_fields => Vec::new(),
            _ => field_changes(old_fields, new_fields, &now.names),
        };
        if differences.is_empty() {
            same += 1;
        } else {
            changed.push(format!("changed | {name} | {}", differences.join(JOB_BETWEEN)));
        }
        if let (Some(old_seconds), Some(new_seconds)) = (old_seconds, new_seconds)
            && new_seconds >= old_seconds * SLOWER_TIMES
            && new_seconds - old_seconds >= SLOWER_SECONDS
        {
            slower.push(format!("slower | {name} | {old_seconds} s -> {new_seconds} s"));
        }
    }
    let gone: Vec<String> = ours.iter().map(|fields| task_key(fields)).filter(|name| !here.contains(name)).map(|name| format!("only in the reference | {name}")).collect();
    let other_sets = if others.is_empty() { String::new() } else { format!(" ({} more task lines of the reference are of sets that this board did not run)", others.len()) };
    let mut lines = vec![format!(
        "compared with the reference: {} and {} task lines{other_sets}; {same} the same, {} changed, {} only in the reference, {} only here; {} slower by {SLOWER_TIMES} times and {SLOWER_SECONDS} s or more",
        ours.len(),
        now.tasks.len(),
        changed.len(),
        gone.len(),
        only_here.len(),
        slower.len()
    )];
    if was.names != now.names {
        lines.push(format!("head | {} -> {}", was.names.join(JOB_BETWEEN), now.names.join(JOB_BETWEEN)));
    }
    lines.extend(changed);
    lines.extend(gone);
    lines.extend(only_here);
    lines.extend(slower);
    if !others.is_empty() {
        if let (Some(old), Some(new)) = (was.whole, now.whole) {
            lines.push(format!("whole run | {old} -> {new}"));
        }
        return lines;
    }
    let old_summary: HashMap<&str, &str> = was.summary.iter().map(|line| (summary_key(line), *line)).collect();
    let new_keys: HashSet<&str> = now.summary.iter().map(|line| summary_key(line)).collect();
    for line in &now.summary {
        match old_summary.get(summary_key(line)) {
            Some(old) if old == line => {}
            Some(old) => lines.push(format!("summary | {old} -> {line}")),
            None => lines.push(format!("summary only here | {line}")),
        }
    }
    lines.extend(was.summary.iter().filter(|line| !new_keys.contains(summary_key(line))).map(|line| format!("summary only in the reference | {line}")));
    if let (Some(old), Some(new)) = (was.whole, now.whole) {
        lines.push(format!("whole run | {old} -> {new}"));
    }
    lines
}
because!(compare_boards, BoardCompare, "what changed from a reference board to a board: a count line, the tasks whose line changed with each changed field, the tasks that only one of them has, the tasks that are slower, the changed lines after the table, and the whole run of each; when the board ran fewer sets than the reference, the tasks of its own sets alone, and no lines after the table, whose counts are of other sets");
