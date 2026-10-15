use crate::base::lanes::{on_lanes, shards};
use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::pattern::choices::{Choice, OwnChoices};
use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::codes::Point;
use crate::tools::pattern::counting::Counting;
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::model::{FIRST_LATER, PatternAnswers, PatternModel, SURE_READINGS, SURE_SHARE, Setting};
use crate::tools::pattern::shared_words::fresh_words;
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::splits::learn_splits;
use crate::tools::pattern::story::Desk;
use crate::tools::pattern::table::{Ready, Tops};
use crate::tools::pattern::told::ToldRules;
use crate::tools::pattern::write_back::{WriteBack, WriteForm};
use crate::tools::plain::Plain;
use patterns::{because, source};

pub struct ChainDepth;
source!(ChainDepth, "the supporting facts of the tasks of Weston and others: a question needs a chain of a few told sentences at most, and a pattern with one sentence more multiplies the patterns that are counted");

pub struct HeldOutTest;
source!(HeldOutTest, "the holdout estimate: of two ways to answer, the one is taken that gets more right on lessons that neither way counted, so a way does not win by having seen its test");

pub const FULL_DEPTH: u32 = 3;
because!(FULL_DEPTH, ChainDepth, "the most told sentences of a pattern: the tasks of the public set that need a chain need this many supporting sentences at most, and each sentence more multiplies the patterns that are counted");

pub const PLAIN_LEAST: usize = 50;
because!(PLAIN_LEAST, HeldOutTest, "plain counting is tried only with this many lessons or more: its test on the last part of the lessons needs enough cases to tell it from the most frequent answer");

pub const PLAIN_PARTS: usize = 5;
because!(PLAIN_PARTS, HeldOutTest, "the lessons are cut into this many parts for the test of plain counting, and the last part is the test");

pub fn point_kinds(words: &PatternWords, ready: &[Ready]) -> Vec<u32> {
    let mut kinds: Vec<u32> = Vec::new();
    for lesson in ready {
        kinds.extend(lesson.case.slots.iter().chain(&lesson.answer).map(|word| words.kind_of(*word)).filter(|kind| *kind != ABSENT));
    }
    kinds.sort_unstable();
    kinds.dedup();
    kinds
}
because!(point_kinds, PatternAnswers, "the kinds that a pointing may use: the kinds of the words that the questions of the lessons name and of the words of their answers, in the order of the names of the kinds; a pointing by any other kind could not lead from a question to an answer");

pub struct ChunkedLoop;
source!(ChunkedLoop, "chunk scheduling of a loop whose steps do not depend on each other, as Kruskal and Weiss studied it: the steps are cut into chunks that the threads take from one shared counter, and with several chunks for each thread the threads end at nearly the same time although the steps differ in length");

const TOP_RUNS: usize = 4;
because!(TOP_RUNS, ChunkedLoop, "the runs of patterns for each thread when the most frequent place of every pattern is found: several runs for each thread, so that a thread with large patterns gives its other runs to the threads that are done");

pub fn learn_counted(words: &PatternWords, symbols: &mut Symbols, ready: &[Ready], set: Setting) -> PatternModel {
    let mut count = Counting::over(words, ready, point_kinds(words, ready), set.star, (SURE_READINGS, SURE_SHARE), set.per_form, set.lanes);
    count.count_one(symbols);
    if set.depth >= FIRST_LATER {
        count.count_more(symbols, FIRST_LATER, None);
    }
    count.make_list(symbols);
    let mut parts = if set.split { learn_splits(&mut count, words, symbols, set) } else { Vec::new() };
    if set.depth >= FULL_DEPTH {
        let open: Vec<u32> = count.open_forms(symbols).into_iter().map(|(asked, _)| asked).collect();
        count.count_more(symbols, FULL_DEPTH, Some(&open));
        count.make_list(symbols);
    }
    let patterns = count.table.parent.len();
    let runs = shards(patterns, set.lanes.max(1) * TOP_RUNS);
    let table = &count.table;
    let found: Vec<Vec<(u32, u32)>> = on_lanes(runs.len(), set.lanes, &|run| {
        let mut tops = Tops::default();
        runs[run].clone().map(|pattern| tops.top(table.of(pattern as u32))).collect()
    });
    let mut place: Vec<u32> = Vec::with_capacity(patterns);
    let mut hit: Vec<u32> = Vec::with_capacity(patterns);
    for (top, hits) in found.into_iter().flatten() {
        place.push(top);
        hit.push(hits);
    }
    let mut answers: FirstSightTable<&[u32], u32> = FirstSightTable::new();
    for lesson in ready {
        answers.add(lesson.answer.as_slice(), 1);
    }
    let default: Vec<u32> = answers.first_most().map(|(answer, _)| answer.to_vec()).unwrap_or_default();
    let left = count.left();
    let Counting { forms, steps, asked, places, mut table, order, point_kinds, .. } = count;
    table.readings = Vec::new();
    let mut rank: Vec<u32> = vec![ABSENT; patterns];
    for (at, rule) in order.iter().enumerate() {
        rank[rule.pattern as usize] = at as u32;
    }
    let steps = steps.freeze();
    for part in &mut parts {
        part.used = order.iter().any(|rule| {
            let pattern = rule.pattern as usize;
            let key = steps.held(table.step[pattern]);
            table.depth[pattern] == 1 && key.point == Point::Split && key.item == part.kind && table.step[table.parent[pattern] as usize] == part.asked
        });
    }
    table.child.freeze();
    PatternModel { depth: set.depth, star: set.star, hash: set.hash, forms: forms.freeze(), steps, asked: asked.freeze(), places, table, place, hit, rank, order, left, default, point_kinds, parts }
}
because!(learn_counted, PatternAnswers, "one counted model from lessons that were read: the patterns with one sentence, then with a second where a pattern is read often and is not sure, the list of rules, the splits for the question forms that the list leaves open, a third sentence for the forms that are still open, the list again, and for each pattern its most frequent place of the answer");

pub struct PatternLearner {
    pub symbols: Symbols,
    pub words: PatternWords,
    pub model: PatternModel,
    pub plain: Option<Plain>,
    pub writes: Vec<WriteForm>,
    pub affirm: Vec<u32>,
    pub choice: Option<Choice>,
    pub reader: Option<ToldRules>,
}
because!(PatternLearner, PatternAnswers, WriteBack, OwnChoices, "the pattern learner after its lessons: the table of words, the words and kinds, the counted model, plain counting where it is used, the statement forms and the affirming word of the write back, the learner of choices, and the reader of told rules where it is on. learn makes it from lessons; said gives the words of an answer and how it came; the tools that build on it read its parts");

pub fn read_lessons(words: &PatternWords, symbols: &mut Symbols, cases: &[(Vec<String>, String, String)], readings: &dyn Fn(&[u32]) -> u32) -> Vec<Ready> {
    let mut desk = Desk::over(words, symbols);
    let mut ready: Vec<Ready> = Vec::with_capacity(cases.len());
    for (sentences, question, answer) in cases {
        let case = desk.case(sentences, question, readings);
        let answer = words.read(desk.names, &answer_words(answer));
        ready.push(Ready { case, answer });
    }
    ready
}
because!(read_lessons, PatternAnswers, "the lessons read into cases and answers, each different told sentence read one time");

pub fn usual_answer(answers: &[Vec<String>]) -> Vec<String> {
    let mut seen: FirstSightTable<&[String], u32> = FirstSightTable::new();
    for answer in answers {
        seen.add(answer.as_slice(), 1);
    }
    seen.first_most().map(|(answer, _)| answer.to_vec()).unwrap_or_default()
}
because!(usual_answer, PatternAnswers, "the answer that a set of lessons has most often, the first of equals in their order: what plain counting must beat to be used");

pub fn tested_plain(cases: &[Case]) -> Option<Plain> {
    if cases.len() < PLAIN_LEAST {
        return None;
    }
    let cut = cases.len() * (PLAIN_PARTS - 1) / PLAIN_PARTS;
    let first = Plain::learn(&cases[..cut]);
    let learned: Vec<Vec<String>> = cases[..cut].iter().map(|case| answer_words(&case.answer)).collect();
    let one = usual_answer(&learned);
    let (mut by_plain, mut by_one) = (0, 0);
    for case in &cases[cut..] {
        let truth = answer_words(&case.answer);
        by_plain += usize::from(first.best(&case.sentences, &case.question) == Some(truth.as_slice()));
        by_one += usize::from(truth == one);
    }
    (by_plain > by_one).then(|| Plain::learn(cases))
}
because!(tested_plain, PatternAnswers, "plain counting of all lessons, when plain counting of the first parts of the lessons gets more of the last part right than the most frequent answer does; nothing else, so that a case that no pattern fits gets the better of the two plain answers");

impl PatternLearner {
    pub fn learn(cases: &[Case], lanes: usize) -> PatternLearner {
        let (mut symbols, words) = fresh_words(cases);
        let set = Setting { depth: FULL_DEPTH, per_form: false, split: true, lanes, star: symbols.number("*"), hash: symbols.number("#") };
        let plain = tested_plain(cases);
        let reader = Some(ToldRules::learn(&words, &mut symbols, cases, SURE_READINGS, SURE_SHARE, lanes)).filter(|reader| reader.on);
        let told: Vec<(Vec<String>, String, String)> = cases
            .iter()
            .map(|case| {
                let sentences = match &reader {
                    Some(reader) => reader.written(&words, &mut symbols, &case.sentences),
                    None => case.sentences.clone(),
                };
                (sentences, case.question.clone(), case.answer.clone())
            })
            .collect();
        let ready = read_lessons(&words, &mut symbols, &told, &|_| 0);
        let model = learn_counted(&words, &mut symbols, &ready, set);
        let rules_on = reader.as_ref().is_some_and(|reader| reader.rules_on);
        let mut learner = PatternLearner { symbols, words, model, plain, writes: Vec::new(), affirm: Vec::new(), choice: None, reader };
        if !rules_on {
            learner.learn_statements(&ready);
        }
        learner.learn_choices(&told, set);
        learner
    }
}
