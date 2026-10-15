use crate::base::symbols::Symbols;
use patterns::{because, source};
use std::collections::HashMap;

pub struct EntryOrder;
source!(EntryOrder, "the compact dictionary of Hettinger: a dense array of entries in the order in which they came, with an index over it; the first of equals wins in that order, so a count gives the same result in every run");

pub struct UnseenWords;
source!(UnseenWords, "a word that no lesson held is read by the place where it stands: it gets the kind that the most frequent lesson text with the same words and kinds in all other places has there, so a rule that was counted on other names is used on a new name");

pub trait WordTexts {
    fn name(&self, number: u32) -> &str;
    fn number(&mut self, text: &str) -> u32;
    fn find(&self, text: &str) -> Option<u32>;
}
because!(WordTexts, UnseenWords, "the texts of words by their numbers, the number of a text, and the number of a text that is there already: the table of the lessons while a model learns, and that table with the new words of one case on top of it while a model answers, so that an answer changes nothing in the model");

impl WordTexts for Symbols {
    fn name(&self, number: u32) -> &str {
        Symbols::name(self, number)
    }

    fn number(&mut self, text: &str) -> u32 {
        Symbols::number(self, text)
    }

    fn find(&self, text: &str) -> Option<u32> {
        Symbols::find(self, text)
    }
}

pub struct Fresh<'a> {
    pub base: &'a Symbols,
    more: Vec<String>,
    at: HashMap<String, u32>,
}
because!(Fresh, UnseenWords, "the words of the lessons with the new words of one case on top: a new word gets the next number after the last word of the lessons, and it is gone when the case is answered");

impl<'a> Fresh<'a> {
    pub fn over(base: &'a Symbols) -> Fresh<'a> {
        Fresh { base, more: Vec::new(), at: HashMap::new() }
    }
}

impl WordTexts for Fresh<'_> {
    fn name(&self, number: u32) -> &str {
        let place = number as usize;
        if place < self.base.len() { self.base.name(number) } else { &self.more[place - self.base.len()] }
    }

    fn number(&mut self, text: &str) -> u32 {
        if let Some(found) = self.base.find(text) {
            return found;
        }
        if let Some(found) = self.at.get(text) {
            return *found;
        }
        let number = (self.base.len() + self.more.len()) as u32;
        self.more.push(text.to_string());
        self.at.insert(text.to_string(), number);
        number
    }

    fn find(&self, text: &str) -> Option<u32> {
        self.base.find(text).or_else(|| self.at.get(text).copied())
    }
}

pub fn text_order<N: WordTexts>(names: &N, a: &[u32], b: &[u32]) -> std::cmp::Ordering {
    let left = a.iter().map(|word| names.name(*word));
    let right = b.iter().map(|word| names.name(*word));
    left.cmp(right)
}
because!(text_order, EntryOrder, "the order of two rows of words: word by word in the order of their texts, a shorter row before a longer one that starts with it; an order by texts and not by numbers, so that it does not depend on which word was seen first");
