use crate::base::words::words;
use crate::tools::call::{Said, StoryAsk};
use crate::tools::pattern::kinds::{IN_LESSONS, JOINT};
use crate::tools::pattern::learn::PatternLearner;
use crate::tools::pattern::model::Hands;
use crate::tools::pattern::names::{Fresh, WordTexts};
use crate::tools::pattern::story::{Desk, Query};
use patterns::{because, source};

pub struct ReadOnce;
source!(ReadOnce, "an inverted index, as a search engine keeps one: a long story is read once into its words and, for each word, the sentences that hold it, and every question of the story is answered from that index; the forms of its sentences are kept from one question to the next, since they depend on the story and on the names that a question binds and not on the question as a whole");

pub struct IndexedStory<'a> {
    learner: &'a PatternLearner,
    sentences: Vec<String>,
    names: Fresh<'a>,
    case: Option<Query>,
    hands: Hands<'a>,
    raws: Vec<Vec<u32>>,
}
because!(IndexedStory, ReadOnce, "a story of the pattern learner read once: the learner, the told sentences, the words of the story that no lesson held, the story read with no question when the short way is open, the working tables with the forms kept from question to question, and the words of each sentence as written, read once. read makes it; short says whether the short way is open; ask answers a question as the learner answers the story with that question");

fn with_local(read: &[u32], local: &[(u32, u32)]) -> Vec<u32> {
    read.iter().map(|word| local.iter().find(|(new, _)| new == word).map_or(*word, |(_, base)| *base)).collect()
}
because!(with_local, ReadOnce, "the words of a text with each new form that the story reads as its other form replaced by that form");

impl<'a> IndexedStory<'a> {
    pub fn read(learner: &'a PatternLearner, sentences: &[String]) -> IndexedStory<'a> {
        let mut names = Fresh::over(&learner.symbols);
        let mut hands = learner.model.hands(&learner.words);
        hands.keep = true;
        let mut kept = None;
        if learner.reader.is_none() && learner.choice.is_none() && learner.writes.is_empty() {
            let case = Desk::over(&learner.words, &mut names).case(sentences, "", &|_| 0);
            let joined = sentences.iter().zip(&case.story.sents).any(|(sentence, told)| {
                let read = learner.words.read(&mut names, &words(sentence));
                with_local(&read, &case.local) != told.words
            });
            if case.back.is_empty() && !joined {
                kept = Some(case);
            }
        }
        let raws: Vec<Vec<u32>> = if kept.is_some() { sentences.iter().map(|sentence| words(sentence).iter().map(|word| names.number(word)).collect()).collect() } else { Vec::new() };
        IndexedStory { learner, sentences: sentences.to_vec(), names, case: kept, hands, raws }
    }

    pub fn short(&self) -> bool {
        self.case.is_some()
    }

    fn fresh(&self, case: &Query, got: &[u32]) -> Vec<usize> {
        let mut places: Vec<u32> = Vec::new();
        for word in got {
            for piece in self.names.name(*word).split(JOINT) {
                if let Some(number) = self.names.find(piece) {
                    places.extend_from_slice(case.story.at(number));
                }
            }
        }
        places.sort_unstable();
        places.dedup();
        if places.len() < self.sentences.len() { places.iter().map(|place| *place as usize).collect() } else { (0..self.sentences.len()).collect() }
    }
}

impl StoryAsk for IndexedStory<'_> {
    fn ask(&mut self, question: &str) -> Said {
        if let Some(case) = self.case.clone() {
            let learner = self.learner;
            let read = learner.words.read(&mut self.names, &words(question));
            let q = with_local(&read, &case.local);
            if q.iter().all(|word| learner.words.has(*word, IN_LESSONS) || !case.story.at(*word).is_empty()) {
                let asked = Query::on(case.story.clone(), q, &learner.words);
                let (got, why) = learner.model.answer_case(&learner.words, &mut self.names, &mut self.hands, &asked, false);
                if !(why.is_none() && learner.plain.is_some()) {
                    let places = self.fresh(&case, &got);
                    let asked: Vec<u32> = words(question).iter().map(|word| self.names.number(word)).collect();
                    let (names, raws) = (&self.names, &self.raws);
                    let here_of = |part: &str| {
                        let wanted = names.find(part)?;
                        let mut forms: Vec<(u32, u32)> = Vec::new();
                        for raw in places.iter().flat_map(|place| raws[*place].iter()).chain(asked.iter()) {
                            if learner.words.norm(*raw) == wanted {
                                match forms.iter_mut().find(|(held, _)| held == raw) {
                                    Some(row) => row.1 += 1,
                                    None => forms.push((*raw, 1)),
                                }
                            }
                        }
                        (!forms.is_empty()).then(|| forms.iter().map(|(raw, count)| (names.name(*raw).to_string(), *count)).collect())
                    };
                    let words = learner.words.raw_words_with(names, &got, &case.local, here_of);
                    return Said { words, note: learner.note(&learner.model, why) };
                }
            }
        }
        let (words, note) = self.learner.said(&self.sentences, question);
        Said { words, note }
    }
}
