use crate::base::symbols::Symbols;
use crate::cases::Case;
use crate::tools::pattern::kinds::PatternWords;
use patterns::{because, source};
use std::cell::RefCell;
use std::marker::PhantomData;

pub struct SharedWords;
source!(SharedWords, "memoization, as Michie named it: a function that is called again with the same arguments gives again the result that it gave; here the words of the first part of the lessons are learned once while the tools of one choice learn from that part, and each tool gets its own copy of the result");

#[derive(Default)]
struct WordsShelf {
    open: Vec<((usize, usize), Option<(Symbols, PatternWords)>)>,
}
because!(WordsShelf, SharedWords, "what one thread keeps while scopes are open: for each open scope, the place and the number of its lessons, and the words learned from them with the table of texts, once some tool has learned them");

thread_local! {
    static WORDS_SHELF: RefCell<WordsShelf> = RefCell::new(WordsShelf::default());
}

pub struct WordsScope<'a> {
    lessons: PhantomData<&'a [Case]>,
}
because!(WordsScope, SharedWords, "the time in which the words learned from some lessons are kept for the next tool on the same thread: the scope holds the lessons, so they can neither change nor go away while it lasts, and a slice of lessons at the same place with the same number of cases is then the same lessons; over opens it, and its end forgets the words");

impl<'a> WordsScope<'a> {
    pub fn over(cases: &'a [Case]) -> WordsScope<'a> {
        WORDS_SHELF.with_borrow_mut(|shelf| shelf.open.push(((cases.as_ptr() as usize, cases.len()), None)));
        WordsScope { lessons: PhantomData }
    }
}

impl Drop for WordsScope<'_> {
    fn drop(&mut self) {
        WORDS_SHELF.with_borrow_mut(|shelf| {
            shelf.open.pop();
        });
    }
}

pub fn fresh_words(cases: &[Case]) -> (Symbols, PatternWords) {
    let key = (cases.as_ptr() as usize, cases.len());
    let kept = WORDS_SHELF.with_borrow(|shelf| shelf.open.last().filter(|(held, _)| *held == key).and_then(|(_, words)| words.clone()));
    if let Some(found) = kept {
        return found;
    }
    let mut symbols = Symbols::new();
    let words = PatternWords::learn(cases, &mut symbols);
    WORDS_SHELF.with_borrow_mut(|shelf| {
        if let Some((held, kept)) = shelf.open.last_mut() {
            if *held == key {
                *kept = Some((symbols.clone(), words.clone()));
            }
        }
    });
    (symbols, words)
}
because!(fresh_words, SharedWords, "the words of some lessons learned into an empty table of texts, as PatternWords::learn gives them: from the scope of these lessons when a tool before learned them, else learned now and kept in the scope for the next tool");
