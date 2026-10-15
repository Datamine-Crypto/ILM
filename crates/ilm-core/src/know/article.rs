use crate::know::signs::{is_space, is_word_sign};
use patterns::{because, source};

pub struct ArticleTokens;
source!(ArticleTokens, "an article as tokens: each word as it is written with its place in the text, and the title of the article as one token wherever the text writes it, so that what stands around a word names the place of the subject and not its name; beside the tokens the starts of the sentences, and for each token its sentence");

pub const SENTENCE_ENDS: &[char] = &['.', '!', '?'];
because!(SENTENCE_ENDS, ArticleTokens, "the signs that end a sentence when space follows them");

const LINE_END: char = '\n';
because!(LINE_END, ArticleTokens, "the end of a line ends a sentence too: a heading or an item of a list has no sign at its end");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub title: bool,
}
because!(Token, ArticleTokens, "one token of an article: where it starts and ends in the text, and whether it is the title of the article in place of a word");

#[derive(Clone, Debug, Default)]
pub struct Article {
    pub tokens: Vec<Token>,
    pub starts: Vec<usize>,
    pub sentence: Vec<usize>,
    pub runs: Vec<(usize, usize)>,
}
because!(Article, ArticleTokens, "the tokens of an article in their order, the places where its sentences start, for each token the number of its sentence, and the runs of word signs that the tokens were made of, so that a reader of every word of the text does not cut it again");

pub fn word_runs(text: &str) -> Vec<(usize, usize)> {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut open: Option<usize> = None;
    for (at, sign) in text.char_indices() {
        match (is_word_sign(sign), open) {
            (true, None) => open = Some(at),
            (false, Some(start)) => {
                runs.push((start, at));
                open = None;
            }
            _ => {}
        }
    }
    if let Some(start) = open {
        runs.push((start, text.len()));
    }
    runs
}
because!(word_runs, ArticleTokens, "the runs of word signs of a text, each with its start and its end");

pub fn sentence_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    let mut signs = text.char_indices().peekable();
    let mut before: Option<char> = None;
    while let Some((_, sign)) = signs.next() {
        let after_end = before.is_some_and(|last| SENTENCE_ENDS.contains(&last)) && is_space(sign);
        if !after_end && sign != LINE_END {
            before = Some(sign);
            continue;
        }
        let mut last = sign;
        while let Some((_, next)) = signs.next_if(|(_, next)| if after_end { is_space(*next) } else { *next == LINE_END }) {
            last = next;
        }
        starts.push(signs.peek().map_or(text.len(), |(at, _)| *at));
        before = Some(last);
    }
    starts
}
because!(sentence_starts, ArticleTokens, "the places where the sentences of a text start: the start of the text, the place after the space that follows a sign of a sentence end, and the place after a run of line ends");

pub fn parse(title: &str, text: &str) -> Article {
    let spans: Vec<(usize, usize)> = if title.is_empty() { Vec::new() } else { text.match_indices(title).map(|(at, found)| (at, at + found.len())).collect() };
    let mut tokens: Vec<Token> = Vec::new();
    let mut span = 0usize;
    let runs = word_runs(text);
    for (start, end) in runs.iter().copied() {
        while span < spans.len() && spans[span].1 <= start {
            span += 1;
        }
        if span < spans.len() && spans[span].0 <= start && start < spans[span].1 {
            if tokens.last().is_none_or(|last| !last.title || last.start != spans[span].0) {
                tokens.push(Token { start: spans[span].0, end: spans[span].1, title: true });
            }
        } else {
            tokens.push(Token { start, end, title: false });
        }
    }
    let starts = sentence_starts(text);
    let mut sentence: Vec<usize> = Vec::with_capacity(tokens.len());
    let mut at = 0usize;
    for token in &tokens {
        while at + 1 < starts.len() && starts[at + 1] <= token.start {
            at += 1;
        }
        sentence.push(at);
    }
    Article { tokens, starts, sentence, runs }
}
because!(parse, ArticleTokens, "the tokens of an article: a word that starts inside a place where the text writes the title is no token of its own, the title is one token there; a word that only reaches into such a place stays a word");

pub fn sentence_of<'a>(article: &Article, text: &'a str, number: usize) -> &'a str {
    let end = article.starts.get(number + 1).copied().unwrap_or(text.len());
    text[article.starts[number]..end].trim_matches(is_space)
}
because!(sentence_of, ArticleTokens, "the text of one sentence of an article without the space around it");

pub fn lowered(written: &str, into: &mut String) {
    into.clear();
    if written.is_ascii() {
        into.push_str(written);
        into.make_ascii_lowercase();
    } else {
        into.push_str(&written.to_lowercase());
    }
}
because!(lowered, ArticleTokens, "a word in small letters, written into a text that the caller keeps, so that the many words of an article need no new memory each");
