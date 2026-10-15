use crate::base::exact::Amount;
use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words};
use crate::cases::Case;
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::list::amounts::{LIST_MOST_DIGITS, LIST_SURE_N, LIST_SURE_SHARE, ListCounts, parts_of};
use crate::tools::list::classes::ListClasses;
use crate::tools::list::words::ListToolWords;
use patterns::{because, source};

pub struct ListAnswers;
source!(ListAnswers, "the answer of a told list is the sum of the amounts of the things that belong, each step an exact addition; its words are those of the lesson answers with this amount, or the digits when the lessons answer in digits. The tool reads a kind only when the lessons are lists with amounts and when a model gives back nearly every answer of its own lessons; else it answers nothing, so it does not answer a kind with no amounts by chance");

const LIST_NO_ANSWER: &str = "no answer: the lessons are no lists with amounts";
because!(LIST_NO_ANSWER, ListAnswers, "the note of a model that does not read its kind");

const LIST_NO_TEXT: &str = "no lesson answer has this amount";
because!(LIST_NO_TEXT, ListAnswers, "the note of a sum that no lesson answer has, so the tool gives no words");

const LIST_ALL_DECIDED: &str = "list";
because!(LIST_ALL_DECIDED, ListAnswers, "the last word of the note when the lessons decide every thing of the list and know every amount");

const LIST_GUESSED: &str = "best";
because!(LIST_GUESSED, ListAnswers, "the last word of the note when a thing or an amount of the list was guessed");

const LIST_PLUS: &str = " + ";
because!(LIST_PLUS, ListAnswers, "the mark between the amounts of a sum as the note writes it");

const LIST_NOTE_BETWEEN: &str = "; ";
because!(LIST_NOTE_BETWEEN, ListAnswers, "the mark that parts the sum, the things that were left out and the last word of an answer note, so that a reader can cut the note into its parts");

pub struct ListModel {
    pub counts: ListCounts,
    pub words: ListToolWords,
    pub classes: ListClasses,
    pub reads: bool,
}
because!(ListModel, ListAnswers, "the model of the list tool: the counts of the lists, the words, the classes of the things, and whether it reads its kind. made makes it and tests it on its own lessons; said gives the words of an answer and its note");

impl ListModel {
    pub fn made(lessons: &Lessons) -> ListModel {
        let cases = lessons.cases;
        let mut symbols = Symbols::new();
        for case in cases {
            for sentence in &case.sentences {
                symbols.numbers(&words(sentence));
            }
            symbols.numbers(&words(&case.question));
            symbols.numbers(&answer_words(&case.answer));
        }
        let rows: Vec<(Vec<Vec<String>>, &Case)> = cases.iter().map(|case| (parts_of(&case.sentences), case)).filter(|(parts, _)| !parts.is_empty()).collect();
        let mut model = ListModel { counts: ListCounts::default(), words: ListToolWords::default(), classes: ListClasses::default(), reads: false };
        if rows.is_empty() {
            return model;
        }
        model.counts = ListCounts::learn(&rows, symbols.clone());
        if !model.counts.reads {
            return model;
        }
        model.words = ListToolWords::learn(cases, symbols);
        let mut lessons: Vec<(Vec<u32>, Vec<(Vec<u32>, Amount)>, Amount)> = Vec::new();
        for (parts, case) in &rows {
            let numbered: Vec<Vec<u32>> = parts.iter().map(|part| model.counts.numbered(part, &mut Vec::new())).collect();
            let items = model.counts.items(&numbered);
            let Some(total) = model.counts.answer_amount(&case.answer) else { continue };
            let amounts: Option<Vec<Amount>> = items.iter().map(|(first, _)| model.counts.amount_of(*first, &[])).collect();
            let Some(amounts) = amounts else { continue };
            let mut things: Vec<(Vec<u32>, Amount)> = Vec::new();
            for ((_, found), value) in items.iter().zip(amounts) {
                let thing = model.words.thing(found);
                match things.iter_mut().find(|(seen, _)| *seen == thing) {
                    Some((_, sum)) => *sum = sum.plus(value).unwrap_or(*sum),
                    None => things.push((thing, value)),
                }
            }
            let question = model.counts.numbered(&words(&case.question), &mut Vec::new());
            lessons.push((question, things, total));
        }
        model.classes = ListClasses::learn(&lessons);
        model.reads = true;
        let back = cases.iter().filter(|case| model.said(&case.sentences, &case.question).words == answer_words(&case.answer)).count();
        model.reads = back as i64 >= LIST_SURE_N && (back as f64) >= LIST_SURE_SHARE * (cases.len() as f64);
        model
    }

    fn said(&self, sentences: &[String], question: &str) -> Said {
        if !self.reads {
            return Said { words: Vec::new(), note: LIST_NO_ANSWER.to_string() };
        }
        let mut fresh: Vec<String> = Vec::new();
        let asked = self.counts.numbered(&words(question), &mut fresh);
        let parts: Vec<Vec<u32>> = parts_of(sentences).iter().map(|part| self.counts.numbered(part, &mut fresh)).collect();
        let mut total = Amount::whole(0);
        let mut sure = true;
        let mut counted: Vec<String> = Vec::new();
        let mut left: Vec<String> = Vec::new();
        for (first, found) in self.counts.items(&parts) {
            let value = match self.counts.amount_of(first, &fresh) {
                Some(value) => value,
                None => {
                    sure = false;
                    Amount::whole(1)
                }
            };
            let thing = self.words.thing(&found);
            let belongs = match self.classes.belongs(&asked, &thing) {
                Some(belongs) => belongs,
                None => {
                    sure = false;
                    self.classes.guess(&asked, &thing)
                }
            };
            if belongs {
                total = total.plus(value).unwrap_or(total);
                counted.push(value.written(LIST_MOST_DIGITS).unwrap_or_default());
            } else {
                left.push(thing.iter().map(|word| self.counts.name(*word, &fresh)).collect::<Vec<&str>>().join(" "));
            }
        }
        let written = total.written(LIST_MOST_DIGITS).unwrap_or_default();
        let found = if self.counts.digits { Some(answer_words(&written)) } else { self.counts.text_of.get(&total).cloned() };
        let mut note = format!("{} = {written}", if counted.is_empty() { Amount::whole(0).written(LIST_MOST_DIGITS).unwrap_or_default() } else { counted.join(LIST_PLUS) });
        if !left.is_empty() {
            note = format!("{note}{LIST_NOTE_BETWEEN}left out: {}", left.join(", "));
        }
        match found {
            None => Said { words: Vec::new(), note: [note.as_str(), LIST_NO_TEXT, LIST_GUESSED].join(LIST_NOTE_BETWEEN) },
            Some(said) => Said { words: said, note: [note.as_str(), if sure { LIST_ALL_DECIDED } else { LIST_GUESSED }].join(LIST_NOTE_BETWEEN) },
        }
    }
}

impl Learner for ListModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        self.said(sentences, question)
    }
}

pub fn learn_list(lessons: Lessons, _lanes: usize) -> Learned {
    Ok(Box::new(ListModel::made(&lessons)))
}
because!(learn_list, ListAnswers, "the list tool as a tool of the one call: it learns from the lessons and always gives a model, which answers nothing when the lessons are no lists with amounts");
