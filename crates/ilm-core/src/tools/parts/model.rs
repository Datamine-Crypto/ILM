use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::split::told_texts;
use crate::cases::Case;
use crate::tools::call::{Learned, Learner, Lessons, Said, Tool};
use crate::tools::parts::chain::Chains;
use crate::tools::parts::classes::{ClassTable, class_search, class_table};
use crate::tools::parts::counted::{FORCED_LESSONS, sure_of};
use crate::tools::parts::cutter::{row_words, told_words};
use crate::tools::parts::found::{CLASS_MOST, FOUND_LESSONS, FormsFound, found_forms};
use crate::tools::parts::grammar::{Grammar, KEPT_NAMES, Meter, Reading};
use crate::tools::parts::language::{WordRow, Word};
use crate::tools::parts::ordered::Kept;
use crate::tools::parts::places::Lexicon;
use crate::tools::parts::rewrite::Rewrites;
use crate::tools::parts::search::Search;
use crate::tools::parts::solver::Light;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::base::lanes::on_lanes;

pub struct ThreeWays;
source!(ThreeWays, "a learner of meaning built from parts with ways of reading chosen by lessons that a way did not learn from: the rows by a search over the states of the words, the class of what is left of a text where the answers are few, the forms found from the lessons, the chains of steps on one state, and one word of signs changed by its condition; a way is taken where the ways before it read too few of the lessons, and where two ways read lessons the last third of the lessons decides: a later way is taken when its model of the first two thirds reads more of the last third than the model of the way that was taken");

const LEARNED_THIRDS: usize = 2;
because!(LEARNED_THIRDS, ThreeWays, "the thirds of the lessons that a way learns from before it reads the rest: most of them, so that a way learns nearly what it learns from all");

const THIRDS: usize = 3;
because!(THIRDS, ThreeWays, "the parts that the lessons are counted in for the split into the lessons learned from and the lessons read");

const CONTROL_TRIALS: usize = 60;
because!(CONTROL_TRIALS, ThreeWays, "the changed lessons of the control with a new word at most: enough to tell a way that follows the change from one that does not");

const CONTROL_HALF: usize = 2;
because!(CONTROL_HALF, ThreeWays, "the rows fail the control when they read fewer than half of the changed lessons: the changed lessons read right times this factor stay below the lessons tried");

const NEW_WORD: &str = "newword";
because!(NEW_WORD, ThreeWays, "the word that the control puts in the place of a word that says itself, when no lesson holds it");

const LONGER_WORD: char = 'x';
because!(LONGER_WORD, ThreeWays, "the letter that is added to the new word of the control while a lesson holds the word");

const NO_SUCH_WORD: Word = Word::MAX;
because!(NO_SUCH_WORD, ThreeWays, "the number of an answer of signs whose word no lesson holds: it equals no word of a lesson, so such an answer is never a right one");

const ANSWER_BETWEEN: &str = " ";
because!(ANSWER_BETWEEN, ThreeWays, "the mark between the words of an answer when they are written as one text for the measure");

const NOTE_ROWS: &str = "parts: the row that is left";
because!(NOTE_ROWS, ThreeWays, "what an answer says of itself when the forms in their order of use made it: so a reader of a file of answers sees which way of the tool gave each");

const NOTE_CLASS: &str = "parts: the class of what is left";
because!(NOTE_CLASS, ThreeWays, "the note of an answer that the class of what is left gives");

const NOTE_FOUND: &str = "parts: the forms found from the lessons";
because!(NOTE_FOUND, ThreeWays, "the note of an answer that the found forms give");

const LATER_PARTS: usize = 3;
because!(LATER_PARTS, ThreeWays, "the parts that the last third of the lessons is counted in for the share that a later way must read");

const LATER_READ: usize = 2;
because!(LATER_READ, ThreeWays, "of those parts a later way must read more than this many, far more than a guess reads where the lessons have few answers to choose from, so that a way that reads lessons by chance does not take the place of the others");

const KEPT_SHARE: usize = 2;
because!(KEPT_SHARE, ThreeWays, "a changed word is tried only where more than one lesson in this many keeps some signs of its word in the answer");

const NOTE_CHAIN: &str = "parts: the chain of steps on one state";
because!(NOTE_CHAIN, ThreeWays, "the note of an answer that the chains of steps give");

const NOTE_REWRITE: &str = "parts: the word changed by its condition";
because!(NOTE_REWRITE, ThreeWays, "the note of an answer that the changed word gives");

const NOTE_DEFAULT: &str = "parts: the most frequent answer";
because!(NOTE_DEFAULT, ThreeWays, "the note of an answer that no way gave: the most frequent answer of the lessons");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Rows,
    Class,
    FormsFound,
    Chain,
    Rewrite,
}
because!(Mode, ThreeWays, "the way of reading that a model took");

pub struct PartsModel {
    symbols: Symbols,
    rank: Vec<u32>,
    default: Vec<String>,
    mode: Mode,
    grammar: Grammar,
    class: Option<ClassTable>,
    found: Option<FormsFound>,
    chain: Option<Chains>,
    rewrite: Option<Rewrites>,
    right: usize,
    pub fault: Option<String>,
}
because!(PartsModel, ThreeWays, "a model of the parts tool: the words with their numbers and their order, the most frequent answer, the way that was taken with what it learned, and the lessons that it reads right; a fault inside the third way is kept and the first two ways stay");

fn last_third(cases: &[Case]) -> Vec<bool> {
    let (keys, sizes) = told_texts(cases);
    let mut first = vec![false; sizes.len()];
    let mut count = 0;
    for (key, size) in sizes.iter().enumerate() {
        if count * THIRDS >= cases.len() * LEARNED_THIRDS {
            break;
        }
        first[key] = true;
        count += size;
    }
    keys.iter().map(|key| !first[*key]).collect()
}
because!(last_third, ThreeWays, "which lessons are in the last third: told texts are taken whole in the order of first sight until two thirds of the lessons are taken, and a case with no told sentence is a story of its own");

fn ranks(names: &[String]) -> Vec<u32> {
    let mut order: Vec<usize> = (0..names.len()).collect();
    order.sort_by(|one, other| names[*one].cmp(&names[*other]));
    let mut rank = vec![0u32; names.len()];
    for (place, word) in order.into_iter().enumerate() {
        rank[word] = place as u32;
    }
    rank
}
because!(ranks, ThreeWays, "the place of each word in the order of the texts of the words");

fn unique(lessons: &[(Vec<Word>, WordRow)]) -> Vec<(Vec<Word>, WordRow)> {
    let mut kept: Kept<(Vec<Word>, WordRow), ()> = Kept::default();
    for lesson in lessons {
        kept.put(lesson.clone(), ());
    }
    kept.into_pairs().map(|(lesson, _)| lesson).collect()
}
because!(unique, ThreeWays, "each text with its answer once, in the order of first sight");

fn grammar_right(grammar: &Grammar, lessons: &[(Vec<Word>, WordRow)], light: &Light) -> usize {
    lessons.iter().filter(|(words, answer)| grammar.reads(words, light).as_ref() == Some(answer)).count()
}
because!(grammar_right, ThreeWays, "the lessons that a grammar reads right in the strict way");

fn said(grammar: &Grammar, words: &[Word], meter: &Meter) -> WordRow {
    grammar.read(words, meter, Reading::plain(false)).ok().flatten().unwrap_or_default()
}
because!(said, ThreeWays, "the answer that a grammar gives in the loose way, and no word when it gives none");

fn control_trials(lessons: &[(Vec<Word>, WordRow)], new_word: Word) -> Vec<(Vec<Word>, WordRow)> {
    let mut in_text: HashMap<Word, usize> = HashMap::new();
    let mut in_answer: HashMap<Word, usize> = HashMap::new();
    let mut in_both: HashMap<Word, usize> = HashMap::new();
    for (words, answer) in lessons {
        let told: HashSet<Word> = words.iter().copied().collect();
        let given: HashSet<Word> = answer.iter().copied().collect();
        for word in &told {
            *in_text.entry(*word).or_default() += 1;
        }
        for word in &given {
            *in_answer.entry(*word).or_default() += 1;
        }
        for word in told.intersection(&given) {
            *in_both.entry(*word).or_default() += 1;
        }
    }
    let itself: HashSet<Word> = in_both.iter().filter(|(word, count)| **count >= FORCED_LESSONS && sure_of(**count, in_text[*word]) && sure_of(**count, in_answer[*word])).map(|(word, _)| *word).collect();
    let mut trials: Vec<(Vec<Word>, WordRow)> = Vec::new();
    for (words, answer) in lessons {
        let mut distinct: Vec<Word> = Vec::new();
        for word in words {
            if !distinct.contains(word) {
                distinct.push(*word);
            }
        }
        for word in distinct {
            let once = |row: &[Word]| row.iter().filter(|other| **other == word).count() == 1;
            if itself.contains(&word) && once(words) && once(answer) && trials.len() < CONTROL_TRIALS {
                let swap = |row: &[Word]| row.iter().map(|other| if *other == word { new_word } else { *other }).collect::<WordRow>();
                trials.push((swap(words), swap(answer)));
            }
        }
        if trials.len() >= CONTROL_TRIALS {
            break;
        }
    }
    trials
}
because!(control_trials, ThreeWays, "the changed lessons of a control of the tool's own for a way that reads the lessons: does it read a lesson with a word that no lesson holds? A word says itself when it stands in the answer of nearly every lesson that holds it in the text and in the text of nearly every lesson that holds it in the answer; in a lesson that holds such a word once in the text and once in the answer the word becomes a new word in both, each such word of a lesson in turn");

struct ThirdGround<'a> {
    lessons: &'a [(Vec<Word>, WordRow)],
    later: &'a [bool],
    chars: &'a HashMap<Word, WordRow>,
    new_word: Word,
    symbols: &'a Symbols,
    lexicon: &'a Lexicon<'a>,
    lanes: usize,
}
because!(ThirdGround, ThreeWays, "what the third way needs from the learning: the lessons and which are in the last third, the signs of each answer word, the new word of the control, the words with their numbers and their order, and the threads of the run");

fn found_says(found: &FormsFound, words: &[Word], ground: &ThirdGround) -> Option<WordRow> {
    let row = found.read(words, ground.lexicon)?;
    if !found.signs {
        return Some(row);
    }
    let text: String = row.iter().map(|word| ground.lexicon.name(*word)).collect();
    Some(vec![ground.symbols.find(&text).unwrap_or(NO_SUCH_WORD)])
}
because!(found_says, ThreeWays, "the answer of the found forms as a row of words: when an answer is read as the row of its signs, the signs are joined to one word again");

fn counted_on(ground: &ThirdGround, lessons: &[&(Vec<Word>, WordRow)], read: &(dyn Fn(&[Word]) -> Option<WordRow> + Sync)) -> usize {
    on_lanes(lessons.len(), ground.lanes, &|piece| read(&lessons[piece].0).as_ref() == Some(&lessons[piece].1)).into_iter().filter(|right| *right).count()
}
because!(counted_on, ThreeWays, "the lessons that a way reads right, read on all threads; a read changes nothing, so the count is the same for every number of threads");

fn third_way(ground: &ThirdGround, rows: (&Grammar, &Grammar), mode: Mode, right: usize, second_right: usize) -> Option<(FormsFound, usize)> {
    let lessons = ground.lessons;
    let (g, inner) = rows;
    let all = unique(lessons);
    let mut given = right;
    let rows_read = mode == Mode::Rows && sure_of(given, lessons.len());
    let trials = if rows_read { control_trials(&all, ground.new_word) } else { Vec::new() };
    let trial_refs: Vec<&(Vec<Word>, WordRow)> = trials.iter().collect();
    let control = (trials.len(), if rows_read { counted_on(ground, &trial_refs, &|words: &[Word]| Some(said(g, words, &Meter::default()))) } else { 0 });
    if sure_of(given, lessons.len()) && !(control.0 >= FORCED_LESSONS && control.1 * CONTROL_HALF < control.0) {
        return None;
    }
    let found = found_forms(&all, ground.chars, ground.lexicon, ground.lanes);
    if found.forms.is_empty() {
        return None;
    }
    let seen: Vec<&(Vec<Word>, WordRow)> = lessons[..lessons.len().min(FOUND_LESSONS)].iter().collect();
    let right_found = counted_on(ground, &seen, &|words: &[Word]| found_says(&found, words, ground)) * lessons.len() / seen.len();
    let taken = if rows_read {
        sure_of(right_found, lessons.len()) && counted_on(ground, &trial_refs, &|words: &[Word]| found_says(&found, words, ground)) > control.1
    } else {
        if mode == Mode::Rows && right_found > given {
            given = counted_on(ground, &lessons.iter().collect::<Vec<_>>(), &|words: &[Word]| Some(said(g, words, &Meter::default())));
        }
        let mut taken = right_found > given;
        if taken && given > 0 {
            let first_lessons: Vec<(Vec<Word>, WordRow)> = lessons.iter().zip(ground.later).filter(|(_, later)| !**later).map(|(lesson, _)| lesson.clone()).collect();
            let first = found_forms(&unique(&first_lessons), ground.chars, ground.lexicon, ground.lanes);
            let last: Vec<&(Vec<Word>, WordRow)> = lessons.iter().zip(ground.later).filter(|(_, later)| **later).map(|(lesson, _)| lesson).collect();
            let second = if mode == Mode::Class { second_right } else { counted_on(ground, &last, &|words: &[Word]| Some(said(inner, words, &Meter::default()))) };
            taken = counted_on(ground, &last, &|words: &[Word]| found_says(&first, words, ground)) > second;
        }
        taken
    };
    taken.then_some((found, right_found))
}
because!(third_way, ThreeWays, "the forms found from the lessons where they read more than the first two ways: tried where those read too few of the lessons, or where the rows read the lessons but not a lesson with a new word; taken where the first two ways read no lesson and they read some, where the last third of the lessons says that they read more, or where they follow the control more often than the rows");

type LaterTaken = (Mode, usize, Option<Chains>, Option<Rewrites>);
because!(LaterTaken, ThreeWays, "a later way that was taken: its mode, the lessons that its model of all lessons reads right, and what it learned");

fn later_way(lessons: &[(Vec<Word>, WordRow)], first: &[(Vec<Word>, WordRow)], last: &[(Vec<Word>, WordRow)], taken_last: usize) -> Option<LaterTaken> {
    let right_of = |read: &dyn Fn(&[Word]) -> Option<WordRow>, part: &[(Vec<Word>, WordRow)]| part.iter().filter(|(words, answer)| read(words).as_ref() == Some(answer)).count();
    let mut best_last = taken_last.max(last.len() * LATER_READ / LATER_PARTS);
    let mut taken: Option<LaterTaken> = None;
    if let Some(inner) = Chains::learn(first) {
        let read_last = right_of(&|words: &[Word]| inner.read(words), last);
        if read_last > best_last {
            best_last = read_last;
            taken = Chains::learn(lessons).map(|model| (Mode::Chain, right_of(&|words: &[Word]| model.read(words), lessons), Some(model), None));
        }
    }
    let inner = Rewrites::learn(first);
    if inner.kept * KEPT_SHARE > first.len() && right_of(&|words: &[Word]| inner.read(words), last) > best_last {
        let model = Rewrites::learn(lessons);
        taken = Some((Mode::Rewrite, right_of(&|words: &[Word]| model.read(words), lessons), None, Some(model)));
    }
    taken
}
because!(later_way, ThreeWays, "the later way that reads the most of the last third of the lessons with a model of the first two thirds, when it reads more than two thirds of it and more of it than the way that was taken; then its model of all lessons. A changed word is tried only where most lessons keep some signs of their word in the answer, as a changed word does");

impl PartsModel {
    pub fn learn(cases: &[Case], lanes: usize) -> PartsModel {
        let mut answers: Kept<Vec<String>, usize> = Kept::default();
        for case in cases {
            answers.count(answer_words(&case.answer), 1);
        }
        let default = answers.most().map(|(answer, _)| answer.clone()).unwrap_or_default();
        let mut symbols = Symbols::new();
        for name in KEPT_NAMES {
            symbols.number(name);
        }
        let lessons: Vec<(Vec<Word>, WordRow)> = cases.iter().map(|case| (symbols.numbers(&told_words(&case.sentences, &case.question)), symbols.numbers(&row_words(&case.answer)))).collect();
        let later = last_third(cases);
        let mut chars: HashMap<Word, WordRow> = HashMap::new();
        for (_, answer) in &lessons {
            for word in answer {
                if !chars.contains_key(word) {
                    let name = symbols.name(*word).to_string();
                    let row: WordRow = name.chars().map(|sign| symbols.number(&sign.to_string())).collect();
                    chars.insert(*word, row);
                }
            }
        }
        let known: HashSet<Word> = lessons.iter().flat_map(|(words, answer)| words.iter().chain(answer)).copied().collect();
        let mut new = NEW_WORD.to_string();
        while symbols.find(&new).is_some_and(|word| known.contains(&word)) {
            new.push(LONGER_WORD);
        }
        let new_word = symbols.number(&new);
        let names = symbols.names().to_vec();
        let rank = ranks(&names);
        let meter = Meter::default();
        let light = Light { meter: &meter, rank: &rank };
        let first: Vec<(Vec<Word>, WordRow)> = lessons.iter().zip(&later).filter(|(_, later)| !**later).map(|(lesson, _)| lesson.clone()).collect();
        let last: Vec<(Vec<Word>, WordRow)> = lessons.iter().zip(&later).filter(|(_, later)| **later).map(|(lesson, _)| lesson.clone()).collect();
        let inner = Search::grammar_of(&first, None, &light);
        let mut second_right = grammar_right(&inner, &last, &light);
        let g = Search::grammar_of(&lessons, Some(&inner), &light);
        let mut right = grammar_right(&g, &lessons, &light);
        let mut mode = Mode::Rows;
        let mut grammar = g.clone();
        let mut class: Option<ClassTable> = None;
        let distinct: HashSet<&WordRow> = lessons.iter().map(|(_, answer)| answer).collect();
        let rank_of = |word: Word| rank.get(word as usize).copied().unwrap_or(word);
        if !sure_of(right, lessons.len()) && distinct.len() > 1 && distinct.len() <= CLASS_MOST {
            let inner_class = class_search(&first, None, &meter, &rank_of);
            let table = class_table(&inner_class, &first, &meter);
            let second = last.iter().filter(|(words, answer)| table.table.get(&inner_class.left(words, &meter)).unwrap_or(&table.default) == answer).count();
            let g2 = class_search(&lessons, Some(&inner_class), &meter, &rank_of);
            let table = class_table(&g2, &lessons, &meter);
            let class_right = lessons.iter().filter(|(words, answer)| table.table.get(&g2.left(words, &meter)).unwrap_or(&table.default) == answer).count();
            if (!g2.forms.is_empty() || !g2.fixed.is_empty()) && g2.right > right {
                mode = Mode::Class;
                grammar = g2;
                right = class_right;
                class = Some(table);
                second_right = second;
            }
        }
        let extra: Vec<String> = Vec::new();
        let lexicon = Lexicon { names: &names, extra: &extra, rank: &rank };
        let ground = ThirdGround { lessons: &lessons, later: &later, chars: &chars, new_word, symbols: &symbols, lexicon: &lexicon, lanes };
        let decided = catch_unwind(AssertUnwindSafe(|| third_way(&ground, (&g, &inner), mode, right, second_right)));
        let (mut found, mut fault) = (None, None);
        match decided {
            Ok(Some((taken, found_right))) => {
                mode = Mode::FormsFound;
                right = found_right;
                found = Some(taken);
            }
            Ok(None) => {}
            Err(why) => fault = Some(why.downcast_ref::<String>().cloned().or_else(|| why.downcast_ref::<&str>().map(|text| text.to_string())).unwrap_or_default()),
        }
        let taken_last = match mode {
            Mode::FormsFound => found.as_ref().map_or(0, |taken| last.iter().filter(|(words, answer)| found_says(taken, words, &ground).as_ref() == Some(answer)).count()),
            _ => second_right,
        };
        let (chain, rewrite);
        (mode, right, chain, rewrite) = later_way(&lessons, &first, &last, taken_last).map_or((mode, right, None, None), |(later, right, chain, rewrite)| (later, right, chain, rewrite));
        PartsModel { symbols, rank, default, mode, grammar, class, found, chain, rewrite, right, fault }
    }

    fn answer_of(&self, sentences: &[String], question: &str) -> Option<(Vec<String>, &'static str)> {
        if self.right == 0 {
            return None;
        }
        let names = self.symbols.names();
        let mut extra: Vec<String> = Vec::new();
        let mut index: HashMap<String, Word> = HashMap::new();
        let mut words: Vec<Word> = Vec::new();
        for text in told_words(sentences, question) {
            let word = match self.symbols.find(&text) {
                Some(word) => word,
                None => *index.entry(text.clone()).or_insert_with(|| {
                    extra.push(text.clone());
                    (names.len() + extra.len() - 1) as Word
                }),
            };
            words.push(word);
        }
        let lexicon = Lexicon { names, extra: &extra, rank: &self.rank };
        let meter = Meter::default();
        let joined = |row: &[Word], between: &str| row.iter().map(|word| lexicon.name(*word)).collect::<Vec<&str>>().join(between);
        match self.mode {
            Mode::FormsFound => {
                let found = self.found.as_ref()?;
                let row = catch_unwind(AssertUnwindSafe(|| found.read(&words, &lexicon))).ok().flatten()?;
                let text = if found.signs { joined(&row, "") } else { joined(&row, ANSWER_BETWEEN) };
                Some((answer_words(&text), NOTE_FOUND))
            }
            Mode::Class => {
                let class = self.class.as_ref()?;
                let left = self.grammar.left(&words, &meter);
                let row = class.table.get(&left).unwrap_or(&class.default);
                Some((answer_words(&joined(row, ANSWER_BETWEEN)), NOTE_CLASS))
            }
            Mode::Chain => {
                let row = self.chain.as_ref()?.read(&words)?;
                Some((answer_words(&joined(&row, ANSWER_BETWEEN)), NOTE_CHAIN))
            }
            Mode::Rewrite => {
                let row = self.rewrite.as_ref()?.read(&words)?;
                Some((answer_words(&joined(&row, ANSWER_BETWEEN)), NOTE_REWRITE))
            }
            Mode::Rows => {
                let row = said(&self.grammar, &words, &meter);
                if row.is_empty() && !self.default.is_empty() {
                    return None;
                }
                Some((answer_words(&joined(&row, ANSWER_BETWEEN)), NOTE_ROWS))
            }
        }
    }
}

impl Learner for PartsModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        match self.answer_of(sentences, question) {
            Some((words, note)) => Said { words, note: note.to_string() },
            None => Said { words: self.default.clone(), note: NOTE_DEFAULT.to_string() },
        }
    }
}

pub fn learn_parts(lessons: Lessons, lanes: usize) -> Learned {
    let model: Box<dyn Learner> = Box::new(PartsModel::learn(lessons.cases, lanes));
    Ok(model)
}
because!(learn_parts, ThreeWays, "the parts tool as a tool of the one call: it learns from the lessons alone and reads every kind, so it never says that it does not read one");

pub const PARTS_TOOL: Tool = Tool { name: "parts", needs_labels: false, probe: false, asks: false, over: None, learn: learn_parts };
because!(PARTS_TOOL, ThreeWays, "the parts tool with its name: it needs no labels of lesson sentences, gives no control by the form of the question, and asks no question");
