use ilm_core::base::first_sight::FirstSightCount;
use ilm_core::base::words::answer_words;
use crate::controls::parts::PartsChange;
use crate::controls::text::capital_name_runs;
use crate::sets::readers::common::python_words;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::{BTreeSet, HashMap, HashSet};

pub struct SentenceForms;
source!(SentenceForms, "the forms of the sentences of three kinds, each counted from the lessons of its kind. A chain of statements about who lies: a name is a word with a capital letter that stands in at least a pair of told sentences, a told sentence with its names taken out is its form, each form turns the truth or keeps it, and which forms turn it and which of the two most frequent answers comes first are the choice that gives the answer of most lessons. A sentence into a question: the first word of a lesson whose answer is the rest of its text means no change, the most frequent other first word asks for the question, the helping verbs are the first words of the answers of questions, and a clause on a noun is marked by who or that, or by commas where the lessons have commas. A sentence into the passive: the first words in the same way, the verbs are the words after was or were in the answers of the lessons, and the pairs of the forms of a noun are the nouns of the lessons that differ by an ending");

const LIES_MARK: &str = "#";
because!(LIES_MARK, SentenceForms, "the sign that stands for a name in the form of a told sentence");

const LIES_NEW_NAME: &str = "Zorblat";
because!(LIES_NEW_NAME, SentenceForms, "a name that no lesson holds, for the person of one more statement");

const LIES_LEAST_TELLS: i64 = 2;
because!(LIES_LEAST_TELLS, SentenceForms, "the told sentences that a word with a capital letter must stand in to be a name");

const LIES_PAIR: usize = 2;
because!(LIES_PAIR, SentenceForms, "the names of a statement of one person about another, and the answers of the kind");

const SORT_LIES_OTHER: &str = "other";
because!(SORT_LIES_OTHER, SentenceForms, "the sort of the change where a statement becomes the other form of statement with its names");

const SORT_LIES_MORE: &str = "more";
because!(SORT_LIES_MORE, SentenceForms, "the sort of the change where one more statement about the asked person is told by a new person, who is asked about then");

const FORM_GAP: &str = " ";
because!(FORM_GAP, SentenceForms, "the mark between the words of a changed sentence");

const FORM_NEW_WORD: &str = "zorblat";
because!(FORM_NEW_WORD, SentenceForms, "a word that no lesson holds, for the last word of a sentence or for a noun");

const QUESTION_MARKS: &[&str] = &["who", "that"];
because!(QUESTION_MARKS, SentenceForms, "the words that start a clause on a noun in English");

const QUESTION_COMMA: &str = ",";
because!(QUESTION_COMMA, SentenceForms, "the mark on both sides of a clause on a noun in German");

const QUESTION_NEW_BUILD: &str = " (the subject has a clause of its own)";
because!(QUESTION_NEW_BUILD, SentenceForms, "the end of the name of a sort of change whose question has a subject with a clause of its own, a build that no question of the lessons has");

const SORT_FORM_KIND: &str = "kind";
because!(SORT_FORM_KIND, SentenceForms, "the sort of the change where the first word becomes the other first word");

const SORT_FORM_VERB: &str = "verb";
because!(SORT_FORM_VERB, SentenceForms, "the sort of the change where the verb becomes another verb");

const SORT_FORM_ITEM: &str = "item";
because!(SORT_FORM_ITEM, SentenceForms, "the sort of the change where a word becomes a word that no lesson holds");

const SORT_FORM_SIDES: &str = "sides";
because!(SORT_FORM_SIDES, SentenceForms, "the sort of the change where the two noun groups change places");

const SORT_FORM_NUMBER: &str = "number";
because!(SORT_FORM_NUMBER, SentenceForms, "the sort of the change where a noun becomes its other form, one or many");

const SORT_FORM_NEW_KEY: &str = "new key";
because!(SORT_FORM_NEW_KEY, SentenceForms, "the sort of the change in a passive where the noun after the verb becomes a word that no lesson holds, so that no table of a tool has it");

const PASSIVE_SINGLE: &str = "was";
because!(PASSIVE_SINGLE, SentenceForms, "the helping verb of the passive for one");

const PASSIVE_MANY: &str = "were";
because!(PASSIVE_MANY, SentenceForms, "the helping verb of the passive when its subject is a noun for more than one");

const PASSIVE_BY: &str = "by";
because!(PASSIVE_BY, SentenceForms, "the word before the doer in the passive");

const PASSIVE_ENDINGS: &[&str] = &["s", "es"];
because!(PASSIVE_ENDINGS, SentenceForms, "the endings that make the form of a noun for many");

const PASSIVE_MANY_END: &str = "s";
because!(PASSIVE_MANY_END, SentenceForms, "the ending of a noun for many, after which the passive takes the verb for many");

const PASSIVE_ONE_END: &str = "us";
because!(PASSIVE_ONE_END, SentenceForms, "the ending of a noun for one that also ends with the ending for many");

const PASSIVE_NOUN_PLACE: usize = 2;
because!(PASSIVE_NOUN_PLACE, SentenceForms, "the place of the noun in the text of a lesson: after the first word and the word before the noun");

const PASSIVE_NEXT: usize = 2;
because!(PASSIVE_NEXT, SentenceForms, "a word of an answer and the word after it, for the verb after was or were");

const PASSIVE_LEAST_GROUP: usize = 2;
because!(PASSIVE_LEAST_GROUP, SentenceForms, "the fewest words of a noun group: a word before the noun and the noun");

type ToldForms = (Vec<(String, Vec<String>)>, String);
because!(ToldForms, SentenceForms, "the form of each told sentence with its names in their order, and the asked name");

fn lies_parts(sentences: &[String], question: &str) -> Option<ToldForms> {
    let mut there: HashMap<&str, i64> = HashMap::new();
    for text in sentences {
        let names: HashSet<&str> = capital_name_runs(text).iter().map(|(start, end)| &text[*start..*end]).collect();
        for name in names {
            *there.entry(name).or_default() += 1;
        }
    }
    let asked_names: Vec<&str> = capital_name_runs(question).iter().map(|(start, end)| &question[*start..*end]).collect();
    let mut names: HashSet<&str> = there.iter().filter(|(_, count)| **count >= LIES_LEAST_TELLS).map(|(name, _)| *name).collect();
    names.extend(asked_names.iter().filter(|name| there.contains_key(*name)));
    let mut told: Vec<(String, Vec<String>)> = Vec::new();
    for text in sentences {
        let mut form = String::new();
        let mut found: Vec<String> = Vec::new();
        let mut last = 0;
        for (start, end) in capital_name_runs(text) {
            let name = &text[start..end];
            if names.contains(name) {
                form.push_str(&text[last..start]);
                form.push_str(LIES_MARK);
                last = end;
                found.push(name.to_string());
            }
        }
        form.push_str(&text[last..]);
        told.push((form, found));
    }
    let asked = asked_names.into_iter().filter(|name| names.contains(name)).last()?;
    Some((told, asked.to_string()))
}
because!(lies_parts, SentenceForms, "the forms of the told sentences of a case with their names, and the asked name; nothing when the question names nobody of the story");

fn lies_filled(form: &str, names: &[&str]) -> String {
    let mut out = String::new();
    let mut names = names.iter();
    let mut pieces = form.split(LIES_MARK);
    out.push_str(pieces.next().unwrap_or_default());
    for piece in pieces {
        out.push_str(names.next().copied().unwrap_or_default());
        out.push_str(piece);
    }
    out
}
because!(lies_filled, SentenceForms, "a form with its names in the places of the signs, in their order");

pub struct LiesForm {
    turn: HashMap<String, bool>,
    first: usize,
    answers: Vec<Vec<String>>,
    two: Vec<String>,
}
because!(LiesForm, SentenceForms, "the reading of a chain of statements: whether each form turns the truth, which answer comes first, the two answers, and the forms of a statement of one person about another. counted takes them from the lessons; exact gives the answer of a case; changes gives its changed forms");

impl LiesForm {
    pub fn counted(lessons: &[Case]) -> Option<LiesForm> {
        let read: Vec<(ToldForms, Vec<String>)> = lessons.iter().filter_map(|case| lies_parts(&case.sentences, &case.question).map(|parts| (parts, answer_words(&case.answer)))).collect();
        let forms: Vec<String> = read.iter().flat_map(|((told, _), _)| told.iter().map(|(form, _)| form.clone())).collect::<BTreeSet<String>>().into_iter().collect();
        let mut counts: FirstSightCount<Vec<String>> = FirstSightCount::new();
        for case in lessons {
            counts.add(answer_words(&case.answer), 1);
        }
        let answers: Vec<Vec<String>> = counts.by_count().into_iter().take(LIES_PAIR).map(|(answer, _)| answer.clone()).collect();
        if answers.len() < LIES_PAIR || forms.len() >= usize::BITS as usize {
            return None;
        }
        let place: HashMap<&str, usize> = forms.iter().enumerate().map(|(at, form)| (form.as_str(), at)).collect();
        let lesson_forms: Vec<(Vec<usize>, &Vec<String>)> = read.iter().map(|((told, _), said)| (told.iter().map(|(form, _)| place[form.as_str()]).collect(), said)).collect();
        let mut best: Option<(usize, usize, usize)> = None;
        for bits in 0..(1usize << forms.len()) {
            for first in [0, 1] {
                let hit = lesson_forms.iter().filter(|(found, said)| answers[(first + found.iter().map(|at| bits >> at & 1).sum::<usize>()) % LIES_PAIR] == **said).count();
                if best.is_none_or(|(top, _, _)| hit > top) {
                    best = Some((hit, bits, first));
                }
            }
        }
        let (_, bits, first) = best?;
        let turn = forms.iter().enumerate().map(|(at, form)| (form.clone(), bits >> at & 1 == 1)).collect();
        let two = forms.iter().filter(|form| form.matches(LIES_MARK).count() == LIES_PAIR).cloned().collect();
        Some(LiesForm { turn, first, answers, two })
    }

    pub fn exact(&self, sentences: &[String], question: &str) -> Option<Vec<String>> {
        let (told, _) = lies_parts(sentences, question)?;
        let mut turned = self.first;
        for (form, _) in &told {
            turned += usize::from(*self.turn.get(form)?);
        }
        Some(self.answers[turned % LIES_PAIR].clone())
    }

    pub fn changes(&self, sentences: &[String], question: &str) -> Vec<PartsChange> {
        let Some((told, asked)) = lies_parts(sentences, question) else { return Vec::new() };
        if self.two.len() != LIES_PAIR {
            return Vec::new();
        }
        let mut out: Vec<PartsChange> = Vec::new();
        for (at, (form, names)) in told.iter().enumerate() {
            let Some(which) = self.two.iter().position(|held| held == form) else { continue };
            if names.len() != LIES_PAIR {
                continue;
            }
            let named: Vec<&str> = names.iter().map(String::as_str).collect();
            let mut changed = sentences.to_vec();
            changed[at] = lies_filled(&self.two[1 - which], &named);
            out.push((SORT_LIES_OTHER.to_string(), changed, question.to_string()));
        }
        for form in &self.two {
            let mut changed = sentences.to_vec();
            changed.push(lies_filled(form, &[LIES_NEW_NAME, asked.as_str()]));
            out.push((SORT_LIES_MORE.to_string(), changed, question.replace(asked.as_str(), LIES_NEW_NAME)));
        }
        out
    }
}

fn first_words(lessons: &[Case]) -> Option<(String, String)> {
    let mut plain: FirstSightCount<String> = FirstSightCount::new();
    for case in lessons {
        let words = python_words(&case.question);
        if words.len() > 1 && answer_words(&words[1..].join(FORM_GAP)) == answer_words(&case.answer) {
            plain.add(words[0].to_string(), 1);
        }
    }
    let plain = plain.first_most()?.0.clone();
    let mut other: FirstSightCount<String> = FirstSightCount::new();
    for case in lessons {
        if let Some(first) = python_words(&case.question).first()
            && *first != plain
        {
            other.add(first.to_string(), 1);
        }
    }
    let other = other.first_most()?.0.clone();
    Some((plain, other))
}
because!(first_words, SentenceForms, "the first word that means no change, the most frequent of the lessons whose answer is the rest of their text, and the most frequent other first word");

fn split_first(question: &str) -> Option<(&str, Vec<&str>)> {
    let words = python_words(question);
    let (first, rest) = words.split_first()?;
    Some((*first, rest.to_vec()))
}
because!(split_first, SentenceForms, "the first word of a text and its other words");

fn last_word(words: &[&str]) -> Option<usize> {
    words.iter().rposition(|word| !answer_words(word).is_empty())
}
because!(last_word, SentenceForms, "the place of the last word that is a word of an answer, not a mark alone");

pub struct QuestionForm {
    plain: String,
    asks: String,
    verbs: Vec<String>,
    german: bool,
}
because!(QuestionForm, SentenceForms, "the reading of a sentence into a question: the first word for no change, the first word for a question, the helping verbs, and whether clauses are marked by commas. counted takes them from the lessons; exact gives the answer of a text; changes gives its changed forms");

impl QuestionForm {
    pub fn counted(lessons: &[Case]) -> Option<QuestionForm> {
        let (plain, asks) = first_words(lessons)?;
        let verbs: BTreeSet<String> = lessons.iter().filter(|case| python_words(&case.question).first() == Some(&asks.as_str())).filter_map(|case| python_words(&case.answer).first().map(|word| word.to_lowercase())).collect();
        let german = lessons.iter().any(|case| python_words(&case.question).contains(&QUESTION_COMMA));
        Some(QuestionForm { plain, asks, verbs: verbs.into_iter().collect(), german })
    }

    fn main_verb(&self, words: &[&str]) -> (Option<usize>, bool) {
        let (mut inside, mut clause) = (false, false);
        for (at, word) in words.iter().enumerate() {
            let low = word.to_lowercase();
            if self.german && *word == QUESTION_COMMA {
                inside = !inside;
                clause = true;
            } else if !self.german && QUESTION_MARKS.contains(&low.as_str()) {
                inside = true;
                clause = true;
            } else if self.verbs.contains(&low) {
                if !inside {
                    return (Some(at), clause);
                }
                if !self.german {
                    inside = false;
                }
            }
        }
        (None, clause)
    }

    pub fn exact(&self, question: &str) -> Option<Vec<String>> {
        let (first, words) = split_first(question)?;
        if first == self.plain {
            return Some(answer_words(&words.join(FORM_GAP)));
        }
        let at = self.main_verb(&words).0?;
        if first != self.asks {
            return None;
        }
        Some(answer_words(&[&[words[at]], &words[..at], &words[at + 1..]].concat().join(FORM_GAP)))
    }

    pub fn changes(&self, sentences: &[String], question: &str) -> Vec<PartsChange> {
        let Some((first, words)) = split_first(question) else { return Vec::new() };
        let (Some(at), clause) = self.main_verb(&words) else { return Vec::new() };
        let other = if first == self.plain { self.asks.as_str() } else { self.plain.as_str() };
        let build = if clause { QUESTION_NEW_BUILD } else { "" };
        let asked_build = if first == self.asks { build } else { "" };
        let mut out: Vec<PartsChange> = Vec::new();
        let kind_build = if other == self.asks { build } else { "" };
        out.push((format!("{SORT_FORM_KIND}{kind_build}"), sentences.to_vec(), [&[other], &words[..]].concat().join(FORM_GAP)));
        let verb = words[at].to_lowercase();
        for other_verb in self.verbs.iter().filter(|held| **held != verb) {
            out.push((format!("{SORT_FORM_VERB}{asked_build}"), sentences.to_vec(), [&[first], &words[..at], &[other_verb.as_str()], &words[at + 1..]].concat().join(FORM_GAP)));
        }
        if let Some(last) = last_word(&words)
            && last != at
        {
            out.push((format!("{SORT_FORM_ITEM}{asked_build}"), sentences.to_vec(), [&[first], &words[..last], &[FORM_NEW_WORD], &words[last + 1..]].concat().join(FORM_GAP)));
        }
        out
    }
}

pub struct PassiveForm {
    plain: String,
    turns: String,
    verbs: Vec<String>,
    pair: HashMap<String, String>,
}
because!(PassiveForm, SentenceForms, "the reading of a sentence into the passive: the first word for no change, the first word for the passive, the verbs, and the other form of each noun that has a pair. counted takes them from the lessons; exact gives the answer of a text; changes gives its changed forms");

type NounGroups<'a> = (Vec<&'a str>, &'a str, Vec<&'a str>);
because!(NounGroups, SentenceForms, "the noun group before the verb, the verb, and the noun group after it");

impl PassiveForm {
    pub fn counted(lessons: &[Case]) -> Option<PassiveForm> {
        let (plain, turns) = first_words(lessons)?;
        let mut verbs: BTreeSet<String> = BTreeSet::new();
        for case in lessons.iter().filter(|case| python_words(&case.question).first() == Some(&turns.as_str())) {
            let words = python_words(&case.answer);
            for pair in words.windows(PASSIVE_NEXT) {
                if [PASSIVE_SINGLE, PASSIVE_MANY].contains(&pair[0]) {
                    verbs.insert(pair[1].to_string());
                }
            }
        }
        let nouns: BTreeSet<String> = lessons.iter().filter_map(|case| python_words(&case.question).get(PASSIVE_NOUN_PLACE).map(|noun| noun.to_string())).collect();
        let mut pair: HashMap<String, String> = HashMap::new();
        for noun in &nouns {
            for end in PASSIVE_ENDINGS {
                let many = format!("{noun}{end}");
                if nouns.contains(&many) {
                    pair.insert(noun.clone(), many.clone());
                    pair.insert(many, noun.clone());
                }
            }
        }
        Some(PassiveForm { plain, turns, verbs: verbs.into_iter().collect(), pair })
    }

    fn groups<'a>(&self, words: &[&'a str]) -> Option<NounGroups<'a>> {
        let at: Vec<usize> = words.iter().enumerate().filter(|(_, word)| self.verbs.iter().any(|verb| verb == *word)).map(|(place, _)| place).collect();
        let [verb] = at[..] else { return None };
        let last = last_word(words)?;
        if verb < PASSIVE_LEAST_GROUP || last < verb + PASSIVE_LEAST_GROUP {
            return None;
        }
        Some((words[..verb].to_vec(), words[verb], words[verb + 1..=last].to_vec()))
    }

    pub fn exact(&self, question: &str) -> Option<Vec<String>> {
        let (first, words) = split_first(question)?;
        if first == self.plain {
            return Some(answer_words(&words.join(FORM_GAP)));
        }
        let (before, verb, after) = self.groups(&words)?;
        if first != self.turns {
            return None;
        }
        let be = if after[1].ends_with(PASSIVE_MANY_END) && !after[1].ends_with(PASSIVE_ONE_END) { PASSIVE_MANY } else { PASSIVE_SINGLE };
        Some(answer_words(&[&after[..], &[be, verb, PASSIVE_BY], &before[..]].concat().join(FORM_GAP)))
    }

    pub fn changes(&self, sentences: &[String], question: &str) -> Vec<PartsChange> {
        let Some((first, words)) = split_first(question) else { return Vec::new() };
        let Some((before, verb, after)) = self.groups(&words) else { return Vec::new() };
        let rest = &words[before.len() + 1 + after.len()..];
        let mut out: Vec<PartsChange> = Vec::new();
        let mut add = |sort: &str, parts: Vec<&str>| out.push((sort.to_string(), sentences.to_vec(), parts.join(FORM_GAP)));
        let other_first = if first == self.plain { self.turns.as_str() } else { self.plain.as_str() };
        add(SORT_FORM_KIND, [&[other_first], &words[..]].concat());
        for other in self.verbs.iter().filter(|held| *held != verb) {
            add(SORT_FORM_VERB, [&[first], &before[..], &[other.as_str()], &after[..], rest].concat());
        }
        add(SORT_FORM_SIDES, [&[first], &after[..], &[verb], &before[..], rest].concat());
        if let Some(other_form) = self.pair.get(after[1]) {
            add(SORT_FORM_NUMBER, [&[first], &before[..], &[verb], &after[..1], &[other_form.as_str()], &after[PASSIVE_LEAST_GROUP..], rest].concat());
        }
        add(SORT_FORM_ITEM, [&[first], &before[..1], &[FORM_NEW_WORD], &before[PASSIVE_LEAST_GROUP..], &[verb], &after[..], rest].concat());
        if first == self.turns {
            add(SORT_FORM_NEW_KEY, [&[first], &before[..], &[verb], &after[..1], &[FORM_NEW_WORD], &after[PASSIVE_LEAST_GROUP..], rest].concat());
        }
        out
    }
}
