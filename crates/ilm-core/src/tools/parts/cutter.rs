use patterns::{because, source};

pub struct PartsCut;
source!(PartsCut, "the cut of a text for a learner of meaning built from parts: the text is read in small letters; a word is a run of letters, digits and low lines of any alphabet, and an apostrophe between two such runs belongs to the word; a sign of arithmetic or a bracket is a word of its own; in a told text a comma, a semicolon and a colon are words too, because such a mark can part the parts of a text, and they are no words of an answer");

const ANSWER_SIGNS: &str = "-+*/=<>()[]{}";
because!(ANSWER_SIGNS, PartsCut, "the marks that are words of their own in an answer and in a text: the signs of arithmetic and the brackets");

const PARTING_MARKS: &str = ",;:";
because!(PARTING_MARKS, PartsCut, "the marks that are words of a told text and of no answer: a mark of this sort can stand between the parts of a text");

const INSIDE_A_WORD: &str = "'’";
because!(INSIDE_A_WORD, PartsCut, "the two forms of the apostrophe: one of them between two runs of letters joins the runs to one word");

const NO_MARKS: &str = "";
because!(NO_MARKS, PartsCut, "an answer has no parting marks: every mark of it that is no sign is left out");

const WORD_LINE_SIGN: char = '_';
because!(WORD_LINE_SIGN, PartsCut, "the low line counts as a letter of a word, as it does in the names of programs");

const BETWEEN_SENTENCES: &str = " ";
because!(BETWEEN_SENTENCES, PartsCut, "the mark between the told sentences and the question when they are read as one text");

fn in_a_word(sign: char) -> bool {
    sign.is_alphanumeric() || sign == WORD_LINE_SIGN
}
because!(in_a_word, PartsCut, "whether a sign is a letter, a digit or the low line, in any alphabet");

fn cut_with(text: &str, marks: &str) -> Vec<String> {
    let low: Vec<char> = text.to_lowercase().chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut at = 0;
    while at < low.len() {
        if !in_a_word(low[at]) {
            if ANSWER_SIGNS.contains(low[at]) || marks.contains(low[at]) {
                out.push(low[at].to_string());
            }
            at += 1;
            continue;
        }
        let mut end = at + 1;
        loop {
            while end < low.len() && in_a_word(low[end]) {
                end += 1;
            }
            let joined = low.get(end).is_some_and(|sign| INSIDE_A_WORD.contains(*sign)) && low.get(end + 1).is_some_and(|sign| in_a_word(*sign));
            if !joined {
                break;
            }
            end += 1;
        }
        out.push(low[at..end].iter().collect());
        at = end;
    }
    out
}
because!(cut_with, PartsCut, "the words of a text in their order, with the given marks as words of their own beside the signs");

pub fn row_words(answer: &str) -> Vec<String> {
    cut_with(answer, NO_MARKS)
}
because!(row_words, PartsCut, "the words of an answer as the rows of this learner hold them");

pub fn told_words(sentences: &[String], question: &str) -> Vec<String> {
    let mut whole: Vec<&str> = sentences.iter().map(String::as_str).collect();
    whole.push(question);
    cut_with(&whole.join(BETWEEN_SENTENCES), PARTING_MARKS)
}
because!(told_words, PartsCut, "the words of a case as one text: its told sentences in their order, then its question");
