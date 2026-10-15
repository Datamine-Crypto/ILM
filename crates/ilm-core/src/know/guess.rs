use crate::know::reader::Lesson;
use patterns::{because, source};
use std::collections::HashMap;

pub struct NameGuess;
source!(NameGuess, "a guess for a fact whose subject has no answer in the text: the subject's own name often says the object (a road named after its state, a school after its town), so an object of the relation that stands as a run of words in the name is taken, the longest first; else the object that the words of the name make most likely by the rule of Bayes with words counted apart, as the classifier that Zhong, Friedman and Chen fit to the training facts of LAMA, which answers many facts from the names alone; every count is made from the lesson facts, and a guess has no sentence behind it, so it is counted apart from the answers that the text gives");

const SEEN_SMOOTH: f64 = 0.1;
because!(SEEN_SMOOTH, NameGuess, "what a word and an object get beside their count of lessons together, so that a word that one object never showed does not rule that object out");

const ALL_SMOOTH: f64 = 0.2;
because!(ALL_SMOOTH, NameGuess, "what an object gets beside its count of lessons when the share of a word in its lessons is made: the two outcomes of a word, there or not, each with the small count beside its count");

#[derive(Clone, Debug, Default)]
struct RelationCounts {
    objects: Vec<(String, usize)>,
    words: HashMap<String, HashMap<usize, usize>>,
}
because!(RelationCounts, NameGuess, "for one relation: its objects in the order of first sight with their counts of lessons, and for each word of a subject's name the count of lessons of each object (by its place) whose subject's name holds that word");

#[derive(Clone, Debug, Default)]
pub struct GuessModel {
    relations: HashMap<String, RelationCounts>,
}
because!(GuessModel, NameGuess, "the counts of the guesses for each relation, from the lesson facts. learn counts them; guess gives the object of a relation for a subject's name: an object that stands in the name as a run of its words, the one with the most signs and then the most frequent, or the object with the best score of Bayes over the words of the name that the lessons of the relation hold; none for a relation that no lesson holds");

fn name_words(name: &str) -> Vec<String> {
    name.split(|sign: char| !sign.is_alphanumeric()).filter(|word| !word.is_empty()).map(str::to_lowercase).collect()
}
because!(name_words, NameGuess, "the words of a name in small letters: the runs of letters and digits");

impl GuessModel {
    pub fn learn(lessons: &[Lesson]) -> GuessModel {
        let mut relations: HashMap<String, RelationCounts> = HashMap::new();
        for fact in lessons {
            let counts = relations.entry(fact.relation.to_string()).or_default();
            let place = match counts.objects.iter().position(|(object, _)| object == fact.object) {
                Some(place) => place,
                None => {
                    counts.objects.push((fact.object.to_string(), 0));
                    counts.objects.len() - 1
                }
            };
            counts.objects[place].1 += 1;
            let mut seen = name_words(fact.subject);
            seen.sort_unstable();
            seen.dedup();
            for word in seen {
                *counts.words.entry(word).or_default().entry(place).or_default() += 1;
            }
        }
        GuessModel { relations }
    }

    pub fn guess(&self, relation: &str, subject: &str) -> Option<&str> {
        let counts = self.relations.get(relation)?;
        let name = name_words(subject);
        let holds = |object: &str| -> bool {
            let run = name_words(object);
            !run.is_empty() && name.windows(run.len()).any(|window| window == run.as_slice())
        };
        let in_name = counts.objects.iter().filter(|(object, _)| holds(object)).max_by(|one, other| one.0.chars().count().cmp(&other.0.chars().count()).then(one.1.cmp(&other.1)).then(other.0.cmp(&one.0)));
        if let Some((object, _)) = in_name {
            return Some(object.as_str());
        }
        let all: usize = counts.objects.iter().map(|(_, count)| count).sum();
        let mut words: Vec<&str> = name.iter().map(String::as_str).filter(|word| counts.words.contains_key(*word)).collect();
        words.sort_unstable();
        words.dedup();
        let mut best: Option<(f64, &str)> = None;
        for (place, (object, count)) in counts.objects.iter().enumerate() {
            let mut score = (*count as f64 / all as f64).ln();
            for word in &words {
                let together = counts.words.get(*word).and_then(|by| by.get(&place)).copied().unwrap_or_default();
                score += ((together as f64 + SEEN_SMOOTH) / (*count as f64 + ALL_SMOOTH)).ln();
            }
            if best.is_none_or(|(top, _)| score > top) {
                best = Some((score, object.as_str()));
            }
        }
        best.map(|(_, object)| object)
    }
}
