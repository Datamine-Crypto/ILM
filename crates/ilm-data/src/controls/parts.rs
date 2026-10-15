use ilm_core::base::first_sight::FirstSightCount;
use ilm_core::base::words::answer_words;
use crate::controls::file::ChangedCase;
use crate::controls::sentences::{LiesForm, PassiveForm, QuestionForm};
use crate::controls::text::paired;
use crate::sets::readers::common::python_words;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::HashSet;

pub struct PartsChanges;
source!(PartsChanges, "changed texts of the families of rows, each read by an exact reading that no learner sees and that must give the answer of the set for a lesson before the lesson is changed. The commands of moves: the interpretation function of their grammar; one word becomes another word of its class, one word is taken out, the two sides of a joining word change places. Brackets to close: a stack of the closers of the open brackets; one more opener at the end, the first closer of the answer put at the end, the last opener that is still open made another opener. Functions on rows in prefix notation: the functions of the grammar of the set; a function word becomes another with as many parts, a function with one part is taken out, the two parts of the first function with two parts change places, the first and the last item get a number that no lesson holds. Whether brackets are matched: one bracket taken out, one made another sort on the same side, one matched pair put at the end. The two answer words of a class kind, and the forms of the sentences of the other kinds, are taken from the lessons. Where the lessons have no told sentence, a changed text that no lesson of the model holds is a sort of its own");

const PARTS_SCAN_KINDS: &[&str] = &["scan_simple", "scan_length", "scan_addprim_jump", "scan_addprim_turn_left"];
because!(PARTS_SCAN_KINDS, PartsChanges, "the four cuts of the set of commands of moves into lessons and tests, which one grammar reads");

const PARTS_PCFG_KINDS: &[&str] = &["pcfg_plain", "pcfg_longer", "pcfg_new"];
because!(PARTS_PCFG_KINDS, PartsChanges, "the kinds of functions on rows in prefix notation");

const PARTS_DYCK: &str = "dyck_languages";
because!(PARTS_DYCK, PartsChanges, "the kind whose answer closes the open brackets");

const PARTS_MATCHED: &str = "cs_parentheses";
because!(PARTS_MATCHED, PartsChanges, "the kind whose answer says whether the brackets are matched");

const PARTS_LIES: &str = "web_of_lies";
because!(PARTS_LIES, PartsChanges, "the kind of a chain of statements about who tells the truth");

const PARTS_PASSIVE: &str = "passiv_en_test";
because!(PARTS_PASSIVE, PartsChanges, "the kind of a sentence into the passive");

const PARTS_QUESTION_KINDS: &[&str] = &["quest_en_test", "quest_de_test"];
because!(PARTS_QUESTION_KINDS, PartsChanges, "the English and the German tests where a sentence becomes a question with its helping verb first");

const SCAN_ACTIONS: &[(&str, &str)] = &[("walk", "I_WALK"), ("look", "I_LOOK"), ("run", "I_RUN"), ("jump", "I_JUMP")];
because!(SCAN_ACTIONS, PartsChanges, "the words of the actions of a command and the moves that they stand for");

const SCAN_TURNS: &[(&str, &str)] = &[("left", "I_TURN_LEFT"), ("right", "I_TURN_RIGHT")];
because!(SCAN_TURNS, PartsChanges, "the words of the sides of a command and the turns that they stand for");

const SCAN_TURN: &str = "turn";
because!(SCAN_TURN, PartsChanges, "the action word that makes no move of its own, only the turn");

const SCAN_OPPOSITE: &str = "opposite";
because!(SCAN_OPPOSITE, PartsChanges, "the word that turns twice before the action");

const SCAN_AROUND: &str = "around";
because!(SCAN_AROUND, PartsChanges, "the word that turns and acts once for each side of a square");

const SCAN_SQUARE: usize = 4;
because!(SCAN_SQUARE, PartsChanges, "the turns of a turn around: one for each side of a square");

const SCAN_TWICE_TIMES: usize = 2;
because!(SCAN_TWICE_TIMES, PartsChanges, "the times that a phrase is done when twice follows it");

const SCAN_THRICE_TIMES: usize = 3;
because!(SCAN_THRICE_TIMES, PartsChanges, "the times that a phrase is done when thrice follows it");

const SCAN_SHORT_PHRASE: usize = 2;
because!(SCAN_SHORT_PHRASE, PartsChanges, "the words of a phrase of an action or a turn with a side and nothing between them");

const SCAN_TWICE: &str = "twice";
because!(SCAN_TWICE, PartsChanges, "the word that does its phrase again");

const SCAN_THRICE: &str = "thrice";
because!(SCAN_THRICE, PartsChanges, "the word that does its phrase again and once more");

const SCAN_AND: &str = "and";
because!(SCAN_AND, PartsChanges, "the word that joins two phrases in their order");

const SCAN_AFTER: &str = "after";
because!(SCAN_AFTER, PartsChanges, "the word that joins two phrases in the other order");

const SCAN_CLASSES: &[&[&str]] = &[&["walk", "look", "run", "jump", "turn"], &["left", "right"], &["twice", "thrice"], &["and", "after"], &["opposite", "around"]];
because!(SCAN_CLASSES, PartsChanges, "the classes of words of the grammar of commands: a word becomes another word of its class");

const SCAN_LONGEST_PHRASE: usize = 3;
because!(SCAN_LONGEST_PHRASE, PartsChanges, "the words of the longest phrase with a side: an action, opposite or around, and the side");

const BRACKET_PAIRS: &[(&str, &str)] = &[("(", ")"), ("[", "]"), ("{", "}"), ("<", ">")];
because!(BRACKET_PAIRS, PartsChanges, "the openers of brackets with their closers, in their order");

const PCFG_COPY: &str = "copy";
because!(PCFG_COPY, PartsChanges, "the function of one part that gives the same row");

const PCFG_REVERSE: &str = "reverse";
because!(PCFG_REVERSE, PartsChanges, "the function of one part that gives the row backwards");

const PCFG_SHIFT: &str = "shift";
because!(PCFG_SHIFT, PartsChanges, "the function of one part that moves the first item to the end");

const PCFG_ECHO: &str = "echo";
because!(PCFG_ECHO, PartsChanges, "the function of one part that gives the last item once more");

const PCFG_SWAP: &str = "swap_first_last";
because!(PCFG_SWAP, PartsChanges, "the function of one part that changes the first and the last item over");

const PCFG_REPEAT: &str = "repeat";
because!(PCFG_REPEAT, PartsChanges, "the function of one part that gives the row twice");

const PCFG_APPEND: &str = "append";
because!(PCFG_APPEND, PartsChanges, "the function of two parts that gives the first row and then the second");

const PCFG_PREPEND: &str = "prepend";
because!(PCFG_PREPEND, PartsChanges, "the function of two parts that gives the second row and then the first");

const PCFG_REMOVE_FIRST: &str = "remove_first";
because!(PCFG_REMOVE_FIRST, PartsChanges, "the function of two parts that gives the second row alone");

const PCFG_REMOVE_SECOND: &str = "remove_second";
because!(PCFG_REMOVE_SECOND, PartsChanges, "the function of two parts that gives the first row alone");

const PCFG_ONE: &[&str] = &[PCFG_COPY, PCFG_REVERSE, PCFG_SHIFT, PCFG_ECHO, PCFG_SWAP, PCFG_REPEAT];
because!(PCFG_ONE, PartsChanges, "the functions of one part in their order, for the change of a function into another");

const PCFG_TWO: &[&str] = &[PCFG_APPEND, PCFG_PREPEND, PCFG_REMOVE_FIRST, PCFG_REMOVE_SECOND];
because!(PCFG_TWO, PartsChanges, "the functions of two parts in their order, for the change of a function into another");

const PCFG_COMMA: &str = ",";
because!(PCFG_COMMA, PartsChanges, "the mark between the two parts of a function");

const PCFG_NEW_NUMBER: u32 = 21;
because!(PCFG_NEW_NUMBER, PartsChanges, "the number that an item gets so that no lesson holds it: higher than the numbers of the items of the set");

const PARTS_GAP: &str = " ";
because!(PARTS_GAP, PartsChanges, "the mark between the words of a changed text and of a new truth");

const PARTS_NEW_SORT: &str = ", no lesson of the model";
because!(PARTS_NEW_SORT, PartsChanges, "the end of the name of a sort for a changed text that no lesson of the model holds");

const SORT_PARTS_WORD: &str = "word";
because!(SORT_PARTS_WORD, PartsChanges, "the sort of the change where a word becomes another word of its class or a function another function");

const SORT_PARTS_OUT: &str = "out";
because!(SORT_PARTS_OUT, PartsChanges, "the sort of the change where a word, a function or a bracket is taken out");

const SORT_PARTS_SIDES: &str = "sides";
because!(SORT_PARTS_SIDES, PartsChanges, "the sort of the change where two sides or two parts change places");

const SORT_PARTS_ITEM: &str = "item";
because!(SORT_PARTS_ITEM, PartsChanges, "the sort of the change where an item gets a number that no lesson holds");

const SORT_PARTS_OPEN: &str = "open";
because!(SORT_PARTS_OPEN, PartsChanges, "the sort of the change where one more opener stands at the end");

const SORT_PARTS_CLOSE: &str = "close";
because!(SORT_PARTS_CLOSE, PartsChanges, "the sort of the change where the first closer of the answer stands at the end");

const SORT_PARTS_KIND: &str = "kind";
because!(SORT_PARTS_KIND, PartsChanges, "the sort of the change where a bracket becomes a bracket of another sort");

const SORT_PARTS_PAIR: &str = "pair";
because!(SORT_PARTS_PAIR, PartsChanges, "the sort of the change where a matched pair stands at the end");

pub type PartsChange = (String, Vec<String>, String);
because!(PartsChange, PartsChanges, "one changed case: the name of its change, its told sentences and its question");

fn scan_move(table: &[(&str, &str)], word: &str) -> Option<String> {
    table.iter().find(|(held, _)| *held == word).map(|(_, said)| said.to_string())
}
because!(scan_move, PartsChanges, "the move of a word of a table, or nothing for a word that the table does not hold");

fn scan_simple(words: &[&str]) -> Option<Vec<String>> {
    let action = words.first().and_then(|word| scan_move(SCAN_ACTIONS, word));
    if words.len() == 1 {
        return action.map(|said| vec![said]);
    }
    let turn = scan_move(SCAN_TURNS, words.last()?)?;
    if !(1..SCAN_LONGEST_PHRASE).contains(&(words.len() - 1)) || (action.is_none() && words[0] != SCAN_TURN) {
        return None;
    }
    let acted: Vec<String> = action.into_iter().collect();
    let mut out: Vec<String> = Vec::new();
    if words.len() == SCAN_SHORT_PHRASE {
        out.push(turn);
        out.extend(acted);
        return Some(out);
    }
    if words[1] == SCAN_OPPOSITE {
        out.push(turn.clone());
        out.push(turn);
        out.extend(acted);
        return Some(out);
    }
    if words[1] == SCAN_AROUND {
        for _ in 0..SCAN_SQUARE {
            out.push(turn.clone());
            out.extend(acted.iter().cloned());
        }
        return Some(out);
    }
    None
}
because!(scan_simple, PartsChanges, "the moves of a phrase with no repetition: an action alone, or an action or a turn with a side, with opposite or around before the side");

fn scan_phrase(words: &[&str]) -> Option<Vec<String>> {
    let times = match words.last() {
        Some(&SCAN_THRICE) => SCAN_THRICE_TIMES,
        Some(&SCAN_TWICE) => SCAN_TWICE_TIMES,
        _ => 1,
    };
    let said = scan_simple(if times > 1 { &words[..words.len() - 1] } else { words })?;
    Some((0..times).flat_map(|_| said.iter().cloned()).collect())
}
because!(scan_phrase, PartsChanges, "the moves of a part of a command between joining words, done once, twice or thrice");

fn scan_exact(question: &str) -> Option<Vec<String>> {
    let words = python_words(question);
    let joins: Vec<usize> = words.iter().enumerate().filter(|(_, word)| [SCAN_AND, SCAN_AFTER].contains(word)).map(|(at, _)| at).collect();
    let said = match joins[..] {
        [] => scan_phrase(&words)?,
        [at] => {
            let (left, right) = (scan_phrase(&words[..at])?, scan_phrase(&words[at + 1..])?);
            if words[at] == SCAN_AND { [left, right].concat() } else { [right, left].concat() }
        }
        _ => return None,
    };
    Some(answer_words(&said.join(PARTS_GAP)))
}
because!(scan_exact, PartsChanges, "the moves of a command as the words of its answer, or nothing when the grammar does not hold the command");

fn scan_changes(sentences: &[String], question: &str) -> Vec<PartsChange> {
    let words = python_words(question);
    let mut out: Vec<PartsChange> = Vec::new();
    let joined = |parts: Vec<&str>| parts.join(PARTS_GAP);
    for (at, word) in words.iter().enumerate() {
        for group in SCAN_CLASSES.iter().filter(|group| group.contains(word)) {
            for other in group.iter().filter(|other| *other != word) {
                let changed = [&words[..at], &[*other], &words[at + 1..]].concat();
                out.push((SORT_PARTS_WORD.to_string(), sentences.to_vec(), joined(changed)));
            }
        }
        out.push((SORT_PARTS_OUT.to_string(), sentences.to_vec(), joined([&words[..at], &words[at + 1..]].concat())));
        if [SCAN_AND, SCAN_AFTER].contains(word) {
            out.push((SORT_PARTS_SIDES.to_string(), sentences.to_vec(), joined([&words[at + 1..], &[*word], &words[..at]].concat())));
        }
    }
    out
}
because!(scan_changes, PartsChanges, "the changed forms of a command: each word made each other word of its class, each word taken out, and the sides of a joining word changed over");

fn is_bracket(word: &str) -> bool {
    BRACKET_PAIRS.iter().any(|(opener, closer)| *opener == word || *closer == word)
}
because!(is_bracket, PartsChanges, "whether a word is an opener or a closer");

fn brackets_of<'a>(sentences: &'a [String], question: &'a str) -> Vec<&'a str> {
    sentences.iter().map(String::as_str).chain([question]).flat_map(python_words).filter(|word| is_bracket(word)).collect()
}
because!(brackets_of, PartsChanges, "the brackets of the told sentences and of the question, in their order");

fn stack_of(signs: &[&str]) -> Option<Vec<&'static str>> {
    let mut stack: Vec<&'static str> = Vec::new();
    for sign in signs {
        match paired(BRACKET_PAIRS, sign) {
            Some(closer) => stack.insert(0, closer),
            None if stack.first() == Some(sign) => {
                stack.remove(0);
            }
            None => return None,
        }
    }
    Some(stack)
}
because!(stack_of, PartsChanges, "the closers of the openers that are still open, the newest first; nothing when a closer does not fit");

fn bracket_place(sentences: &[String]) -> Option<usize> {
    sentences.iter().rposition(|sentence| python_words(sentence).iter().any(|word| is_bracket(word)))
}
because!(bracket_place, PartsChanges, "the place of the last told sentence that holds a bracket: the input of a case of brackets");

fn with_words(sentences: &[String], place: usize, words: &[&str]) -> Vec<String> {
    let mut out = sentences.to_vec();
    out[place] = words.join(PARTS_GAP);
    out
}
because!(with_words, PartsChanges, "the told sentences with new words in place of the sentence at a place");

fn dyck_exact(sentences: &[String], question: &str) -> Option<Vec<String>> {
    let stack = stack_of(&brackets_of(sentences, question))?;
    (!stack.is_empty()).then(|| answer_words(&stack.join(PARTS_GAP)))
}
because!(dyck_exact, PartsChanges, "the closers that the open brackets need, as the words of the answer; nothing when none is open or a closer does not fit");

fn dyck_changes(sentences: &[String], question: &str) -> Vec<PartsChange> {
    let Some(place) = bracket_place(sentences) else { return Vec::new() };
    let words = python_words(&sentences[place]);
    let mut out: Vec<PartsChange> = Vec::new();
    for (opener, _) in BRACKET_PAIRS {
        out.push((SORT_PARTS_OPEN.to_string(), with_words(sentences, place, &[&words[..], &[*opener]].concat()), question.to_string()));
    }
    let stack = stack_of(&brackets_of(sentences, question)).unwrap_or_default();
    if stack.len() > 1 {
        out.push((SORT_PARTS_CLOSE.to_string(), with_words(sentences, place, &[&words[..], &[stack[0]]].concat()), question.to_string()));
    }
    if stack.is_empty() {
        return out;
    }
    let mut depth = 0usize;
    let mut open_at: Option<usize> = None;
    for at in (0..words.len()).rev() {
        if BRACKET_PAIRS.iter().any(|(_, closer)| *closer == words[at]) {
            depth += 1;
        } else if paired(BRACKET_PAIRS, words[at]).is_some() {
            if depth == 0 {
                open_at = Some(at);
                break;
            }
            depth -= 1;
        }
    }
    if let Some(at) = open_at {
        for (opener, _) in BRACKET_PAIRS.iter().filter(|(opener, _)| *opener != words[at]) {
            out.push((SORT_PARTS_KIND.to_string(), with_words(sentences, place, &[&words[..at], &[*opener], &words[at + 1..]].concat()), question.to_string()));
        }
    }
    out
}
because!(dyck_changes, PartsChanges, "the changed forms of a case of brackets: each opener once more at the end, the first closer of the answer at the end when the answer has more than one closer, and the last opener that is still open made each other opener");

fn pcfg_one(function: &str, row: Vec<String>) -> Vec<String> {
    let length = row.len();
    let head = length.min(1);
    let last = &row[length.saturating_sub(1)..];
    match function {
        PCFG_REVERSE => row.iter().rev().cloned().collect(),
        PCFG_SHIFT => [&row[head..], &row[..head]].concat(),
        PCFG_ECHO => [&row[..], last].concat(),
        PCFG_SWAP if length > 1 => [last, &row[1..length - 1], &row[..1]].concat(),
        PCFG_REPEAT => [&row[..], &row[..]].concat(),
        _ => row,
    }
}
because!(pcfg_one, PartsChanges, "the row that a function of one part gives");

fn pcfg_two(function: &str, first: Vec<String>, second: Vec<String>) -> Vec<String> {
    match function {
        PCFG_APPEND => [first, second].concat(),
        PCFG_PREPEND => [second, first].concat(),
        PCFG_REMOVE_FIRST => second,
        _ => first,
    }
}
because!(pcfg_two, PartsChanges, "the row that a function of two parts gives");

fn pcfg_row_at(words: &[&str], at: usize) -> Option<(Vec<String>, usize)> {
    let word = *words.get(at)?;
    if PCFG_ONE.contains(&word) {
        let (row, end) = pcfg_row_at(words, at + 1)?;
        return Some((pcfg_one(word, row), end));
    }
    if PCFG_TWO.contains(&word) {
        let (first, middle) = pcfg_row_at(words, at + 1)?;
        if words.get(middle) != Some(&PCFG_COMMA) {
            return None;
        }
        let (second, end) = pcfg_row_at(words, middle + 1)?;
        return Some((pcfg_two(word, first, second), end));
    }
    let end = (at..words.len()).find(|place| words[*place] == PCFG_COMMA || PCFG_ONE.contains(&words[*place]) || PCFG_TWO.contains(&words[*place])).unwrap_or(words.len());
    (end > at).then(|| (words[at..end].iter().map(|word| word.to_string()).collect(), end))
}
because!(pcfg_row_at, PartsChanges, "the row of the part that starts at a word and the place after it, or nothing: a function word with its parts, or a run of items");

fn pcfg_exact(question: &str) -> Option<Vec<String>> {
    let words = python_words(question);
    let (row, end) = pcfg_row_at(&words, 0)?;
    (end == words.len() && !row.is_empty()).then(|| answer_words(&row.join(PARTS_GAP)))
}
because!(pcfg_exact, PartsChanges, "the row of a whole text as the words of its answer, or nothing when the text is no whole part");

fn pcfg_changes(sentences: &[String], question: &str) -> Vec<PartsChange> {
    let words = python_words(question);
    let mut out: Vec<PartsChange> = Vec::new();
    let mut add = |sort: &str, parts: Vec<&str>| out.push((sort.to_string(), sentences.to_vec(), parts.join(PARTS_GAP)));
    let mut sided = false;
    for (at, word) in words.iter().enumerate() {
        for group in [PCFG_ONE, PCFG_TWO].iter().filter(|group| group.contains(word)) {
            for other in group.iter().filter(|other| *other != word) {
                add(SORT_PARTS_WORD, [&words[..at], &[*other], &words[at + 1..]].concat());
            }
        }
        if PCFG_ONE.contains(word) {
            add(SORT_PARTS_OUT, [&words[..at], &words[at + 1..]].concat());
        }
        if PCFG_TWO.contains(word) && !sided {
            let first = pcfg_row_at(&words, at + 1);
            let second = first.as_ref().and_then(|(_, middle)| pcfg_row_at(&words, middle + 1));
            if let (Some((_, middle)), Some((_, end))) = (first, second) {
                sided = true;
                add(SORT_PARTS_SIDES, [&words[..at + 1], &words[(middle + 1).min(end)..end], &[PCFG_COMMA], &words[at + 1..middle], &words[end..]].concat());
            }
        }
    }
    let items: Vec<usize> = words.iter().enumerate().filter(|(_, word)| **word != PCFG_COMMA && !PCFG_ONE.contains(word) && !PCFG_TWO.contains(word)).map(|(at, _)| at).collect();
    let mut ends: Vec<usize> = items.first().into_iter().chain(items.last()).copied().collect();
    ends.dedup();
    for at in ends {
        let item = format!("{}{PCFG_NEW_NUMBER}", words[at].trim_end_matches(|sign: char| sign.is_ascii_digit()));
        add(SORT_PARTS_ITEM, [&words[..at], &[item.as_str()], &words[at + 1..]].concat());
    }
    out
}
because!(pcfg_changes, PartsChanges, "the changed forms of a text of functions: each function word made each other function with as many parts, each function of one part taken out, the parts of the first function of two parts changed over, and the first and the last item given a new number");

fn matched(sentences: &[String], question: &str) -> bool {
    let mut stack: Vec<&str> = Vec::new();
    for sign in brackets_of(sentences, question) {
        match paired(BRACKET_PAIRS, sign) {
            Some(closer) => stack.push(closer),
            None => {
                if stack.pop() != Some(sign) {
                    return false;
                }
            }
        }
    }
    stack.is_empty()
}
because!(matched, PartsChanges, "whether the brackets are matched: every pair can be taken away");

pub struct MatchedForm {
    word: Vec<(bool, Vec<String>)>,
    sorts: Vec<&'static str>,
}
because!(MatchedForm, PartsChanges, "the answer words of matched and of unmatched brackets, the most frequent of the lessons of each, and the openers that the told sentences of the lessons hold");

fn matched_form(lessons: &[Case]) -> MatchedForm {
    let mut said: Vec<(bool, FirstSightCount<Vec<String>>)> = Vec::new();
    for case in lessons {
        let ok = matched(&case.sentences, &case.question);
        let place = said.iter().position(|(held, _)| *held == ok).unwrap_or_else(|| {
            said.push((ok, FirstSightCount::new()));
            said.len() - 1
        });
        said[place].1.add(answer_words(&case.answer), 1);
    }
    let word = said.iter().filter_map(|(ok, found)| found.first_most().map(|(answer, _)| (*ok, answer.clone()))).collect();
    let told: HashSet<String> = lessons.iter().flat_map(|case| python_words(&case.sentences.join(PARTS_GAP)).into_iter().map(String::from).collect::<Vec<String>>()).collect();
    let sorts = BRACKET_PAIRS.iter().map(|(opener, _)| *opener).filter(|opener| told.contains(*opener)).collect();
    MatchedForm { word, sorts }
}
because!(matched_form, PartsChanges, "the form of the kind of matched brackets, counted from its lessons");

fn matched_changes(form: &MatchedForm, sentences: &[String], question: &str) -> Vec<PartsChange> {
    let Some(place) = bracket_place(sentences) else { return Vec::new() };
    let words = python_words(&sentences[place]);
    let mut out: Vec<PartsChange> = Vec::new();
    for (at, word) in words.iter().enumerate() {
        if !is_bracket(word) {
            continue;
        }
        out.push((SORT_PARTS_OUT.to_string(), with_words(sentences, place, &[&words[..at], &words[at + 1..]].concat()), question.to_string()));
        for opener in &form.sorts {
            let other = if paired(BRACKET_PAIRS, word).is_some() { *opener } else { paired(BRACKET_PAIRS, opener).unwrap_or_default() };
            if other != *word {
                out.push((SORT_PARTS_KIND.to_string(), with_words(sentences, place, &[&words[..at], &[other], &words[at + 1..]].concat()), question.to_string()));
            }
        }
    }
    for opener in &form.sorts {
        out.push((SORT_PARTS_PAIR.to_string(), with_words(sentences, place, &[&words[..], &[*opener, paired(BRACKET_PAIRS, opener).unwrap_or_default()]].concat()), question.to_string()));
    }
    out
}
because!(matched_changes, PartsChanges, "the changed forms of a case of matched brackets: each bracket taken out and made each other sort on its side, and a pair of each sort put at the end");

enum PartsReading {
    Scan,
    Dyck,
    Pcfg,
    Matched(MatchedForm),
    Lies(LiesForm),
    Questions(QuestionForm),
    Passive(PassiveForm),
}
because!(PartsReading, PartsChanges, "the exact reading of a kind with what it took from the lessons. exact gives the answer of a case; changes gives the changed forms of a case");

impl PartsReading {
    fn exact(&self, sentences: &[String], question: &str) -> Option<Vec<String>> {
        match self {
            PartsReading::Scan => scan_exact(question),
            PartsReading::Dyck => dyck_exact(sentences, question),
            PartsReading::Pcfg => pcfg_exact(question),
            PartsReading::Matched(form) => {
                let ok = matched(sentences, question);
                form.word.iter().find(|(held, _)| *held == ok).map(|(_, answer)| answer.clone())
            }
            PartsReading::Lies(form) => form.exact(sentences, question),
            PartsReading::Questions(form) => form.exact(question),
            PartsReading::Passive(form) => form.exact(question),
        }
    }

    fn changes(&self, sentences: &[String], question: &str) -> Vec<PartsChange> {
        match self {
            PartsReading::Scan => scan_changes(sentences, question),
            PartsReading::Dyck => dyck_changes(sentences, question),
            PartsReading::Pcfg => pcfg_changes(sentences, question),
            PartsReading::Matched(form) => matched_changes(form, sentences, question),
            PartsReading::Lies(form) => form.changes(sentences, question),
            PartsReading::Questions(form) => form.changes(sentences, question),
            PartsReading::Passive(form) => form.changes(sentences, question),
        }
    }
}

fn parts_reading(kind: &str, lessons: &[Case]) -> Option<PartsReading> {
    if PARTS_SCAN_KINDS.contains(&kind) {
        return Some(PartsReading::Scan);
    }
    if PARTS_PCFG_KINDS.contains(&kind) {
        return Some(PartsReading::Pcfg);
    }
    if PARTS_QUESTION_KINDS.contains(&kind) {
        return QuestionForm::counted(lessons).map(PartsReading::Questions);
    }
    match kind {
        PARTS_DYCK => Some(PartsReading::Dyck),
        PARTS_MATCHED => Some(PartsReading::Matched(matched_form(lessons))),
        PARTS_LIES => LiesForm::counted(lessons).map(PartsReading::Lies),
        PARTS_PASSIVE => PassiveForm::counted(lessons).map(PartsReading::Passive),
        _ => None,
    }
}
because!(parts_reading, PartsChanges, "the exact reading of a kind, with what it takes from the lessons; nothing for a kind that this control does not read or whose lessons do not give its form");

pub fn parts_changes(lessons: &[Case], whole: bool, kind: &str) -> Result<Vec<ChangedCase>, String> {
    let Some(reading) = parts_reading(kind, lessons) else { return Err("no reading of this control for the kind".to_string()) };
    let (first, last): (Vec<usize>, Vec<usize>) = if whole { ((0..lessons.len()).collect(), (0..lessons.len()).collect()) } else { inner_parts(lessons) };
    let untold = first.iter().all(|place| lessons[*place].sentences.is_empty());
    let told: HashSet<&str> = if untold { first.iter().map(|place| lessons[*place].question.as_str()).collect() } else { HashSet::new() };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        if reading.exact(&case.sentences, &case.question) != Some(answer_words(&case.answer)) {
            continue;
        }
        for (name, sentences, question) in reading.changes(&case.sentences, &case.question) {
            let Some(truth) = reading.exact(&sentences, &question) else { continue };
            let new = !told.is_empty() && !told.contains(question.as_str());
            let sort = if new { format!("{name}{PARTS_NEW_SORT}") } else { name };
            out.push(ChangedCase { place: Some(place), sort, sentences, question, truth: truth.join(PARTS_GAP) });
        }
    }
    Ok(out)
}
because!(parts_changes, PartsChanges, "the changed texts of the lessons that the model does not learn, or of all lessons for a model of all lessons, whose exact reading gives the answer of the set, each with the truth of the exact reading");
