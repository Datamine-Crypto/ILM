use ilm_core::cases::Case;
use ilm_core::choice::picked;
use ilm_core::base::lanes::on_lanes_then;
use ilm_core::tools::call::Tool;
use crate::measure::kind::{ABOVE, Asked, Ground, LEARNED, Row, SHARE_PLACES, measure, share_that_counts};
use ilm_core::classes::WordClasses;
use ilm_data::sets::folder::{Loaded, Source, final_file, kind_weight, load, load_pair, new_words};
use ilm_core::split::{SEALED_AFTER, parts};
use patterns::{because, source};
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
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

fn run(job: &Job, sources: &[Source], tools: &[Tool], asked: &Asked, classes: Option<&WordClasses>) -> Row {
    let from = &sources[job.from];
    let (name, loaded, last) = match &job.pair {
        Some((small, large)) => (format!("{}:{small}>{large}", from.name), load_pair(from, small, large), large.as_str()),
        None => (job.kind.clone(), load(from, &job.kind), job.kind.as_str()),
    };
    let Ok(loaded) = loaded else { return Row { kind: name, note: NO_CASES.to_string(), ..Row::default() } };
    let words = if job.new_words { new_words(from, &job.kind) } else { None };
    let asked = Asked { fit: job.fit, held: from.held, new_words: job.new_words, ..*asked };
    let last_part = if asked.sealed { Some(final_part(from, last, asked.fit, asked.held)) } else { None };
    measure(&Ground { name: &name, cases: &loaded.kind.cases, labels: &loaded.kind.labels, start: loaded.start, stories: loaded.stories, final_part: last_part.as_deref(), sealed: None, new_words: words.as_deref(), answerer: None, classes }, tools, &asked)
}
because!(run, TaskBoard, "one job of the board through the measure; a pair is the cases of its small kind and then of its large kind, and its final part is the one of its large kind");

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

struct Passes {
    rows: Vec<Row>,
    few: Vec<Option<Row>>,
    first_seconds: f64,
}
because!(Passes, TaskBoard, "the rows of the jobs of the first pass and of the pairs in the order of the jobs, the row with few lessons of each learned kind of a counted task at the place of its job, and the seconds until the last job of the first pass ended");

fn run_all(jobs: &[Job], first_count: usize, sources: &[Source], tools: &[Tool], asked: &Asked, classes: Option<&WordClasses>) -> Passes {
    let started = Instant::now();
    let alone = Asked { lanes: 1, ..*asked };
    let weights: Vec<u64> = jobs.iter().map(|job| job_weight(job, sources)).collect();
    let mut order: Vec<usize> = (0..jobs.len()).collect();
    order.sort_by_key(|at| std::cmp::Reverse(weights[*at]));
    let last_end = AtomicU64::new(0);
    let (done, few_done) = on_lanes_then(
        order.len(),
        asked.lanes,
        &|at| {
            let row = run(&jobs[order[at]], sources, tools, &alone, classes);
            last_end.fetch_max(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
            row
        },
        &|at, row| order[at] < first_count && few_counts(&jobs[order[at]], sources) && kind_passed(&jobs[order[at]], row, sources),
        &|at| run(&few_job(&jobs[order[at]]), sources, tools, &alone, classes),
    );
    let mut rows: Vec<Row> = (0..jobs.len()).map(|_| Row::default()).collect();
    let mut few: Vec<Option<Row>> = (0..jobs.len()).map(|_| None).collect();
    for ((at, row), few_row) in order.iter().zip(done).zip(few_done) {
        rows[*at] = row;
        few[*at] = few_row;
    }
    Passes { rows, few, first_seconds: Duration::from_nanos(last_end.into_inner()).as_secs_f64() }
}
because!(run_all, TaskBoard, "the jobs of the first pass and the pairs side by side on the threads, each job on one thread, the largest guess first so that no thread starts a large job when the others are near their end; a kind of the first pass that is learned, in a task that is counted, asks for its job with few lessons at once, so that the threads that would wait for the last long jobs of the first pass do that work; the rows come back in the order of the jobs");

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
}
because!(Counts, TaskBoard, "the numbers of a board: kinds and tasks, how many are learned and in how many sets, how many learned tasks have stronger lessons, are measured and pass with new words or hold no name at all, are learned from few lessons, hold from a small form to a large one, and are text that people wrote");

pub struct Board {
    pub text: Vec<String>,
    pub counts: Counts,
}
because!(Board, TaskBoard, "a board as the lines that are printed and as its numbers");

pub fn board(sources: &[Source], tools: &[Tool], asked: &Asked, classes: Option<&WordClasses>) -> Board {
    let started = Instant::now();
    let first_jobs: Vec<Job> = sources
        .iter()
        .enumerate()
        .flat_map(|(from, source)| source.tasks.iter().flat_map(move |(_, kinds)| kinds.iter().map(move |kind| Job { from, kind: kind.clone(), pair: None, fit: source.fit, new_words: true })))
        .collect();
    let pair_jobs: Vec<Job> = sources
        .iter()
        .enumerate()
        .flat_map(|(from, source)| source.pairs.iter().map(move |(task, small, large)| Job { from, kind: task.clone(), pair: Some((small.clone(), large.clone())), fit: source.fit, new_words: false }))
        .collect();
    let first_count = first_jobs.len();
    let mut together = first_jobs;
    together.extend(pair_jobs);
    let Passes { mut rows, few: mut few_by_job, first_seconds } = run_all(&together, first_count, sources, tools, asked, classes);
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
        .map(|(at, (job, _))| few_by_job[at].take().unwrap_or_else(|| run(&few_job(job), sources, tools, &alone, classes)))
        .collect();
    let seconds = started.elapsed().as_secs_f64();

    let row_of = |from: usize, kind: &str| first_jobs.iter().position(|job| job.from == from && job.kind == kind);
    let few_of = |from: usize, kind: &str| few_jobs.iter().position(|job| job.from == from && job.kind == kind).is_some_and(|at| kind_passed(&few_jobs[at], &few_rows[at], sources));
    let mut counts = Counts { kinds: first_jobs.len(), kinds_learned: is_learned.iter().filter(|learned| **learned).count(), ..Counts::default() };
    let mut sets: Vec<&str> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("set | task | text by | lessons | kinds learned | task learned | lowest share | control: question alone, highest | new words, lowest share | learned from {FEW} lessons | small to large | seconds"));
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
            let lessons = format!("text, answer{}{}", if with_labels { ", labels" } else { "" }, if source.stronger.is_empty() { String::new() } else { format!(", {}", source.stronger) });
            if whole {
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
                let mut notes: Vec<&str> = mine.iter().map(|row| row.note.as_str()).collect();
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
            lines.push(format!("{} | {task} | {by} | {lessons} | {got} of {} | {task_learned} | {low:.places$} | {question:.places$} | {with_words} | {from_few} | {large} | {all_seconds:.none$}", source.set, kinds.len(), places = SHARE_PLACES, none = 0));
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
            "{} kinds in {} tasks; {} kinds learned; {} tasks learned in {} sets; whole run {seconds:.none$} s on {} threads (first pass {first_seconds:.none$} s)",
            counts.kinds,
            counts.tasks,
            counts.kinds_learned,
            counts.tasks_learned,
            counts.sets,
            asked.lanes,
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
    text.push(format!("breadth: {} tasks learned in {} sets ({} of the tasks are learned with stronger lessons: labels of the lesson sentences that the set gives)", counts.tasks_learned, counts.sets, counts.labelled));
    text.push(format!(
        "new words: {} of {apply} learned tasks pass where new words apply ({} are measured, {} have no table that changes a read case and count as not passed; {} more tasks hold no name, place or thing, as their set says)",
        counts.word_passed,
        counts.word_tasks,
        apply - counts.word_tasks,
        counts.no_words
    ));
    text.push(format!("learn small, read large: {} tasks", counts.small_large));
    text.push(format!("few lessons: {} of {} learned tasks are learned from {FEW} lessons", counts.few, counts.tasks_learned));
    text.push(format!("text that people wrote: {} learned tasks ({} with stronger lessons)", counts.people, counts.people_labelled));
    Board { text, counts }
}
because!(board, TaskBoard, "the board in three passes: every kind with the lessons that its set names, read also with new words; every learned kind of a counted task again from few lessons; the pairs of a small and a large kind. A task is learned when all its kinds pass, and a task that passes by a regularity of its set is printed and not counted");
