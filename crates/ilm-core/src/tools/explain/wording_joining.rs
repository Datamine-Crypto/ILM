use crate::cases::Case;
use crate::tools::explain::learn::{ExplainModel, model_of};
use crate::tools::explain::text::{fact_numbers, read_topic};
use crate::tools::explain::words::{WordId, word_number, word_text};
use crate::base::words::first_capital;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct LearnedJoining;
source!(LearnedJoining, "the ways of joining of a language of commands learned from its lessons, as a child learns what 'twice' does from hearing it with words it knows: the meaning of a word is the token that its lessons always hold and the other lessons never hold (cross-situational learning, Siskind); a word with no such token is a joining word; a small search proposes, for each way that a word joins the parts of a command, the answer form made of the answers of its parts and the token of its word that reads the most lessons right, one way at a time until nothing improves (coordinate ascent); the search only proposes: its result is written as lines of the library, and the engine's reading of those lines is the model and is judged on the lessons, so a reader can check every learned rule");

const MOST_REPEATS: usize = 4;
because!(MOST_REPEATS, LearnedJoining, "the most times that an answer form repeats the answer of a part or the token of a word: enough for a part said as many times as a full turn has quarter turns (around), few enough that the forms stay a small set");

const MOST_FORM_PIECES: usize = 8;
because!(MOST_FORM_PIECES, LearnedJoining, "the most pieces of an answer form that joins a part and the token of a word after a joining word: a token and a part, as many times over as a part may be said");

const MOST_SWEEPS: usize = 8;
because!(MOST_SWEEPS, LearnedJoining, "the most rounds of the search over the ways of joining; a round that improves nothing ends it sooner");

const JOINED_TOPIC: &str = "learned joining";
because!(JOINED_TOPIC, LearnedJoining, "the name of the text of the learned lines, which the note of an answer names");

const CONDITION_JOIN: &str = "and";
because!(CONDITION_JOIN, LearnedJoining, "the word that parts the conditions of a rule, so a joining word of the same letters is written with a capital to stay a word");

const WORD_MARK: &str = "<word>";
because!(WORD_MARK, LearnedJoining, "the mark in a line of the library that the joining word or the word of meaning takes");

const FORM_MARK: &str = "<form>";
because!(FORM_MARK, LearnedJoining, "the mark in a line of the library that the learned answer form takes");

const TOKEN_MARK: &str = "<token>";
because!(TOKEN_MARK, LearnedJoining, "the mark in a line of the library that the token of a word of meaning takes");

const JOINED_LINE_BREAK: &str = "\n";
because!(JOINED_LINE_BREAK, LearnedJoining, "the break between the learned lines, as between the lines of a text of the library");

const PIECE_GAP: &str = " ";
because!(PIECE_GAP, LearnedJoining, "the space between the pieces of an answer form, as the library writes the words of a sentence");

const COMMAND_LINE: &str = "If the question is A then A is a command.";
because!(COMMAND_LINE, LearnedJoining, "the question of a lesson is the command that the lines read");

const JOINED_LINE: &str = "If A <word> B is a command then A <word> B is joined.";
because!(JOINED_LINE, LearnedJoining, "a command with a word that joins two parts is not read as one part");

const FIRST_PART_LINE: &str = "If A <word> B is a command then A is a part.";
because!(FIRST_PART_LINE, LearnedJoining, "the words before a word that joins two parts are a part to read");

const SECOND_PART_LINE: &str = "If A <word> B is a command then B is a part.";
because!(SECOND_PART_LINE, LearnedJoining, "the words after a word that joins two parts are a part to read");

const BETWEEN_LINE: &str = "If A <word> B is a command and the actions of A are C and the actions of B are D then the actions of A <word> B are <form>.";
because!(BETWEEN_LINE, LearnedJoining, "the answer of a command joined by a word between two parts, in the learned order of the answers of the parts");

const WHOLE_PART_LINE: &str = "If A is a command and not A is joined then A is a part.";
because!(WHOLE_PART_LINE, LearnedJoining, "a command with no word that joins two parts is one part");

const MEANING_FACT: &str = "The meaning of <word> is <token>.";
because!(MEANING_FACT, LearnedJoining, "the token that the lessons give a word of meaning, as a fact of the learned text");

const ALONE_LINE: &str = "If R is a part and the meaning of R is K then the actions of R are K.";
because!(ALONE_LINE, LearnedJoining, "a word of meaning alone answers its token");

const HEAD_LINE: &str = "If <word> R is a part and the meaning of R is K then the actions of <word> R are <form>.";
because!(HEAD_LINE, LearnedJoining, "a joining word before a word of meaning, alone in a part, answers the token in the learned number of times");

const HEAD_OF_TWO_LINE: &str = "If T <word> R is a part and the meaning of R is K and not the meaning of T is J then the actions of T <word> R are <form>.";
because!(HEAD_OF_TWO_LINE, LearnedJoining, "a joining word between a word of no meaning and a word of meaning answers the token in the learned number of times");

const BEFORE_PART_LINE: &str = "If A <word> R is a part and the meaning of R is K then A is a part.";
because!(BEFORE_PART_LINE, LearnedJoining, "the words before a joining word that comes before a word of meaning are a part to read");

const BEFORE_LINE: &str = "If A <word> R is a part and the meaning of R is K and the actions of A are B then the actions of A <word> R are <form>.";
because!(BEFORE_LINE, LearnedJoining, "a part, a joining word and a word of meaning answer the learned mix of the answer of the part and the token");

const MEANING_PART_LINE: &str = "If S <word> is a part and the meaning of S is J then S is a part.";
because!(MEANING_PART_LINE, LearnedJoining, "a word of meaning before another word of meaning, alone, is a part to read");

const MEANING_LONG_PART_LINE: &str = "If A S <word> is a part and the meaning of S is J then A S is a part.";
because!(MEANING_LONG_PART_LINE, LearnedJoining, "the words before a word of meaning are a part to read when they end in a word of meaning");

const MEANING_LINE: &str = "If A <word> is a part and the meaning of <word> is K and the actions of A are B then the actions of A <word> are <form>.";
because!(MEANING_LINE, LearnedJoining, "a part and a word of meaning after it answer the learned mix of the answer of the part and the token");

const AFTER_PART_LINE: &str = "If A <word> is a part then A is a part.";
because!(AFTER_PART_LINE, LearnedJoining, "the words before a joining word at the end of a part are a part to read");

const AFTER_LINE: &str = "If A <word> is a part and the actions of A are B then the actions of A <word> are <form>.";
because!(AFTER_LINE, LearnedJoining, "a part and a joining word after it answer the answer of the part in the learned number of times");

const ANSWER_LINE: &str = "To answer \"A\": B if the actions of A are B.";
because!(ANSWER_LINE, LearnedJoining, "the answer of a question is the actions of its command");

const FIRST_PLACE: &str = "C";
because!(FIRST_PLACE, LearnedJoining, "the place of the answer of the first part in the line of a word between two parts");

const SECOND_PLACE: &str = "D";
because!(SECOND_PLACE, LearnedJoining, "the place of the answer of the second part in the line of a word between two parts");

const PART_PLACE: &str = "B";
because!(PART_PLACE, LearnedJoining, "the place of the answer of the one part in the line of a way with one part");

const TOKEN_PLACE: &str = "K";
because!(TOKEN_PLACE, LearnedJoining, "the place of the token of the word of meaning in the line of a way of joining");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum JoinShape {
    Between,
    After,
    Meaning,
    Before,
    Head,
    HeadOfTwo,
}
because!(JoinShape, LearnedJoining, "a way that a word joins the parts of a command: between two parts, after a part, a word of meaning after a part, a joining word between a part and a word of meaning, a joining word before a word of meaning, and a joining word between a word of no meaning and a word of meaning");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FormPiece {
    First,
    Second,
    Mark,
}
because!(FormPiece, LearnedJoining, "a piece of an answer form: the answer of the first part (the only part, for a way with one part), the answer of the second part, or the token of the word of meaning");

type Form = Vec<FormPiece>;
because!(Form, LearnedJoining, "an answer form, piece by piece");

type JoinKey = (JoinShape, WordId);
because!(JoinKey, LearnedJoining, "a way of joining and the word that joins by it");

struct JoiningGrammar {
    meaning: HashMap<WordId, WordId>,
    between: Vec<WordId>,
    after: HashSet<WordId>,
    forms: HashMap<JoinKey, Form>,
}
because!(JoiningGrammar, LearnedJoining, "what the search proposes, and its own reading, which only proposes forms (the engine reads the written lines): the token of each word of meaning, the words that join two parts, the other joining words, and the answer form of each way of joining; form gives the answer form of a way of joining and notes that it was asked; phrase reads a part with no word that joins two parts, from its last word back: a word of meaning alone, after a word of no meaning, after a joining word (after a word of no meaning, or after a part), or after a part; a joining word after a part; command reads a command at the first word that joins two parts, or as one part; reads says whether a lesson is read right");

struct CommandLesson {
    command: Vec<WordId>,
    tokens: Vec<WordId>,
}
because!(CommandLesson, LearnedJoining, "a lesson as the words of its command, as the rules see them, and the tokens of its answer as the set writes them");

pub struct Joining {
    pub lines: Vec<String>,
    pub proposed_right: usize,
    pub model: ExplainModel,
}
because!(Joining, LearnedJoining, "the learned joining of a kind: the lines of the library, the lessons that the search reads right with what it proposes, and the model, which is the engine's reading of the lines");

fn in_letters(words: &mut Vec<WordId>) {
    words.sort_by_key(|word| word_text(*word));
    words.dedup();
}
because!(in_letters, LearnedJoining, "words in the order of their letters, each once, so that the search and its lines are the same in every run, whatever numbers the words have");

fn key_order(key: &JoinKey) -> (JoinShape, &'static str) {
    (key.0, word_text(key.1))
}
because!(key_order, LearnedJoining, "the order of the ways of joining: by their shape, then by the letters of their word, the same in every run");

fn meanings(lessons: &[CommandLesson]) -> HashMap<WordId, WordId> {
    let mut words: Vec<WordId> = lessons.iter().flat_map(|lesson| lesson.command.iter().copied()).collect();
    in_letters(&mut words);
    let mut out = HashMap::new();
    for word in words {
        let holding: Vec<&CommandLesson> = lessons.iter().filter(|lesson| lesson.command.contains(&word)).collect();
        let Some((first, rest)) = holding.split_first() else { continue };
        let mut common: Vec<WordId> = first.tokens.iter().copied().filter(|token| rest.iter().all(|lesson| lesson.tokens.contains(token))).collect();
        in_letters(&mut common);
        if let Some(token) = common.into_iter().find(|token| lessons.iter().all(|lesson| lesson.command.contains(&word) || !lesson.tokens.contains(token))) {
            out.insert(word, token);
        }
    }
    out
}
because!(meanings, LearnedJoining, "the token of each word that every lesson holding the word holds in its answer and no other lesson holds, the first in order of letters when several do");

fn joining_between(lessons: &[CommandLesson], joining: &[WordId]) -> Vec<WordId> {
    joining
        .iter()
        .filter(|word| {
            let holding: Vec<&CommandLesson> = lessons.iter().filter(|lesson| lesson.command.contains(word)).collect();
            !holding.is_empty() && holding.iter().all(|lesson| lesson.command.iter().filter(|other| other == word).count() == 1 && lesson.command.first() != Some(word) && lesson.command.last() != Some(word))
        })
        .copied()
        .collect()
}
because!(joining_between, LearnedJoining, "the joining words that join two parts: each stands once in a command, never first and never last");

fn spread(form: &Form, first: &[WordId], second: &[WordId], mark: &[WordId]) -> Vec<WordId> {
    form.iter()
        .flat_map(|piece| match piece {
            FormPiece::First => first.to_vec(),
            FormPiece::Second => second.to_vec(),
            FormPiece::Mark => mark.to_vec(),
        })
        .collect()
}
because!(spread, LearnedJoining, "the answer that a form gives from the answers of the parts and the token of the word");

impl JoiningGrammar {
    fn form(&self, shape: JoinShape, word: WordId, used: &mut Option<&mut HashSet<JoinKey>>) -> Option<&Form> {
        let key = (shape, word);
        if let Some(asked) = used.as_deref_mut() {
            asked.insert(key);
        }
        self.forms.get(&key)
    }

    fn phrase(&self, words: &[WordId], used: &mut Option<&mut HashSet<JoinKey>>) -> Option<Vec<WordId>> {
        let (last, rest) = words.split_last()?;
        if let Some(token) = self.meaning.get(last) {
            let mark = std::slice::from_ref(token);
            if rest.is_empty() {
                return Some(mark.to_vec());
            }
            if let [head] = rest
                && !self.meaning.contains_key(head)
            {
                return Some(spread(self.form(JoinShape::Head, *head, used)?, &[], &[], mark));
            }
            if let Some((joiner, inner)) = rest.split_last()
                && self.after.contains(joiner)
            {
                if let [head] = inner
                    && !self.meaning.contains_key(head)
                {
                    return Some(spread(self.form(JoinShape::HeadOfTwo, *joiner, used)?, &[], &[], mark));
                }
                let part = self.phrase(inner, used)?;
                return Some(spread(self.form(JoinShape::Before, *joiner, used)?, &part, &[], mark));
            }
            let part = self.phrase(rest, used)?;
            return Some(spread(self.form(JoinShape::Meaning, *last, used)?, &part, &[], mark));
        }
        if self.after.contains(last) {
            let part = self.phrase(rest, used)?;
            return Some(spread(self.form(JoinShape::After, *last, used)?, &part, &[], &[]));
        }
        None
    }

    fn command(&self, words: &[WordId], used: &mut Option<&mut HashSet<JoinKey>>) -> Option<Vec<WordId>> {
        for joiner in &self.between {
            if let Some(at) = words.iter().position(|word| word == joiner) {
                let first = self.phrase(&words[..at], used)?;
                let second = self.phrase(&words[at + 1..], used)?;
                return Some(spread(self.form(JoinShape::Between, *joiner, used)?, &first, &second, &[]));
            }
        }
        self.phrase(words, used)
    }

    fn reads(&self, lesson: &CommandLesson) -> bool {
        self.command(&lesson.command, &mut None).as_ref() == Some(&lesson.tokens)
    }
}

fn forms_of(shape: JoinShape) -> Vec<Form> {
    let repeats = |piece: FormPiece| -> Vec<Form> { (1..=MOST_REPEATS).map(|count| vec![piece; count]).collect() };
    let mixed = |most: usize| -> Vec<Form> {
        let mut out: Vec<Form> = vec![Vec::new()];
        let mut all = Vec::new();
        for _ in 0..most {
            out = out.iter().flat_map(|form| [FormPiece::First, FormPiece::Mark].map(|piece| form.iter().copied().chain([piece]).collect::<Form>())).collect();
            all.extend(out.iter().filter(|form| form.contains(&FormPiece::First)).cloned());
        }
        all
    };
    match shape {
        JoinShape::Between => vec![vec![FormPiece::First, FormPiece::Second], vec![FormPiece::Second, FormPiece::First]],
        JoinShape::After => repeats(FormPiece::First),
        JoinShape::Head | JoinShape::HeadOfTwo => repeats(FormPiece::Mark),
        JoinShape::Meaning => mixed(MOST_REPEATS),
        JoinShape::Before => mixed(MOST_FORM_PIECES).into_iter().filter(|form| form.contains(&FormPiece::Mark)).collect(),
    }
}
because!(forms_of, LearnedJoining, "the answer forms that a way of joining may have: the two orders of two parts; a part said again up to four times; a token said again; or the answer of a part and the token mixed in any order (with the part at least once, and the token too for a joining word before a word of meaning)");

fn proposed(lessons: &[CommandLesson]) -> (JoiningGrammar, usize) {
    let meaning = meanings(lessons);
    let mut joining: Vec<WordId> = lessons.iter().flat_map(|lesson| lesson.command.iter().copied()).filter(|word| !meaning.contains_key(word)).collect();
    in_letters(&mut joining);
    let between = joining_between(lessons, &joining);
    let after: HashSet<WordId> = joining.iter().filter(|word| !between.contains(word)).copied().collect();
    let mut keys: Vec<JoinKey> = between.iter().map(|word| (JoinShape::Between, *word)).collect();
    keys.extend(after.iter().flat_map(|word| [JoinShape::After, JoinShape::Before, JoinShape::HeadOfTwo].map(|shape| (shape, *word))));
    keys.extend(joining.iter().map(|word| (JoinShape::Head, *word)));
    keys.extend(meaning.keys().map(|word| (JoinShape::Meaning, *word)));
    let forms = keys.into_iter().filter_map(|key| forms_of(key.0).into_iter().next().map(|form| (key, form))).collect();
    let mut grammar = JoiningGrammar { meaning, between, after, forms };
    let mut using: HashMap<JoinKey, Vec<usize>> = HashMap::new();
    for (at, lesson) in lessons.iter().enumerate() {
        let mut asked = HashSet::new();
        let _ = grammar.command(&lesson.command, &mut Some(&mut asked));
        for key in asked {
            using.entry(key).or_default().push(at);
        }
    }
    grammar.forms.retain(|key, _| using.contains_key(key));
    let mut keys: Vec<JoinKey> = grammar.forms.keys().copied().collect();
    keys.sort_by_key(key_order);
    let mut right: Vec<bool> = lessons.iter().map(|lesson| grammar.reads(lesson)).collect();
    for _ in 0..MOST_SWEEPS {
        let mut moved = false;
        for key in &keys {
            let Some(asking) = using.get(key) else { continue };
            let Some(kept) = grammar.forms.get(key).cloned() else { continue };
            let mut best = (asking.iter().filter(|at| right[**at]).count(), kept.clone());
            for form in forms_of(key.0).into_iter().filter(|form| *form != kept) {
                if best.0 == asking.len() {
                    break;
                }
                grammar.forms.insert(*key, form.clone());
                let mut count = usize::default();
                for (done, at) in asking.iter().enumerate() {
                    if count + (asking.len() - done) <= best.0 {
                        break;
                    }
                    count += usize::from(grammar.reads(&lessons[*at]));
                }
                if count > best.0 {
                    best = (count, form);
                }
            }
            moved |= best.1 != kept;
            grammar.forms.insert(*key, best.1);
            for at in asking {
                right[*at] = grammar.reads(&lessons[*at]);
            }
        }
        if !moved {
            break;
        }
    }
    let read_right = right.iter().filter(|read| **read).count();
    (grammar, read_right)
}
because!(proposed, LearnedJoining, "the grammar that the search proposes from the lessons, and the lessons it reads right: the meanings of words, the words that join two parts and the other joining words, the ways of joining that the lessons ask for, each with the first of its forms, then each way in turn given the form that reads right the most of the lessons that ask for it, the first such form in their order, round after round until nothing improves; a form stops being read once it cannot read more lessons than the best so far, and the forms stop once one reads them all, which changes no choice");

fn written(word: &str) -> String {
    if word == CONDITION_JOIN { first_capital(word) } else { word.to_string() }
}
because!(written, LearnedJoining, "a word of a command as a line of the library writes it, with a capital where the small form would part the conditions");

fn joined_form_text(shape: JoinShape, form: &Form) -> String {
    let place = |piece: &FormPiece| match (shape, piece) {
        (JoinShape::Between, FormPiece::First) => FIRST_PLACE,
        (JoinShape::Between, _) => SECOND_PLACE,
        (_, FormPiece::Mark) => TOKEN_PLACE,
        _ => PART_PLACE,
    };
    form.iter().map(place).collect::<Vec<&str>>().join(PIECE_GAP)
}
because!(joined_form_text, LearnedJoining, "an answer form in the places of the line of its way of joining");

fn grammar_lines(grammar: &JoiningGrammar) -> Vec<String> {
    let fill = |line: &str, word: &WordId, form: &str| line.replace(WORD_MARK, &written(word_text(*word))).replace(FORM_MARK, form);
    let mut keys: Vec<(&JoinKey, &Form)> = grammar.forms.iter().collect();
    keys.sort_by_key(|(key, _)| key_order(key));
    let mut lines = vec![COMMAND_LINE.to_string()];
    for ((shape, word), form) in keys.iter().filter(|((shape, _), _)| *shape == JoinShape::Between) {
        let form = joined_form_text(*shape, form);
        lines.extend([JOINED_LINE, FIRST_PART_LINE, SECOND_PART_LINE, BETWEEN_LINE].map(|line| fill(line, word, &form)));
    }
    lines.push(WHOLE_PART_LINE.to_string());
    let mut meanings: Vec<(&WordId, &WordId)> = grammar.meaning.iter().collect();
    meanings.sort_by_key(|(word, token)| (word_text(**word), word_text(**token)));
    lines.extend(meanings.into_iter().map(|(word, token)| MEANING_FACT.replace(WORD_MARK, &written(word_text(*word))).replace(TOKEN_MARK, word_text(*token))));
    lines.push(ALONE_LINE.to_string());
    for ((shape, word), form) in keys {
        let text = joined_form_text(*shape, form);
        let shape_lines: &[&str] = match shape {
            JoinShape::Between => &[],
            JoinShape::After => &[AFTER_PART_LINE, AFTER_LINE],
            JoinShape::Meaning => &[MEANING_PART_LINE, MEANING_LONG_PART_LINE, MEANING_LINE],
            JoinShape::Before => &[BEFORE_PART_LINE, BEFORE_LINE],
            JoinShape::Head => &[HEAD_LINE],
            JoinShape::HeadOfTwo => &[HEAD_OF_TWO_LINE],
        };
        lines.extend(shape_lines.iter().map(|line| fill(line, word, &text)));
    }
    lines.push(ANSWER_LINE.to_string());
    lines
}
because!(grammar_lines, LearnedJoining, "the proposed grammar as lines of the library: the command of the question, each word between two parts with the parts it makes and its answer, the one part of a command with no such word, the meanings as facts, a word of meaning alone, then each way of joining with the parts it makes and its answer, and the question form; a part is made only where the search reads one, so the engine reads the same parts");

pub fn learn_joining(cases: &[Case]) -> Result<Joining, String> {
    let lessons: Vec<CommandLesson> = cases.iter().map(|case| CommandLesson { command: fact_numbers(&case.question), tokens: case.answer.split_whitespace().map(word_number).collect() }).collect();
    let (grammar, proposed_right) = proposed(&lessons);
    let lines = grammar_lines(&grammar);
    let topic = read_topic(JOINED_TOPIC, &lines.join(JOINED_LINE_BREAK))?;
    Ok(Joining { lines, proposed_right, model: model_of(vec![topic]) })
}
because!(learn_joining, LearnedJoining, "the learned joining of the lessons of a kind: the grammar that the search proposes, written as lines of the library and read by the library reader, whose reading by the engine is the model; the fault of the reader when a line does not read");
