use crate::cases::Case;
use crate::tools::explain::chain::{empty_filling, fit_into, reads_as_number, worked_words};
use crate::tools::explain::text::{ANY_FIRST, LogicPiece, MARKED_FIRST, ONE_SIGN_FIRST, PLAIN_PLACES, Pattern, Reckoning, fact_words, place_letter, word_not_mark};
use crate::tools::explain::wording::{CONDITIONS_JOINED, WordingLesson, library_writing, places_of};
use crate::tools::explain::words::{WORD_IS_DIGITS, WordId, word_has, word_number, word_text};
use patterns::{because, source};

pub struct LearnedForms;
source!(LearnedForms, "the answer of a kind whose worked examples show an output made of the words of their input, learned as a pupil copies the form of a worked example (programming by demonstration, Lieberman): each word of the output is the word at one place of the input, the same place in every example, the same word in every example, or one step of arithmetic on the words at two places of the input, the same step in every example; the input is the one told sentence of an example, or its question when it tells none; a place of the input whose word changes from example to example is an open place of one word (of one sign when it may hold a mark), a place whose word stays is a fixed word, and the input after the last place that the output reads is one run of any signs when its words change; the examples are grouped by the lengths of their input and their output, each group of enough examples whose every word is so explained gives one answer, and the answers become forms of the learned wording, written as question forms of the library, so a person reads where each word of the output comes from");

const FORM_SUPPORT: usize = 3;
because!(FORM_SUPPORT, LearnedForms, "the fewest examples of one length of input and output that give an answer form, so that a place of the output rests on more than a chance case");

const FORM_GAP: &str = " ";
because!(FORM_GAP, LearnedForms, "the space between an answer and the next");

const WORKED_STEPS: &[(Reckoning, &str)] = &[(Reckoning::Plus, "plus"), (Reckoning::Minus, "minus"), (Reckoning::Times, "times"), (Reckoning::Over, "divided by")];
because!(WORKED_STEPS, LearnedForms, "the steps of arithmetic that a word of the output may be, with the words by which a condition of the library names each, tried in this order");

const WORKED_IS: &str = "is";
because!(WORKED_IS, LearnedForms, "the word before the result of a step of arithmetic in a condition of the library");

struct Example {
    input: Vec<String>,
    output: Vec<String>,
}
because!(Example, LearnedForms, "one worked example: the words of its input (its one told sentence, or its question) and the words of its answer, as the rules read them");

enum OutputSource {
    Fixed(String),
    Place(usize),
    Worked(usize, usize, usize, Option<String>),
}
because!(OutputSource, LearnedForms, "where a word of the output comes from: the same word in every example, the word at a place of the input, or a step of arithmetic (by its number in the list of steps) on the words at two places of the input, with a number added to its result when every example adds the same number");

struct GroupForm {
    input: Pattern,
    output: Pattern,
    worked: Vec<String>,
}
because!(GroupForm, LearnedForms, "the answer form of a group of examples: the input with fixed words, open places and a run at its end, the output with fixed words and places, and the conditions of the steps of arithmetic that fill some places of the output");

fn worked_step(group: &[&Example], varying: &[usize], at: usize) -> Option<(usize, usize, usize, Option<String>)> {
    let first = group.first()?;
    let result = |example: &Example, one: usize, other: usize, reckoning: Reckoning| worked_words(reckoning, &[word_number(&example.input[one])], &[word_number(&example.input[other])]).0;
    let wanted = |example: &Example| vec![word_number(&example.output[at])];
    for one in varying {
        for other in varying.iter().filter(|other| *other != one) {
            for (step, (reckoning, _)) in WORKED_STEPS.iter().enumerate() {
                let holds = |example: &Example| result(example, *one, *other, *reckoning) == Some(wanted(example));
                if holds(first) && group.iter().all(|example| holds(example)) {
                    return Some((*one, *other, step, None));
                }
            }
        }
    }
    for one in varying {
        for other in varying.iter().filter(|other| *other != one) {
            for (step, (reckoning, _)) in WORKED_STEPS.iter().enumerate() {
                let added = |example: &Example| result(example, *one, *other, *reckoning).and_then(|value| worked_words(Reckoning::Minus, &wanted(example), &value).0);
                let Some(more) = added(first) else { continue };
                if more.len() == 1 && word_has(more[0], WORD_IS_DIGITS) && group.iter().all(|example| added(example).as_ref() == Some(&more)) {
                    return Some((*one, *other, step, Some(word_text(more[0]).to_string())));
                }
            }
        }
    }
    None
}
because!(worked_step, LearnedForms, "the first step of arithmetic on the words at two changing places of the input, in their order and the order of the steps, whose result as the engine writes it is the word of the output in every example of the group; else the first such step whose result falls short of the output by the same whole number in every example, which is then added (one more than the sum); nothing when none is");

fn group_form(group: &[&Example]) -> Option<GroupForm> {
    let first = group.first()?;
    let fixed: Vec<bool> = (0..first.input.len()).map(|at| group.iter().all(|example| example.input[at] == first.input[at])).collect();
    let varying: Vec<usize> = (0..first.input.len()).filter(|at| !fixed[*at]).collect();
    let mut taken: Vec<OutputSource> = Vec::new();
    let mut needed: Vec<bool> = vec![false; first.input.len()];
    for (at, word) in first.output.iter().enumerate() {
        if group.iter().all(|example| example.output[at] == *word) {
            taken.push(OutputSource::Fixed(word.clone()));
            continue;
        }
        if let Some(from) = varying.iter().copied().find(|place| group.iter().all(|example| example.output[at] == example.input[*place])) {
            needed[from] = true;
            taken.push(OutputSource::Place(from));
            continue;
        }
        let (one, other, step, more) = worked_step(group, &varying, at)?;
        needed[one] = true;
        needed[other] = true;
        taken.push(OutputSource::Worked(one, other, step, more));
    }
    let tail = needed.iter().rposition(|need| *need).map_or(0, |last| last + 1);
    let run = varying.iter().any(|at| *at >= tail);
    let mut input: Pattern = Vec::new();
    let mut places: Vec<Option<usize>> = vec![None; first.input.len()];
    let (mut next_word, mut next_sign) = (PLAIN_PLACES, ONE_SIGN_FIRST);
    for (at, word) in first.input.iter().enumerate().take(if run { tail } else { first.input.len() }) {
        if fixed[at] {
            input.push(LogicPiece::Word(word_number(word)));
            continue;
        }
        let (next, last) = if group.iter().all(|example| word_not_mark(&example.input[at])) { (&mut next_word, MARKED_FIRST) } else { (&mut next_sign, ANY_FIRST) };
        if *next >= last {
            return None;
        }
        places[at] = Some(*next);
        input.push(LogicPiece::OpenWithEnd(*next, String::new()));
        *next += 1;
    }
    if run {
        input.push(LogicPiece::Open(ANY_FIRST));
    }
    let mut output: Pattern = Vec::new();
    let mut worked: Vec<String> = Vec::new();
    for piece in taken {
        match piece {
            OutputSource::Fixed(word) => output.push(LogicPiece::Word(word_number(&word))),
            OutputSource::Place(from) => output.push(LogicPiece::OpenWithEnd(places[from]?, String::new())),
            OutputSource::Worked(one, other, step, more) => {
                let fresh = 1 + usize::from(more.is_some());
                if next_word + fresh > MARKED_FIRST {
                    return None;
                }
                let result = next_word;
                next_word += fresh;
                worked.push([place_letter(places[one]?).as_str(), WORKED_STEPS[step].1, place_letter(places[other]?).as_str(), WORKED_IS, place_letter(result).as_str()].join(FORM_GAP));
                let said = match more {
                    Some(number) => {
                        worked.push([place_letter(result).as_str(), WORKED_STEPS[usize::default()].1, number.as_str(), WORKED_IS, place_letter(result + 1).as_str()].join(FORM_GAP));
                        result + 1
                    }
                    None => result,
                };
                output.push(LogicPiece::OpenWithEnd(said, String::new()));
            }
        }
    }
    Some(GroupForm { input, output, worked })
}
because!(group_form, LearnedForms, "the answer form of a group of examples of one length of input and of output: each word of the output a fixed word where every example has the same word, else the first changing place of the input whose word every example has there, else the first step of arithmetic on two changing places that gives it; the input with a fixed word where every example has the same word and an open place where the words change (one word, or one sign when some example has a mark there), and after the last place that the output reads, one run of any signs when a word there changes; a step's result fills a new place of one word; nothing when some word of the output is none of these, or the places run out");

fn grouped(examples: &[Example]) -> Vec<Vec<&Example>> {
    let mut groups: Vec<((usize, usize), Vec<&Example>)> = Vec::new();
    for example in examples {
        let key = (example.input.len(), example.output.len());
        match groups.iter_mut().find(|(known, _)| *known == key) {
            Some((_, group)) => group.push(example),
            None => groups.push((key, vec![example])),
        }
    }
    groups.sort_by(|one, other| other.1.len().cmp(&one.1.len()).then_with(|| one.0.cmp(&other.0)));
    groups.into_iter().map(|(_, group)| group).filter(|group| group.len() >= FORM_SUPPORT).collect()
}
because!(grouped, LearnedForms, "the examples in groups of one length of input and of output, the largest group first, the groups of too few examples left out");

pub(super) fn worked_conditions(form: &Pattern, held: &[&WordingLesson]) -> Vec<(String, String)> {
    let own = places_of(form);
    let numbered = |word: &WordId| reads_as_number(*word);
    let mut read: Vec<(Vec<Option<WordId>>, WordId)> = Vec::new();
    for lesson in held {
        let answer = word_number(&lesson.answer.concat());
        let mut found = Vec::new();
        fit_into(form, &lesson.question, &empty_filling(), &mut found);
        let (true, Some(filling)) = (numbered(&answer), found.first()) else { return Vec::new() };
        read.push((own.iter().map(|place| match filling.get(*place) {
            Some([word]) if numbered(word) => Some(*word),
            _ => None,
        }).collect(), answer));
    }
    let free: Vec<usize> = (0..PLAIN_PLACES).filter(|place| !own.contains(place)).take(WORKED_FREE_PLACES).collect();
    let (Some(result), Some(added)) = (free.first(), free.get(1)) else { return Vec::new() };
    let mut plain: Vec<(String, String)> = Vec::new();
    let mut more: Vec<(String, String)> = Vec::new();
    for one in 0..own.len() {
        for other in (0..own.len()).filter(|other| *other != one) {
            for (reckoning, words) in WORKED_STEPS.iter() {
                let results: Vec<(Option<Vec<WordId>>, WordId)> = read.iter().map(|(values, answer)| (values[one].zip(values[other]).and_then(|(first, second)| worked_words(*reckoning, &[first], &[second]).0), *answer)).collect();
                let step = [place_letter(own[one]).as_str(), *words, place_letter(own[other]).as_str(), WORKED_IS, place_letter(*result).as_str()].join(FORM_GAP);
                let right: Vec<bool> = results.iter().map(|(value, answer)| value.as_deref() == Some(std::slice::from_ref(answer))).collect();
                if right.contains(&true) {
                    plain.push((step.clone(), place_letter(*result)));
                }
                let differences: Vec<Option<Vec<WordId>>> = results.iter().filter_map(|(value, answer)| value.as_ref().map(|value| worked_words(Reckoning::Minus, &[*answer], value).0)).collect();
                if let Some(Some([number])) = differences.first().map(|difference| difference.as_deref())
                    && differences.len() >= WORKED_SUPPORT
                    && right.contains(&false)
                    && word_has(*number, WORD_IS_DIGITS)
                    && differences.iter().all(|difference| difference.as_deref() == Some(std::slice::from_ref(number)))
                {
                    let then = [place_letter(*result).as_str(), WORKED_STEPS[usize::default()].1, word_text(*number), WORKED_IS, place_letter(*added).as_str()].join(FORM_GAP);
                    more.push((format!("{step}{CONDITIONS_JOINED}{then}"), place_letter(*added)));
                }
            }
        }
    }
    plain.extend(more);
    plain
}
because!(worked_conditions, LearnedForms, "the conditions of answers that a step of arithmetic works out from two places of a form that hold a number in the lessons of its sample (as a step reads one: below nothing, with a point, or a whole number over another), each answer the result in a free place, when every lesson of the sample answers one number: each step on two such places, in either order, that answers some lesson right, and after them each step whose result falls short of every answer by the same whole number, with that number added (one more than the sum), as a worked example shows its step to a pupil; none for a form whose answers or places hold no numbers");

const WORKED_FREE_PLACES: usize = 2;
because!(WORKED_FREE_PLACES, LearnedForms, "the free places of a worked answer: the result of the step, and the result with the whole number added");

const WORKED_SUPPORT: usize = 3;
because!(WORKED_SUPPORT, LearnedForms, "the fewest lessons on which the result of a step must fall short of the answer by the same whole number before that number is added, so that it rests on more than a chance case");

pub(super) fn answers_numbered(held: &[&WordingLesson]) -> bool {
    !held.is_empty() && held.iter().all(|lesson| reads_as_number(word_number(&lesson.answer.concat())))
}
because!(answers_numbered, LearnedForms, "whether every lesson of a sample answers one number, as a step of arithmetic may work it out from numbers of the question (the measure cuts a number below nothing into its sign and its digits, which together read as the number)");

pub(super) fn told_patterns(cases: &[&Case]) -> Vec<(String, String)> {
    let examples: Option<Vec<Example>> = cases
        .iter()
        .map(|case| match case.sentences.as_slice() {
            [sentence] => Some(Example { input: fact_words(sentence), output: fact_words(&case.answer) }),
            _ => None,
        })
        .collect();
    let Some(examples) = examples.filter(|examples| !examples.is_empty()) else { return Vec::new() };
    grouped(&examples)
        .iter()
        .filter_map(|group| group_form(group))
        .filter(|form| form.output.iter().any(|piece| !matches!(piece, LogicPiece::Word(_))))
        .map(|form| (std::iter::once(library_writing(&form.input)).chain(form.worked).collect::<Vec<String>>().join(CONDITIONS_JOINED), library_writing(&form.output)))
        .collect()
}
because!(told_patterns, LearnedForms, "the conditions of answers made of the words of the one told sentence of each lesson of a sample, as worked examples show them: for each group of one length of input and output whose every output word is a fixed word, a word at a place of the input or a step of arithmetic, the largest group first, the told sentence in the form of its input with the steps, and the output as the answer that the places fill, when some word of it comes from the input (a label alone is no answer made of the input); none when a lesson tells more or fewer sentences than one");

pub(super) fn question_forms(cases: &[&Case]) -> Vec<Pattern> {
    let examples: Vec<Example> = cases.iter().filter(|case| case.sentences.is_empty()).map(|case| Example { input: fact_words(&case.question), output: fact_words(&case.answer) }).collect();
    grouped(&examples).iter().filter_map(|group| group_form(group)).filter(|form| !form.worked.is_empty()).map(|form| form.input).collect()
}
because!(question_forms, LearnedForms, "the forms of questions that worked examples show when a question holds its own data and too many runs change for a cut of the question to give a form (a sum among choices to pick from): for each group of one length of question and answer, the largest first, whose answer a step of arithmetic works out, the question with a fixed word where every example has the same word, a place where the words change, and after the last place that the answer reads, one run of any signs");

pub(super) fn patterns_told(cases: &[&Case]) -> bool {
    !told_patterns(cases).is_empty()
}
because!(patterns_told, LearnedForms, "whether the answers of some lessons are made of the words of their one told sentence, as an answer form of worked examples shows them");
