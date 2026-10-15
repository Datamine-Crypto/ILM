use crate::tools::parts::language::{Word, WordRow};
use crate::tools::pattern::hash::CheapMap;
use patterns::{because, source};
use std::sync::Arc;

pub struct ChangedCopies;
source!(ChangedCopies, "a table that many copies share, each copy with a short list of its own changes, in the way of the persistent data structures of Driscoll, Sarnak, Sleator and Tarjan: a copy that changes one entry does not copy the whole table, and a lookup reads the short list before the shared table");

pub const CHANGES_MOST: usize = 8;
because!(CHANGES_MOST, ChangedCopies, "the changes that a copy keeps beside the shared table: with more, the copy puts them into a table of its own, so that a lookup reads only a few changes before the shared table");

#[derive(Clone, Debug, Default)]
pub struct WordSays {
    shared: Arc<CheapMap<Word, WordRow>>,
    changes: Arc<Vec<(Word, Option<WordRow>)>>,
}
because!(WordSays, ChangedCopies, "what each word of a grammar says: a shared table and the changes of this copy, each a word with its new row or with nothing when the word says nothing any more. of makes it from a table; get and keys read it as one table; set changes one word, and a copy shares the table and the changes until one of them changes");

impl WordSays {
    pub fn of(table: CheapMap<Word, WordRow>) -> WordSays {
        WordSays { shared: Arc::new(table), changes: Arc::default() }
    }

    pub fn get(&self, word: &Word) -> Option<&WordRow> {
        match self.changes.iter().find(|(held, _)| held == word) {
            Some((_, row)) => row.as_ref(),
            None => self.shared.get(word),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &Word> {
        let shared = self.shared.keys().filter(|word| !self.changes.iter().any(|(held, _)| held == *word));
        shared.chain(self.changes.iter().filter(|(_, row)| row.is_some()).map(|(word, _)| word))
    }

    pub fn set(&mut self, word: Word, row: Option<WordRow>) {
        let changes = Arc::make_mut(&mut self.changes);
        match changes.iter_mut().find(|(held, _)| *held == word) {
            Some(change) => change.1 = row,
            None => changes.push((word, row)),
        }
        if changes.len() > CHANGES_MOST {
            let table = Arc::make_mut(&mut self.shared);
            for (word, row) in changes.drain(..) {
                match row {
                    Some(row) => {
                        table.insert(word, row);
                    }
                    None => {
                        table.remove(&word);
                    }
                }
            }
        }
    }
}
