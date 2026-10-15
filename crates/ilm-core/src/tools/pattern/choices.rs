use crate::base::words::{answer_words, words};
use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::learn::{PatternLearner, learn_counted, read_lessons};
use crate::tools::pattern::list::Rule;
use crate::tools::pattern::model::{Hands, PatternModel, SURE_READINGS, SURE_SHARE, Setting, Why};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{Fresh, WordTexts};
use crate::tools::pattern::story::Desk;
use patterns::{because, source};

pub struct OwnChoices;
source!(OwnChoices, "a question that holds its own choices is read as one question for each choice with the answer yes or no, the reduction of a choice among many to binary decisions; the word that cuts the questions into choices is found by counting");

pub const MOST_CHOICES: usize = 3000;
because!(MOST_CHOICES, OwnChoices, "the most single choices that the learner of choices is taught on: a question with many choices gives many lessons, and this many are enough to count the patterns of a choice");

pub const PAIR_WORDS: usize = 2;
because!(PAIR_WORDS, OwnChoices, "a word cuts a question into choices only when the question holds it this many times or more: one mark gives one choice, and one choice is no choice");

pub struct Choice {
    pub mark: u32,
    pub tail: Vec<u32>,
    pub model: PatternModel,
}
because!(Choice, OwnChoices, "what was counted for questions that hold their own choices: the word that cuts a question into choices, the words that end every question, and the learner of single choices");

impl PatternLearner {
    pub fn cut_choices(&self, names: &impl WordTexts, question: &str, mark: u32, tail: &[u32]) -> Option<(Vec<String>, Vec<Vec<String>>)> {
        let mut raw = words(question);
        let mut read: Vec<u32> = raw.iter().map(|word| names.find(word).map_or(ABSENT, |number| self.words.norm(number))).collect();
        if !tail.is_empty() && read.ends_with(tail) {
            raw.truncate(raw.len() - tail.len());
            read.truncate(raw.len());
        }
        let at: Vec<usize> = (0..read.len()).filter(|place| read[*place] == mark).collect();
        if at.len() < PAIR_WORDS {
            return None;
        }
        let mut choices: Vec<Vec<String>> = Vec::new();
        for (number, from) in at.iter().enumerate() {
            let to = at.get(number + 1).copied().unwrap_or(raw.len());
            let choice = raw[from + 1..to].to_vec();
            if !choice.is_empty() && !choices.contains(&choice) {
                choices.push(choice);
            }
        }
        Some((raw[..at[0]].to_vec(), choices))
    }

    pub fn learn_choices(&mut self, cases: &[(Vec<String>, String, String)], set: Setting) {
        if (cases.len() as u32) < SURE_READINGS {
            return;
        }
        let read = |text: &[String]| -> Vec<u32> { text.iter().map(|word| self.symbols.find(word).map_or(ABSENT, |number| self.words.norm(number))).collect() };
        let questions: Vec<Vec<u32>> = cases.iter().map(|(_, question, _)| read(&words(question))).collect();
        let answers: Vec<Vec<u32>> = cases.iter().map(|(_, _, answer)| read(&answer_words(answer))).collect();
        let need = SURE_SHARE * cases.len() as f64;
        let mut tail: Vec<u32> = Vec::new();
        loop {
            let mut last: FirstSightTable<u32, u32> = FirstSightTable::new();
            for question in &questions {
                if question.len() > tail.len() && question.ends_with(&tail) {
                    last.add(question[question.len() - tail.len() - 1], 1);
                }
            }
            match last.first_most() {
                Some((word, count)) if f64::from(*count) >= need => tail.insert(0, *word),
                _ => break,
            }
        }
        let mut twice: FirstSightTable<u32, u32> = FirstSightTable::new();
        for question in &questions {
            let mut seen: FirstSightTable<u32, u32> = FirstSightTable::new();
            for word in question {
                seen.add(*word, 1);
            }
            for (word, count) in seen.rows() {
                if *count as usize >= PAIR_WORDS {
                    twice.add(*word, 1);
                }
            }
        }
        let mut best: Option<(u32, u32)> = None;
        for (mark, count) in twice.by_count() {
            if f64::from(*count) < need {
                break;
            }
            let mut hit = 0;
            for ((_, question, _), answer) in cases.iter().zip(&answers) {
                if let Some((_, choices)) = self.cut_choices(&self.symbols, question, *mark, &tail) {
                    hit += u32::from(choices.iter().any(|choice| read(choice) == *answer));
                }
            }
            if f64::from(hit) >= need && best.is_none_or(|(top, _)| hit > top) {
                best = Some((hit, *mark));
            }
        }
        let Some((_, mark)) = best else { return };
        let mut sub: Vec<(Vec<String>, String, String)> = Vec::new();
        for ((sentences, question, _), answer) in cases.iter().zip(&answers) {
            if let Some((head, choices)) = self.cut_choices(&self.symbols, question, mark, &tail) {
                for choice in choices {
                    let said = if read(&choice) == *answer { "yes" } else { "no" };
                    let asked: Vec<String> = head.iter().chain(&choice).cloned().collect();
                    sub.push((sentences.clone(), asked.join(" "), said.to_string()));
                }
            }
        }
        sub.truncate(MOST_CHOICES);
        let ready = read_lessons(&self.words, &mut self.symbols, &sub, &|_| 0);
        let model = learn_counted(&self.words, &mut self.symbols, &ready, Setting { per_form: true, split: false, ..set });
        self.choice = Some(Choice { mark, tail, model });
    }

    pub fn sub_said(&self, model: &PatternModel, hands: &mut Hands, sentences: &[String], question: &str) -> (Vec<String>, Option<Why>) {
        let mut names = Fresh::over(&self.symbols);
        let mut desk = Desk::over(&self.words, &mut names);
        let case = desk.case(sentences, question, &|form| model.readings(form));
        let (mut got, why) = model.answer_case(&self.words, desk.names, hands, &case, false);
        for word in got.iter_mut() {
            if let Some((_, new)) = case.back.iter().find(|(stand, _)| stand == word) {
                *word = *new;
            }
        }
        (self.words.raw_words(&names, &got, sentences, question, &case.local), why)
    }

    pub fn choose(&self, sentences: &[String], question: &str) -> Option<(Vec<String>, Why)> {
        let choice = self.choice.as_ref()?;
        let (head, choices) = self.cut_choices(&self.symbols, question, choice.mark, &choice.tail)?;
        let mut hands = choice.model.hands(&self.words);
        let mut best: Option<(Vec<String>, Why)> = None;
        for found in choices {
            let asked: Vec<String> = head.iter().chain(&found).cloned().collect();
            let (got, why) = self.sub_said(&choice.model, &mut hands, sentences, &asked.join(" "));
            let Some(why) = why.filter(|why| why.list && got == ["yes"]) else { continue };
            let rule = Rule { pattern: why.pattern, place: why.place, hit: why.hit, readings: why.readings };
            let better = best.as_ref().is_none_or(|(_, top)| {
                let held = Rule { pattern: top.pattern, place: top.place, hit: top.hit, readings: top.readings };
                rule.share() > held.share() || (rule.share() == held.share() && rule.readings > held.readings)
            });
            if better {
                best = Some((found, why));
            }
        }
        best
    }
}
