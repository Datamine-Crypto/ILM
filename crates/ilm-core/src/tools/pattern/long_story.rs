use crate::base::lanes::hardware_lanes;
use crate::base::words::answer_words;
use crate::split::parts;
use crate::cases::{Case, head_value, read_kind};
use crate::tools::all_tools;
use crate::tools::call::{Learner, Lessons, read_story};
use crate::cases::START_NAME;
use patterns::{because, source};
use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub struct LongStory;
source!(LongStory, "a model with no window: a story is read sentence by sentence into words and an index, and a rule points at a sentence by its words and its form wherever it stands, so the facts that a question needs can stand any distance back; measured with filler of other stories whose people and things carry marks of their own, so that the filler says nothing about what the question asks");

const LESSONS: usize = 1000;
because!(LESSONS, LongStory, "the lessons of the model of the measure: the most lessons of a kind that a learner counts on");

const HELD: usize = 400;
because!(HELD, LongStory, "the read cases of a kind, as the measure of the kinds takes them");

const FAR_LENGTHS: [usize; 4] = [1_000, 10_000, 100_000, 1_000_000];
because!(FAR_LENGTHS, LongStory, "the lengths of a story in words for one question with its facts at the start: each ten times the one before, up to a story far longer than any window of a network");

const MANY_LENGTHS: [usize; 3] = [10_000, 100_000, 1_000_000];
because!(MANY_LENGTHS, LongStory, "the lengths of a story in words that is read once and asked about people all through it");

const CHECK_WORDS: usize = 20_000;
because!(CHECK_WORDS, LongStory, "the length of the story on which every answer of the index is compared with the answer of the model for the whole story");

const CHECK_QUESTIONS: usize = 200;
because!(CHECK_QUESTIONS, LongStory, "the questions of the check of the index against the model");

const FAR_CASES: usize = 10;
because!(FAR_CASES, LongStory, "the read cases for each length of the measure with the facts at the start");

const MOST_SECONDS: f64 = 120.0;
because!(MOST_SECONDS, LongStory, "a length of the measure with the facts at the start stops when one answer takes longer than this many seconds");

const MANY_QUESTIONS: usize = 300;
because!(MANY_QUESTIONS, LongStory, "the questions asked of one long story, spread over all of it");

const SPREAD_STEP: usize = 7919;
because!(SPREAD_STEP, LongStory, "a prime that spreads the lesson stories that fill one story, so that two read cases take different filler");

const THOUSANDS: f64 = 1000.0;
because!(THOUSANDS, LongStory, "milliseconds in a second, for the time of one question");

fn marked_text(text: &str, own: &HashSet<String>, mark: usize) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / LETTERS_PER_MARK);
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if !word.is_empty() {
            out.push_str(word);
            if own.contains(&word.to_lowercase()) {
                out.push_str(&format!("x{mark}"));
            }
            word.clear();
        }
    };
    for sign in text.chars() {
        if sign.is_ascii_alphabetic() {
            word.push(sign);
        } else {
            flush(&mut word, &mut out);
            out.push(sign);
        }
    }
    flush(&mut word, &mut out);
    out
}
because!(marked_text, LongStory, "a text with each word of a person or a thing marked with the number of its story, so that the people and things of one story are words of their own");

const LETTERS_PER_MARK: usize = 8;
because!(LETTERS_PER_MARK, LongStory, "room for the marks of a marked_text text: about one mark for each few letters");

fn word_count(texts: &[String]) -> usize {
    texts.iter().map(|text| text.split_whitespace().count()).sum()
}
because!(word_count, LongStory, "the words of some texts as a reader counts them, cut at white space");

fn long_story_of(read: &[Case], own: &HashSet<String>, wanted: usize) -> (Vec<String>, Vec<(String, String)>) {
    let (mut sentences, mut questions) = (Vec::new(), Vec::new());
    let (mut count, mut mark) = (0, 0);
    while count < wanted {
        let case = &read[mark % read.len()];
        mark += 1;
        for sentence in &case.sentences {
            let new = marked_text(sentence, own, mark);
            count += new.split_whitespace().count();
            sentences.push(new);
        }
        questions.push((marked_text(&case.question, own, mark), marked_text(&case.answer, own, mark)));
    }
    (sentences, questions)
}
because!(long_story_of, LongStory, "one long story of read cases one after the other, each with its people and things marked as its own, and the question of each with its answer");

fn far(model: &dyn Learner, learn: &[Case], read: &[Case], own: &HashSet<String>) {
    let others: Vec<&Vec<String>> = learn.iter().map(|case| &case.sentences).collect();
    for length in FAR_LENGTHS {
        let (mut right, mut done) = (0, 0);
        let mut seconds: Vec<f64> = Vec::new();
        let mut size = 0;
        for (number, case) in read.iter().take(FAR_CASES).enumerate() {
            let mut story = case.sentences.clone();
            let mut count = word_count(&story);
            let mut mark = 0;
            while count < length {
                mark += 1;
                for sentence in others[(number * SPREAD_STEP + mark) % others.len()] {
                    let new = marked_text(sentence, own, mark);
                    count += new.split_whitespace().count();
                    story.push(new);
                }
            }
            size = story.len();
            let started = Instant::now();
            let said = model.answer(&story, &case.question);
            seconds.push(started.elapsed().as_secs_f64());
            right += usize::from(said.words == answer_words(&case.answer));
            done += 1;
            if seconds[seconds.len() - 1] > MOST_SECONDS {
                break;
            }
        }
        let longest = seconds.iter().copied().fold(f64::default(), f64::max);
        let mean = seconds.iter().sum::<f64>() / seconds.len().max(1) as f64;
        println!("   {length} words ({size} sentences): {right} of {done} right; seconds for one answer: {mean:.2} (the longest {longest:.2})");
        if longest > MOST_SECONDS {
            println!("   stopped: one answer took longer than the bound");
            break;
        }
    }
}
because!(far, LongStory, "one question for a story: the story of a read case stands first, then lesson stories with words of their own until the length is reached, then the question; the story is read again for each question; for each length the answers right and the seconds of one answer");

fn many(model: &dyn Learner, read: &[Case], own: &HashSet<String>) {
    let (sentences, questions) = long_story_of(read, own, CHECK_WORDS);
    let mut story = read_story(model, &sentences);
    let checked = &questions[..questions.len().min(CHECK_QUESTIONS)];
    let same = checked.iter().filter(|(question, _)| story.ask(question) == model.answer(&sentences, question)).count();
    println!("   check on a story of {CHECK_WORDS} words: the same words and note as the model for {same} of {} questions", checked.len());
    for length in MANY_LENGTHS {
        let (sentences, questions) = long_story_of(read, own, length);
        let started = Instant::now();
        let mut story = read_story(model, &sentences);
        let read_seconds = started.elapsed().as_secs_f64();
        let step = (questions.len() / MANY_QUESTIONS).max(1);
        let picked: Vec<&(String, String)> = questions.iter().step_by(step).take(MANY_QUESTIONS).collect();
        let started = Instant::now();
        let right = picked.iter().filter(|(question, answer)| story.ask(question).words == answer_words(answer)).count();
        let ask_seconds = started.elapsed().as_secs_f64();
        println!(
            "   {length} words, {} sentences, {} stories in it: read once in {read_seconds:.2} s; {} questions about people all through the story: {right} right; {:.1} ms for a question",
            sentences.len(),
            questions.len(),
            picked.len(),
            THOUSANDS * ask_seconds / picked.len().max(1) as f64
        );
    }
}
because!(many, LongStory, "one story for many questions: a check that every answer of the story read once is the answer of the model for the whole story, then for each length the seconds to read the story once, the answers right and the time of one question");

pub fn long_story_command(given: &[String]) -> ExitCode {
    let [which, path, own_path] = given else {
        eprintln!("usage: ilm patterns long-story far|many <case file of a kind> <file of the words of people and things, one on a line>");
        return ExitCode::FAILURE;
    };
    let (kind, own) = match (read_kind(Path::new(path)), std::fs::read_to_string(own_path)) {
        (Ok(kind), Ok(text)) => (kind, text.lines().map(|line| line.trim().to_lowercase()).filter(|line| !line.is_empty()).collect::<HashSet<String>>()),
        _ => {
            eprintln!("{path} or {own_path} can not be read");
            return ExitCode::FAILURE;
        }
    };
    let start = head_value(&kind, START_NAME).and_then(|text| text.parse::<usize>().ok());
    let stories = head_value(&kind, FIT_STORIES_NAME).and_then(|text| text.parse::<usize>().ok()) == Some(1);
    let cut = parts(&kind.cases, LESSONS, HELD, start, stories);
    let learn: Vec<Case> = cut.learn.iter().map(|place| kind.cases[*place].clone()).collect();
    let read: Vec<Case> = cut.read.iter().map(|place| kind.cases[*place].clone()).collect();
    let started = Instant::now();
    let taken = match crate::choice::take(&all_tools(), Lessons::of(&learn), hardware_lanes(), false) {
        Ok(taken) => taken,
        Err(why) => {
            eprintln!("{why}");
            return ExitCode::FAILURE;
        }
    };
    let told: usize = learn.iter().map(|case| case.sentences.len()).sum();
    println!("{path}: {} lessons, tool {}, model in {:.0} s; a lesson story has {:.0} sentences", learn.len(), taken.named, started.elapsed().as_secs_f64(), told as f64 / learn.len().max(1) as f64);
    match which.as_str() {
        "far" => far(&taken, &learn, &read, &own),
        _ => many(&taken, &read, &own),
    }
    ExitCode::SUCCESS
}
because!(long_story_command, LongStory, "the measures of a very long story for one kind: far, with the facts at the start of a story of up to a million words, and many, with one story read once and asked about people all through it; the model is the one of the choice among all tools");


const FIT_STORIES_NAME: &str = "fit_stories";
because!(FIT_STORIES_NAME, LongStory, "the name of the value of a case file that says that the lessons are counted in stories and not in cases");
