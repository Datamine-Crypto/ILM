use crate::controls::file::ChangedCase;
use crate::controls::text::{number_runs, with_numbers};
use crate::sets::readers::common::stripped;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};

pub struct ArithmeticChanges;
source!(ArithmeticChanges, "changed sums of two numbers with choices: the question gives two numbers before its question mark and the choices of the answer after it, and the operation is the last part of the name of the kind. A sum is changed so that its result is new and is no choice: the second number gets one more, or for a division the first number gets the second added to it. Then the old answer among the choices becomes the new result and the choices turn round by one place, so that the place of the right choice moves; or the choices stay and the new result is no choice. A learner that calculates follows both changes; a learner that takes a place among the choices does not");

const ARITH_ASK: char = '?';
because!(ARITH_ASK, ArithmeticChanges, "the mark between the sum of a question and its choices");

const ARITH_KIND_PARTS: char = '_';
because!(ARITH_KIND_PARTS, ArithmeticChanges, "the mark before the operation in the name of a kind");

pub const ARITH_ADD: &str = "add";
because!(ARITH_ADD, ArithmeticChanges, "the operation of a kind of sums");

pub const ARITH_SUB: &str = "sub";
because!(ARITH_SUB, ArithmeticChanges, "the operation of a kind of differences");

pub const ARITH_MUL: &str = "mul";
because!(ARITH_MUL, ArithmeticChanges, "the operation of a kind of products");

const ARITH_DIV: &str = "div";
because!(ARITH_DIV, ArithmeticChanges, "the operation of a kind of quotients, whose change keeps the quotient whole");

const ARITH_TURNED: &str = "turned";
because!(ARITH_TURNED, ArithmeticChanges, "the sort of the change where the new result takes the place of the old answer among the choices and the choices turn round");

const ARITH_NOT_AMONG: &str = "not among the choices";
because!(ARITH_NOT_AMONG, ArithmeticChanges, "the sort of the change where the choices stay, so that the new result is no choice");

pub fn arithmetic_result(operation: &str, first: i128, second: i128) -> Option<i128> {
    match operation {
        ARITH_ADD => first.checked_add(second),
        ARITH_SUB => first.checked_sub(second),
        ARITH_MUL => first.checked_mul(second),
        ARITH_DIV => (second != 0 && first % second == 0).then(|| first / second),
        _ => None,
    }
}
because!(arithmetic_result, ArithmeticChanges, "the result of the operation of a kind on two numbers; nothing for a quotient that is not whole or a number too large to hold");

fn arithmetic_values(text: &str) -> Option<Vec<i128>> {
    number_runs(text, true).iter().map(|(start, end)| text[*start..*end].parse::<i128>().ok()).collect()
}
because!(arithmetic_values, ArithmeticChanges, "the numbers of a text in their order");

fn arithmetic_change(operation: &str, question: &str, answer: &str, turned: bool) -> Option<(String, String)> {
    let (head, tail) = question.split_once(ARITH_ASK).unwrap_or((question, ""));
    let [first, second] = arithmetic_values(head)?[..] else { return None };
    let mut choices = arithmetic_values(tail)?;
    let (first, second) = if operation == ARITH_DIV { (first.checked_add(second)?, second) } else { (first, second.checked_add(1)?) };
    let truth = arithmetic_result(operation, first, second)?;
    let old = stripped(answer).parse::<i128>().ok()?;
    if !choices.contains(&old) || choices.contains(&truth) {
        return None;
    }
    if turned {
        for choice in &mut choices {
            if *choice == old {
                *choice = truth;
            }
        }
        choices.rotate_right(1);
    }
    let mark = if question.contains(ARITH_ASK) { ARITH_ASK.to_string() } else { String::new() };
    let numbers: Vec<String> = [first, second].iter().map(i128::to_string).collect();
    let choices: Vec<String> = choices.iter().map(i128::to_string).collect();
    let changed = format!("{}{mark}{}", with_numbers(head, &number_runs(head, true), &numbers), with_numbers(tail, &number_runs(tail, true), &choices));
    Some((changed, truth.to_string()))
}
because!(arithmetic_change, ArithmeticChanges, "the changed question of a lesson and its new result, or nothing when the question does not have two numbers before its mark, when the new result is not whole, when the old answer is no choice, or when the new result is a choice already");

pub fn arithmetic_changes(lessons: &[Case], whole: bool, kind: &str) -> Vec<ChangedCase> {
    let operation = kind.rsplit_once(ARITH_KIND_PARTS).map_or("", |(_, operation)| operation);
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        for turned in [true, false] {
            if let Some((question, truth)) = arithmetic_change(operation, &case.question, &case.answer, turned) {
                let sort = if turned { ARITH_TURNED } else { ARITH_NOT_AMONG };
                out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences: case.sentences.clone(), question, truth });
            }
        }
    }
    out
}
because!(arithmetic_changes, ArithmeticChanges, "the changed questions of the lessons that the model does not learn, or of all lessons for a model of all lessons, each with its two sorts of change");
