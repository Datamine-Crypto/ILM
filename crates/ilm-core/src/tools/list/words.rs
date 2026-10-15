use crate::base::symbols::Symbols;
use crate::cases::Case;
use crate::tools::pattern::kinds::PatternWords;
use patterns::{because, source};

pub struct SameWords;
source!(SameWords, "the list tool counts a thing by the one form of its words, as the pattern learner reads the words of its lessons: a plural and its singular are one word when both stand in the texts, and an answer word that its story never holds is the one story word of the same first letter that comes with it in nearly every such lesson; so two cars and a car are one thing");

#[derive(Clone, Debug, Default)]
pub struct ListToolWords {
    pub symbols: Symbols,
    known: usize,
    forms: PatternWords,
}
because!(ListToolWords, SameWords, "the words of the lessons as numbers with the one form of each word, made by the word model of the pattern learner from all cases of the lessons. learn makes it; norm gives the one form of a word; thing gives the numbers of the one forms of some words, and a word that no lesson held keeps its own number, above the numbers of the lessons, so that a thing with such a word is no thing of the tables");

impl ListToolWords {
    pub fn learn(cases: &[Case], symbols: Symbols) -> ListToolWords {
        let mut made = ListToolWords { known: symbols.len(), symbols, forms: PatternWords::default() };
        made.forms.learn_same(&mut made.symbols, cases);
        made.forms.learn_answer_words(&mut made.symbols, cases);
        made
    }

    pub fn norm(&self, word: u32) -> u32 {
        self.forms.norm(word)
    }

    pub fn thing(&self, found: &[u32]) -> Vec<u32> {
        found.iter().map(|word| if (*word as usize) < self.known { self.forms.norm(*word) } else { *word }).collect()
    }
}
