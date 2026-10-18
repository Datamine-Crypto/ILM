use crate::cases::Case;
use crate::tools::explain::bridges::place_map;
use crate::tools::explain::chain::{Facts, Filling, empty_filling, fill, fit_into};
use crate::tools::explain::text::{AskedForm, Condition, LogicPiece, Pattern, text_words, word_not_mark};
use crate::tools::explain::words::{WordId, word_number, word_text};
use crate::tools::explain::wording::{FORM_WORD_GAP, STATEMENT_END, places_of};
use patterns::{because, source};
use std::collections::HashSet;

pub struct AbducedLessons;
source!(AbducedLessons, "abduction as Peirce named it, the inference from a result back to the case that explains it under a known rule: a question form of the text says which facts give an answer (son if A is a parent of B and B is D and C means a D child), so the answer of a lesson tells the facts that its story must make (Ashley is a parent of Nicholas, Nicholas is male), while the facts that the form's other answers need do not hold; and a told sentence that holds the values of a question and an answer (Ashley daughter Lillian) tells a lesson of one step outright, as the story answers that question itself");

pub(super) const STATED_HOLDS: &str = "yes";
because!(STATED_HOLDS, AbducedLessons, "the answer of a lesson that states a fact that the answer of its own lesson needs");

pub(super) const STATED_FAILS: &str = "no";
because!(STATED_FAILS, AbducedLessons, "the answer of a lesson that states a fact that another answer of the question form needs, which the answer given rules out");

pub(super) fn one_step_lessons(lessons: &[(&Case, Vec<String>)], answers: &HashSet<String>) -> Vec<Case> {
    let mut out: Vec<Case> = Vec::new();
    for (case, values) in lessons {
        for sentence in &case.sentences {
            let written: Vec<String> = text_words(sentence).into_iter().filter(|word| word_not_mark(word)).collect();
            if written.len() != values.len() + 1 {
                continue;
            }
            let said: Vec<usize> = (0..written.len()).filter(|at| answers.contains(&written[*at].to_lowercase())).collect();
            let [answer] = said.as_slice() else { continue };
            let named: Vec<&String> = written.iter().enumerate().filter(|(at, _)| at != answer).map(|(_, word)| word).collect();
            if !named.iter().all(|word| word.chars().next().is_some_and(char::is_uppercase)) {
                continue;
            }
            let mut question = case.question.clone();
            let mut whole = true;
            for (old, new) in values.iter().zip(&named) {
                whole &= question.matches(old.as_str()).count() == 1;
                question = question.replacen(old.as_str(), new, 1);
            }
            let lesson = Case { sentences: vec![sentence.clone()], question, answer: written[*answer].to_lowercase() };
            if whole && !out.iter().any(|known| known.sentences == lesson.sentences && known.question == lesson.question) {
                out.push(lesson);
            }
        }
    }
    out
}
because!(one_step_lessons, AbducedLessons, "the lessons of one step that the told sentences of some lessons tell: a told sentence of as many names as the question has values and one word more that the lessons give as an answer, as the question of its lesson with those names in its places, in the order of the sentence, answered by that word, with that sentence alone as its story; each once");

struct Resolved {
    filling: Filling,
    facts: Vec<Pattern>,
    bound: Vec<(usize, Pattern)>,
}
because!(Resolved, AbducedLessons, "one answer of a question form resolved for a lesson: the filling of its places, the facts that the story must make, and each place that a known fact filled with the sentence of that known fact");

fn resolved(base: &Facts, known: usize, conditions: &[Condition], filling: Filling) -> Option<Resolved> {
    let mut filling = filling;
    let mut bound: Vec<(usize, Pattern)> = Vec::new();
    let mut open: Vec<&Pattern> = conditions.iter().map(|condition| if let Condition::Fact { pattern, .. } = condition { Some(pattern) } else { None }).collect::<Option<Vec<&Pattern>>>()?;
    loop {
        let mut grew = false;
        open.retain(|pattern| {
            if fill(pattern, &filling).is_some() {
                return true;
            }
            let fits: Vec<Filling> = (0..known).flat_map(|place| {
                let mut found = Vec::new();
                fit_into(pattern, base.fact(place), &filling, &mut found);
                found
            }).collect();
            match fits.as_slice() {
                [only] => {
                    bound.extend(places_of(pattern).into_iter().filter(|place| !filling.filled(*place)).map(|place| (place, (*pattern).clone())));
                    filling = only.clone();
                    grew = true;
                    false
                }
                _ => true,
            }
        });
        if !grew {
            break;
        }
    }
    if open.iter().any(|pattern| fill(pattern, &filling).is_none()) {
        return None;
    }
    let facts: Vec<Pattern> = open.into_iter().filter(|pattern| fill(pattern, &filling).is_some_and(|words| !(0..known).any(|place| base.fact(place) == words.as_slice()))).cloned().collect();
    Some(Resolved { filling, facts, bound })
}
because!(resolved, AbducedLessons, "the conditions of one answer of a question form under the filling that the question and the answer give: each condition that one known fact alone fits fills its places from that fact (son means a D child: D is male), again while one does, and the conditions left, all filled, are the facts that the story must make; nothing when a condition is not a sentence, or stays open, as the answer then does not tell what its story holds");

pub(super) fn abduced_facts(base: &Facts, form: &AskedForm, (values, answer): (&[Vec<WordId>], &[WordId]), order: &[usize]) -> (Vec<(Vec<WordId>, Pattern)>, Vec<(Vec<WordId>, Pattern)>) {
    let known = base.added_to(&Facts::default());
    let mut asked = empty_filling();
    for (value, place) in values.iter().zip(order) {
        asked.set(*place, value.clone());
    }
    let mut held: Vec<(Vec<WordId>, Pattern)> = Vec::new();
    let mut ruled_out: Vec<(Vec<WordId>, Pattern)> = Vec::new();
    for reply in &form.replies {
        let mut said = Vec::new();
        fit_into(&reply.words, answer, &asked, &mut said);
        let answered = said.into_iter().next().and_then(|filling| resolved(base, known, &reply.conditions, filling));
        match answered {
            Some(found) => {
                for pattern in &found.facts {
                    if let Some(words) = fill(pattern, &found.filling)
                        && !held.iter().any(|(known_words, _)| *known_words == words)
                    {
                        held.push((words, pattern.clone()));
                    }
                    for other in alternatives(base, known, pattern, (&found.filling, &found.bound)) {
                        ruled_out.push((other, pattern.clone()));
                    }
                }
            }
            None => {
                for condition in &reply.conditions {
                    if let Condition::Fact { pattern, .. } = condition
                        && places_of(pattern).iter().all(|place| asked.filled(*place))
                        && let Some(words) = fill(pattern, &asked)
                    {
                        ruled_out.push((words, pattern.clone()));
                    }
                }
            }
        }
    }
    let mut out: Vec<(Vec<WordId>, Pattern)> = Vec::new();
    for (words, pattern) in ruled_out {
        if !held.iter().any(|(known_words, _)| *known_words == words) && held.iter().any(|(_, known)| place_map(known, &pattern).is_some()) && !out.iter().any(|(known_words, _)| *known_words == words) {
            out.push((words, pattern));
        }
    }
    (held, out)
}
because!(abduced_facts, AbducedLessons, "the facts that a lesson's answer needs under a question form of the text, each with the sentence of the form that it fills, with the question's values in its places in the order given, and the facts of the same sentences that it rules out: an answer of the form whose words fit the lesson's answer and whose conditions resolve gives its facts as held, and each place of them that a known fact filled gives, with the other values that the known facts of that sentence give it, the same fact as ruled out (male for female); an answer that does not fit gives its conditions that the question's values alone fill as ruled out, kept only when it is a sentence of a held fact with its places in another order (Lillian is a parent of Ashley beside Ashley is a parent of Lillian), as such facts tell which way a sentence reads, while the sentences of the other answers (a spouse, a sibling) would bury the held facts among many");

fn alternatives(base: &Facts, known: usize, pattern: &Pattern, (filling, bound): (&Filling, &[(usize, Pattern)])) -> Vec<Vec<WordId>> {
    let Some(words) = fill(pattern, filling) else { return Vec::new() };
    let mut out: Vec<Vec<WordId>> = Vec::new();
    for (place, source) in bound.iter().filter(|(place, _)| places_of(pattern).contains(place)) {
        for at in 0..known {
            let mut fits = Vec::new();
            fit_into(source, base.fact(at), &empty_filling(), &mut fits);
            for value in fits.iter().filter_map(|found| found.get(*place).map(<[WordId]>::to_vec)) {
                let mut other = filling.clone();
                other.set(*place, value);
                if let Some(found) = fill(pattern, &other)
                    && found != words
                    && !out.contains(&found)
                {
                    out.push(found);
                }
            }
        }
    }
    out
}
because!(alternatives, AbducedLessons, "the same fact as a held one with a place that a known fact filled given another value that the known facts of that sentence give it (Lillian is male beside Lillian is female, as the relations hold both a male and a female child): the facts that the held one rules out");

pub(super) fn form_answers(base: &Facts, form: &AskedForm) -> HashSet<String> {
    let known = base.added_to(&Facts::default());
    let mut out: HashSet<String> = HashSet::new();
    for reply in &form.replies {
        let [LogicPiece::Open(answer)] = reply.words.as_slice() else { continue };
        for condition in &reply.conditions {
            let Condition::Fact { pattern, .. } = condition else { continue };
            if !places_of(pattern).contains(answer) {
                continue;
            }
            for at in 0..known {
                let mut fits = Vec::new();
                fit_into(pattern, base.fact(at), &empty_filling(), &mut fits);
                out.extend(fits.iter().filter_map(|found| match found.get(*answer) {
                    Some([word]) => Some(word_text(*word).to_string()),
                    _ => None,
                }));
            }
        }
    }
    out
}
because!(form_answers, AbducedLessons, "the words that a question form can give as an answer: the values that the known facts give the answer's place in a sentence of facts that holds it (wife, from Wife means a female spouse), so that a told sentence that names a relation which no lesson asks (Clarence wife Juanita) is a lesson of one step too");

pub(super) fn statement_lessons(story: &[String], (held, ruled_out): (&[(Vec<WordId>, Pattern)], &[(Vec<WordId>, Pattern)]), names: &HashSet<String>) -> Vec<(Pattern, Case)> {
    let said = |words: &[WordId]| -> String {
        let text: Vec<String> = words.iter().map(|word| {
            let plain = word_text(*word);
            match names.iter().find(|name| name.to_lowercase() == plain) {
                Some(name) => name.clone(),
                None => plain.to_string(),
            }
        }).collect();
        format!("{}{STATEMENT_END}", capitalized(&text.join(FORM_WORD_GAP)))
    };
    held.iter().map(|(words, pattern)| (pattern.clone(), Case { sentences: story.to_vec(), question: said(words), answer: STATED_HOLDS.to_string() }))
        .chain(ruled_out.iter().map(|(words, pattern)| (pattern.clone(), Case { sentences: story.to_vec(), question: said(words), answer: STATED_FAILS.to_string() })))
        .collect()
}
because!(statement_lessons, AbducedLessons, "the lessons that state the facts of a lesson's answer, each with the lesson's story and the sentence of the form that it states: a held fact answered yes, a ruled out fact answered no, each written as a sentence with the names as the story writes them");

pub(super) fn capitalized(text: &str) -> String {
    let mut letters = text.chars();
    letters.next().map_or_else(String::new, |first| first.to_uppercase().chain(letters).collect())
}
because!(capitalized, AbducedLessons, "a sentence with its first letter written large, as a told sentence starts");

pub(super) fn capital_words(lessons: &[&Case]) -> HashSet<String> {
    lessons.iter().flat_map(|case| case.sentences.iter().chain(std::iter::once(&case.question))).flat_map(|text| text_words(text)).filter(|word| word.chars().next().is_some_and(char::is_uppercase) && word.chars().skip(1).any(char::is_lowercase)).collect()
}
because!(capital_words, AbducedLessons, "the words that the told sentences and questions of some lessons write with a capital letter first, as names are written, to write the stated facts with their names as the story writes them");

pub(super) fn word_ids(words: &[String]) -> Vec<WordId> {
    words.iter().map(|word| word_number(&word.to_lowercase())).collect()
}
because!(word_ids, AbducedLessons, "the numbers of written words as the rules see them, in small letters");
