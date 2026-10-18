use crate::base::lanes::on_spare_lanes;
use crate::base::words::answer_words;
use crate::cases::Case;
use crate::choice::{inner_parts, picked_labels, right_of};
use crate::tools::call::{Learner, Lessons, Reading};
use crate::tools::explain::costs::{WAY_BARS, WAY_BEYOND, WAY_JOINING, WAY_LABEL_LIST, WAY_MEANINGS, WAY_RECKONING, WAY_REWRITE, cost_scope, learning_note, spent_way};
use crate::tools::explain::learn::{ExplainModel, Library, model_of};
use crate::tools::explain::text::{LogicPiece, Topic};
use crate::tools::explain::words::word_text;
use crate::tools::explain::wording::{WordedModel, bridged_written, facts_beside_written, worded_model, worded_search};
use crate::tools::explain::wording_joining::learn_joining;
use crate::tools::explain::wording_rewrite::{RewriteModel, rewrite_kind};
use crate::tools::explain::wording_meanings::meanings_topic;
use crate::tools::explain::wording_numbers::{NUMBER_FACT_WORDS, reckoned_topics};
use crate::tools::explain::wording_words::{WORD_RIGHT_PERCENTS, word_topics};
use crate::tools::plain::learn_question;
use patterns::{because, source};
use std::cell::OnceCell;
use std::collections::HashMap;

pub struct OneReading;
source!(OneReading, "the one learner of the reading of a kind: every way that learns a reading from the lessons (the list over the words of the told text, the ways to reckon over the numbers of a story, the meanings of function words over lists, the ways of joining of a language of commands, the rewrites of a sentence into another build, and the learned wording beside the written texts) is judged on the last lessons of the kind against the same bars, worked out once: how the written texts that fit read those lessons and how the question alone reads them; the first way that passes gives the reading");

const WHOLE_OF_LAST: usize = 100;
because!(WHOLE_OF_LAST, OneReading, "the whole of the last lessons in hundredths, against which the share that the written texts read is taken");

enum LearnedAgain {
    Labels(usize),
    Reckoned(usize),
    Meant,
    Joined,
    Rewritten,
}
because!(LearnedAgain, OneReading, "how a learned way learns again from all lessons once it is taken: the list over the words of the told text or the ways to reckon at the trust that answered the most last lessons, the meanings, the joining, or the rewrites of sentences");

struct WayJudged {
    right: usize,
    again: LearnedAgain,
}
because!(WayJudged, OneReading, "a learned way judged on the last lessons of a kind: how many it answers right when learned from the first lessons, and how it learns again from all of them");

type WayJudge = fn(&KindBars) -> Option<WayJudged>;
because!(WayJudge, OneReading, "a learned way as the one learner judges it on the bars of a kind");

const LEARNED_WAYS: &[(&str, WayJudge)] = &[(WAY_LABEL_LIST, label_judged), (WAY_RECKONING, reckoned_judged), (WAY_MEANINGS, meant_judged), (WAY_JOINING, joined_judged), (WAY_REWRITE, rewrite_judged)];
because!(LEARNED_WAYS, OneReading, "the learned ways in the order in which they are judged: the first that passes the bars is taken");

fn passes(bars: &KindBars, right: usize) -> bool {
    let last = bars.last_cases.len();
    let above_alone = || right * WHOLE_OF_LAST >= bars.alone() * WHOLE_OF_LAST + last * ABOVE_ALONE_POINTS;
    if bars.plain.is_some() { right >= bars.written() && above_alone() } else { above_alone() && right >= bars.written() }
}
because!(passes, OneReading, "whether a learned way passes the bars of a kind: it answers clearly more of the last lessons than the question alone, as a learned kind stands above its controls on the board, and at least as many as the written texts that fit (or the question forms learned for a text of rules), so that a learned reading takes the place of a written one that does no better, and a way that answers a lesson or two more than the question alone (rewrites on the functions of a row) is no reading; the cheap bar is asked first");

const ABOVE_ALONE_POINTS: usize = 5;
because!(ABOVE_ALONE_POINTS, OneReading, "the hundredths of the last lessons by which a learned way must stand above the question alone, as a learned kind stands above its controls on the board: a difference that the chance of a few hundred lessons does not give");

fn learned_again(bars: &KindBars, again: &LearnedAgain) -> Option<Box<dyn Learner>> {
    let every: Vec<&Case> = bars.lessons.cases.iter().collect();
    let model = match again {
        LearnedAgain::Labels(percent) => model_of(vec![word_topics(&every, &[*percent]).into_iter().next()??]),
        LearnedAgain::Reckoned(percent) => model_of(vec![reckoned_topics(numbers_text(bars.library)?, &every, &[*percent]).into_iter().next()??]),
        LearnedAgain::Meant => model_of(vec![meanings_topic(&bars.library.topics, &every)?]),
        LearnedAgain::Joined => learn_joining(bars.lessons.cases).ok()?.model,
        LearnedAgain::Rewritten => return RewriteModel::learn(&every).map(|model| Box::new(model) as Box<dyn Learner>),
    };
    Some(Box::new(model.learned().framed(bars.lessons.cases)))
}
because!(learned_again, OneReading, "the model of a taken way learned again from all lessons, as the board learns a tool again once it is chosen");

pub(super) fn learned_reading(library: &Library, lessons: &Lessons, plain: Option<&ExplainModel>, lanes: usize) -> (Option<Box<dyn Learner>>, Option<WordedModel>) {
    let bars = KindBars::of(library, lessons, plain, lanes);
    for &(way, judge) in LEARNED_WAYS {
        let mut scope = cost_scope(Some(way), None);
        let Some(judged) = judge(&bars) else { continue };
        if passes(&bars, judged.right)
            && let Some(model) = learned_again(&bars, &judged.again)
        {
            scope.taken();
            return (Some(model), None);
        }
    }
    (None, spent_way(WAY_BEYOND, || worded_beyond(&bars)))
}
because!(learned_reading, OneReading, "the reading of a kind that its lessons teach, by the first learned way in order that passes the bars of the kind and learns again from all lessons, and when none does, the learned wording that may read the kind beside the written texts that fit it");

const TABLE_HEAD: &str = "the learned ways judged on the last lessons of the learning, in order (the first that passes is taken): ";
because!(TABLE_HEAD, OneReading, "the words before the bars in the table of the learned ways of a kind");

const NO_WAY: &str = "does not apply to this kind";
because!(NO_WAY, OneReading, "what the table says of a learned way that has no candidate for the kind (another form of question, no numbers text, told sentences for the joining)");

const WAY_TAKEN: &str = ", passes, taken";
because!(WAY_TAKEN, OneReading, "the words after the first learned way that passes the bars");

const WAY_PASSES: &str = ", passes, after one taken";
because!(WAY_PASSES, OneReading, "the words after a learned way that passes the bars after another was taken");

const WAY_BELOW: &str = ", below the bars";
because!(WAY_BELOW, OneReading, "the words after a learned way that does not pass the bars");

pub(super) fn reading_table(library: &Library, lessons: &Lessons, plain: Option<&ExplainModel>, lanes: usize) -> Vec<String> {
    let bars = KindBars::of(library, lessons, plain, lanes);
    let last = bars.last_cases.len();
    let mut lines = vec![format!("{TABLE_HEAD}the written texts read {} of {last} (the learned wording, when no written text fits), the question alone {} of {last}", bars.written(), bars.alone())];
    let mut taken = false;
    for &(way, judge) in LEARNED_WAYS {
        lines.push(match judge(&bars) {
            None => format!("  {way}: {NO_WAY}"),
            Some(judged) => {
                let mark = match (passes(&bars, judged.right), taken) {
                    (true, false) => WAY_TAKEN,
                    (true, true) => WAY_PASSES,
                    (false, _) => WAY_BELOW,
                };
                taken |= mark == WAY_TAKEN;
                format!("  {way}: {} of {last}{mark}", judged.right)
            }
        });
    }
    lines
}
because!(reading_table, OneReading, "the table of the learned ways of a kind, as the one learner judges them: the bars (the written texts and the question alone on the last lessons) and each way with the last lessons it answers right, whether it passes, and which is taken; every way is judged, also after one was taken, so a person sees how near each came");

struct KindBars<'a> {
    library: &'a Library,
    lessons: &'a Lessons<'a>,
    plain: Option<&'a ExplainModel>,
    lanes: usize,
    first: Vec<usize>,
    last_cases: Vec<Case>,
    written: OnceCell<usize>,
    written_rights: OnceCell<Vec<bool>>,
    alone: OnceCell<usize>,
}
because!(KindBars, OneReading, "the lessons of a kind cut into its first and last lessons, with the bars on the last ones that a learned way must pass, each worked out once for all the ways that ask for it: how the written topics that fit read each last lesson, the written bar (or, when no written topic fits, the question forms learned for a text of rules), and the question alone");

impl<'a> KindBars<'a> {
    fn of(library: &'a Library, lessons: &'a Lessons<'a>, plain: Option<&'a ExplainModel>, lanes: usize) -> KindBars<'a> {
        let (first, last) = inner_parts(lessons.cases);
        let last_cases: Vec<Case> = last.iter().map(|at| lessons.cases[*at].clone()).collect();
        KindBars { library, lessons, plain, lanes, first, last_cases, written: OnceCell::new(), written_rights: OnceCell::new(), alone: OnceCell::new() }
    }

    fn first_cases(&self) -> Vec<&Case> {
        self.first.iter().map(|at| &self.lessons.cases[*at]).collect()
    }

    fn written_rights(&self) -> Option<&Vec<bool>> {
        let plain = self.plain?;
        Some(self.written_rights.get_or_init(|| {
            let _bars = cost_scope(Some(WAY_BARS), None);
            case_rights(plain, &self.last_cases, self.lanes)
        }))
    }

    fn written(&self) -> usize {
        *self.written.get_or_init(|| match self.written_rights() {
            Some(rights) => rights.iter().filter(|right| **right).count(),
            None => {
                let _bars = cost_scope(Some(WAY_BARS), None);
                let first_owned: Vec<Case> = self.first.iter().map(|at| self.lessons.cases[*at].clone()).collect();
                let first_labels = picked_labels(self.lessons.labels, &self.first);
                worded_model(self.library, &self.lessons.part(&first_owned, &first_labels)).map_or(usize::default(), |model| {
                    let right = right_of(&model, &self.last_cases, self.lanes);
                    if model.reading() == Some(Reading::Mixed) { right.saturating_sub(1) } else { right }
                })
            }
        })
    }

    fn alone(&self) -> usize {
        *self.alone.get_or_init(|| {
            let _bars = cost_scope(Some(WAY_BARS), None);
            alone_right(self.lessons, &self.first, &self.last_cases, self.lanes)
        })
    }
}

fn worded_beyond(bars: &KindBars) -> Option<WordedModel> {
    let written = bars.written_rights()?;
    let read = written.iter().filter(|right| **right).count();
    if read * WHOLE_OF_LAST >= bars.last_cases.len() * WRITTEN_READS_PERCENT {
        learning_note(format!("the written texts read {read} of {} last lessons, at the bar or above it", bars.last_cases.len()));
        return None;
    }
    if let Some(place) = bars.plain.and_then(ExplainModel::first_topic) {
        let first = bars.first_cases();
        if let Some(model) = bridged_written(bars.library, place, bars.lessons, (&first, &bars.last_cases), read).or_else(|| facts_beside_written(bars.library, (&first, &bars.last_cases), read)) {
            return Some(model);
        }
    }
    learning_note(format!("the written texts read {read} of {} last lessons, below the bar, so the learned wording was searched beside them without the rule step and the told side", bars.last_cases.len()));
    let first_owned: Vec<Case> = bars.first.iter().map(|at| bars.lessons.cases[*at].clone()).collect();
    let first_labels = picked_labels(bars.lessons.labels, &bars.first);
    let learned = case_rights(&worded_search(bars.library, &bars.lessons.part(&first_owned, &first_labels), false)?, &bars.last_cases, bars.lanes);
    if learned.iter().zip(written).any(|(learned, written)| *written && !*learned) || learned == *written {
        return None;
    }
    worded_search(bars.library, bars.lessons, false)
}
because!(worded_beyond, OneReading, "the question forms learned for a text of rules, with no told side, in place of the written topics that fit a kind, when the written topics answer fewer of its last lessons than a learned kind must (first the written topic that reads the kind best with the bridges of the invented states of the kind and the kinds before it, or else with the facts of knowledge that it lacks and the lessons show, when they answer more of its last lessons) and the learned forms, learned from the first lessons, answer every last lesson that the written topics answer and more, then learned again from all lessons: a learned reading that loses a lesson that the written one reads may read the lessons by their defaults and not the told text; the learning notes that the search ran without the rule step and the told side");

const WRITTEN_READS_PERCENT: usize = 95;
because!(WRITTEN_READS_PERCENT, OneReading, "the share of the last lessons from which the written topics that fit a kind read it as a learned kind must, the line of the board: below it the learned wording of a text of rules is tried beside them, and above it the search for a wording would cost the board for a few lessons more");

fn alone_right(lessons: &Lessons, first: &[usize], last_cases: &[Case], lanes: usize) -> usize {
    let first_owned: Vec<Case> = first.iter().map(|at| lessons.cases[*at].clone()).collect();
    let alone = learn_question(Lessons::of(&first_owned), lanes).map_or(usize::default(), |model| right_of(model.as_ref(), last_cases, lanes));
    let mut counts: HashMap<Vec<String>, usize> = HashMap::new();
    for case in &first_owned {
        *counts.entry(answer_words(&case.answer)).or_default() += 1;
    }
    let one = counts.into_iter().max_by(|one, other| one.1.cmp(&other.1).then_with(|| other.0.cmp(&one.0))).map_or(usize::default(), |(answer, _)| last_cases.iter().filter(|case| answer_words(&case.answer) == answer).count());
    alone.max(one)
}
because!(alone_right, OneReading, "the last lessons that the question alone answers right, by plain counting of the question or by the one most common answer of the first lessons, whichever answers more: a learned list that does no better reads nothing in the told text, and the board counts such a kind as not learned");

fn case_rights(model: &dyn Learner, cases: &[Case], lanes: usize) -> Vec<bool> {
    on_spare_lanes(cases.len(), lanes, &|place| model.answer(&cases[place].sentences, &cases[place].question).words == answer_words(&cases[place].answer))
}
because!(case_rights, OneReading, "for each case whether a model answers it with exactly the words of the true answer, the cases read on the threads, so that a choice made for each question of a kind reads each case once");

fn numbers_text(library: &Library) -> Option<&Topic> {
    library.topics.iter().find(|topic| {
        topic.questions.is_empty()
            && topic.rules.iter().any(|rule| {
                let fixed: Vec<&str> = rule.outcome.iter().filter_map(|logic_piece| if let LogicPiece::Word(word) = logic_piece { Some(word_text(*word)) } else { None }).collect();
                fixed == NUMBER_FACT_WORDS
            })
    })
}
because!(numbers_text, OneReading, "the text of the library that tells the numbers of a story in their order, found by the fact that its rules make");

fn label_judged(bars: &KindBars) -> Option<WayJudged> {
    let (right, percent) = word_topics(&bars.first_cases(), WORD_RIGHT_PERCENTS).into_iter().zip(WORD_RIGHT_PERCENTS).filter_map(|(topic, percent)| Some((right_of(&model_of(vec![topic?]), &bars.last_cases, bars.lanes), *percent))).max_by(|one, other| one.0.cmp(&other.0).then_with(|| one.1.cmp(&other.1)))?;
    Some(WayJudged { right, again: LearnedAgain::Labels(percent) })
}
because!(label_judged, OneReading, "for a kind whose one question asks for a label of the told text, the decision list over the words of the told sentences (the learned wording over a text with no rules), learned from the first lessons at the trust in a word that answers the most of the last lessons");

fn reckoned_judged(bars: &KindBars) -> Option<WayJudged> {
    let numbers = numbers_text(bars.library)?;
    let (right, percent) = reckoned_topics(numbers, &bars.first_cases(), WORD_RIGHT_PERCENTS).into_iter().zip(WORD_RIGHT_PERCENTS).filter_map(|(topic, percent)| Some((right_of(&model_of(vec![topic?]), &bars.last_cases, bars.lanes), *percent))).max_by(|one, other| one.0.cmp(&other.0).then_with(|| one.1.cmp(&other.1)))?;
    Some(WayJudged { right, again: LearnedAgain::Reckoned(percent) })
}
because!(reckoned_judged, OneReading, "for a kind of word problems, the learned ways to reckon over the numbers of a story (the learned wording's list of words and ways to reckon), with the text that tells the numbers, learned from the first lessons at the trust that answers the most of the last lessons");

fn meant_judged(bars: &KindBars) -> Option<WayJudged> {
    let meant = meanings_topic(&bars.library.topics, &bars.first_cases())?;
    Some(WayJudged { right: right_of(&model_of(vec![meant]), &bars.last_cases, bars.lanes), again: LearnedAgain::Meant })
}
because!(meant_judged, OneReading, "for a kind whose questions are function words over lists, the meanings of its function words learned over the text of functions on lists (meanings), learned from the first lessons");

fn joined_judged(bars: &KindBars) -> Option<WayJudged> {
    if bars.lessons.cases.iter().any(|case| !case.sentences.is_empty()) {
        return None;
    }
    let first_cases: Vec<Case> = bars.first.iter().map(|at| bars.lessons.cases[*at].clone()).collect();
    let joined = learn_joining(&first_cases).ok()?;
    Some(WayJudged { right: right_of(&joined.model, &bars.last_cases, bars.lanes), again: LearnedAgain::Joined })
}
fn rewrite_judged(bars: &KindBars) -> Option<WayJudged> {
    let first = bars.first_cases();
    if !rewrite_kind(&first) {
        return None;
    }
    let model = RewriteModel::learn(&first)?;
    Some(WayJudged { right: right_of(&model, &bars.last_cases, bars.lanes), again: LearnedAgain::Rewritten })
}
because!(rewrite_judged, OneReading, "for a kind of sentences written again in another build (rewrite_kind), the learned rewrites of its units, learned from the first lessons");

because!(joined_judged, OneReading, "for a kind of commands with no told sentences, the ways of joining of its language learned from its lessons (joining), learned from the first lessons");
