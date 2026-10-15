use crate::base::words::words;
use crate::tools::call::{Learned, Learner, Lessons, Said, StoryAsk};
use crate::tools::pattern::indexed::IndexedStory;
use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::finder::words_at;
use crate::tools::pattern::kinds::{IN_LESSONS, JOINT, PatternWords};
use crate::tools::pattern::learn::PatternLearner;
use crate::tools::pattern::list::Rule;
use crate::tools::pattern::model::{PatternAnswers, PatternModel, Why};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{Fresh, WordTexts};
use crate::tools::pattern::story::Desk;
use patterns::{because, source};

pub struct WrittenForms;
source!(WrittenForms, "an answer is given in the words as the case writes them: of the forms that the lessons read as one word, the one that the lesson answers hold; and a new word gets the change of ending that the lesson answers show where they differ from their story");

pub const CHANGE_LEAST: u32 = 5;
because!(CHANGE_LEAST, WrittenForms, "a change of ending between the story and the answer is used on a new word only when this many lesson answers or more show it");

impl PatternWords {
    pub fn raw_words<N: WordTexts>(&self, names: &N, got: &[u32], sentences: &[String], question: &str, local: &[(u32, u32)]) -> Vec<String> {
        let mut told: Option<FirstSightTable<String, FirstSightTable<String, u32>>> = None;
        let norm_text = |raw: &str| names.find(raw).map_or_else(|| raw.to_string(), |number| names.name(self.norm(number)).to_string());
        let here_of = |part: &str| {
            let told = told.get_or_insert_with(|| {
                let mut table: FirstSightTable<String, FirstSightTable<String, u32>> = FirstSightTable::new();
                for text in sentences.iter().map(String::as_str).chain(std::iter::once(question)) {
                    for raw in words(text) {
                        table.entry(norm_text(&raw), FirstSightTable::new()).add(raw, 1);
                    }
                }
                table
            });
            told.get(&part.to_string()).map(|forms| forms.rows().to_vec())
        };
        self.raw_words_with(names, got, local, here_of)
    }

    pub fn raw_words_with<N: WordTexts, F: FnMut(&str) -> Option<Vec<(String, u32)>>>(&self, names: &N, got: &[u32], local: &[(u32, u32)], mut here_of: F) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for word in got {
            for part in names.name(*word).split(JOINT) {
                let number = names.find(part);
                let forms = number.and_then(|number| self.answer_forms.get(&number)).filter(|forms| !forms.rows().is_empty());
                if let Some(forms) = forms.filter(|forms| forms.rows().len() == 1) {
                    out.push(names.name(forms.rows()[0].0).to_string());
                    continue;
                }
                let here = here_of(part);
                if let Some(forms) = forms {
                    let mut both: Vec<(u32, &str)> = forms.rows().iter().map(|(raw, count)| (*count, names.name(*raw))).filter(|(_, raw)| here.as_ref().is_some_and(|here| here.iter().any(|(held, _)| held == raw))).collect();
                    both.sort_unstable();
                    let raw = both.last().map(|(_, raw)| *raw).or_else(|| forms.first_most().map(|(raw, _)| names.name(*raw)));
                    out.push(raw.unwrap_or(part).to_string());
                    continue;
                }
                let mut top: Option<&(String, u32)> = None;
                for row in here.iter().flatten() {
                    if top.is_none_or(|held| row.1 > held.1) {
                        top = Some(row);
                    }
                }
                let mut raw = top.map_or_else(|| part.to_string(), |(raw, _)| raw.clone());
                let known = number.is_some_and(|number| self.has(number, IN_LESSONS) || local.iter().any(|(new, base)| *new == number || *base == number));
                if !known {
                    let fit = self.answer_changes.iter().filter(|((old, _), count)| *count >= CHANGE_LEAST && !old.is_empty() && raw.ends_with(old.as_str()) && raw.len() > old.len());
                    if let Some(((old, new), _)) = fit.max_by_key(|((old, new), _)| (old.len(), old.as_str(), new.as_str())) {
                        raw = format!("{}{new}", &raw[..raw.len() - old.len()]);
                    }
                }
                out.push(raw);
            }
        }
        out
    }
}

impl PatternLearner {
    pub fn said(&self, sentences: &[String], question: &str) -> (Vec<String>, String) {
        let rewritten;
        let sentences = match &self.reader {
            Some(reader) => {
                let mut names = Fresh::over(&self.symbols);
                if let Some((answer, (absent, asked, other))) = reader.answer(&self.words, &mut names, sentences, question, &|form| self.model.readings(form)) {
                    let note = format!("told rules | absent {absent} | asked form told {asked} | other form told {other}");
                    return (self.words.raw_words(&names, &answer, sentences, question, &[]), note);
                }
                rewritten = reader.written(&self.words, &mut Fresh::over(&self.symbols), sentences);
                &rewritten
            }
            None => sentences,
        };
        if let Some((found, why)) = self.choose(sentences, question) {
            return (found, self.note(self.choice.as_ref().map_or(&self.model, |choice| &choice.model), Some(why)));
        }
        let written;
        let sentences = if self.writes.is_empty() {
            sentences
        } else {
            written = self.written(sentences);
            &written
        };
        let mut hands = self.model.hands(&self.words);
        let mut names = Fresh::over(&self.symbols);
        let mut desk = Desk::over(&self.words, &mut names);
        let case = desk.case(sentences, question, &|form| self.model.readings(form));
        let (mut got, why) = self.model.answer_case(&self.words, desk.names, &mut hands, &case, false);
        if why.is_none() {
            if let Some(plain) = &self.plain {
                return (plain.best(sentences, question).map(<[String]>::to_vec).unwrap_or_default(), String::new());
            }
        }
        for word in got.iter_mut() {
            if let Some((_, new)) = case.back.iter().find(|(stand, _)| stand == word) {
                *word = *new;
            }
        }
        (self.words.raw_words(&names, &got, sentences, question, &case.local), self.note(&self.model, why))
    }

    pub fn note(&self, model: &PatternModel, why: Option<Why>) -> String {
        let Some(why) = why else { return String::new() };
        let rule = Rule { pattern: why.pattern, place: why.place, hit: why.hit, readings: why.readings };
        let end = if why.list { "list" } else { "best" };
        format!("{} | {} | {} | {} | {} | {end}", model.text(&self.words, &self.symbols, why.pattern), model.place_name(&self.symbols, why.place), rule.share(), why.readings, why.hit)
    }
}

impl Learner for PatternLearner {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let (words, note) = self.said(sentences, question);
        Said { words, note }
    }

    fn story<'a>(&'a self, sentences: &[String]) -> Option<Box<dyn StoryAsk + 'a>> {
        Some(Box::new(IndexedStory::read(self, sentences)))
    }

    fn question_alone(&self, sentences: &[String], question: &str) -> Option<Vec<String>> {
        let mut names = Fresh::over(&self.symbols);
        let mut desk = Desk::over(&self.words, &mut names);
        let case = desk.case(sentences, question, &|form| self.model.readings(form));
        let bare = self.model.asked.seek(case.form.as_slice()).and_then(|asked| self.model.table.child.get(ABSENT, asked));
        let mut said = match bare {
            Some(pattern) => self.model.places.row(self.model.place[pattern as usize]).and_then(|place| words_at(place, &case, &[])).unwrap_or_default(),
            None => self.model.default.clone(),
        };
        for word in said.iter_mut() {
            if let Some((_, new)) = case.back.iter().find(|(stand, _)| stand == word) {
                *word = *new;
            }
        }
        Some(self.words.raw_words(&names, &said, sentences, question, &[]))
    }
}

pub fn learn_patterns(lessons: Lessons, lanes: usize) -> Learned {
    let learner = PatternLearner::learn(lessons.cases, lanes);
    Ok(Box::new(learner))
}
because!(learn_patterns, PatternAnswers, "the pattern learner as a tool of the one call: it reads every kind of lessons and never says that it does not read a kind");
