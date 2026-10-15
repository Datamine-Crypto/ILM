use crate::base::symbols::Symbols;
use crate::know::signs::is_space;
use patterns::{because, source};

pub struct SentenceTokens;
source!(SentenceTokens, "a sentence as tokens for a reader of rules: a run of letters and digits is one token in small letters, every other mark that is no blank is a token of its own, and for each token it is kept whether it was written with a capital letter, because a capital letter inside a sentence is how a text writes a name");

const RULE_SLOT: &str = "_";
because!(RULE_SLOT, SentenceTokens, "the token that stands in the key of a piece where a thing stands in the sentence");

const RULE_START: &str = "<";
because!(RULE_START, SentenceTokens, "the token before the first token of a sentence");

const RULE_END: &str = ">";
because!(RULE_END, SentenceTokens, "the token after the last token of a sentence");

const RULE_EDGE: &str = "|";
because!(RULE_EDGE, SentenceTokens, "the token that stands next to the marks before a piece when they reach to the piece itself");

#[derive(Clone, Debug, Default)]
pub(super) struct RuleSentence {
    pub(super) toks: Vec<u32>,
    pub(super) caps: Vec<bool>,
    pub(super) left: Option<u32>,
    pub(super) later: Option<bool>,
}
because!(RuleSentence, SentenceTokens, "the tokens of a sentence as numbers, for each token whether it was written with a capital letter, and what is known of the place of the sentence in its theory: how many told sentences stand after it, and whether a later sentence holds a thing that it speaks about (nothing for a sentence that stands alone)");

pub(super) fn rule_sentence<N: FnMut(&str) -> u32>(text: &str, name: &mut N) -> RuleSentence {
    let mut out = RuleSentence::default();
    let mut lower = String::new();
    let mut marks = text.char_indices().peekable();
    while let Some((start, mark)) = marks.next() {
        if is_space(mark) {
            continue;
        }
        lower.clear();
        if mark.is_ascii_alphanumeric() {
            let mut end = start + mark.len_utf8();
            while let Some((at, next)) = marks.peek().copied() {
                if !next.is_ascii_alphanumeric() {
                    break;
                }
                end = at + next.len_utf8();
                marks.next();
            }
            lower.push_str(&text[start..end]);
            lower.make_ascii_lowercase();
        } else {
            lower.extend(mark.to_lowercase());
        }
        out.toks.push(name(&lower));
        out.caps.push(mark.is_uppercase());
    }
    out
}
because!(rule_sentence, SentenceTokens, "the tokens of a text in their order, each with the number that the given table of tokens has for it");

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RuleMarks {
    pub(super) slot: u32,
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) edge: u32,
}
because!(RuleMarks, SentenceTokens, "the numbers of the tokens that the reader itself puts into the key of a piece: the place of a thing, the start and the end of the sentence, and the edge of the marks before a piece");

impl RuleMarks {
    pub(super) fn new(symbols: &mut Symbols) -> RuleMarks {
        RuleMarks { slot: symbols.number(RULE_SLOT), start: symbols.number(RULE_START), end: symbols.number(RULE_END), edge: symbols.number(RULE_EDGE) }
    }
}
