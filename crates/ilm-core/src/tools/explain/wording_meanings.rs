use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::call::Learner;
use crate::tools::explain::chain::rules_key;
use crate::tools::explain::learn::model_of;
use crate::tools::explain::text::{Condition, LogicPiece, Topic, fact_words, read_topic, word_not_mark};
use crate::tools::explain::words::{word_text, word_texts};
use patterns::{because, source};
use std::collections::HashSet;

pub struct LearnedMeanings;
source!(LearnedMeanings, "the meanings of the function words of a kind learned from its lessons over the functions on lists that a text of the library names by what they do, as a child learns a new word by trying the meanings it knows on the cases where the word is heard (cross-situational learning, Siskind): a function word is a word of the questions that no answer holds; each is tried with each meaning of the text on a few lessons where it is the only word whose meaning is not yet known, and keeps the meaning that reads all of them right, one word after another until no word gains one (coordinate ascent); the engine reads each trial, so the meanings are those of the text, and the learned lines are words of the library that a person can read");

const FUNCTION_KIND: &[&str] = &["is", "a", "function", "word"];
because!(FUNCTION_KIND, LearnedMeanings, "the words after a word in the fact that names it a function word, which the text of functions on lists reads ('not B holds a function word') to find the innermost function");

const TRIAL_LESSONS: usize = 3;
because!(TRIAL_LESSONS, LearnedMeanings, "the lessons on which a meaning of a word is tried, those whose lists are longest first: enough that functions which act alike on a short list (a list of one word said as it is, backward or with its ends swapped) are told apart, few enough that the trials cost little");

const MEANINGS_NAME: &str = "meanings of function words";
because!(MEANINGS_NAME, LearnedMeanings, "what the note of an answer names as the text that gave it, so that a reader of the note tells a learned meaning from a written text");

const JOIN_MARK: &str = "-";
because!(JOIN_MARK, LearnedMeanings, "the mark that joins the words of a function word of several words into one word, as a hyphen between two letters joins a word, so that a place of one word holds it");

const MEANING_END: &str = ".";
because!(MEANING_END, LearnedMeanings, "the mark that ends each learned fact of a meaning");

const JOIN_START: &str = "Write AAAA ";
because!(JOIN_START, LearnedMeanings, "the words that open a learned line that joins a function word of several words, with the run before it");

const JOIN_AS: &str = " BBBB as AAAA ";
because!(JOIN_AS, LearnedMeanings, "the words between the words of a function word and its joined form in a learned line, with the run after it");

const JOIN_END: &str = " BBBB.";
because!(JOIN_END, LearnedMeanings, "the end of a learned line that joins a function word, with the run after it");

const FUNCTION_FACT: &str = " is a function word.";
because!(FUNCTION_FACT, LearnedMeanings, "the end of the learned fact that names a function word");

const VALUE_ANSWER: &str = "To answer \"AAAA\": B if the value is B , end and not B holds a function word.";
because!(VALUE_ANSWER, LearnedMeanings, "the learned question form: any question is answered by its value once no function word is left in it, as the text of functions on lists writes the value of a question");

struct FunctionWord {
    words: Vec<String>,
    joined: String,
}
because!(FunctionWord, LearnedMeanings, "a function word of a kind: its words as the rules read them, and the one word that joins them");

fn functions_text(topics: &[Topic]) -> Option<&Topic> {
    topics.iter().find(|topic| topic.questions.is_empty() && topic.rules.iter().any(|rule| rule.absent.iter().any(|condition| matches!(condition, Condition::HoldsKind { kind, .. } if word_texts(kind) == FUNCTION_KIND))))
}
because!(functions_text, LearnedMeanings, "the text of the library that names the functions on lists, found by the condition that its lines read: that a list holds no function word");

fn meanings_of(text: &Topic) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for rule in &text.rules {
        for condition in rule.conditions.iter().skip(1) {
            let Condition::Fact { pattern, .. } = condition else { continue };
            let [LogicPiece::OpenWithEnd(_, ending), rest @ ..] = pattern.as_slice() else { continue };
            let words: Option<Vec<&str>> = rest.iter().map(|logic_piece| if let LogicPiece::Word(word) = logic_piece { Some(word_text(*word)) } else { None }).collect();
            if let Some(words) = words.filter(|words| ending.is_empty() && !words.is_empty() && words.as_slice() != &FUNCTION_KIND[..]) {
                let meaning = words.join(" ");
                if !found.contains(&meaning) {
                    found.push(meaning);
                }
            }
        }
    }
    found
}
because!(meanings_of, LearnedMeanings, "the meanings that the text of functions on lists names, in the order of its lines: the words after the place of the function in the first condition of each line that reads a function, each meaning once");

fn function_words(cases: &[&Case]) -> Vec<FunctionWord> {
    let answered: HashSet<String> = cases.iter().flat_map(|case| case.answer.split_whitespace().map(str::to_lowercase)).collect();
    let mut seen: Vec<FunctionWord> = Vec::new();
    for case in cases {
        for token in case.question.split_whitespace() {
            let low = token.to_lowercase();
            if !word_not_mark(&low) || answered.contains(&low) {
                continue;
            }
            let words = fact_words(&low);
            if words.is_empty() || words.iter().any(|word| !word.chars().all(char::is_alphabetic)) || seen.iter().any(|known| known.words == words) {
                continue;
            }
            seen.push(FunctionWord { joined: words.join(JOIN_MARK), words });
        }
    }
    seen
}
because!(function_words, LearnedMeanings, "the function words of the lessons: the words of the questions, as they are written, that no answer holds, each with its words as the rules read them (a low line parts them) and the one word that joins them; only words of letters, so that the hyphen joins them into one word");

fn learned_text(functions: &[FunctionWord], meanings: &[String], chosen: &[Option<usize>]) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut joined: Vec<&FunctionWord> = functions.iter().filter(|function| function.words.len() > 1).collect();
    joined.sort_by(|one, other| other.words.len().cmp(&one.words.len()));
    lines.extend(joined.iter().map(|function| format!("{JOIN_START}{}{JOIN_AS}{}{JOIN_END}", function.words.join(" "), function.joined)));
    lines.extend(functions.iter().map(|function| format!("{}{FUNCTION_FACT}", function.joined)));
    lines.extend(functions.iter().zip(chosen).filter_map(|(function, meaning)| meaning.map(|at| format!("{} {}{MEANING_END}", function.joined, meanings[at]))));
    lines.push(VALUE_ANSWER.to_string());
    lines.join("\n")
}
because!(learned_text, LearnedMeanings, "the learned lines: a line that joins each function word of several words (the longest first), a fact that names each function word, a fact of the meaning of each word that has one, and the question form of the value");

fn with_text(text: &Topic, lines: &str) -> Option<Topic> {
    let read = read_topic(MEANINGS_NAME, lines).ok()?;
    let rules: Vec<_> = read.rules.iter().chain(&text.rules).cloned().collect();
    let facts: Vec<Vec<_>> = text.facts.iter().chain(&read.facts).cloned().collect();
    Some(Topic { name: MEANINGS_NAME.to_string(), rules_key: rules_key(&rules), rules, facts, questions: read.questions, reply_lines: read.reply_lines, builds_on: Vec::new(), lines: read.lines + text.lines })
}
because!(with_text, LearnedMeanings, "the learned lines read as a text of the library with the text of functions on lists under them: the learned lines first, so that a function word is joined before a function is read, then the lines of the text; the facts of both, and the learned question form");

fn reads_all(text: &Topic, lines: &str, cases: &[&Case]) -> bool {
    let Some(topic) = with_text(text, lines) else { return false };
    let model = model_of(vec![topic]);
    cases.iter().all(|case| model.answer(&case.sentences, &case.question).words == answer_words(&case.answer))
}
because!(reads_all, LearnedMeanings, "whether the engine reads every one of some lessons right with the learned lines over the text of functions on lists");

fn held_functions(cases: &[&Case], functions: &[FunctionWord]) -> Vec<Vec<usize>> {
    cases
        .iter()
        .map(|case| {
            let tokens: Vec<Vec<String>> = case.question.split_whitespace().map(|token| fact_words(&token.to_lowercase())).collect();
            (0..functions.len()).filter(|at| tokens.contains(&functions[*at].words)).collect()
        })
        .collect()
}
because!(held_functions, LearnedMeanings, "the function words that each lesson holds, found once for every trial");

fn trial_cases<'a>(cases: &[&'a Case], held: &[Vec<usize>], known: &[bool], word: usize) -> Vec<&'a Case> {
    let mut fitting: Vec<&Case> = cases.iter().zip(held).filter(|(_, held)| held.contains(&word) && held.iter().all(|at| *at == word || known[*at])).map(|(case, _)| *case).collect();
    fitting.sort_by_key(|case| std::cmp::Reverse(case.answer.split_whitespace().count()));
    fitting.truncate(TRIAL_LESSONS);
    fitting
}
because!(trial_cases, LearnedMeanings, "the lessons on which a meaning of one word is tried: those that hold the word and no other function word whose meaning is not yet known, the longest answers first, at most a few");

pub fn meanings_topic(topics: &[Topic], cases: &[&Case]) -> Option<Topic> {
    let (text, lines) = meanings_lines(topics, cases)?;
    with_text(text, &lines)
}
because!(meanings_topic, LearnedMeanings, "the learned lines of the meanings of a kind read as a text of the library over the text of functions on lists, and nothing when no word gains a meaning");

pub fn meanings_lines<'a>(topics: &'a [Topic], cases: &[&Case]) -> Option<(&'a Topic, String)> {
    if cases.iter().any(|case| !case.sentences.is_empty()) {
        return None;
    }
    let text = functions_text(topics)?;
    let meanings = meanings_of(text);
    let functions = function_words(cases);
    if functions.is_empty() || meanings.is_empty() {
        return None;
    }
    let held = held_functions(cases, &functions);
    let mut chosen: Vec<Option<usize>> = vec![None; functions.len()];
    loop {
        let mut gained = false;
        for word in 0..functions.len() {
            if chosen[word].is_some() {
                continue;
            }
            let known: Vec<bool> = chosen.iter().map(Option::is_some).collect();
            let trials = trial_cases(cases, &held, &known, word);
            if trials.is_empty() {
                continue;
            }
            let found = (0..meanings.len()).find(|meaning| {
                let mut tried = chosen.clone();
                tried[word] = Some(*meaning);
                reads_all(text, &learned_text(&functions, &meanings, &tried), &trials)
            });
            if found.is_some() {
                chosen[word] = found;
                gained = true;
            }
        }
        if !gained {
            break;
        }
    }
    if chosen.iter().all(Option::is_none) {
        return None;
    }
    Some((text, learned_text(&functions, &meanings, &chosen)))
}
because!(meanings_lines, LearnedMeanings, "the learned lines of a kind whose questions are function words over lists, with no told sentences, and the text of functions on lists that they build on: the meaning of each function word, tried word by word on a few lessons where it is the only word not yet known, the first meaning of the text that reads them all, in rounds until no word gains one; nothing when the library has no text of functions on lists, the lessons tell sentences, no word of the questions is missing from the answers, or no word gains a meaning");
