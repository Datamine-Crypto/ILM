use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::codes::{ASKED_SLOT, Point};
use crate::tools::pattern::kinds::{IN_LESSONS, JOINT};
use crate::tools::pattern::learn::PatternLearner;
use crate::tools::pattern::model::{SURE_READINGS, SURE_SHARE};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{Fresh, WordTexts};
use crate::tools::pattern::story::{Query, Desk};
use crate::tools::pattern::table::Ready;
use patterns::{because, source};

pub struct WriteBack;
source!(WriteBack, "forward chaining to a fixed point, as in the production systems of Forgy and in Datalog: a rule is used again on its own result; a statement that a rule of the list affirms is written into the story as a told sentence, round by round, until a round adds nothing");

pub const MOST_ROUNDS: usize = 8;
because!(MOST_ROUNDS, WriteBack, "the most rounds in which statements are written back into a story: the chains of the public sets end sooner, and the limit bounds the work for a story where a rule gives a new statement in every round");

pub const AFFIRM_WORDS: usize = 1;
because!(AFFIRM_WORDS, WriteBack, "the word that affirms a statement is a single word, as true or yes: an answer of more words is the content of a statement and not its truth");

pub struct WriteForm {
    pub template: Vec<u32>,
    pub slots: Vec<(u32, Vec<u32>)>,
}
because!(WriteForm, WriteBack, "a question form whose questions have the form of a told sentence: the form, and for each of its slots the kind of the words that filled it in the lessons, with these words");

impl PatternLearner {
    pub fn learn_statements(&mut self, ready: &[Ready]) {
        let mut forms: FirstSightTable<&[u32], (u32, u32, Vec<FirstSightTable<u32, u32>>)> = FirstSightTable::new();
        let mut told: FirstSightTable<&[u32], u32> = FirstSightTable::new();
        for lesson in ready {
            let case = &lesson.case;
            let seen = forms.entry(case.form.as_slice(), (0, 0, Vec::new()));
            seen.0 += 1;
            let shape = self.words.shape_of(&case.q, &case.story.extra);
            if self.words.sent_shapes.get(&case.q.len()).is_some_and(|shapes| shapes.contains(&shape)) {
                seen.1 += 1;
                for (place, word) in case.slots.iter().enumerate() {
                    if seen.2.len() <= place {
                        seen.2.resize(place + 1, FirstSightTable::new());
                    }
                    seen.2[place].add(*word, 1);
                }
                if case.story.sents.iter().any(|sent| sent.words == case.q) {
                    told.add(lesson.answer.as_slice(), 1);
                }
            }
        }
        let all: u32 = told.rows().iter().map(|(_, count)| *count).sum();
        let Some((answer, hit)) = told.first_most() else { return };
        if all < SURE_READINGS || f64::from(*hit) < SURE_SHARE * f64::from(all) || answer.len() != AFFIRM_WORDS {
            return;
        }
        self.affirm = answer.to_vec();
        for (form, (cases, as_sentence, fillers)) in forms.rows() {
            if *cases < SURE_READINGS || f64::from(*as_sentence) < SURE_SHARE * f64::from(*cases) || fillers.is_empty() {
                continue;
            }
            let mut slots: Vec<(u32, Vec<u32>)> = Vec::new();
            for filler in fillers {
                let mut kinds: FirstSightTable<u32, u32> = FirstSightTable::new();
                for (word, count) in filler.rows() {
                    if self.words.kind_of(*word) != ABSENT {
                        kinds.add(self.words.kind_of(*word), *count);
                    }
                }
                if let Some((kind, _)) = kinds.first_most() {
                    slots.push((*kind, filler.rows().iter().map(|(word, _)| *word).collect()));
                }
            }
            if slots.len() == fillers.len() {
                self.writes.push(WriteForm { template: form.to_vec(), slots });
            }
        }
    }

    pub fn written(&self, sentences: &[String]) -> Vec<String> {
        let mut told: Vec<String> = sentences.to_vec();
        let mut hands = self.model.hands(&self.words);
        for _ in 0..MOST_ROUNDS {
            let mut names = Fresh::over(&self.symbols);
            let mut desk = Desk::over(&self.words, &mut names);
            let base = desk.case(&told, "", &|form| self.model.readings(form));
            let story = &base.story;
            let story_words = story.told_words();
            let mut new: Vec<Vec<u32>> = Vec::new();
            for form in &self.writes {
                let options: Vec<Vec<u32>> = form.slots.iter().map(|(kind, held)| story_words.iter().copied().filter(|word| self.words.kind_with(*word, &story.extra) == *kind && (held.contains(word) || !self.words.has(*word, IN_LESSONS))).collect()).collect();
                let mut pick: Vec<usize> = vec![0; options.len()];
                let mut more = options.iter().all(|option| !option.is_empty());
                while more {
                    let combo: Vec<u32> = pick.iter().zip(&options).map(|(at, option)| option[*at]).collect();
                    let unlike = (0..combo.len()).all(|place| !combo[..place].contains(&combo[place]));
                    let q: Vec<u32> = form.template.iter().map(|cell| if cell & ASKED_SLOT != 0 { combo.get((cell & !ASKED_SLOT) as usize).copied().unwrap_or(*cell) } else { *cell }).collect();
                    if unlike && !story.sents.iter().any(|sent| sent.words == q) {
                        let case = Query::on(story.clone(), q.clone(), &self.words);
                        if case.form == form.template {
                            let (got, why) = self.model.answer_case(&self.words, desk.names, &mut hands, &case, true);
                            let by_sentence = why.is_some_and(|why| self.model.table.chain(why.pattern).first().is_some_and(|step| self.model.steps.held(*step).point != Point::Split));
                            if by_sentence && got == self.affirm {
                                new.push(q);
                            }
                        }
                    }
                    more = false;
                    for place in (0..pick.len()).rev() {
                        pick[place] += 1;
                        if pick[place] < options[place].len() {
                            more = true;
                            break;
                        }
                        pick[place] = 0;
                    }
                }
            }
            if new.is_empty() {
                break;
            }
            let mut shapes: Vec<Vec<u32>> = story.sents.iter().map(|sent| self.words.shape_of(&sent.words, &story.extra)).collect();
            for q in new {
                let shape = self.words.shape_of(&q, &story.extra);
                let at = shapes.iter().rposition(|held| *held == shape).map_or(told.len(), |place| place + 1);
                let text: Vec<String> = q.iter().map(|word| desk.names.name(*word).replace(JOINT, " ")).collect();
                told.insert(at, format!("{}.", text.join(" ")));
                shapes.insert(at, shape);
            }
        }
        told
    }
}
