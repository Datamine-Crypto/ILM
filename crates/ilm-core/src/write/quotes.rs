use patterns::{because, source};

pub struct QuotedText;
source!(QuotedText, "a text written inside quotes, with a backslash before a quote and before a line end, stays on one line and can be read back sign for sign");

pub fn quoted_text(text: &str) -> String {
    let quote = if text.contains(WRITE_SINGLE_QUOTE) && !text.contains(WRITE_DOUBLE_QUOTE) { WRITE_DOUBLE_QUOTE } else { WRITE_SINGLE_QUOTE };
    let mut out = String::new();
    out.push(quote);
    for sign in text.chars() {
        if sign == quote || sign == WRITE_BACKSLASH {
            out.push(WRITE_BACKSLASH);
            out.push(sign);
        } else if let Some((_, letter)) = WRITE_NAMED_ESCAPES.iter().find(|(named, _)| *named == sign) {
            out.push(WRITE_BACKSLASH);
            out.push(*letter);
        } else {
            out.push(sign);
        }
    }
    out.push(quote);
    out
}
because!(quoted_text, QuotedText, "a text inside quotes: the single quote, or the double quote when the text holds a single quote and no double one; a quote of the same kind, the backslash, the tab and the line ends with a backslash. Forms with the same counts are put in the order of such texts of their bound facts, so that their order is the same in every run");

const WRITE_SINGLE_QUOTE: char = '\'';
because!(WRITE_SINGLE_QUOTE, QuotedText, "the quote that stands around a text in quotes when nothing speaks for the other one");

const WRITE_DOUBLE_QUOTE: char = '"';
because!(WRITE_DOUBLE_QUOTE, QuotedText, "the quote for a text that holds a single quote and no double one, so that no quote inside it needs a backslash");

const WRITE_BACKSLASH: char = '\\';
because!(WRITE_BACKSLASH, QuotedText, "the sign that is written before a quote, before itself and before the letter of a tab or a line end");

const WRITE_NAMED_ESCAPES: &[(char, char)] = &[('\n', 'n'), ('\t', 't'), ('\r', 'r')];
because!(WRITE_NAMED_ESCAPES, QuotedText, "the signs that are written as a backslash and a letter, each with its letter: the line feed, the tab and the carriage return; the files of rows write the same signs in the same way, so that a text with a line end stays on one line");

pub fn escaped_back(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut signs = text.chars();
    while let Some(sign) = signs.next() {
        if sign != WRITE_BACKSLASH {
            out.push(sign);
            continue;
        }
        match signs.next() {
            Some(letter) => match WRITE_NAMED_ESCAPES.iter().find(|(_, named)| *named == letter) {
                Some((plain, _)) => out.push(*plain),
                None => out.push(letter),
            },
            None => out.push(WRITE_BACKSLASH),
        }
    }
    out
}
because!(escaped_back, QuotedText, "a text of a file of rows as its set had it: a backslash with the letter of a tab or of a line end is that sign again, a backslash before another sign gives that sign");
