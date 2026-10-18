use ilm_core::base::first_sight::FirstSightCount;
use ilm_core::base::words::answer_words;
use crate::controls::arith::{ARITH_ADD, ARITH_MUL, ARITH_SUB, arithmetic_result};
use crate::controls::expression::{TRUTH_CLOSE, TRUTH_FALSE, TRUTH_OPEN, TRUTH_TRUE, TRUTH_WORDS, number_value, truth_value};
use crate::controls::file::ChangedCase;
use crate::controls::text::{number_runs, whole_numbers, with_numbers};
use crate::sets::readers::common::{python_words, stripped};
use ilm_core::base::signs::{is_space, is_word_sign};
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};

pub struct MoreChanges;
source!(MoreChanges, "changed cases with a known true answer for kinds that a tool counts: each kind is read by its form with an exact reading, which must give the answer of the set for a lesson before the lesson is changed, and the same reading gives the truth of the changed case. A sum of two numbers in a ring or in plain numbers: the second number gets one more. A copy or a turned list of a told item: its first letter becomes the next letter of the alphabet. The three numbers of a told date or item in one fixed order and writing, both counted from the lessons: the last digit of the longest number becomes the next digit. An expression of numbers: its last number gets one more. An expression of truths: its last truth word becomes the other one. The words of a list in the order of their letters: the first word gets letters that put it at the end. Two orders in quotes: they change places, and the other letter is right. The free time between visits: the times of one visit become the times of the old answer, with the visits in the order of time or with the changed sentence where it stood. The year of a told fact that the question names: that year gets one more");

const MORE_MODULAR: &str = "modarith_";
because!(MORE_MODULAR, MoreChanges, "the start of the name of a kind of sums in a ring, before its operation");

const MORE_PLUS: &str = "plus";
because!(MORE_PLUS, MoreChanges, "the end of the name of a kind of sums in a ring whose answer has one added, before the one");

const MORE_KIND_PARTS: char = '_';
because!(MORE_KIND_PARTS, MoreChanges, "the mark between the operation and the rest of the name of a kind of sums in a ring");

const MORE_ADDITION: &str = "uicl_addition";
because!(MORE_ADDITION, MoreChanges, "the kind of sums of two numbers");

const MORE_IDENTITY: &str = "uicl_identity";
because!(MORE_IDENTITY, MoreChanges, "the kind whose answer is the told item as it stands");

const MORE_BACKWARDS: &[&str] = &["uicl_reverse", "uicl_reverse_back"];
because!(MORE_BACKWARDS, MoreChanges, "the kinds whose answer is the told list backwards");

const MORE_THREE_KINDS: &[&str] = &["uicl_dates", "uicl_dates_back", "uicl_numbers", "uicl_numbers_back"];
because!(MORE_THREE_KINDS, MoreChanges, "the kinds whose answer is the three numbers of the told item in one fixed order and writing");

const MORE_EXPRESSION: &str = "multistep_arithmetic";
because!(MORE_EXPRESSION, MoreChanges, "the kind whose answer is the value of an expression of numbers");

const MORE_TRUTHS: &str = "boolean_expressions";
because!(MORE_TRUTHS, MoreChanges, "the kind whose answer is the value of an expression of truths");

const MORE_SORTING: &str = "word_sorting";
because!(MORE_SORTING, MoreChanges, "the kind whose answer is the words of a list in the order of their letters");

const MORE_ORDERS: &str = "hyperbaton";
because!(MORE_ORDERS, MoreChanges, "the kind whose question gives two orders in quotes, each after its letter");

const MORE_VISITS: &str = "temporal_sequences";
because!(MORE_VISITS, MoreChanges, "the kind whose answer is the free time between told visits");

const MORE_YEAR: &str = "tot_event_at_what_time";
because!(MORE_YEAR, MoreChanges, "the kind whose answer is the year of a told fact that the question names");

const SORT_SECOND: &str = "the second number";
because!(SORT_SECOND, MoreChanges, "the part of a sum that gets one more, in the name of its change");

const SORT_LAST: &str = "the last number";
because!(SORT_LAST, MoreChanges, "the part of an expression that gets one more, in the name of its change");

const SORT_YEAR: &str = "the year of the told fact";
because!(SORT_YEAR, MoreChanges, "the part of a story of facts that gets one more, in the name of its change");

const SORT_LETTER: &str = "the first letter becomes the next letter";
because!(SORT_LETTER, MoreChanges, "the change of a told list whose answer is a copy or a turned list");

const SORT_DIGIT: &str = "the last digit of the longest number becomes the next digit";
because!(SORT_DIGIT, MoreChanges, "the change of a told item whose answer is its numbers in a fixed order");

const SORT_TRUTH: &str = "the last truth word is the other one";
because!(SORT_TRUTH, MoreChanges, "the change of an expression of truths");

const SORT_WORD_END: &str = "the first word of the list goes to the end";
because!(SORT_WORD_END, MoreChanges, "the change of a list whose answer is its words in the order of letters");

const SORT_ORDERS: &str = "the two orders change over";
because!(SORT_ORDERS, MoreChanges, "the change of a question with two orders in quotes");

const SORT_VISITS_IN_ORDER: &str = "the times of one visit become the old answer, the visits in the order of time";
because!(SORT_VISITS_IN_ORDER, MoreChanges, "the change of a story of visits after which the visits stand in the order of their times again, as in every story of the set");

const SORT_VISITS_WHERE: &str = "the same, the changed sentence left where it stood";
because!(SORT_VISITS_WHERE, MoreChanges, "the change of a story of visits after which the changed sentence stands where it stood");

const MORE_COLON: char = ':';
because!(MORE_COLON, MoreChanges, "the mark after which a told sentence gives its item or its list");

const MORE_COMMA: char = ',';
because!(MORE_COMMA, MoreChanges, "the mark between the parts of a told list");

const MORE_LIST_CUT: &str = ", ";
because!(MORE_LIST_CUT, MoreChanges, "the mark between the parts of a list that is written backwards");

const MORE_GAP: &str = " ";
because!(MORE_GAP, MoreChanges, "the mark between words that are written anew");

const MORE_AFTER_COLON: &str = ": ";
because!(MORE_AFTER_COLON, MoreChanges, "the marks between the head of a told sentence and the list that is written anew");

const MORE_TO_THE_END: &str = "zz";
because!(MORE_TO_THE_END, MoreChanges, "the letters put before a word so that it comes after every word of a list in the order of letters");

const MORE_EQUALS: char = '=';
because!(MORE_EQUALS, MoreChanges, "the mark at the end of an expression whose value is asked");

const MORE_ASK: char = '?';
because!(MORE_ASK, MoreChanges, "the mark after which a question gives its choices");

const MORE_QUOTE: char = '"';
because!(MORE_QUOTE, MoreChanges, "the mark on both sides of an order in a question");

const MORE_END: char = '.';
because!(MORE_END, MoreChanges, "the mark at the end of an answer of times and of a told fact");

const MORE_SLOT: &str = "{}";
because!(MORE_SLOT, MoreChanges, "the place of a number in the writing of an answer of three numbers");

const MORE_THREE: usize = 3;
because!(MORE_THREE, MoreChanges, "the numbers of a told item that the kinds of dates and numbers put in a fixed order: a day, a month and a year");

const MORE_TWO: usize = 2;
because!(MORE_TWO, MoreChanges, "the numbers of a sum, the orders of a question, and the fewest words of a list that can change");

const MORE_ALPHABET: u8 = 26;
because!(MORE_ALPHABET, MoreChanges, "the letters of the alphabet, after whose last letter the first comes again");

const MORE_SMALL_A: u8 = b'a';
because!(MORE_SMALL_A, MoreChanges, "where the alphabet of small letters starts, so that the letter after the last one is the first");

const MORE_CAPITAL_A: u8 = b'A';
because!(MORE_CAPITAL_A, MoreChanges, "where the alphabet of capital letters starts, so that a capital letter stays a capital letter");

const MORE_DIGITS: u8 = 10;
because!(MORE_DIGITS, MoreChanges, "the digits, after whose last digit the first comes again");

const MORE_HALF_DAY: u128 = 12;
because!(MORE_HALF_DAY, MoreChanges, "the hours of half a day, for the hour of a time before or after noon");

const MORE_AFTER_NOON: char = 'p';
because!(MORE_AFTER_NOON, MoreChanges, "the first letter of the word after the number of a time after noon");

const FACT_WAS: &str = " was the ";
because!(FACT_WAS, MoreChanges, "the words between the person and the role of a told fact");

const FACT_OF: &str = " of ";
because!(FACT_OF, MoreChanges, "the word between the role and the whole of a told fact and of a question");

const FACT_FROM: &str = " from ";
because!(FACT_FROM, MoreChanges, "the word before the year when a told fact starts");

const FACT_TO: &str = " to ";
because!(FACT_TO, MoreChanges, "the word before the year when a told fact stops");

const FACT_LINE: &str = "\n";
because!(FACT_LINE, MoreChanges, "the end of a line that may follow a told fact");

const ASK_START: &str = "start";
because!(ASK_START, MoreChanges, "the word of a question that asks for the year when a fact starts");

const ASK_STOP: &str = "stop";
because!(ASK_STOP, MoreChanges, "the word of a question that asks for the year when a fact stops");

const ASK_BEING: &str = "being the ";
because!(ASK_BEING, MoreChanges, "the words of a question before the role that it asks about");

const ASK_ARTICLE: &str = "the ";
because!(ASK_ARTICLE, MoreChanges, "a word that may stand before the whole of a fact in a question");

const ASK_ENTITY: &str = "entity ";
because!(ASK_ENTITY, MoreChanges, "a word that may stand before the name of the whole of a fact in a question");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoreKind {
    Modular(&'static str, bool),
    Addition,
    Identity,
    Backwards,
    Three,
    Expression,
    Truths,
    Sorting,
    Orders,
    Visits,
    Year,
}
because!(MoreKind, MoreChanges, "the reading of a kind: sums in a ring with their operation and whether one is added, sums of two numbers, a copy, a list backwards, three numbers, an expression of numbers, an expression of truths, sorted words, two orders, visits, or the year of a fact");

fn more_kind(kind: &str) -> Option<MoreKind> {
    if let Some(rest) = kind.strip_prefix(MORE_MODULAR) {
        let (operation, end) = rest.split_once(MORE_KIND_PARTS).unwrap_or((rest, ""));
        let operation = [ARITH_ADD, ARITH_SUB, ARITH_MUL].into_iter().find(|known| *known == operation)?;
        let plus = format!("{MORE_PLUS}{}", 1);
        return match end {
            "" => Some(MoreKind::Modular(operation, false)),
            _ if end == plus => Some(MoreKind::Modular(operation, true)),
            _ => None,
        };
    }
    match kind {
        MORE_ADDITION => Some(MoreKind::Addition),
        MORE_IDENTITY => Some(MoreKind::Identity),
        _ if MORE_BACKWARDS.contains(&kind) => Some(MoreKind::Backwards),
        _ if MORE_THREE_KINDS.contains(&kind) => Some(MoreKind::Three),
        MORE_EXPRESSION => Some(MoreKind::Expression),
        MORE_TRUTHS => Some(MoreKind::Truths),
        MORE_SORTING => Some(MoreKind::Sorting),
        MORE_ORDERS => Some(MoreKind::Orders),
        MORE_VISITS => Some(MoreKind::Visits),
        MORE_YEAR => Some(MoreKind::Year),
        _ => None,
    }
}
because!(more_kind, MoreChanges, "the reading of a kind from its name, or nothing for a kind that this control does not read");

fn one_more_sort(what: &str) -> String {
    format!("{what} gets {} more", 1)
}
because!(one_more_sort, MoreChanges, "the name of a change where a number gets one more");

fn partition(text: &str, mark: char) -> (&str, &str, &str) {
    match text.split_once(mark) {
        Some((head, tail)) => (head, &text[head.len()..head.len() + mark.len_utf8()], tail),
        None => (text, "", ""),
    }
}
because!(partition, MoreChanges, "the text before the first mark, the mark, and the text after it; the whole text and nothing more when the mark is not there");

fn digit_runs(text: &str) -> Vec<&str> {
    number_runs(text, false).iter().map(|(start, end)| &text[*start..*end]).collect()
}
because!(digit_runs, MoreChanges, "the runs of digits of a text in their order");

fn last_number_more(text: &str) -> Option<String> {
    let runs = number_runs(text, true);
    let (start, end) = *runs.last()?;
    let value = text[start..end].parse::<i128>().ok()?.checked_add(1)?;
    Some(format!("{}{value}{}", &text[..start], &text[end..]))
}
because!(last_number_more, MoreChanges, "the text with one added to its last number");

fn told_item(sentences: &[String]) -> Option<&str> {
    let [sentence] = sentences else { return None };
    sentence.split_once(MORE_COLON).map(|(_, item)| stripped(item))
}
because!(told_item, MoreChanges, "the item after the colon of the one told sentence");

fn quoted(text: &str) -> Vec<(usize, usize)> {
    let marks: Vec<usize> = text.match_indices(MORE_QUOTE).map(|(at, _)| at).collect();
    marks.chunks_exact(MORE_TWO).map(|pair| (pair[0] + MORE_QUOTE.len_utf8(), pair[1])).collect()
}
because!(quoted, MoreChanges, "the places of the texts between pairs of quotes, read from the left");

fn sign_run(signs: &[char], start: usize, test: impl Fn(char) -> bool) -> usize {
    let mut end = start;
    while end < signs.len() && test(signs[end]) {
        end += 1;
    }
    end
}
because!(sign_run, MoreChanges, "the end of the run of signs that pass a test from a place on");

fn span_end(signs: &[char], at: usize) -> Option<usize> {
    let digit = |sign: char| sign.is_ascii_digit();
    let small = |sign: char| sign.is_ascii_lowercase();
    let gap = MORE_GAP.chars().next()?;
    let number = sign_run(signs, at, digit);
    let word = sign_run(signs, sign_run(signs, number, is_space), small);
    let next = sign_run(signs, word + 1, small);
    let second = sign_run(signs, next + 1, digit);
    let last = sign_run(signs, sign_run(signs, second, is_space), small);
    let joined = signs.get(word) == Some(&gap) && signs.get(next) == Some(&gap);
    let filled = number > at && word > sign_run(signs, number, is_space) && next > word + 1 && second > next + 1 && last > sign_run(signs, second, is_space);
    (joined && filled).then_some(last)
}
because!(span_end, MoreChanges, "the end of a span of times that starts at a place, such as a number and its word, two words, and a number and its word; nothing when no span starts there");

fn time_spans(text: &str) -> Vec<String> {
    let signs: Vec<char> = text.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut at = 0;
    while at < signs.len() {
        match span_end(&signs, at) {
            Some(end) => {
                out.push(signs[at..end].iter().collect());
                at = end;
            }
            None => at += 1,
        }
    }
    out
}
because!(time_spans, MoreChanges, "the spans of times of a text in their order, read from the left with no overlap");

fn span_hour(span: &str) -> u128 {
    let signs: Vec<char> = span.chars().collect();
    let number = sign_run(&signs, 0, |sign| sign.is_ascii_digit());
    let digits: String = signs[..number].iter().collect();
    let word = sign_run(&signs, number, is_space);
    let after_noon = signs.get(word) == Some(&MORE_AFTER_NOON);
    digits.parse::<u128>().unwrap_or_default() % MORE_HALF_DAY + if after_noon { MORE_HALF_DAY } else { 0 }
}
because!(span_hour, MoreChanges, "the hour of the day at which a span of times starts");

struct TextCursor<'a> {
    text: &'a str,
    at: usize,
}
because!(TextCursor, MoreChanges, "a place in a text from which its parts are read one after the other: word reads a run of word signs, digits a run of digits, fixed a given text");

impl<'a> TextCursor<'a> {
    fn run(&mut self, test: impl Fn(char) -> bool) -> Option<&'a str> {
        let rest = &self.text[self.at..];
        let length: usize = rest.chars().take_while(|sign| test(*sign)).map(char::len_utf8).sum();
        self.at += length;
        (length > 0).then(|| &rest[..length])
    }

    fn word(&mut self) -> Option<&'a str> {
        self.run(is_word_sign)
    }

    fn digits(&mut self) -> Option<&'a str> {
        self.run(|sign| sign.is_ascii_digit())
    }

    fn fixed(&mut self, part: &str) -> Option<()> {
        self.text[self.at..].starts_with(part).then(|| self.at += part.len())
    }
}

struct ToldFact<'a> {
    who: &'a str,
    role: &'a str,
    whose: &'a str,
    from: &'a str,
    to: &'a str,
}
because!(ToldFact, MoreChanges, "a told fact of roles: the person, the role, the whole, the year of the start and the year of the stop");

fn told_fact(sentence: &str) -> Option<ToldFact<'_>> {
    let mut cursor = TextCursor { text: sentence, at: 0 };
    let who = cursor.word()?;
    cursor.fixed(FACT_WAS)?;
    let role = cursor.word()?;
    cursor.fixed(FACT_OF)?;
    let whose = cursor.word()?;
    cursor.fixed(FACT_FROM)?;
    let from = cursor.digits()?;
    cursor.fixed(FACT_TO)?;
    let to = cursor.digits()?;
    cursor.fixed(&MORE_END.to_string())?;
    let rest = &sentence[cursor.at..];
    (rest.is_empty() || rest == FACT_LINE).then_some(ToldFact { who, role, whose, from, to })
}
because!(told_fact, MoreChanges, "the person, the role, the whole, the year of the start and the year of the stop of a told sentence that is a fact of roles and nothing more");

fn asked_at(text: &str, at: usize) -> Option<(&str, &str, &str)> {
    let mut cursor = TextCursor { text, at };
    let who = cursor.word()?;
    cursor.fixed(MORE_GAP)?;
    for word in [ASK_START, ASK_STOP] {
        let mut tried = TextCursor { text, at: cursor.at };
        if tried.fixed(word).and_then(|_| tried.fixed(MORE_GAP)).is_some() {
            cursor.at = tried.at;
            break;
        }
    }
    cursor.fixed(ASK_BEING)?;
    let role = cursor.word()?;
    cursor.fixed(FACT_OF)?;
    for (the, entity) in [(true, true), (true, false), (false, true), (false, false)] {
        let mut tried = TextCursor { text, at: cursor.at };
        if (the && tried.fixed(ASK_ARTICLE).is_none()) || (entity && tried.fixed(ASK_ENTITY).is_none()) {
            continue;
        }
        if let Some(whose) = tried.word() {
            return Some((who, role, whose));
        }
    }
    None
}
because!(asked_at, MoreChanges, "the person, the role and the whole that a question names, read from a place: the words that may stand before the whole are taken where the name of the whole still follows them");

fn asked_fact(question: &str) -> Option<(&str, bool, &str, &str)> {
    let text = stripped(question);
    let (who, role, whose) = text.char_indices().find_map(|(at, _)| asked_at(text, at))?;
    Some((who, question.contains(ASK_START), role, whose))
}
because!(asked_fact, MoreChanges, "the person, whether the start is asked, the role and the whole of a question about the year of a fact");

struct ThreeForm {
    order: Option<[usize; MORE_THREE]>,
    writing: String,
}
because!(ThreeForm, MoreChanges, "the order of the three numbers of a told item in the answer and the writing of an answer, both counted from the lessons: the order that gives the answer of most lessons and the most frequent answer with its numbers taken out, the first seen of equals");

fn three_orders() -> Vec<[usize; MORE_THREE]> {
    let mut out: Vec<[usize; MORE_THREE]> = Vec::new();
    for first in 0..MORE_THREE {
        for second in 0..MORE_THREE {
            for third in 0..MORE_THREE {
                if first != second && first != third && second != third {
                    out.push([first, second, third]);
                }
            }
        }
    }
    out
}
because!(three_orders, MoreChanges, "the orders of three places, in the order of their numbers");

fn three_form(lessons: &[Case]) -> ThreeForm {
    let mut orders: FirstSightCount<[usize; MORE_THREE]> = FirstSightCount::new();
    let mut writings: FirstSightCount<String> = FirstSightCount::new();
    for case in lessons {
        let told = digit_runs(told_item(&case.sentences).unwrap_or_default());
        let said = digit_runs(&case.answer);
        if told.len() == MORE_THREE && said.len() == MORE_THREE {
            for order in three_orders() {
                if order.iter().map(|place| told[*place]).eq(said.iter().copied()) {
                    orders.add(order, 1);
                }
            }
        }
        let answer = stripped(&case.answer);
        let runs = number_runs(answer, false);
        writings.add(with_numbers(answer, &runs, &vec![MORE_SLOT.to_string(); runs.len()]), 1);
    }
    ThreeForm { order: orders.first_most().map(|(order, _)| *order), writing: writings.first_most().map(|(writing, _)| writing.clone()).unwrap_or_default() }
}
because!(three_form, MoreChanges, "the form of the answers of a kind of three numbers, counted from its lessons");

fn three_answer(form: &ThreeForm, sentences: &[String]) -> Option<String> {
    let numbers = digit_runs(told_item(sentences).unwrap_or_default());
    let order = form.order?;
    if numbers.len() != MORE_THREE || form.writing.matches(MORE_SLOT).count() != MORE_THREE {
        return None;
    }
    let mut out = form.writing.clone();
    for place in order {
        out = out.replacen(MORE_SLOT, numbers[place], 1);
    }
    Some(out)
}
because!(three_answer, MoreChanges, "the three numbers of a told item in the order and the writing of the lessons");

fn more_exact(kind: MoreKind, form: &ThreeForm, sentences: &[String], question: &str) -> Option<String> {
    match kind {
        MoreKind::Modular(operation, plus) => {
            let [first, second] = whole_numbers(question, true)?[..] else { return None };
            Some(arithmetic_result(operation, first, second)?.checked_add(i128::from(plus))?.to_string())
        }
        MoreKind::Addition => {
            let [first, second] = whole_numbers(question, false)?[..] else { return None };
            Some(first.checked_add(second)?.to_string())
        }
        MoreKind::Identity => told_item(sentences).map(String::from),
        MoreKind::Backwards => {
            let parts: Vec<&str> = told_item(sentences)?.split(MORE_COMMA).map(stripped).rev().collect();
            Some(parts.join(MORE_LIST_CUT))
        }
        MoreKind::Three => three_answer(form, sentences),
        MoreKind::Expression => number_value(stripped(stripped(question).trim_end_matches(MORE_EQUALS))),
        MoreKind::Truths => {
            let spaced = question.replace(TRUTH_OPEN, &format!("{MORE_GAP}{TRUTH_OPEN}{MORE_GAP}")).replace(TRUTH_CLOSE, &format!("{MORE_GAP}{TRUTH_CLOSE}{MORE_GAP}"));
            let words = python_words(&spaced);
            let end = words.iter().rposition(|word| TRUTH_WORDS.contains(word)).map_or(0, |last| last + 1);
            let words = &words[..end];
            if words.is_empty() || words.iter().any(|word| !TRUTH_WORDS.contains(word)) {
                return None;
            }
            truth_value(words)
        }
        MoreKind::Sorting => {
            let (_, listed) = sentences.last()?.split_once(MORE_COLON)?;
            let mut words = python_words(listed);
            words.sort_unstable();
            Some(words.join(MORE_GAP))
        }
        MoreKind::Orders => None,
        MoreKind::Visits => {
            let told = sentences.join(MORE_GAP);
            let choices = time_spans(partition(question, MORE_ASK).2);
            let free: Vec<&String> = choices.iter().filter(|choice| !told.contains(choice.as_str())).collect();
            match free[..] {
                [one] => Some(one.clone()),
                _ => None,
            }
        }
        MoreKind::Year => {
            let (who, start, role, whose) = asked_fact(question)?;
            let found: Vec<ToldFact> = sentences.iter().filter_map(|sentence| told_fact(sentence)).filter(|fact| (fact.who, fact.role, fact.whose) == (who, role, whose)).collect();
            match &found[..] {
                [fact] => Some(if start { fact.from } else { fact.to }.to_string()),
                _ => None,
            }
        }
    }
}
because!(more_exact, MoreChanges, "the answer of a case by the exact reading of its kind, or nothing when the case does not have the form of its kind");

fn next_letter(sentences: &[String]) -> Option<Vec<String>> {
    let (head, colon, tail) = partition(sentences.first()?, MORE_COLON);
    let (at, letter) = tail.char_indices().find(|(_, sign)| sign.is_ascii_alphabetic())?;
    let first = if letter.is_ascii_lowercase() { MORE_SMALL_A } else { MORE_CAPITAL_A };
    let new = char::from((letter as u8 - first + 1) % MORE_ALPHABET + first);
    Some(vec![format!("{head}{colon}{}{new}{}", &tail[..at], &tail[at + 1..])])
}
because!(next_letter, MoreChanges, "the told sentence with the first letter of its list made the next letter of the alphabet");

fn digit_more(sentences: &[String]) -> Option<Vec<String>> {
    let (head, colon, tail) = partition(sentences.first()?, MORE_COLON);
    let runs = number_runs(tail, false);
    let mut longest: Option<(usize, usize)> = None;
    for run in runs {
        if longest.is_none_or(|(start, end)| run.1 - run.0 > end - start) {
            longest = Some(run);
        }
    }
    let at = longest?.1 - 1;
    let radix = u32::from(MORE_DIGITS);
    let digit = (char::from(tail.as_bytes()[at]).to_digit(radix)? + 1) % radix;
    Some(vec![format!("{head}{colon}{}{digit}{}", &tail[..at], &tail[at + 1..])])
}
because!(digit_more, MoreChanges, "the told sentence with the last digit of the longest number of its item made the next digit");

fn other_truth_word(question: &str) -> Option<String> {
    let mut words: Vec<String> = question.split(MORE_GAP).map(String::from).collect();
    let brackets: Vec<char> = TRUTH_OPEN.chars().chain(TRUTH_CLOSE.chars()).collect();
    let at = words.iter().rposition(|word| [TRUTH_TRUE, TRUTH_FALSE].contains(&word.trim_matches(brackets.as_slice())))?;
    let hold = char::from(0u8).to_string();
    words[at] = words[at].replace(TRUTH_TRUE, &hold).replace(TRUTH_FALSE, TRUTH_TRUE).replace(&hold, TRUTH_FALSE);
    Some(words.join(MORE_GAP))
}
because!(other_truth_word, MoreChanges, "the expression with its last truth word made the other one");

fn word_to_the_end(sentences: &[String]) -> Option<Vec<String>> {
    let (last, before) = sentences.split_last()?;
    let (head, _, tail) = partition(last, MORE_COLON);
    let words = python_words(tail);
    if words.len() < MORE_TWO {
        return None;
    }
    let mut out = before.to_vec();
    out.push(format!("{head}{MORE_AFTER_COLON}{MORE_TO_THE_END}{}{MORE_GAP}{}", words[0], words[1..].join(MORE_GAP)));
    Some(out)
}
because!(word_to_the_end, MoreChanges, "the told sentences with the first word of the list given letters that put it at the end");

fn orders_changed_over(question: &str) -> Option<String> {
    let [(first_start, first_end), (second_start, second_end)] = quoted(question)[..] else { return None };
    Some(format!("{}{}{}{}{}", &question[..first_start], &question[second_start..second_end], &question[first_end..second_start], &question[first_start..first_end], &question[second_end..]))
}
because!(orders_changed_over, MoreChanges, "the question with its two orders in quotes in each other's place");

fn other_letter(question: &str, answer: &str) -> Option<String> {
    let said = stripped(answer);
    let letters: Vec<&str> = quoted(question).iter().filter_map(|(start, _)| python_words(&question[..start - MORE_QUOTE.len_utf8()]).last().copied()).collect();
    letters.into_iter().find(|letter| *letter != said).map(String::from)
}
because!(other_letter, MoreChanges, "the letter of the other choice: of the words right before the quotes, the one that is not the old answer");

fn visit_moved(sentences: &[String], question: &str, answer: &str) -> Option<Vec<String>> {
    let choices = time_spans(partition(question, MORE_ASK).2);
    let old = stripped(answer).trim_end_matches(MORE_END);
    let known = choices.iter().any(|choice| choice == old);
    let moved = choices.iter().find(|choice| *choice != old && sentences.iter().filter(|sentence| sentence.contains(choice.as_str())).count() == 1 && known)?;
    Some(sentences.iter().map(|sentence| sentence.replace(moved.as_str(), old)).collect())
}
because!(visit_moved, MoreChanges, "the told sentences with the times of one visit made the times of the old answer, so that the old times of that visit are free");

fn visit_moved_in_order(sentences: &[String], question: &str, answer: &str) -> Option<Vec<String>> {
    let mut out = visit_moved(sentences, question, answer)?;
    let places: Vec<usize> = out.iter().enumerate().filter(|(_, sentence)| !time_spans(sentence).is_empty()).map(|(at, _)| at).collect();
    let mut visits: Vec<String> = places.iter().map(|at| out[*at].clone()).collect();
    visits.sort_by_key(|visit| time_spans(visit).first().map(|span| span_hour(span)).unwrap_or_default());
    for (at, visit) in places.iter().zip(visits) {
        out[*at] = visit;
    }
    Some(out)
}
because!(visit_moved_in_order, MoreChanges, "the same change with the told visits in the order of their times again");

fn year_more(sentences: &[String], question: &str) -> Option<Vec<String>> {
    let (who, start, role, whose) = asked_fact(question)?;
    let mut out: Vec<String> = Vec::new();
    for sentence in sentences {
        let fact = told_fact(sentence).filter(|fact| (fact.who, fact.role, fact.whose) == (who, role, whose));
        let Some(fact) = fact else {
            out.push(sentence.clone());
            continue;
        };
        let (from, to) = (fact.from.parse::<u128>().ok()?, fact.to.parse::<u128>().ok()?);
        let (from, to) = if start { (from + 1, to) } else { (from, to + 1) };
        out.push(format!("{who}{FACT_WAS}{role}{FACT_OF}{whose}{FACT_FROM}{from}{FACT_TO}{to}{MORE_END}"));
    }
    Some(out)
}
because!(year_more, MoreChanges, "the told sentences with one added to the asked year of the named fact");

fn more_changed(kind: MoreKind, case: &Case) -> Vec<(String, Option<(Vec<String>, String)>)> {
    let (sentences, question) = (&case.sentences, &case.question);
    let same = |changed: Option<Vec<String>>| changed.map(|told| (told, question.clone()));
    match kind {
        MoreKind::Modular(..) | MoreKind::Addition => vec![(one_more_sort(SORT_SECOND), last_number_more(question).map(|changed| (sentences.clone(), changed)))],
        MoreKind::Identity | MoreKind::Backwards => vec![(SORT_LETTER.to_string(), same(next_letter(sentences)))],
        MoreKind::Three => vec![(SORT_DIGIT.to_string(), same(digit_more(sentences)))],
        MoreKind::Expression => vec![(one_more_sort(SORT_LAST), last_number_more(question).map(|changed| (sentences.clone(), changed)))],
        MoreKind::Truths => vec![(SORT_TRUTH.to_string(), other_truth_word(question).map(|changed| (sentences.clone(), changed)))],
        MoreKind::Sorting => vec![(SORT_WORD_END.to_string(), same(word_to_the_end(sentences)))],
        MoreKind::Orders => vec![(SORT_ORDERS.to_string(), orders_changed_over(question).map(|changed| (sentences.clone(), changed)))],
        MoreKind::Visits => vec![(SORT_VISITS_IN_ORDER.to_string(), same(visit_moved_in_order(sentences, question, &case.answer))), (SORT_VISITS_WHERE.to_string(), same(visit_moved(sentences, question, &case.answer)))],
        MoreKind::Year => vec![(one_more_sort(SORT_YEAR), same(year_more(sentences, question)))],
    }
}
because!(more_changed, MoreChanges, "the changes of a case of a kind, each with its name, in their order; nothing for a change that the case does not allow");

pub fn more_changes(lessons: &[Case], whole: bool, kind: &str) -> Result<Vec<ChangedCase>, String> {
    let Some(reading) = more_kind(kind) else { return Err("no reading of this control for the kind".to_string()) };
    let form = if reading == MoreKind::Three { three_form(lessons) } else { ThreeForm { order: None, writing: String::new() } };
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        if reading != MoreKind::Orders && more_exact(reading, &form, &case.sentences, &case.question).is_none_or(|exact| answer_words(&exact) != answer_words(&case.answer)) {
            continue;
        }
        for (sort, made) in more_changed(reading, case) {
            let Some((sentences, question)) = made else { continue };
            let truth = if reading == MoreKind::Orders { other_letter(&case.question, &case.answer) } else { more_exact(reading, &form, &sentences, &question) };
            if let Some(truth) = truth {
                out.push(ChangedCase { place: Some(place), sort, sentences, question, truth });
            }
        }
    }
    Ok(out)
}
because!(more_changes, MoreChanges, "the changed cases of the lessons that the model does not learn, or of all lessons for a model of all lessons, whose exact reading gives the answer of the set, each with the truth of the exact reading of the changed case");
