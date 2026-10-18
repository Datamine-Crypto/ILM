use std::borrow::Cow;
use crate::base::first_sight::NumberTable;
use crate::base::lanes::{hardware_lanes, on_lanes};
use crate::base::words::answer_words;
use crate::cases::Case;
use crate::choice::inner_parts;
use crate::tools::call::{AnswerMark, Learner, Lessons, Reading, Said};
use crate::tools::explain::marks::{SeenFrames, answer_mark};
use crate::tools::explain::known_good::{source_kind, store_key, stored_groups, stored_lines};
use crate::tools::explain::costs::{constants_left_out, WAY_ALONE, WAY_ANSWERED, WAY_BRIDGES, WAY_FACTS, WAY_TAUGHT, WAY_FINAL, WAY_FORMS, WAY_PROMISE, WAY_RULES, WAY_TOLD, cost_scope, learning_note, search_ended};
use crate::base::lanes::{MOST_LEARNING_WORK, Worked, add_scope_work, in_work_scope, in_work_scope_offering, learned_within, note_store_taken, scope_spent, with_work_scope, work_left};
use crate::tools::explain::story_disk::{LEARNED_RULES, LEARNED_TOLD, kept_learning};
use crate::tools::explain::chain::{adds, asks, rule_text, chain_words_once, reads_none, reads_question, reply_masks, reply_patterns, reply_read_bound, chain_facts_once, Facts, Filling, chain_facts, chain_facts_within, empty_filling, fill, fit_into, holds_words, idle_sentences, keeps_history, knowledge_base, reply, rules_key, told_with_question};
use crate::tools::explain::curriculum::{asked_line, taught_before};
use crate::tools::explain::states::{StateCandidate, event_word, person_state, state_candidates, told_forms_below_bar};
use crate::tools::explain::scopes::{known_scope, merged_rules, shape_values};
use crate::tools::explain::abduction::{STATED_HOLDS, abduced_facts, form_answers, one_step_lessons, statement_lessons, word_ids, capital_words};
use crate::tools::explain::bridges::{MOST_BRIDGE_PLACES, kept_splits, orders};
use crate::tools::explain::wording_patterns::{answers_numbered, patterns_told, question_forms, told_patterns, worked_conditions};
use crate::tools::explain::why::missing_step;
use crate::tools::explain::learn::{Library, hand_rules, library, proof_text};
use crate::tools::explain::provenance::{answer_steps, used_facts, used_rules};
use crate::tools::explain::derive::{answer_proof, proof_lines};
use crate::tools::explain::discovery::{Judged, RuleSearch, SearchStep, WrongLesson, progress, STOP_ALL_RIGHT, STOP_NO_GAIN, STOP_NO_RULE, STOP_RAN_AWAY, WAY_ALIGNED, WAY_FORMED, WAY_LABEL, WAY_REPLY, WAY_SPOKEN, FormJoin};
use crate::tools::explain::text::{PartReading, HOLDS_WORDS, NOT_START, filled_form, plain_piece, plain_filled, question_form, quoted_answers, ANY_FIRST, ANY_PLACES, GROUPED_FIRST, MARKED_FIRST, ONE_SIGN_FIRST, OPEN_PLACES, PLAIN_PLACES, QUESTION_TOLD, WORD_ENDINGS, AskedForm, Condition, Reply, LogicPiece, Moment, Pattern, ReplyLine, ToldRule, Topic, fact_numbers, fact_words, place_letter, read_topic, text_words, word_not_mark, LineKind};
use crate::tools::explain::words::{WordId, word_number, word_numbers, word_text, word_texts};
use crate::base::words::first_capital;
use patterns::{because, source};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError, Weak};
use std::time::Instant;

pub struct LearnedWording;
source!(LearnedWording, "the wording of a set learned from its lessons in place of a text that a person writes: each question becomes a form whose open places stand for the runs of words that change from lesson to lesson, and each form gets answers whose conditions are lines of the library made from the facts that the rules of a text make (one fact, a fact compared with a word of the question, or two facts compared), or, for a kind whose one question asks for a label of the told text (the intent of a request), from the words that a told sentence holds, each read by the reader of the library and judged by the engine on the lessons, chosen one after another by the lessons that they answer right, as a decision list is learned (Rivest); an answer that holds otherwise comes last, only after an answer with a condition and only when the lessons that no condition covers agree on it (for a list over the words of the told text, the most common answer of those lessons), so that no answer comes from the question alone");

pub static SEARCH_PATHS: AtomicBool = AtomicBool::new(false);
because!(SEARCH_PATHS, LearnedWording, "whether the search for a learned wording says on the error output which of its branches ran for a kind and how long each took (the plain wording, the rules of a concept for each text, the told side for each text, the wording learned again); off unless a command asks, so that a run pays nothing for it");

fn path_note(lessons: &Lessons, what: &str, started: Instant) {
    if SEARCH_PATHS.load(Ordering::Relaxed) {
        let asked = lessons.cases.first().map_or("", |case| case.question.as_str());
        eprintln!("{PATH_MARK} {asked} | {} lessons | {what} | {:.1} s", lessons.cases.len(), started.elapsed().as_secs_f64());
    }
}
because!(path_note, LearnedWording, "one line of the branches of the search for a learned wording: the first question of the lessons, which names the kind, how many lessons the search learns from (the first part inside the choice of the tool, all of them, or a few), the branch with what it gave, and its seconds");

const PATH_LINES_GAP: &str = " / ";
because!(PATH_LINES_GAP, LearnedWording, "the mark between the learned lines that a line of the branches of the search lists");

const PATH_MARK: &str = "search path:";
because!(PATH_MARK, LearnedWording, "the start of a line of the branches of the search, by which a reader finds them among other lines of the error output");

const FIXED_SHARE_PERCENT: usize = 4;
because!(FIXED_SHARE_PERCENT, LearnedWording, "a word of a question that the story does not hold is a fixed word of its form when it stands in at least this share of the questions, as the words of a question frame come again and again while names and numbers change");

const TOLD_FIXED_SHARES: &[usize] = &[FIXED_SHARE_PERCENT, 30];
because!(TOLD_FIXED_SHARES, LearnedWording, "the two readings of a word of a question that the story holds too: fixed at the same share as any word (a verb of the frame that the story tells, such as reached), or only at a far larger share (a common noun of the frame, such as class, while the names of classes change); the forms of both readings are learned, and a question takes the learned form with the most lessons that fits it");

const REWRITTEN_FIXED_PERCENT: usize = 50;
because!(REWRITTEN_FIXED_PERCENT, LearnedWording, "a word of a told sentence that a line of the text wrote again is a fixed word of the form of that sentence when it stands in at least this share of the lessons whose sentences were written again: the words that the line leaves come again, while the values change");

const MOST_REWRITTEN_FORMS: usize = 16;
because!(MOST_REWRITTEN_FORMS, LearnedWording, "the most forms of told sentences written again that become sources of conditions, the most common first, so that the judging of conditions stays small");

const MOST_FORM_PLACES: usize = 4;
because!(MOST_FORM_PLACES, LearnedWording, "a learned form has at most this many open places: the forms of a frame name a few things and values, while a question cut into many runs of rare words has no frame, and every way to give the places of a fact to the places of such a form is too many to try");

pub(super) const WHOLE_SHARE: usize = 100;
because!(WHOLE_SHARE, LearnedWording, "the count of a share in hundredths, so that a count of questions compares with a percent in whole numbers");

const MOST_LEARNED_ANSWERS: usize = 3;
because!(MOST_LEARNED_ANSWERS, LearnedWording, "a learned form keeps few answers, as the written forms of the library do, so that each answer rests on many lessons and a long list cannot learn the lessons one by one");

pub(super) const LEAST_SUPPORT: usize = 3;
because!(LEAST_SUPPORT, LearnedWording, "the fewest lessons on which an answer must hold, so that an answer rests on more than a chance case");

const LEAST_RIGHT_PERCENT: usize = 90;
because!(LEAST_RIGHT_PERCENT, LearnedWording, "the share of the lessons where its condition holds that an answer must answer right");

const BROAD_PERCENT: usize = 50;
because!(BROAD_PERCENT, LearnedWording, "a fact with a free value is compared with other values only when it holds on at least this share of the lessons of a form, so that the search for comparisons stays small and starts from facts that the stories tell about the things of the question");

const FORM_SAMPLE: usize = 50;
because!(FORM_SAMPLE, LearnedWording, "the lessons of a form on which its candidate conditions are judged, as many as the sample on which the engine keeps a written text, so that the search stays small while the share of a learned form is still counted on all its lessons");

const SCREEN_SAMPLE: usize = 10;
because!(SCREEN_SAMPLE, LearnedWording, "the first lessons of a form on which a candidate condition must hold at least once to be judged at all, as most candidates of a text that does not read the kind hold on none, and on whose misses a second fact must give the answer once");

const COMPARING: &[&str] = &["is more than", "is less than", "is at least", "is at most"];
because!(COMPARING, LearnedWording, "the comparisons of two numbers that a learned condition may make, in the words of the engine");

const PROBE: &str = "probe";
because!(PROBE, LearnedWording, "the answer and the name of the one-line text that reads a candidate condition: only whether its condition holds counts");

pub(super) const STATEMENT_END: &str = ".";
because!(STATEMENT_END, LearnedWording, "the mark that closes a learned form of a statement, as a told sentence ends");

pub(super) const QUESTION_END: &str = "?";
because!(QUESTION_END, LearnedWording, "the mark that closes a learned form of a question");

pub(super) const FORM_WORD_GAP: &str = " ";
because!(FORM_WORD_GAP, LearnedWording, "the space between the words of a written form");

pub(super) const CONDITIONS_JOINED: &str = " and ";
because!(CONDITIONS_JOINED, LearnedWording, "the word between the parts of a learned condition, as in a line of the library");

const NO_SUCH_TEXT: &str = "no text of the library has the name";
because!(NO_SUCH_TEXT, LearnedWording, "what the learning says when the named text is not in the library");

const LINE_START: &str = "To answer \"";
because!(LINE_START, LearnedWording, "the words that open a written question form of the library");

const LINE_MIDDLE: &str = "\": ";
because!(LINE_MIDDLE, LearnedWording, "the marks between the question of a written form and its answers");

const LEARNED_IF: &str = " if ";
because!(LEARNED_IF, LearnedWording, "the word between an answer and its condition in a written form");

const LEARNED_OTHERWISE: &str = " otherwise";
because!(LEARNED_OTHERWISE, LearnedWording, "the word after an answer that holds when no answer before it holds, as a written form ends it");

pub struct LearnedForm {
    pub question: Pattern,
    pub closed_as_question: bool,
    pub answers: Vec<(Vec<String>, String)>,
    pub lessons: usize,
    pub right: usize,
    pub otherwise: Option<Vec<String>>,
    pub(super) taught: Vec<usize>,
    pub(super) probes: Vec<AskedForm>,
    pub(super) fills: Vec<bool>,
}
because!(LearnedForm, LearnedWording, "one learned form: the question with its open places, whether its questions end with a question mark, its answers in order (the words of the answer and its condition as a line of the library writes it), the answer that holds when none of them holds, if one was learned, the lessons of the form with those that it answers right, which lessons they are, each answer read by the reader of the library, and for each answer whether its words are a place that its condition fills from the facts of the story (the book that a person ends with) and not a label");

pub struct WordingReport {
    pub told: Vec<String>,
    pub forms: Vec<LearnedForm>,
    pub checked: usize,
    pub learned_right: usize,
    pub written_right: usize,
    pub seconds: (f64, f64),
    pub below_bar: Vec<(String, usize, usize)>,
}
because!(WordingReport, LearnedWording, "what the learning of a wording gives (with the seconds of the searches for rules and for readings of a curriculum, apart): the learned rules that read the told sentences, if asked for, the learned forms, and on the lessons that it did not learn from, how many cases the learned forms and the written forms of the text answer right; for a curriculum, the told sentences that too few of the lessons of its search for states hold for a state to read them");

pub(super) struct QuestionWords {
    shares: HashMap<String, usize>,
    questions: usize,
    text: Option<TextWords>,
    slot: HashSet<String>,
    freed: HashSet<String>,
}
because!(QuestionWords, LearnedWording, "how many questions of the first lessons hold each word, and how many questions there are, with the words of the text that the forms were learned for, the words that their places in the questions fix and those that their places leave open, so that a question is cut as the forms were cut");

fn question_words(cases: &[&Case]) -> QuestionWords {
    let mut shares: HashMap<String, usize> = HashMap::new();
    for case in cases {
        let words: HashSet<String> = fact_words(&case.question).into_iter().collect();
        for word in words {
            *shares.entry(word).or_default() += 1;
        }
    }
    QuestionWords { shares, questions: cases.len(), text: None, slot: HashSet::new(), freed: HashSet::new() }
}
because!(question_words, LearnedWording, "the words of the questions of the first lessons with the number of questions that hold each");

struct QuestionSlots {
    fillers: HashMap<(String, String), HashSet<String>>,
    stands: HashMap<String, Vec<(String, String)>>,
    told_too: HashSet<String>,
}
because!(QuestionSlots, LearnedWording, "the places of the words of the questions of some lessons: the words that stand between each pair of neighbours, the neighbours of each use of each word, and the words that the story of a lesson tells beside its question");

fn question_slots(cases: &[&Case]) -> QuestionSlots {
    let mut slots = QuestionSlots { fillers: HashMap::new(), stands: HashMap::new(), told_too: HashSet::new() };
    for case in cases {
        let question = fact_words(&case.question);
        let told: HashSet<String> = case.sentences.iter().flat_map(|sentence| fact_words(sentence)).collect();
        for (at, word) in question.iter().enumerate() {
            let before = at.checked_sub(1).map_or_else(String::new, |earlier| context_word(&question[earlier]));
            let after = question.get(at + 1).map_or_else(String::new, |later| context_word(later));
            slots.fillers.entry((before.clone(), after.clone())).or_default().insert(word.clone());
            slots.stands.entry(word.clone()).or_default().push((before, after));
            if told.contains(word) {
                slots.told_too.insert(word.clone());
            }
        }
    }
    slots
}
because!(question_slots, LearnedWording, "where the words of the questions of some lessons stand: between which neighbours, which words share each pair of neighbours, and which words the story of a lesson tells beside its question");

fn few_fillers(slots: &QuestionSlots, places: &[(String, String)]) -> usize {
    places.iter().filter(|context| slots.fillers.get(*context).map_or(0, HashSet::len) <= MOST_FRAME_FILLERS).count()
}
because!(few_fillers, LearnedWording, "how many uses of a word stand between neighbours that few words share");

fn placed_words(cases: &[&Case], words: &QuestionWords) -> (HashSet<String>, HashSet<String>) {
    let slots = question_slots(cases);
    let mut slot: HashSet<String> = HashSet::new();
    let mut freed: HashSet<String> = HashSet::new();
    for (word, places) in &slots.stands {
        if !word_not_mark(word) || is_value(word) || places.len() < LEAST_SUPPORT {
            continue;
        }
        let common = words.shares.get(word).copied().unwrap_or_default() * WHOLE_SHARE >= words.questions * FIXED_SHARE_PERCENT;
        let few = few_fillers(&slots, places);
        if !common && !slots.told_too.contains(word) && few * WHOLE_SHARE >= places.len() * FRAME_PLACE_PERCENT {
            slot.insert(word.clone());
        } else if common && (places.len() - few) * WHOLE_SHARE >= places.len() * FRAME_PLACE_PERCENT {
            freed.insert(word.clone());
        }
    }
    (slot, freed)
}
because!(placed_words, LearnedWording, "the words of the questions that their places fix or leave open, against their shares: fixed, a word too rare for a share that no story of its lessons tells, in enough questions, that stands between neighbours that few words share, as one of a few choices of a frame (atleast or atmost between has and a number; a frame that only some questions of a kind use splits the share of its words among its choices); open, a word common enough for a share that stands between neighbours that many words share, as one value among many (a colour among fifteen between are and in), so that one form reads every value of the frame and not one form for each value");

fn untold_frame(cases: &[&Case]) -> HashSet<WordId> {
    let slots = question_slots(cases);
    slots
        .stands
        .iter()
        .filter(|(word, places)| word_not_mark(word) && !is_value(word) && places.len() >= LEAST_SUPPORT && !slots.told_too.contains(*word) && few_fillers(&slots, places) * WHOLE_SHARE >= places.len() * FRAME_PLACE_PERCENT)
        .map(|(word, _)| word_number(word))
        .collect()
}
because!(untold_frame, LearnedWording, "the words of the frame of the questions that no story of their lessons tells, whatever their share: each stands between neighbours that few words share, as a word of one of several question forms of a kind (didn't, did, not), while a value of the stories (australia, garden) is told");

struct TextWords {
    named: HashSet<String>,
    before_places: HashSet<String>,
}
because!(TextWords, LearnedWording, "the words that the rules of a text name in their conditions and outcomes, the words of the relations that the text reads, and the words that stand right before an open place in a rule that may start on the stories of a kind, which those rules read apart from the thing after them");

#[derive(Clone, Copy)]
enum Cut<'a> {
    Share(usize),
    Named(&'a TextWords, bool),
    Slots,
    Values,
}
because!(Cut, LearnedWording, "a reading of the words of a question that the story holds too: fixed when common enough in the questions, at one of the shares, or fixed only when the rules of the text name them, with or without the word before a run joined to it, as the rules name the words of a relation (left, gave, afraid) and never the names of the things that a story tells, however few such things a kind has and however often its questions name each; or, in the reading by places, fixed as at the first share and also when their places fix them, with each number a place of its own, so that a number of the question can be compared with a number of the story; or, in the reading by values, fixed as at the first share unless their places leave them open, as a colour among many");

fn rule_words(topic: &Topic, cases: &[&Case]) -> TextWords {
    rule_words_told(topic, &first_told(cases))
}

fn first_told(cases: &[&Case]) -> Vec<Vec<WordId>> {
    cases.iter().take(RELEVANCE_SAMPLE).flat_map(|case| case.sentences.iter().map(|sentence| fact_numbers(sentence))).collect()
}
because!(first_told, LearnedWording, "the told sentences of the first lessons, read into words, in which rule_words looks for the rules that start on a story");

fn rule_words_told(topic: &Topic, told: &[Vec<WordId>]) -> TextWords {
    let mut out = TextWords { named: HashSet::new(), before_places: HashSet::new() };
    for rule in &topic.rules {
        let read: Vec<&Pattern> = rule.conditions.iter().chain(&rule.absent).filter_map(|condition| match condition {
            Condition::Fact { pattern, .. } | Condition::Count { pattern, .. } => Some(pattern),
            _ => None,
        }).collect();
        let starts = read.iter().any(|pattern| told.iter().any(|sentence| holds_words(pattern, sentence)));
        for pattern in read.into_iter().chain([&rule.outcome]) {
            for (at, logic_piece) in pattern.iter().enumerate() {
                let LogicPiece::Word(word) = logic_piece else { continue };
                out.named.insert(word_text(*word).to_string());
                if starts && matches!(pattern.get(at + 1), Some(LogicPiece::Open(_) | LogicPiece::OpenWithEnd(..))) {
                    out.before_places.insert(word_text(*word).to_string());
                }
            }
        }
    }
    out
}
because!(rule_words_told, LearnedWording, "rule_words over the told sentences of the first lessons already read into words");
because!(rule_words, LearnedWording, "the words that the rules of a text name, and those that stand right before an open place in a rule with a condition whose fixed words some told sentence of the first lessons holds in order, as a rule that cannot start on the stories of a kind says nothing of how they name their things");

fn cuts<'a>(text: &'a TextWords, words: &QuestionWords) -> Vec<Cut<'a>> {
    let by_places = (!words.slot.is_empty()).then_some(Cut::Slots);
    let by_values = (!words.freed.is_empty()).then_some(Cut::Values);
    TOLD_FIXED_SHARES.iter().map(|told_least| Cut::Share(*told_least)).chain([Cut::Named(text, false), Cut::Named(text, true)]).chain(by_places).chain(by_values).collect()
}
because!(cuts, LearnedWording, "the readings of the words of the questions for a text: each share, then the words that its rules name, first as they stand and then with the word before a run joined to it, then the reading by places when some words of the questions are fixed by their places alone, and last the reading by values when some common words are left open by their places");

fn read_by_share(cut: Cut) -> bool {
    matches!(cut, Cut::Share(_) | Cut::Slots | Cut::Values)
}
because!(read_by_share, LearnedWording, "whether a reading fixes the words of the questions by their shares, as the readings by places and by values do too, and not by the words of the rules of the text: only then may a filled answer name a fixed word that the stories tell");

fn fixed_in(word: &str, told: &HashSet<String>, words: &QuestionWords, cut: Cut) -> bool {
    let common = |least: usize| words.shares.get(word).copied().unwrap_or_default() * WHOLE_SHARE >= words.questions * least;
    !word_not_mark(word)
        || (!is_value(word)
            && match cut {
                Cut::Share(told_least) => common(if told.contains(word) { told_least } else { FIXED_SHARE_PERCENT }),
                Cut::Named(text, _) if told.contains(word) => text.named.contains(word),
                Cut::Named(..) => common(FIXED_SHARE_PERCENT),
                Cut::Slots => words.slot.contains(word) || common(FIXED_SHARE_PERCENT),
                Cut::Values => !words.freed.contains(word) && common(FIXED_SHARE_PERCENT),
            })
}
because!(fixed_in, LearnedWording, "whether a word of a question is a fixed word of its form: a mark always, a number never, as a number is a value in a told sentence too, a word of the story only when it is common enough in the questions for the reading at hand or, in the reading by the text, when its rules name it, and any other word when it is common, or in the reading by places when its places fix it, and not in the reading by values when its places leave it open");

pub(super) struct SaidCase {
    pub(super) sentences: Vec<Vec<String>>,
    told: HashSet<String>,
    written: Vec<String>,
    pub(super) question: Vec<String>,
}
because!(SaidCase, LearnedWording, "the words of a lesson that the form of its question reads: the words of each told sentence and the set of them, and the question as it is written and as the rules see it; found once for lessons on which many readings are judged, as no reading changes them");

pub(super) fn said_case(case: &Case) -> SaidCase {
    let sentences: Vec<Vec<String>> = case.sentences.iter().map(|sentence| fact_words(sentence)).collect();
    let told: HashSet<String> = sentences.iter().flatten().cloned().collect();
    SaidCase { sentences, told, written: text_words(&case.question), question: fact_words(&case.question) }
}
because!(said_case, LearnedWording, "the words of one lesson that the form of its question reads");

pub(crate) fn one_question(cases: &[&Case]) -> Option<Vec<String>> {
    let first = fact_words(&cases.first()?.question);
    cases.iter().all(|case| fact_words(&case.question) == first).then_some(first)
}
because!(one_question, LearnedWording, "the words of the one question of a kind, when every lesson asks the same, and nothing for a kind whose questions differ");

#[derive(Clone, Copy)]
pub(super) struct ListShape {
    pub(super) most: usize,
    pub(super) right_percent: usize,
    pub(super) agreed_otherwise: bool,
}
because!(ListShape, LearnedWording, "how long a decision list of one form may grow, the share of the lessons where a condition holds that it must answer right, and whether the answer otherwise needs the lessons left to agree on it as a learned answer must (and a condition of the label that would be the answer otherwise is passed over), or is the most common answer of those lessons");

const FACT_LIST: ListShape = ListShape { most: MOST_LEARNED_ANSWERS, right_percent: LEAST_RIGHT_PERCENT, agreed_otherwise: true };
because!(FACT_LIST, LearnedWording, "the list of a form over the facts of a text: a few answers, each right on nearly all the lessons where its condition holds, and an answer otherwise only when the lessons left agree on it");

fn form_of(case: &Case, words: &QuestionWords, cut: Cut) -> Option<Pattern> {
    form_in(&said_case(case), words, cut)
}
because!(form_of, LearnedWording, "the form of the question of a lesson whose words are not found yet");

fn form_in(said: &SaidCase, words: &QuestionWords, cut: Cut) -> Option<Pattern> {
    let SaidCase { sentences, told, written, question } = said;
    let mut pattern: Pattern = Vec::new();
    let mut places = 0usize;
    for (at, word) in question.iter().enumerate() {
        let named = at > usize::default()
            && written.get(at).is_some_and(|text| text.chars().next().is_some_and(char::is_uppercase) && text.chars().nth(1).is_some())
            && written.get(at - 1).is_some_and(|before| !SENTENCE_STARTS.contains(&before.as_str()))
            && told.contains(word);
        if !named && fixed_in(word, &told, words, cut) {
            pattern.push(LogicPiece::Word(word_number(word)));
            continue;
        }
        let number_beside = matches!(cut, Cut::Slots) && at > usize::default() && (is_value(word) || is_value(&question[at - 1]));
        if let Some(LogicPiece::Open(_)) = pattern.last()
            && !number_beside
        {
            continue;
        }
        if places == MOST_FORM_PLACES {
            return None;
        }
        let before = at.checked_sub(1).map(|earlier| &question[earlier]);
        if let Cut::Named(text, true) = cut
            && before.is_some_and(|before| word_not_mark(before) && !text.before_places.contains(before) && sentences.iter().any(|sentence| sentence.windows(WORD_PAIR).any(|pair| pair[0] == *before && pair[1] == *word)))
        {
            pattern.pop();
        }
        pattern.push(LogicPiece::Open(places));
        places += 1;
    }
    Some(pattern)
}
because!(form_in, LearnedWording, "the form of a question: fixed words stay, and each run of other words becomes one open place, also a word of the story that the question writes with a capital inside a sentence, as a name is written, however common it is in the questions (three friends asked about by turns); in the second reading by the text, a fixed word right before a run joins the run when a told sentence holds it right before the first word of the run and no rule of the text that can start on the stories holds it right before an open place, as the story names a thing with the word before it (the red sphere) and rules that never read that word apart from a thing read the thing with it; in the reading by places, a number is a place of its own and not part of a run of words beside it, as a comparison reads the number alone; nothing for a question with more runs than a learned form may have");

const WORD_PAIR: usize = 2;
because!(WORD_PAIR, LearnedWording, "a word and the word right after it in a told sentence, as a story names a thing with the word before it");

const SENTENCE_STARTS: &[&str] = &[".", "?", "!", ":"];

const NAMED_PERCENT: usize = 50;
because!(NAMED_PERCENT, LearnedWording, "the least share of the uses of a word inside a sentence where it is written with a capital, for the word to be a name: a common word has a capital inside a sentence only by chance, as in a title");
because!(SENTENCE_STARTS, LearnedWording, "the marks after which a word starts a sentence and has a capital for that alone, not because it is a name");

pub(super) fn places_of(pattern: &Pattern) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for logic_piece in pattern {
        if let LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) = logic_piece
            && !out.contains(place)
        {
            out.push(*place);
        }
    }
    out
}
because!(places_of, LearnedWording, "the open places of a pattern in the order in which they first stand");

pub(super) fn renamed(pattern: &Pattern, from: &[usize], to: &[usize]) -> Pattern {
    let new_place = |place: &usize| from.iter().position(|old| old == place).map_or(*place, |at| to[at]);
    pattern
        .iter()
        .map(|logic_piece| match logic_piece {
            LogicPiece::Open(place) => LogicPiece::Open(new_place(place)),
            LogicPiece::OpenWithEnd(place, ending) => LogicPiece::OpenWithEnd(new_place(place), ending.clone()),
            LogicPiece::Word(word) => LogicPiece::Word(word.clone()),
        })
        .collect()
}
because!(renamed, LearnedWording, "a pattern with its open places renamed to places of a form, so that a fact that a rule makes can be asked with the words of a question");

pub(super) fn fixed_count(pattern: &Pattern) -> usize {
    pattern.iter().filter(|logic_piece| matches!(logic_piece, LogicPiece::Word(_))).count()
}
because!(fixed_count, LearnedWording, "how many fixed words a form has: of two forms that answer the same lessons, the one with more fixed words says more of the question");

fn place_choices(count: usize, places: usize, free: usize) -> Vec<Vec<usize>> {
    if count == 0 {
        return vec![Vec::new()];
    }
    let mut out: Vec<Vec<usize>> = Vec::new();
    for shorter in place_choices(count - 1, places, free) {
        for place in (0..places).chain([free]) {
            if !shorter.contains(&place) {
                let mut longer = shorter.clone();
                longer.push(place);
                out.push(longer);
            }
        }
    }
    out
}
because!(place_choices, LearnedWording, "every way to give each of a number of places of a pattern its own place of a form, or the one free place, which the question does not fill, so that a condition may name someone or a value that the question does not name");

pub(crate) fn library_writing(pattern: &Pattern) -> String {
    pattern
        .iter()
        .enumerate()
        .map(|(at, logic_piece)| match logic_piece {
            LogicPiece::Word(word) if JOINING_WORDS.contains(&word_text(*word)) || (at == 0 && word_text(*word).chars().nth(1).is_some()) => first_capital(word_text(*word)),
            LogicPiece::Word(word) => word_text(*word).to_string(),
            LogicPiece::Open(place) => place_letter(*place),
            LogicPiece::OpenWithEnd(place, ending) => format!("{}{ending}", place_letter(*place)),
        })
        .collect::<Vec<String>>()
        .join(FORM_WORD_GAP)
}
because!(library_writing, LearnedWording, "a pattern as a text of the library writes it: each open place as its letter, the word and with a capital (a capital And stays inside one condition, as the library writes a list in a rule), and the first word with a capital when it has more than one letter, so that a fact that starts with not or latest is read as a fact and not as a word of the engine; a word of one letter keeps its small letter, as a capital one would be an open place");

const JOINING_WORDS: &[&str] = &["and", "then"];
because!(JOINING_WORDS, LearnedWording, "the words that the reader of the library takes as borders between two conditions or between the conditions and the outcome when they are written in small letters, so a form writes them with a capital to keep them words");

pub(super) struct WordingCandidate {
    text: String,
    pub(super) probe: Arc<AskedForm>,
    single: Option<Pattern>,
    free: bool,
    key: u64,
}
because!(WordingCandidate, LearnedWording, "a candidate condition of an answer: its text as a line of the library writes it, the question form with that condition as read by the reader of the library, for a condition of one fact with one free value its pattern, from which comparisons are made, and whether it names a place that the question does not fill, through which a second fact may give the answer; and a hash of its text and of the question form, by which its checks are kept");

pub(super) fn probed(form_text: &str, condition: &str, single: Option<Pattern>, free: bool) -> Option<WordingCandidate> {
    let line = format!("{LINE_START}{form_text}{LINE_MIDDLE}{PROBE}{LEARNED_IF}{condition}{STATEMENT_END}");
    let mut hasher = DefaultHasher::new();
    line.hash(&mut hasher);
    let shards = PROBES.get_or_init(|| (0..CHECK_SHARDS).map(|_| Mutex::default()).collect());
    let shard = &shards[hasher.finish() as usize % CHECK_SHARDS];
    let kept = shard.lock().ok().and_then(|kept| kept.get(&line).cloned());
    let (probe, key) = match kept {
        Some(read) => read?,
        None => {
            let read = read_topic(PROBE, &line).ok().and_then(|topic| topic.questions.into_iter().next()).map(|probe| {
                let mut hasher = DefaultHasher::new();
                condition.hash(&mut hasher);
                probe.question.hash(&mut hasher);
                (Arc::new(probe), hasher.finish())
            });
            if let Ok(mut kept) = shard.lock() {
                if kept.len() >= MOST_KEPT_PROBES / CHECK_SHARDS {
                    kept.clear();
                }
                kept.insert(line, read.clone());
            }
            read?
        }
    };
    Some(WordingCandidate { text: condition.to_string(), probe, single, free, key })
}
because!(probed, LearnedWording, "a candidate condition read as the reader of the library reads a written line, so that a learned condition means what a written one means; each line is read once in a run and kept, as the same few thousand lines come again in every judge of a reading");

const MOST_KEPT_PROBES: usize = 1_000_000;
because!(MOST_KEPT_PROBES, LearnedWording, "the most candidate lines kept as read in all parts of their table, about a gigabyte, so that a run that learns wordings for many kinds cannot fill the memory of the machine; a part that is full starts again empty, as a kept line is only a reading that can be made again");

static PROBES: OnceLock<Vec<Mutex<HashMap<String, Option<(Arc<AskedForm>, u64)>>>>> = OnceLock::new();
because!(PROBES, LearnedWording, "the candidate lines of the run as the reader of the library read them, with the key of each, or nothing for a line that it cannot read; the lines come from the forms of the questions and the facts that the rules make, so they stay few (some tens of thousands on a LoNLI kind)");

fn condition_holds(candidate: &WordingCandidate, lesson: &WordingLesson) -> bool {
    let key = (lesson.print, candidate.key);
    let shards = CHECKS.get_or_init(|| (0..CHECK_SHARDS).map(|_| Mutex::default()).collect());
    let shard = &shards[(lesson.print ^ candidate.key) as usize % CHECK_SHARDS];
    if let Some(held) = shard.lock().ok().and_then(|kept| kept.get(&key).copied()) {
        return held;
    }
    let held = reply(std::slice::from_ref(candidate.probe.as_ref()), &lesson.question, &Facts::given_again(&lesson.facts)).is_some();
    if let Ok(mut kept) = shard.lock() {
        if kept.len() >= MOST_KEPT_CHECKS / CHECK_SHARDS {
            kept.clear();
        }
        kept.insert(key, held);
    }
    held
}
because!(condition_holds, LearnedWording, "whether a candidate condition holds for a lesson, as the engine answers a question with the facts of its story, on a new layer over them with its own count of the facts read, so that a check never counts against another and the result of a check depends only on the condition and the lesson; a result is kept by the fingerprint of the lesson and the key of the condition, since a reading of one sentence form changes the facts of few lessons and the same check comes again for the others");

const MOST_KEPT_CHECKS: usize = 64_000_000;
because!(MOST_KEPT_CHECKS, LearnedWording, "the most kept results of checks in all parts of the table, about two gigabytes, so that a long kind cannot fill the memory of the machine; a part that is full starts again empty, as a kept result is only a check that can be made again");

const CHECK_SHARDS: usize = 64;
because!(CHECK_SHARDS, LearnedWording, "the parts of the table of kept checks, each with its own lock, so that the lanes that judge readings side by side seldom wait for each other");

static CHECKS: OnceLock<Vec<Mutex<NumberTable<(u64, u64), bool>>>> = OnceLock::new();
because!(CHECKS, LearnedWording, "the kept results of the condition checks of the run, by the fingerprint of the lesson (its facts with their places in the story and its question) and the key of the condition");

enum KeptRow {
    Held(u64),
    Screened,
}
because!(KeptRow, LearnedWording, "a kept row of a candidate condition on the sample of a form: the lessons where it holds, one bit for each, or the sign that it held on none of the first lessons and was not judged further");

const MOST_KEPT_ROWS: usize = 16_000_000;
because!(MOST_KEPT_ROWS, LearnedWording, "the most kept rows in all parts of their table, about half a gigabyte, so that a long kind cannot fill the memory of the machine; a part that is full starts again empty, as a kept row is only checks that can be made again");

const _: () = assert!(SCREEN_SAMPLE <= FORM_SAMPLE && FORM_SAMPLE < u64::BITS as usize);

static ROWS: OnceLock<Vec<Mutex<NumberTable<(u64, u64), KeptRow>>>> = OnceLock::new();
because!(ROWS, LearnedWording, "the kept rows of the run, by the key of the condition and the fingerprint of the sample of lessons of a form (the fingerprints of its lessons in order)");

fn sample_print(held: &[&WordingLesson]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for lesson in held {
        lesson.print.hash(&mut hasher);
    }
    hasher.finish()
}
because!(sample_print, LearnedWording, "the fingerprint of the sample of lessons of a form, from the fingerprints of its lessons in order, by which the rows of conditions on it are kept: a reading of one sentence form leaves the facts of most samples as they were");

fn candidate_row(candidate: &WordingCandidate, held: &[&WordingLesson], print: u64, screened: bool) -> Option<Vec<bool>> {
    let key = (candidate.key, print);
    let shards = ROWS.get_or_init(|| (0..CHECK_SHARDS).map(|_| Mutex::default()).collect());
    let shard = &shards[(candidate.key ^ print) as usize % CHECK_SHARDS];
    let screen_bits = (1u64 << SCREEN_SAMPLE.min(held.len())) - 1;
    let unpacked = |bits: u64| -> Vec<bool> { (0..held.len()).map(|at| bits >> at & 1 == 1).collect() };
    match shard.lock().ok().as_deref().and_then(|kept| kept.get(&key)) {
        Some(KeptRow::Held(bits)) if !screened || bits & screen_bits != 0 => return Some(unpacked(*bits)),
        Some(_) if screened => return None,
        _ => {}
    }
    let row = if screened && !held.iter().take(SCREEN_SAMPLE).any(|lesson| condition_holds(candidate, lesson)) {
        KeptRow::Screened
    } else {
        KeptRow::Held(held.iter().enumerate().filter(|(_, lesson)| condition_holds(candidate, lesson)).fold(0u64, |bits, (at, _)| bits | 1 << at))
    };
    let out = match row {
        KeptRow::Held(bits) => Some(unpacked(bits)),
        KeptRow::Screened => None,
    };
    if let Ok(mut kept) = shard.lock() {
        if kept.len() >= MOST_KEPT_ROWS / CHECK_SHARDS {
            kept.clear();
        }
        kept.insert(key, row);
    }
    out
}
because!(candidate_row, LearnedWording, "where a candidate condition holds on the sample of lessons of a form, or nothing when it is screened and holds on none of the first lessons; a row is kept by the key of the condition and the fingerprint of the sample, and made again from the kept checks when the sample has changed");

fn with_form_places(pattern: &Pattern, form: &Pattern) -> Pattern {
    let own = places_of(form);
    let mapped = |place: usize| own.get(place).copied().unwrap_or(place);
    pattern
        .iter()
        .map(|logic_piece| match logic_piece {
            LogicPiece::Open(place) => LogicPiece::Open(mapped(*place)),
            LogicPiece::OpenWithEnd(place, ending) => LogicPiece::OpenWithEnd(mapped(*place), ending.clone()),
            LogicPiece::Word(word) => LogicPiece::Word(*word),
        })
        .collect()
}
because!(with_form_places, LearnedWording, "a condition made for the places of a form numbered from the first in order, with each such place written as the place of the form itself, so that a form whose place may hold any signs (a run of a question) gets conditions that name that place; the same pattern for a form of plain places");

fn run_fits(form: &Pattern, question: &[WordId]) -> bool {
    if !form.iter().any(|logic_piece| matches!(logic_piece, LogicPiece::Open(place) if *place >= ANY_FIRST)) {
        return false;
    }
    let mut found = Vec::new();
    fit_into(form, question, &empty_filling(), &mut found);
    !found.is_empty()
}
because!(run_fits, LearnedWording, "whether a question fits a learned run form, by the words of the form and its run of any signs, as the engine fits a question form: a run form is not one that the form of a single question gives, so it is found by fitting");

const MOST_RUN_PLACES: usize = 2;
const _: () = assert!(MOST_RUN_PLACES <= ANY_PLACES);
because!(MOST_RUN_PLACES, LearnedWording, "the most places of any signs of a learned run form: a question holds a stretch of data, or a pair of them to compare (phrases in the order of their words), and each place more multiplies the ways to fit a question and the conditions to judge, so the stretches that the fewest frame words part become one until no more are left");

pub(super) fn common_words(one: &[String], other: &[String]) -> Vec<String> {
    let mut longest = vec![vec![usize::default(); other.len() + 1]; one.len() + 1];
    for at in (0..one.len()).rev() {
        for to in (0..other.len()).rev() {
            longest[at][to] = if one[at] == other[to] { longest[at + 1][to + 1] + 1 } else { longest[at + 1][to].max(longest[at][to + 1]) };
        }
    }
    let mut out: Vec<String> = Vec::new();
    let (mut at, mut to) = (usize::default(), usize::default());
    while at < one.len() && to < other.len() {
        if one[at] == other[to] {
            out.push(one[at].clone());
            at += 1;
            to += 1;
        } else if longest[at + 1][to] >= longest[at][to + 1] {
            at += 1;
        } else {
            to += 1;
        }
    }
    out
}
because!(common_words, LearnedWording, "the longest run of words, in order but not side by side, that two questions both hold, as a table of the longest such runs from each pair of places, the words of the first question kept where two runs are as long");

fn gaps_of(frame: &[String], words: &[String]) -> Vec<usize> {
    let mut gaps: Vec<usize> = Vec::new();
    let mut from = usize::default();
    for word in frame {
        let at = (from..words.len()).find(|at| words[*at] == *word).unwrap_or(words.len());
        gaps.push(at.saturating_sub(from));
        from = (at + 1).min(words.len());
    }
    gaps.push(words.len().saturating_sub(from));
    gaps
}
because!(gaps_of, LearnedWording, "how many words of a question stand before, between and after the words of a frame, each frame word taken at its first place after the one before it");

fn run_form(cases: &[&Case]) -> Option<Pattern> {
    let questions: Vec<Vec<String>> = cases.iter().map(|case| fact_words(&case.question)).collect();
    let (first, rest) = questions.split_first()?;
    let mut frame: Vec<String> = rest.iter().fold(first.clone(), |frame, words| common_words(&frame, words));
    let runs = loop {
        let gaps: Vec<Vec<usize>> = questions.iter().map(|words| gaps_of(&frame, words)).collect();
        let mixed = (0..=frame.len()).find(|gap| gaps.iter().any(|counts| counts[*gap] == usize::default()) && gaps.iter().any(|counts| counts[*gap] > usize::default()));
        if let Some(gap) = mixed {
            if frame.is_empty() {
                return None;
            }
            frame.remove(gap.min(frame.len() - 1));
            continue;
        }
        let runs: Vec<bool> = (0..=frame.len()).map(|gap| gaps.iter().all(|counts| counts[gap] > usize::default())).collect();
        let changing: Vec<usize> = (0..runs.len()).filter(|gap| runs[*gap]).collect();
        if changing.len() <= MOST_RUN_PLACES {
            break runs;
        }
        let (from, to) = changing.iter().zip(changing.iter().skip(1)).min_by_key(|(from, to)| **to - **from)?;
        frame.drain(*from..*to);
    };
    let places = runs.iter().filter(|run| **run).count();
    if frame.is_empty() || places == usize::default() {
        return None;
    }
    let mut pattern: Pattern = Vec::new();
    let mut place = ANY_FIRST;
    for (gap, run) in runs.iter().enumerate() {
        if *run {
            pattern.push(LogicPiece::Open(place));
            place += 1;
        }
        if let Some(word) = frame.get(gap) {
            pattern.push(LogicPiece::Word(word_number(word)));
        }
    }
    Some(pattern)
}
because!(run_form, LearnedWording, "the one form of questions that no form of enough lessons covers: the words that all of them hold in the same order, and a place that may hold any signs wherever every question has words between two of them, as a question that holds its data (an expression, a path, two phrases to compare) changes in long stretches whose words, frequent or not, are values; a frame word before which some questions have words and others none leaves the frame, and when more stretches change than a learned run form takes, the two that the fewest frame words part become one; nothing when no word is shared or no stretch changes");

fn holds_runs(form: &Pattern) -> bool {
    form.iter().any(|logic_piece| matches!(logic_piece, LogicPiece::Open(place) if *place >= ANY_FIRST))
}
because!(holds_runs, LearnedWording, "whether a form has a place of any signs: such a place holds a stretch of data, which a fact names only when the facts of the story hold that stretch, as the forms of the facts that hold the words of the question or the answer find it, so the outcomes and the read facts of the rules with their places given to the places of the form are not tried for it, nor the filled answers that give the places of a fact every place, free place and fixed word of the form, which grow past any count with a long frame");

const MOST_ASKED_FORMS: usize = 16;
because!(MOST_ASKED_FORMS, LearnedWording, "the most forms of facts of the stories that hold the words of the questions which become sources of conditions, the most common first, so that the judging of conditions stays small");

fn ending_of(word: WordId, run: &[WordId]) -> Option<&'static str> {
    let [stem] = run else { return None };
    let text = word_text(word);
    WORD_ENDINGS.iter().copied().find(|ending| text.strip_suffix(*ending) == Some(word_text(*stem)))
}
because!(ending_of, LearnedWording, "the ending that a word of a fact adds to the one word of a place of the question (chases for chase, calls for call), so that a fact that writes the verb of a question with -s still names that place");

fn asked_pattern(fact: &[WordId], runs: &[(usize, &[WordId])]) -> Option<Pattern> {
    let mut pattern: Pattern = Vec::new();
    let mut at = usize::default();
    let mut replaced = false;
    while at < fact.len() {
        let found = runs.iter().filter(|(_, words)| !words.is_empty() && fact[at..].starts_with(words)).min_by_key(|(_, words)| std::cmp::Reverse(words.len()));
        match found {
            Some((place, words)) => {
                pattern.push(LogicPiece::Open(*place));
                at += words.len();
                replaced = true;
            }
            None => {
                match runs.iter().find_map(|(place, words)| ending_of(fact[at], words).map(|ending| (*place, ending))) {
                    Some((place, ending)) => {
                        pattern.push(LogicPiece::OpenWithEnd(place, ending.to_string()));
                        replaced = true;
                    }
                    None => pattern.push(LogicPiece::Word(fact[at])),
                }
                at += 1;
            }
        }
    }
    (replaced && fixed_count(&pattern) > usize::default()).then_some(pattern)
}
because!(asked_pattern, LearnedWording, "a fact of a story as a condition of a form: each run of its words that a place of the question holds becomes that place (the longest run first, and of runs as long the first place), a word that adds an ending to the one word of a place becomes that place with the ending, every other word stays; nothing when the fact holds no words of the question or only them");

fn count_form(pattern: Pattern, seen: &mut HashSet<Pattern>, counts: &mut HashMap<Pattern, usize>, order: &mut Vec<Pattern>) {
    if !seen.insert(pattern.clone()) {
        return;
    }
    let count = counts.entry(pattern.clone()).or_default();
    if *count == usize::default() {
        order.push(pattern);
    }
    *count += 1;
}
because!(count_form, LearnedWording, "one more lesson that holds a form of a fact, counted once for each lesson however many of its facts have that form, with the order in which the forms were first seen");

fn counted_forms(counts: &HashMap<Pattern, usize>, order: Vec<Pattern>) -> Vec<Pattern> {
    let mut kept: Vec<(Pattern, usize)> = order.into_iter().map(|pattern| { let count = counts[&pattern]; (pattern, count) }).filter(|(_, count)| *count >= LEAST_SUPPORT).collect();
    kept.sort_by(|one, other| other.1.cmp(&one.1));
    kept.into_iter().take(MOST_ASKED_FORMS).map(|(pattern, _)| pattern).collect()
}
because!(counted_forms, LearnedWording, "the forms of facts that come again in enough lessons, the most common first and of those as common the first seen, at most as many as the judging of conditions takes");

fn asked_forms(form: &Pattern, lessons: &[&WordingLesson], base: &Facts) -> (Vec<Pattern>, Vec<Pattern>) {
    let own = places_of(form);
    let start = base.added_to(&Facts::default());
    let (mut plain_counts, mut filled_counts): (HashMap<Pattern, usize>, HashMap<Pattern, usize>) = (HashMap::new(), HashMap::new());
    let (mut plain_order, mut filled_order): (Vec<Pattern>, Vec<Pattern>) = (Vec::new(), Vec::new());
    for lesson in lessons {
        let mut fillings = Vec::new();
        fit_into(form, &lesson.question, &empty_filling(), &mut fillings);
        let Some(filling) = fillings.first() else { continue };
        let runs: Vec<(usize, &[WordId])> = own.iter().enumerate().filter_map(|(at, place)| Some((at, filling.get(*place)?))).collect();
        let answer: Vec<WordId> = lesson.answer.iter().map(|word| word_number(word)).collect();
        let free = if lesson.answer.iter().any(|word| !word_not_mark(word)) { ANY_FIRST + own.iter().filter(|place| **place >= ANY_FIRST).count() } else { own.len() };
        let mut answered = runs.clone();
        answered.push((free, answer.as_slice()));
        let (mut plain_seen, mut filled_seen): (HashSet<Pattern>, HashSet<Pattern>) = (HashSet::new(), HashSet::new());
        for place in start..start + lesson.facts.added_to(base) {
            let fact = lesson.facts.fact(place);
            if let Some(pattern) = asked_pattern(fact, &runs) {
                count_form(pattern, &mut plain_seen, &mut plain_counts, &mut plain_order);
            }
            if let Some(pattern) = asked_pattern(fact, &answered).filter(|pattern| pattern.iter().any(|logic_piece| matches!(logic_piece, LogicPiece::Open(at) if *at == free))) {
                count_form(pattern, &mut filled_seen, &mut filled_counts, &mut filled_order);
            }
        }
    }
    (counted_forms(&plain_counts, plain_order), counted_forms(&filled_counts, filled_order))
}
because!(asked_forms, LearnedWording, "the forms of the facts of the stories of the sample of a form, told or made by the rules, that hold the words of the question, with those words as the places of the form, kept when they come again in enough lessons: a story that says Bob is not big answers the question Bob is big through the form A is not B, which no rule of the text makes as its outcome; and the forms of the facts that hold the answer of the lesson too, with the answer as a free place, as conditions of an answer that the facts fill (Wren is a stool gives B if A is a B), a place of any signs for an answer that holds a mark (the closing brackets of an input)");

fn asked_candidates(form: &Pattern, form_text: &str, asked: &[Pattern]) -> Vec<WordingCandidate> {
    let mut seen: HashSet<String> = HashSet::new();
    asked.iter().map(|pattern| library_writing(&with_form_places(pattern, form))).filter(|text| seen.insert(text.clone())).filter_map(|text| probed(form_text, &text, None, false)).collect()
}
because!(asked_candidates, LearnedWording, "the conditions of the forms of facts that hold the words of the question, each in the places of the form as it stands, with no place left free");

fn asked_filled(form: &Pattern, places: usize, form_text: &str, filled: &[Pattern]) -> Vec<(WordingCandidate, String)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut question = FormReading::default();
    filled
        .iter()
        .filter_map(|pattern| {
            let free = pattern.iter().find_map(|logic_piece| match logic_piece {
                LogicPiece::Open(place) if *place >= places => Some(*place),
                _ => None,
            })?;
            filled_candidate(form_text, &library_writing(&with_form_places(pattern, form)), place_letter(free), (&mut seen, &mut question))
        })
        .collect()
}
because!(asked_filled, LearnedWording, "the filled answers of the forms of facts that hold the answer of their lessons: the free place after those of the form (of words, or of any signs) is the answer, and the condition the form of the fact");

#[derive(Clone, Copy)]
struct Reach {
    as_told: bool,
    wide: bool,
    timed: bool,
}
because!(Reach, LearnedWording, "which conditions of one fact are made for a form: with the form itself as a told fact or not; in the wide reach also the facts that the rules read, each fact also with its open places that stand side by side read as one; and each condition also as held only by the latest fact that fits it, when asked for");

fn source_patterns(topic: &Topic, form: &Pattern, reach: Reach, rewritten: &[Pattern]) -> (Vec<Pattern>, usize) {
    let mut sources: Vec<Pattern> = if reach.as_told { vec![form.clone()] } else { Vec::new() };
    for outcome in topic.rules.iter().map(|rule| &rule.outcome).filter(|_| !holds_runs(form)).chain(rewritten) {
        if !sources.contains(outcome) {
            sources.push(outcome.clone());
        }
    }
    let narrow = sources.len();
    if reach.wide && !holds_runs(form) {
        let read = topic.rules.iter().flat_map(|rule| &rule.conditions).filter_map(|condition| match condition {
            Condition::Fact { pattern, .. } => Some(pattern),
            _ => None,
        });
        for pattern in read {
            if !sources.contains(pattern) {
                sources.push(pattern.clone());
            }
        }
        let joined: Vec<Pattern> = sources.iter().map(|pattern| places_joined(pattern)).collect();
        for pattern in joined {
            if !sources.contains(&pattern) {
                sources.push(pattern);
            }
        }
    }
    (sources, narrow)
}
because!(source_patterns, LearnedWording, "the facts from which the conditions of one fact are made, with how many come first in the narrow reach: the form itself as a told fact when asked for, every fact that a rule of the text makes and every form of the told sentences that a line of the text wrote again; in the wide reach then every fact that a rule reads, and each fact with its open places that stand side by side read as one place");

fn places_joined(pattern: &Pattern) -> Pattern {
    let mut out: Pattern = Vec::new();
    for logic_piece in pattern {
        if matches!(logic_piece, LogicPiece::Open(_)) && matches!(out.last(), Some(LogicPiece::Open(_))) {
            continue;
        }
        out.push(logic_piece.clone());
    }
    out
}
because!(places_joined, LearnedWording, "a pattern with each run of open places side by side read as one place, as an answer of a question may be all the words of a fact that a rule makes in several places (a route of two steps)");

fn single_candidates(topic: &Topic, form: &Pattern, places: usize, form_text: &str, reach: Reach, rewritten: &[Pattern]) -> Vec<WordingCandidate> {
    let (sources, narrow) = source_patterns(topic, form, reach, rewritten);
    singles_of((&sources, narrow), (form, places, form_text), reach.timed, rewritten)
}
because!(single_candidates, LearnedWording, "the conditions of one fact for a form, from the facts of the reach of a text");

fn singles_of((sources, narrow): (&[Pattern], usize), (form, places, form_text): (&Pattern, usize, &str), timed: bool, rewritten: &[Pattern]) -> Vec<WordingCandidate> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<WordingCandidate> = Vec::new();
    for (at, source) in sources.iter().enumerate() {
        let from = places_of(source);
        if places == usize::default() || (from.is_empty() && rewritten.contains(source)) {
            let text = library_writing(source);
            if fixed_count(source) > usize::default()
                && seen.insert(text.clone())
                && let Some(candidate) = probed(form_text, &text, None, false)
            {
                out.push(candidate);
            }
            continue;
        }
        if from.is_empty() || from.len() > places + 1 {
            continue;
        }
        for to in place_choices(from.len(), places, places).into_iter().filter(|to| to.iter().any(|place| *place < places)) {
            let condition = with_form_places(&renamed(source, &from, &to), form);
            let text = library_writing(&condition);
            let free = to.contains(&places);
            let single = (free && at < narrow).then(|| condition.clone());
            if seen.insert(text.clone())
                && let Some(candidate) = probed(form_text, &text, single, free)
            {
                out.push(candidate);
            }
        }
    }
    if timed {
        let latest: Vec<WordingCandidate> = out.iter().filter_map(|candidate| probed(form_text, &format!("{LATEST_START}{}", candidate.text), None, candidate.free)).collect();
        out.extend(latest);
    }
    out
}
because!(singles_of, LearnedWording, "the conditions of one fact: each fact of the reach with its open places given to the places of the form in every way, one place left free at most and one place of the form named at least, since a fact that names nothing of a question with places holds on every story of the question; for a form with no place, each such fact as it stands, its places free, since then whether the story holds such a fact tells the answer, and so too a told sentence written again into a form with no place, which holds for some stories of the question and not for others; when asked for, each also as held only by the latest fact that fits it, after all the plain ones, so that of two that answer the same lessons the plain one comes first; only the facts of the narrow reach are compared, so that the comparisons stay few");

const LATEST_START: &str = "latest ";
const BEFORE_START: &str = "before that time ";
because!(BEFORE_START, LearnedWording, "the words that open a second condition held only by the latest fact before the fact of the first condition, in the words of the library, as where a person was when a thing was dropped is the place that the story told last before the drop");
const AT_START: &str = "at that time ";
because!(AT_START, LearnedWording, "the words that open a second condition held only by the latest fact up to the time of the fact of the first condition, in the words of the library, as the state of a thing when another fact came");
because!(LATEST_START, LearnedWording, "the word that opens a condition held only by the latest fact that fits it, in the words of the library, as the state of a thing (where a person is, what a person holds) is the last that the story told of it");

const SECOND_TIMES: &[&str] = &[LATEST_START, BEFORE_START, AT_START, ""];
because!(SECOND_TIMES, LearnedWording, "the ways a second fact of an answer of two facts is read, in the order in which they are tried: held only by the latest fact, by the latest before the first fact, by the latest up to it, or at any time; a fact held at a time is one of the facts that hold it at any time, so when the plain second fact holds on no lesson where the first holds, none of the others does");

fn free_place_of_kind(source: usize, places: usize) -> usize {
    if source < PLAIN_PLACES {
        places
    } else if source < MARKED_FIRST {
        source
    } else if source < GROUPED_FIRST {
        MARKED_FIRST + places
    } else if source < ONE_SIGN_FIRST {
        GROUPED_FIRST + places
    } else {
        source
    }
}
because!(free_place_of_kind, LearnedWording, "the place that a filled answer gives, of the same kind as the place of the fact that fills it: the place after those of the form for a place of words, of marks or of groups, so that a name with marks (a title with a hyphen) is said whole, and its own place for a place of one word or of signs, which no place of the form takes");

#[derive(Clone, Copy, PartialEq)]
enum FilledTarget {
    Place(usize),
    Free(usize),
    Word(WordId),
}
because!(FilledTarget, LearnedWording, "what a place of a fact becomes in the condition of a filled answer: a place of the form, a free place whose words the answer holds (the first or the second), or a fixed word of the form that the stories tell (a name that every question of the form asks about)");

const MOST_FREE_PLACES: usize = 2;
because!(MOST_FREE_PLACES, LearnedWording, "the most free places of the condition of a filled answer: an answer may be a span with a start and an end (the hours of a visit), while more free places would make too many conditions to judge");

fn filled_targets(count: usize, places: usize, constants: &[WordId]) -> Vec<Vec<FilledTarget>> {
    if count == usize::default() {
        return vec![Vec::new()];
    }
    let mut out: Vec<Vec<FilledTarget>> = Vec::new();
    for shorter in filled_targets(count - 1, places, constants) {
        let options = (0..places).map(FilledTarget::Place).chain((0..MOST_FREE_PLACES).map(FilledTarget::Free)).chain(constants.iter().map(|word| FilledTarget::Word(*word)));
        let named = shorter.iter().any(|target| matches!(target, FilledTarget::Word(_)));
        for option in options {
            if !shorter.contains(&option) && !(named && matches!(option, FilledTarget::Word(_))) {
                let mut longer = shorter.clone();
                longer.push(option);
                out.push(longer);
            }
        }
    }
    out
}
because!(filled_targets, LearnedWording, "every way to give each of a number of places of a fact a place of the form, a free place or a fixed word of the form, each used once and one fixed word at most, as a filled answer names the one thing that every question of its form asks about, while a form whose stories tell many of its fixed words would give more ways than can be judged");

fn filled_condition(outcome: &Pattern, from: &[usize], targets: &[FilledTarget], places: usize) -> Option<(Pattern, Pattern)> {
    let mut first_free: Option<usize> = None;
    let mut last_free: Option<usize> = None;
    let mut out: Pattern = Vec::new();
    for logic_piece in outcome {
        let (place, ending) = match logic_piece {
            LogicPiece::Word(word) => {
                out.push(LogicPiece::Word(*word));
                continue;
            }
            LogicPiece::Open(place) => (*place, String::new()),
            LogicPiece::OpenWithEnd(place, ending) => (*place, ending.clone()),
        };
        let at = from.iter().position(|known| *known == place)?;
        let target = match targets[at] {
            FilledTarget::Place(target) => target,
            FilledTarget::Free(which) => {
                first_free.get_or_insert(out.len());
                last_free = Some(out.len());
                free_place_of_kind(place, places + which)
            }
            FilledTarget::Word(word) if ending.is_empty() => {
                out.push(LogicPiece::Word(word));
                continue;
            }
            FilledTarget::Word(_) => return None,
        };
        out.push(if ending.is_empty() { LogicPiece::Open(target) } else { LogicPiece::OpenWithEnd(target, ending) });
    }
    let said = out[first_free?..=last_free?].to_vec();
    Some((out, said))
}
because!(filled_condition, LearnedWording, "the condition of a filled answer: the fact with each of its places given its target, a free place of the same kind as in the fact, and the words of the answer: the stretch of the fact from its first free place to its last (a span: T to U); nothing when a place with an ending would become a fixed word");

fn filled_candidates(form: &Pattern, places: usize, form_text: &str, (sources, constants, rewritten): (&[Pattern], &[WordId], &[Pattern]), (timed, keep): (bool, &mut dyn FnMut(&WordingCandidate) -> bool)) -> Vec<(WordingCandidate, String)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut question = FormReading::default();
    let mut out: Vec<(WordingCandidate, String)> = Vec::new();
    if holds_runs(form) {
        return out;
    }
    let mut latest: Vec<(String, String)> = Vec::new();
    for outcome in sources {
        let from = places_of(outcome);
        if from.is_empty() || from.len() > places + constants.len().min(1) + 1 {
            continue;
        }
        let freed = |targets: &Vec<FilledTarget>, which: usize| targets.contains(&FilledTarget::Free(which));
        let named = |targets: &Vec<FilledTarget>| targets.iter().any(|target| !matches!(target, FilledTarget::Free(_)));
        let told_again = rewritten.contains(outcome);
        for targets in filled_targets(from.len(), places, constants).into_iter().filter(|targets| freed(targets, usize::default()) && (told_again || named(targets))) {
            if freed(&targets, 1) && targets.iter().position(|target| *target == FilledTarget::Free(1)) < targets.iter().position(|target| *target == FilledTarget::Free(usize::default())) {
                continue;
            }
            let Some((pattern, said_pattern)) = filled_condition(outcome, &from, &targets, places).map(|(pattern, said)| (with_form_places(&pattern, form), with_form_places(&said, form))) else { continue };
            let condition = library_writing(&pattern);
            let said = said_pattern.iter().map(|logic_piece| match logic_piece {
                LogicPiece::Word(word) => word_text(*word).to_string(),
                LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => place_letter(*place),
            }).collect::<Vec<String>>().join(FORM_WORD_GAP);
            if timed {
                latest.push((format!("{LATEST_START}{condition}"), said.clone()));
            }
            out.extend(filled_candidate(form_text, &condition, said, (&mut seen, &mut question)).filter(|(candidate, _)| keep(candidate)));
        }
    }
    for (condition, said) in latest {
        out.extend(filled_candidate(form_text, &condition, said, (&mut seen, &mut question)).filter(|(candidate, _)| keep(candidate)));
    }
    out
}
because!(filled_candidates, LearnedWording, "the conditions of one fact that a rule of the text makes, or of a form of a told sentence that a line of the text wrote again (which may name nothing of the question, as it is the question or the story written again), whose free places give the answer: each place of the fact given to a place of the form or a fixed word of the form that the stories tell, and one or two of them left free, of the same kind as in the fact (the second free place stands after the first); the answer is the stretch of the fact from its first free place to its last, as a written line answers 'BB if R ends with BB' or 'T to U if the span T to U is free for R at the B'; when asked for, each condition also as held only by the latest fact that fits it, after all the plain ones; a candidate that the caller does not keep (one whose answers may hold on no lesson) is left out as soon as it is read, so that the many candidates of a form are not held at once");

fn made_facts(topic: &Topic, rewritten: &[Pattern]) -> Vec<Pattern> {
    topic.rules.iter().map(|rule| &rule.outcome).chain(rewritten).cloned().collect()
}
because!(made_facts, LearnedWording, "the facts whose places a filled answer gives: every fact that a rule of the text makes, and every form of the told sentences that a line of the text wrote again");

fn filled_line(form_text: &str, condition: &str, said: &str) -> String {
    format!("{LINE_START}{form_text}{LINE_MIDDLE}{said}{LEARNED_IF}{condition}{STATEMENT_END}")
}
because!(filled_line, LearnedWording, "the line of a condition with a filled answer as the library writes it, by which its candidate is told apart from the others of a form");

#[derive(Default)]
struct FormReading {
    question: Option<Pattern>,
    held: HashMap<String, PartReading>,
}
because!(FormReading, LearnedWording, "what the reader of the library read of the lines of the filled answers of one form: its question, read once, and each part of their conditions, read once, as the filled answers of a form are many and most share their conditions, or the parts of them, with others (a first condition joined with each of thousands of second facts)");

fn filled_probe(form_text: &str, (said, held): (&str, &str), line: &str, read: &mut FormReading) -> Option<AskedForm> {
    if plain_filled(form_text, said, held) {
        return filled_form(form_text, (said, held), &mut read.question, &mut read.held).ok();
    }
    match quoted_answers(line) {
        Some((asked, answers)) if asked == form_text => question_form(asked, answers, &mut read.question).ok(),
        _ => read_topic(PROBE, line).ok().and_then(|topic| topic.questions.into_iter().next()),
    }
}
because!(filled_probe, LearnedWording, "the question form of the line of a filled answer as the reader of the library reads it, its question read once for all the lines of a form; a plain line read by its parts, each text of its conditions once for the form");

fn filled_candidate(form_text: &str, condition: &str, said: String, (seen, question): (&mut HashSet<String>, &mut FormReading)) -> Option<(WordingCandidate, String)> {
    let line = filled_line(form_text, condition, &said);
    if !seen.insert(line.clone()) {
        return None;
    }
    let probe = filled_probe(form_text, (&said, condition), &line, question)?;
    let mut hasher = DefaultHasher::new();
    line.hash(&mut hasher);
    Some((WordingCandidate { text: condition.to_string(), probe: Arc::new(probe), single: None, free: false, key: hasher.finish() }, said))
}
because!(filled_candidate, LearnedWording, "a condition with a filled answer as the reader of the library reads its line, once for each line of a form, with the hash of the line by which its checks are kept; the line is not kept for the run, as the filled answers of a form are many and seldom come again; the question of the form is read once for all its lines, as the lines of one form share it");

fn joined_seconds(sources: &[Pattern], form: &Pattern, places: usize) -> (Vec<(String, String)>, Vec<String>) {
    let shared = places;
    let further = places + 1;
    let mut seconds: Vec<(String, String)> = Vec::new();
    let mut lone: Vec<String> = Vec::new();
    for source in sources {
        let from = places_of(source);
        if from.is_empty() || from.len() > places + JOINED_FREE_PLACES {
            continue;
        }
        for to in place_choices(from.len(), places + 1, further).into_iter().filter(|to| to.contains(&shared)) {
            let second = library_writing(&with_form_places(&renamed(source, &from, &to), form));
            let both = to.contains(&further);
            if both && to.iter().any(|place| *place < places) {
                lone.extend([second.clone(), format!("{LATEST_START}{second}")]);
            }
            let answer = place_letter(if both { further } else { shared });
            seconds.push((second, answer));
        }
    }
    (seconds, lone)
}
because!(joined_seconds, LearnedWording, "the second facts of answers of two facts joined by a value that the question does not name, each with its answer: a fact of the reach that names the free place of a first condition, which the caller reads in each of the ways of SECOND_TIMES, the other places given to places of the form or to one more free place, the answer that further place when it names it, else the shared one, as a person's kind gives the colour of the kind and a person's need gives the place that people with that need go to; and the facts alone with two free places, whose answer is the first of them, as the giver of a thing is asked while the receiver is not named");

const HALF_PLACES: usize = 2;
because!(HALF_PLACES, LearnedWording, "the places of each half of a joined condition: one place of the question and the free value that the halves share (B is the number C, the money of A after the change is C), so that each half ties the value to one word of the question");

const MOST_HALVES: usize = 24;
because!(MOST_HALVES, LearnedWording, "the most halves that are joined in pairs for one form, the first found, so that the pairs stay few hundred at most");

fn joined_labels(topic: &Topic, form: &Pattern, form_text: &str, rewritten: &[Pattern], (held, print): (&[&WordingLesson], u64), misses: &[usize]) -> Vec<WordingCandidate> {
    let places = places_of(form).len();
    let (sources, _) = source_patterns(topic, form, Reach { as_told: false, wide: true, timed: false }, rewritten);
    let mut halves: Vec<(usize, String, Vec<bool>)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for source in &sources {
        let from = places_of(source);
        if from.len() != HALF_PLACES {
            continue;
        }
        for question in 0..places {
            for to in [[question, places], [places, question]] {
                let text = library_writing(&with_form_places(&renamed(source, &from, &to), form));
                if halves.len() >= MOST_HALVES || !seen.insert(text.clone()) {
                    continue;
                }
                if let Some(candidate) = probed(form_text, &text, None, true)
                    && let Some(row) = candidate_row(&candidate, held, print, false)
                    && misses.iter().any(|at| row[*at])
                {
                    halves.push((question, text, row));
                }
            }
        }
    }
    let mut out: Vec<WordingCandidate> = Vec::new();
    for (at, (one_place, one, one_row)) in halves.iter().enumerate() {
        for (other_place, other, other_row) in &halves[at + 1..] {
            if one_place != other_place && misses.iter().any(|miss| one_row[*miss] && other_row[*miss]) {
                out.extend(probed(form_text, &format!("{one}{CONDITIONS_JOINED}{other}"), None, false));
            }
        }
    }
    out
}
because!(joined_labels, LearnedWording, "the conditions of a label of facts joined by a value that the question does not name: each fact of the wide reach with two places, one given a place of the question and the other the free value, kept when it holds on a lesson that the list does not answer yet, and each pair of such halves that name two places of the question and both hold on a lesson not yet answered (entailment if B is the number C and the money of A after the change is C, where the story writes the number in words)");

fn event_joins(sources: &[Pattern], form: &Pattern, places: usize) -> Vec<(String, String)> {
    let free = places;
    let mut out: Vec<(String, String)> = Vec::new();
    for source in sources {
        let from = places_of(source);
        if from.len() < EVENT_SECOND_PLACES || from.len() > places {
            continue;
        }
        for to in place_choices(from.len(), places, free).into_iter().filter(|to| !to.contains(&free)) {
            let first = library_writing(&with_form_places(&renamed(source, &from, &to), form));
            for earlier in place_choices(from.len(), places, free).into_iter().filter(|earlier| earlier.contains(&free) && earlier.iter().any(|place| to.contains(place))) {
                out.push((first.clone(), library_writing(&with_form_places(&renamed(source, &from, &earlier), form))));
            }
        }
    }
    out
}
because!(event_joins, LearnedWording, "the two facts of an answer that an event of the things that the question names and the same fact of one of them at that time give, as where a thing was before a place is read from the fact that the thing is in that place, read at every time it held, the latest first, and, before that time, the fact of the place of the thing: a first fact whose places are all places of the question, and the same fact again with the answer in a free place and a place of the question that the first fact names; the first fact read as held only by its latest telling would ask for the state of the thing now, which the story may have changed since, and a second fact of another kind read before that time (Yesterday A went to the C, before a fact of this morning) would read the order of the telling where the words of the story give another");

const EVENT_TIMES: [&str; 2] = [BEFORE_START, AT_START];
because!(EVENT_TIMES, LearnedWording, "the ways the second fact of an answer read at the time of a first fact is read: by the latest fact before it, or the latest up to it; read at any time it would not hang on the first fact");

const EVENT_SECOND_PLACES: usize = 2;
because!(EVENT_SECOND_PLACES, LearnedWording, "the fewest places of a fact whose event and earlier value give an answer: the place that the event and its earlier value share, and the place of the answer");

const JOINED_FREE_PLACES: usize = 2;
because!(JOINED_FREE_PLACES, LearnedWording, "the places that a second fact may name beyond the places of the form: the free place that it shares with the first condition and one more, whose words may be the answer");

fn told_constants(form: &Pattern, told: &[&HashSet<String>]) -> Vec<WordId> {
    let mut out: Vec<WordId> = Vec::new();
    for logic_piece in form {
        if let LogicPiece::Word(word) = logic_piece
            && word_not_mark(word_text(*word))
            && !out.contains(word)
            && told.iter().filter(|words| words.contains(word_text(*word))).count() * WHOLE_SHARE >= told.len() * BROAD_PERCENT
        {
            out.push(*word);
        }
    }
    out
}
because!(told_constants, LearnedWording, "the fixed words of a form that the stories of its lessons tell, most of them: a name that every question of the form asks about stays a fixed word when it is common in the questions, and a filled answer may name it; only for a form of a reading by a share, as in the reading by the text the fixed words that the stories tell are the words of relations that the rules name, not values");

fn filled_row(candidate: &WordingCandidate, held: &[&WordingLesson], may: &[bool]) -> (Vec<bool>, Vec<bool>) {
    held.iter()
        .zip(may)
        .map(|(lesson, may)| match may.then(|| reply(std::slice::from_ref(candidate.probe.as_ref()), &lesson.question, &Facts::given_again(&lesson.facts))).flatten() {
            Some(answered) => (true, answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP)) == lesson.answer),
            None => (false, false),
        })
        .unzip()
}
because!(filled_row, LearnedWording, "where a condition with a filled answer holds on the sample of a form, and where the words that it fills are the answer of the lesson; a lesson on which no answer may hold is not asked, as its reply would be none");

fn may_reply_rows(probe: &AskedForm, facts: &[&Facts], known: &mut HashMap<Vec<WordId>, Vec<bool>>) -> Vec<bool> {
    let mut out = vec![false; facts.len()];
    for answer in &probe.replies {
        let mut row = vec![true; facts.len()];
        for condition in &answer.conditions {
            let Condition::Fact { pattern, when: Moment::Any } = condition else { continue };
            let words: Vec<WordId> = pattern.iter().filter_map(|logic_piece| match logic_piece {
                LogicPiece::Word(word) => Some(*word),
                LogicPiece::Open(_) | LogicPiece::OpenWithEnd(..) => None,
            }).collect();
            let held = known.entry(words).or_insert_with_key(|words| facts.iter().map(|facts| facts.holds_in_order(words)).collect());
            for (may, holds) in row.iter_mut().zip(held.iter()) {
                *may &= *holds;
            }
        }
        for (may, holds) in out.iter_mut().zip(row) {
            *may |= holds;
        }
    }
    out
}
because!(may_reply_rows, LearnedWording, "on which lessons of a sample some answer of a probe may hold: a fact condition read at any time holds only on a fact that holds its fixed words in order, as the engine passes over every other fact, so a lesson on which some condition of each answer finds no such fact gets no reply; the test of the words of a condition on the lessons is kept for the form, as many candidates of a form share the words of a condition and differ only in their open places");

fn compared_candidates(singles: &[&WordingCandidate], form: &Pattern, form_text: &str) -> Vec<WordingCandidate> {
    let own = places_of(form);
    let places = own.len();
    let free = place_letter(places);
    let other = place_letter(places + 1);
    let mut out: Vec<WordingCandidate> = Vec::new();
    for single in singles {
        for place in &own {
            for comparing in COMPARING {
                let text = format!("{}{CONDITIONS_JOINED}{free} {comparing} {}", single.text, place_letter(*place));
                out.extend(probed(form_text, &text, None, false));
            }
        }
    }
    for (at, first) in singles.iter().enumerate() {
        for second in &singles[at + 1..] {
            let Some(pattern) = &second.single else { continue };
            let from = places_of(pattern);
            let to: Vec<usize> = from.iter().map(|place| if *place == places { places + 1 } else { *place }).collect();
            let shifted = library_writing(&renamed(pattern, &from, &to));
            for comparing in COMPARING {
                let text = format!("{}{CONDITIONS_JOINED}{shifted}{CONDITIONS_JOINED}{free} {comparing} {other}", first.text);
                out.extend(probed(form_text, &text, None, false));
            }
        }
    }
    out
}
because!(compared_candidates, LearnedWording, "the conditions that compare values: a fact with a free value compared with a word of the question, and two facts with free values compared with each other, made only from facts that hold broadly");

pub(super) struct WordingLesson {
    pub(super) question: Vec<WordId>,
    facts: Arc<Facts>,
    pub(super) answer: Vec<String>,
    print: u64,
}
because!(WordingLesson, LearnedWording, "a lesson read for learning a wording: its question, the facts of its story, the words of its answer, and a fingerprint of its facts and its question, by which the checks of conditions on it are kept");

pub(super) fn lesson_of(question: Vec<WordId>, facts: Arc<Facts>, answer: Vec<String>) -> WordingLesson {
    let mut hasher = DefaultHasher::new();
    facts.fingerprint().hash(&mut hasher);
    question.hash(&mut hasher);
    WordingLesson { print: hasher.finish(), question, facts, answer }
}
because!(lesson_of, LearnedWording, "a lesson read for learning, with the fingerprint of its facts and its question made once, so that each check of a condition on it finds the kept result by a number");

fn value_holding(candidate: &WordingCandidate, places: usize) -> Option<String> {
    let [reply] = candidate.probe.replies.as_slice() else { return None };
    let [Condition::Fact { pattern, when: Moment::Latest | Moment::Any }] = reply.conditions.as_slice() else { return None };
    let (Some(LogicPiece::Open(asked)), Some(frame)) = (pattern.last(), pattern.get(..pattern.len() - 1)) else { return None };
    let frame = frame.to_vec();
    let framed = places_of(&frame);
    if *asked >= places || framed.is_empty() || framed.iter().any(|place| *place >= places || place == asked) {
        return None;
    }
    let value = place_letter(places);
    Some(format!("{LATEST_START}{} {value}{CONDITIONS_JOINED}{value} {} {}", library_writing(&frame), HOLDS_WORDS.join(FORM_WORD_GAP), place_letter(*asked)))
}
because!(value_holding, LearnedWording, "a condition that reads the latest whole value of a state and asks whether it holds the asked word, made from a condition that reads the latest value of a state as the asked word itself: the latest value of the state of A may be a value of several places (the B or the C of an either), which a later value of one place replaces, while a condition with the places of such a value (The is of A is B or the C) finds the latest fact of that shape alone, also after a later move to another place ended it");

fn loose_conditions(candidates: &[WordingCandidate], form: &Pattern) -> Vec<bool> {
    let own = places_of(form);
    candidates
        .iter()
        .map(|candidate| {
            let conditions: Vec<&Pattern> = candidate.probe.replies.iter().flat_map(|reply| &reply.conditions).filter_map(|condition| match condition {
                Condition::Fact { pattern, .. } => Some(pattern),
                _ => None,
            }).collect();
            let named: Vec<usize> = conditions.iter().flat_map(|pattern| places_of(pattern)).collect();
            own.iter().any(|place| !named.contains(place))
                || conditions.iter().any(|pattern| !form.is_empty() && pattern.windows(form.len()).any(|stretch| stretch == form.as_slice()))
        })
        .collect()
}
because!(loose_conditions, LearnedWording, "which conditions may hold on every lesson without reading the story: one that leaves a place of the question unnamed (A is a C, for a question about A and B: A is some kind of thing in every story), and one that holds the whole form of the question (the question is ... as told), which every question of the form makes true by itself; a free place beside every place of the question is a reading (C is a child of A, for A has no child)");

fn row_lists(rows: &[Vec<bool>]) -> Vec<Vec<usize>> {
    rows.iter().map(|row| (0..row.len()).filter(|lesson| row[*lesson]).collect()).collect()
}
because!(row_lists, LearnedWording, "the lessons where each candidate condition holds, as a list in the order of the lessons, from its row");

pub(super) struct ListKeys {
    pub(super) of: Vec<Vec<usize>>,
    pub(super) count: usize,
}
because!(ListKeys, LearnedWording, "what may answer each lesson of a list, by places among the different keys of the lessons (a label, the words of an answer, or a way to work an answer out, of which a lesson may have several), and how many different keys there are: a lesson is right under a chosen key that is one of its own");

pub(super) fn answer_keys<'a>(lessons: &[&'a WordingLesson]) -> (Vec<&'a Vec<String>>, ListKeys) {
    let mut distinct: Vec<&Vec<String>> = lessons.iter().map(|lesson| &lesson.answer).collect();
    distinct.sort();
    distinct.dedup();
    let of: Vec<Vec<usize>> = lessons.iter().map(|lesson| vec![distinct.binary_search(&&lesson.answer).unwrap_or_default()]).collect();
    let count = distinct.len();
    (distinct, ListKeys { of, count })
}
because!(answer_keys, LearnedWording, "the different answers of some lessons in order, and the key of each lesson its own answer's place among them, so that answers are counted by their places and of answers as frequent the first in order is taken");

pub(super) fn decision_list(keys: &ListKeys, lists: &[Vec<usize>], (filled, loose): (&[Option<Vec<bool>>], &[bool]), shape: ListShape) -> Vec<usize> {
    let mut counts: Vec<usize> = vec![usize::default(); keys.count];
    let mut touched: Vec<usize> = Vec::new();
    let mut open: Vec<bool> = vec![true; keys.of.len()];
    let mut chosen: Vec<usize> = Vec::new();
    let everywhere: Vec<bool> = lists.iter().enumerate().map(|(at, list)| list.len() == keys.of.len() && loose.get(at).copied().unwrap_or_default() && !matches!(filled.get(at), Some(Some(_)))).collect();
    for _ in 0..shape.most {
        let otherwise = (shape.agreed_otherwise && !chosen.is_empty()).then(|| key_otherwise(keys, &open, shape)).flatten();
        let mut open_counts: Vec<usize> = vec![usize::default(); keys.count];
        let mut open_total = 0usize;
        for held in keys.of.iter().zip(&open).filter(|(_, open)| **open).map(|(held, _)| held) {
            open_total += 1;
            for key in held {
                open_counts[*key] += 1;
            }
        }
        let mut best: Option<(usize, usize)> = None;
        let mut best_of_story: Option<(usize, usize)> = None;
        for (at, list) in lists.iter().enumerate() {
            let filled_on = filled.get(at).and_then(Option::as_ref);
            let mut count = 0usize;
            let mut filled_right = 0usize;
            for lesson in list.iter().filter(|lesson| open[**lesson]) {
                count += 1;
                match filled_on {
                    Some(right_on) => filled_right += usize::from(right_on[*lesson]),
                    None => {
                        for key in &keys.of[*lesson] {
                            if counts[*key] == usize::default() {
                                touched.push(*key);
                            }
                            counts[*key] += 1;
                        }
                    }
                }
            }
            let left_top = if shape.agreed_otherwise && filled_on.is_none() { (0..keys.count).map(|key| open_counts[key] - counts[key]).max().unwrap_or_default() } else { usize::default() };
            let left_agree = left_top >= LEAST_SUPPORT && left_top * WHOLE_SHARE >= (open_total - count) * shape.right_percent;
            let mut top: Option<(usize, usize)> = None;
            for key in touched.drain(..) {
                if top.is_none_or(|(known, right)| counts[key] > right || (counts[key] == right && key < known)) {
                    top = Some((key, counts[key]));
                }
                counts[key] = usize::default();
            }
            if count == usize::default() {
                continue;
            }
            let right = if filled_on.is_some() { filled_right } else { top.map_or(usize::default(), |(_, right)| right) };
            if right < LEAST_SUPPORT || right * WHOLE_SHARE < count * shape.right_percent {
                continue;
            }
            let label = top.map(|(key, _)| key);
            if filled_on.is_none() && label.is_some() && label == otherwise {
                continue;
            }
            let score = right.saturating_sub(count - right) + if left_agree { left_top } else { usize::default() };
            if best.is_none_or(|(best_score, _)| score > best_score) {
                best = Some((score, at));
            }
            if !everywhere[at] && best_of_story.is_none_or(|(best_score, _)| score > best_score) {
                best_of_story = Some((score, at));
            }
        }
        let picked = match best {
            Some((_, at)) if everywhere[at] && !chosen.is_empty() => break,
            Some((_, at)) if everywhere[at] => best_of_story,
            other => other,
        };
        let Some((_, at)) = picked else { break };
        chosen.push(at);
        for lesson in &lists[at] {
            open[*lesson] = false;
        }
    }
    chosen
}
because!(decision_list, LearnedWording, "the conditions of the answers of one form, chosen one after another: the condition that answers right the most lessons not yet answered, less those that it answers wrong, and, in a list whose answer otherwise needs the lessons left to agree, more the lessons that it leaves to that answer when they agree (a condition of the few maybe lessons that leaves only no lessons answers those too, while a condition of some no lessons that leaves no and maybe mixed answers only its own), if it is right on enough lessons and on the share of the list of the lessons where it holds, an answer of a label right where one of the lesson's keys is the key that most of them have, a filled answer right where it fills the answer of the lesson; the lessons that it covers leave before the next is chosen, and of conditions with the same score the first, so a plain fact comes before a comparison and a label before a filled answer; after an answer, a condition of the label that the answer otherwise would give the lessons left is passed over, as it adds nothing to that answer and only fills the short list (a closed world answers false if The A Bs the C. true otherwise., not true if The C Bs the C); a loose condition (a place of the question left unnamed, or the question itself as told) that holds on every lesson of the sample says nothing of the story: it is never the first answer, where it would make the whole form an answer by default, and after an answer it ends the list, so that the lessons left go to the answer otherwise, which says what it is; a condition that names only the places of the question (contradiction if A tried to B, for A did not try to B) is a reading of the story even when every lesson holds it");

fn key_otherwise(keys: &ListKeys, open: &[bool], shape: ListShape) -> Option<usize> {
    let mut counts: Vec<usize> = vec![usize::default(); keys.count];
    let mut remaining = 0usize;
    for held in keys.of.iter().zip(open).filter(|(_, open)| **open).map(|(held, _)| held) {
        remaining += 1;
        for key in held {
            counts[*key] += 1;
        }
    }
    let top = (0..keys.count).filter(|key| counts[*key] > usize::default()).max_by(|one, other| counts[*one].cmp(&counts[*other]).then_with(|| other.cmp(one)))?;
    (counts[top] >= LEAST_SUPPORT && counts[top] * WHOLE_SHARE >= remaining * shape.right_percent).then_some(top)
}
because!(key_otherwise, LearnedWording, "the key that the answer otherwise would give the lessons that no chosen condition answers yet, when they agree on it as a learned answer must: a later condition of that key answers nothing that the answer otherwise does not");

pub(super) fn list_answers(keys: &ListKeys, lists: &[Vec<usize>], filled: &[Option<Vec<bool>>], chosen: &[usize]) -> Vec<Option<usize>> {
    let mut counts: Vec<usize> = vec![usize::default(); keys.count];
    let mut covered: Vec<bool> = vec![false; keys.of.len()];
    let mut answers: Vec<Option<usize>> = Vec::new();
    for at in chosen {
        for lesson in lists[*at].iter().filter(|lesson| !covered[**lesson]) {
            for key in &keys.of[*lesson] {
                counts[*key] += 1;
            }
        }
        for lesson in &lists[*at] {
            covered[*lesson] = true;
        }
        let top = (0..keys.count).filter(|key| counts[*key] > usize::default()).max_by(|one, other| counts[*one].cmp(&counts[*other]).then_with(|| other.cmp(one)));
        answers.push(top.filter(|_| filled[*at].is_none()));
        counts.fill(usize::default());
    }
    answers
}
because!(list_answers, LearnedWording, "the key that each chosen condition of a list gives, in order: the most frequent key of the lessons that it covers after the conditions before it, and nothing for a condition that fills its answer from the facts");

fn list_misses(lessons: &[&WordingLesson], held: &[Vec<bool>], (filled, loose): (&[Option<Vec<bool>>], &[bool])) -> Vec<usize> {
    let lists = row_lists(held);
    let (_, keys) = answer_keys(lessons);
    let chosen = decision_list(&keys, &lists, (filled, loose), FACT_LIST);
    let answers = list_answers(&keys, &lists, filled, &chosen);
    let otherwise = majority_otherwise(&keys, &lists, &chosen, FACT_LIST);
    missed(&keys, (&lists, filled), (&chosen, &answers, otherwise))
}
because!(list_misses, LearnedWording, "the lessons of the sample of a form that the decision list of the candidates so far does not answer right: a second fact is sought only for them, as a form that single facts answer needs no join of two");

pub(super) fn missed(keys: &ListKeys, (lists, filled): (&[Vec<usize>], &[Option<Vec<bool>>]), (chosen, answers, otherwise): (&[usize], &[Option<usize>], Option<usize>)) -> Vec<usize> {
    let mut first: Vec<Option<usize>> = vec![None; keys.of.len()];
    for (step, at) in chosen.iter().enumerate().filter(|(step, at)| filled[**at].is_some() || answers[*step].is_some()) {
        for lesson in &lists[*at] {
            first[*lesson] = first[*lesson].or(Some(step));
        }
    }
    let holds = |key: Option<usize>, place: usize| key.is_some_and(|key| keys.of[place].contains(&key));
    (0..keys.of.len())
        .filter(|place| match first[*place] {
            Some(step) => !filled[chosen[step]].as_ref().map_or_else(|| holds(answers[step], *place), |right| right[*place]),
            None => !holds(otherwise, *place),
        })
        .collect()
}
because!(missed, LearnedWording, "the lessons that a decision list answers wrong or not at all: the first chosen condition that holds on a lesson gives its key or the words that it fills, and a lesson that no chosen condition holds on gets the key otherwise; a key is right on a lesson that has it");

fn answers_held(held: &[&WordingLesson]) -> bool {
    let named = held.iter().filter(|lesson| !lesson.answer.is_empty() && lesson.answer.iter().all(|word| lesson.facts.holds_word(word_number(word)))).count();
    named >= LEAST_SUPPORT && named * WHOLE_SHARE >= held.len() * BROAD_PERCENT
}
because!(answers_held, LearnedWording, "whether the answers of a form may be values of the facts: every word of the answer stands in some fact of the lesson, its story, what the rules made of it or what the text knows, in at least half the lessons of the sample, as a place or a colour is, while a label such as entailment stands in no fact; only then are its filled answers made at all (a label is no value of a fact), with the wide reach, the latest fact and the second facts");

pub(super) fn majority_otherwise(keys: &ListKeys, lists: &[Vec<usize>], chosen: &[usize], shape: ListShape) -> Option<usize> {
    let mut covered: Vec<bool> = vec![false; keys.of.len()];
    for at in chosen {
        for lesson in &lists[*at] {
            covered[*lesson] = true;
        }
    }
    let mut counts: Vec<usize> = vec![usize::default(); keys.count];
    let mut rest = 0usize;
    for held in keys.of.iter().zip(&covered).filter(|(_, covered)| !**covered).map(|(held, _)| held) {
        rest += 1;
        for key in held {
            counts[*key] += 1;
        }
    }
    let top = (0..keys.count).filter(|key| counts[*key] > usize::default()).max_by(|one, other| counts[*one].cmp(&counts[*other]).then_with(|| other.cmp(one)))?;
    (!shape.agreed_otherwise || (!chosen.is_empty() && counts[top] >= LEAST_SUPPORT && counts[top] * WHOLE_SHARE >= rest * shape.right_percent)).then_some(top)
}
because!(majority_otherwise, LearnedWording, "the answer of a form when none of its learned conditions holds: the key of the lessons that no condition covers, when they agree on it as a learned answer must and there are enough of them, and only after an answer with a condition, so that no form answers from the question alone; a label such as neutral is often the answer when the story gives nothing for the question; for a list over the words of the told text, the most common key of those lessons, as a request that no learned word reads is most often of the most common kind");

fn learned_answer(form: &LearnedForm, lesson: &WordingLesson) -> Option<Vec<String>> {
    form.probes
        .iter()
        .zip(&form.answers)
        .zip(&form.fills)
        .find_map(|((probe, (answer, _)), fill)| {
            let answered = reply(std::slice::from_ref(probe), &lesson.question, &Facts::given_again(&lesson.facts))?;
            Some(if *fill { answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP)) } else { answer.clone() })
        })
        .or_else(|| form.otherwise.clone())
}
because!(learned_answer, LearnedWording, "the answer of a learned form to a lesson: the first answer whose condition holds, its label or the words that it fills, else the answer that holds otherwise, and nothing when none holds");

pub(super) fn told_of<'a>(sentences: &'a [String], question: &str, rules: &[ToldRule]) -> Cow<'a, [String]> {
    if sentences.is_empty() {
        return Cow::Owned(vec![format!("{QUESTION_TOLD}{question}")]);
    }
    told_with_question(sentences, question, rules)
}
because!(told_of, LearnedWording, "the told sentences of a case for a learned wording: its story, with its question when a rule of the text reads it; for a case that tells no sentence, its question as the one told sentence, since then the question holds the data that the rules of a text read (an expression, a list of words, a path)");

fn bounded_words(case: &Case, told: &[Vec<WordId>], base: &Arc<Facts>, topic: &Topic) -> Option<Facts> {
    let bound = case.sentences.len() + TRIAL_FACTS;
    let most = base.added_to(&Facts::default()) + bound;
    let reads = TRIAL_READS.max(case.sentences.len() * TRIAL_READS_PER_SENTENCE);
    let facts = chain_words_once(told, base, (&topic.rules, topic.rules_key), (most, reads, TRIAL_MORE_WORDS));
    (facts.added_to(base) < bound && !facts.exhausted()).then_some(facts)
}
because!(bounded_words, LearnedWording, "bounded_facts for a chain that is not kept, from the told sentences of the case already read into words");

pub(super) fn bounded_facts(case: &Case, base: &Arc<Facts>, topic: &Topic, kept: bool) -> Option<Facts> {
    let rules = &topic.rules;
    let bound = case.sentences.len() + TRIAL_FACTS;
    let told = told_of(&case.sentences, &case.question, rules);
    let most = base.added_to(&Facts::default()) + bound;
    let reads = TRIAL_READS.max(case.sentences.len() * TRIAL_READS_PER_SENTENCE);
    let facts = if kept {
        chain_facts_within(&told, base, (rules, topic.rules_key), most, reads)
    } else {
        chain_facts_once(&told, base, (rules, topic.rules_key), (most, reads, TRIAL_MORE_WORDS))
    };
    (facts.added_to(base) < bound && !facts.exhausted()).then_some(facts)
}
because!(bounded_facts, LearnedWording, "the facts of one story for a text under the bounds of a trial, the bound on the facts read growing with a long story, or nothing when its rules run away on that story: a text that runs away on any lesson of a kind does not read it, and learning its wording would cost the bound of the engine on every story; a chain that is not kept also runs away when it makes a fact longer than all its told sentences together and a few words more");

const TRIAL_READS_PER_SENTENCE: usize = 500;
because!(TRIAL_READS_PER_SENTENCE, LearnedWording, "the facts that the engine may read for each told sentence of a long story of a trial: each sentence starts a few joins over the facts told before it, so a story of a hundred sentences that a text reads needs more reads than one of three (bAbI's stories of three facts hold some hundreds of sentences), while a story of up to forty sentences keeps the bound of a short story");

const TRIAL_MORE_WORDS: usize = 20;
because!(TRIAL_MORE_WORDS, LearnedWording, "the words beyond all the told sentences of a story that a fact of a trial may hold: the rules of a text that reads a kind make facts from parts of the told sentences with a few words of their own (at most five more on the LoNLI kinds), while a learned reading whose outcome holds its own condition with more words makes a longer fact in each round");

pub(super) fn learn_forms(topic: &Topic, base: &Arc<Facts>, fit: &[&Case]) -> (Vec<LearnedForm>, QuestionWords) {
    let fit_facts: Vec<Option<Arc<Facts>>> = fit.iter().map(|case| bounded_facts(case, base, topic, true).map(Arc::new)).collect();
    let said: Vec<SaidCase> = fit.iter().map(|case| said_case(case)).collect();
    let (forms, words) = learn_forms_with(topic, base, (fit, &said), &fit_facts);
    (engine_order(topic, forms, fit, &fit_facts), words)
}
because!(learn_forms, LearnedWording, "the learned forms of the questions of the first lessons for one text, with the facts of each lesson found and kept for later reads, in the order in which the engine reads the lessons best");

fn engine_order(topic: &Topic, forms: Vec<LearnedForm>, fit: &[&Case], fit_facts: &[Option<Arc<Facts>>]) -> Vec<LearnedForm> {
    let in_order: Vec<usize> = (0..forms.len()).collect();
    let mut specific = in_order.clone();
    specific.sort_by(|one, other| fixed_count(&forms[*other].question).cmp(&fixed_count(&forms[*one].question)));
    if specific == in_order {
        return forms;
    }
    let right = |order: &[usize]| -> usize {
        let Ok(read) = read_topic(&topic.name, &order.iter().map(|at| form_line(&forms[*at])).collect::<Vec<String>>().join(LEARNED_LINE_BREAK)) else { return usize::default() };
        fit.iter()
            .zip(fit_facts)
            .filter(|(case, facts)| facts.as_ref().is_some_and(|facts| reply(&read.questions, &fact_numbers(&case.question), &Facts::given_again(facts)).is_some_and(|answered| answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP)) == answer_words(&case.answer))))
            .count()
    };
    if right(&specific) <= right(&in_order) {
        return forms;
    }
    let mut slots: Vec<Option<LearnedForm>> = forms.into_iter().map(Some).collect();
    specific.into_iter().filter_map(|at| slots[at].take()).collect()
}
because!(engine_order, LearnedWording, "the learned forms in the order in which the engine, which tries the forms of a text one after another and takes the first answer that holds, reads the first lessons best: the forms with the most lessons first, or, when it reads more of them right, the forms with the most fixed words first, as a general form (A is B of C) whose answers hold for a question of a narrower form (A is child of B) would answer it wrongly before the narrower form is tried");

pub(super) struct FormsGround {
    words: QuestionWords,
    told: Vec<Vec<WordId>>,
    asked: Vec<Vec<WordId>>,
    forms: Vec<Vec<Option<Pattern>>>,
}
because!(FormsGround, LearnedWording, "what the forms of the questions of some lessons rest on whatever the rules of a text: the words of the questions with their places, the told sentences of the first lessons read into words, each question as numbers, and the form of each question under each cut that no text names (the shares, the places, the values), found once for many texts judged on the same lessons");

fn forms_ground(fit: &[&Case], said: &[SaidCase]) -> FormsGround {
    let mut words = question_words(fit);
    let (slot, freed) = placed_words(fit, &words);
    words.slot = slot;
    words.freed = freed;
    let forms = said.iter().map(|one| plain_cuts(&words).into_iter().map(|cut| form_in(one, &words, cut)).collect()).collect();
    FormsGround { told: first_told(fit), asked: said.iter().map(|one| word_numbers(&one.question)).collect(), forms, words }
}
because!(forms_ground, LearnedWording, "the ground of the forms of the questions of some lessons, found once");

fn checked_ground(said: &[SaidCase], words: &QuestionWords) -> FormsGround {
    let forms = said.iter().map(|one| plain_cuts(words).into_iter().map(|cut| form_in(one, words, cut)).collect()).collect();
    FormsGround { words: QuestionWords { shares: HashMap::new(), questions: usize::default(), text: None, slot: HashSet::new(), freed: HashSet::new() }, told: Vec::new(), asked: said.iter().map(|one| word_numbers(&one.question)).collect(), forms }
}
because!(checked_ground, LearnedWording, "the forms of the questions of the lessons that are checked, under the words of the lessons that the forms were learned from, for each cut that no text names, and each question as numbers, found once for many texts");

fn plain_cuts(words: &QuestionWords) -> Vec<Cut<'static>> {
    let by_places = (!words.slot.is_empty()).then_some(Cut::Slots);
    let by_values = (!words.freed.is_empty()).then_some(Cut::Values);
    TOLD_FIXED_SHARES.iter().map(|told_least| Cut::Share(*told_least)).chain(by_places).chain(by_values).collect()
}
because!(plain_cuts, LearnedWording, "the cuts of cuts() that no text names, in their order there: each share, then the reading by places and by values when some word is placed or freed");

fn plain_place(cut: Cut, words: &QuestionWords) -> Option<usize> {
    match cut {
        Cut::Share(told_least) => TOLD_FIXED_SHARES.iter().position(|share| *share == told_least),
        Cut::Named(..) => None,
        Cut::Slots => Some(TOLD_FIXED_SHARES.len()),
        Cut::Values => Some(TOLD_FIXED_SHARES.len() + usize::from(!words.slot.is_empty())),
    }
}
because!(plain_place, LearnedWording, "the place of a cut among plain_cuts, where the form of a question under it was kept, and none for a cut by the words of a text");

fn form_kept(ground: &FormsGround, (said, at): (&SaidCase, usize), words: &QuestionWords, cut: Cut) -> Option<Pattern> {
    match plain_place(cut, words) {
        Some(place) => ground.forms[at][place].clone(),
        None => form_in(said, words, cut),
    }
}
because!(form_kept, LearnedWording, "the form of a question under a cut: kept in the ground for a cut that no text names, else found");

fn learn_forms_with(topic: &Topic, base: &Facts, (fit, said): (&[&Case], &[SaidCase]), fit_facts: &[Option<Arc<Facts>>]) -> (Vec<LearnedForm>, QuestionWords) {
    let ground = forms_ground(fit, said);
    let (kept, text) = learn_forms_grounded(topic, base, (fit, said, &ground), fit_facts);
    let mut words = ground.words;
    words.text = Some(text);
    (kept, words)
}

fn learn_forms_grounded(topic: &Topic, base: &Facts, (fit, said, ground): (&[&Case], &[SaidCase], &FormsGround), fit_facts: &[Option<Arc<Facts>>]) -> (Vec<LearnedForm>, TextWords) {
    let words = &ground.words;
    let text = rule_words_told(topic, &ground.told);
    let mut groups: Vec<(Pattern, bool, Vec<usize>)> = Vec::new();
    let mut by_share: HashSet<Pattern> = HashSet::new();
    for cut in cuts(&text, words) {
        for (at, case) in fit.iter().enumerate().filter(|(at, _)| fit_facts[*at].is_some()) {
            let Some(form) = form_kept(ground, (&said[at], at), words, cut) else { continue };
            if read_by_share(cut) {
                by_share.insert(form.clone());
            }
            let closed_as_question = case.question.trim_end().ends_with(QUESTION_END);
            match groups.iter_mut().find(|(known, _, _)| *known == form) {
                Some((_, _, taught)) => {
                    if !taught.contains(&at) {
                        taught.push(at);
                    }
                }
                None => groups.push((form, closed_as_question, vec![at])),
            }
        }
    }
    let covered: HashSet<usize> = groups.iter().filter(|(_, _, taught)| taught.len() >= LEAST_SUPPORT).flat_map(|(_, _, taught)| taught.iter().copied()).collect();
    let uncovered: Vec<usize> = (0..fit.len()).filter(|at| fit_facts[*at].is_some() && !covered.contains(at)).collect();
    if uncovered.len() >= LEAST_SUPPORT
        && let Some(form) = run_form(&uncovered.iter().map(|at| fit[*at]).collect::<Vec<&Case>>())
        && !groups.iter().any(|(known, _, _)| *known == form)
    {
        let closed_as_question = fit[uncovered[0]].question.trim_end().ends_with(QUESTION_END);
        by_share.insert(form.clone());
        groups.push((form, closed_as_question, uncovered.clone()));
    }
    for form in question_forms(&uncovered.iter().map(|at| fit[*at]).collect::<Vec<&Case>>()) {
        let taught: Vec<usize> = uncovered.iter().copied().filter(|at| {
            let mut found = Vec::new();
            fit_into(&form, &ground.asked[*at], &empty_filling(), &mut found);
            !found.is_empty()
        }).collect();
        if taught.len() >= LEAST_SUPPORT && !groups.iter().any(|(known, _, _)| *known == form) {
            let closed_as_question = fit[taught[0]].question.trim_end().ends_with(QUESTION_END);
            groups.push((form, closed_as_question, taught));
        }
    }
    let read: Vec<Option<WordingLesson>> = fit.iter().zip(&ground.asked).zip(fit_facts).map(|((case, asked), facts)| Some(lesson_of(asked.clone(), facts.clone()?, answer_words(&case.answer)))).collect();
    let lessons_of = |taught: &[usize]| -> Vec<&WordingLesson> { taught.iter().filter_map(|at| read[*at].as_ref()).collect() };
    let mut forms: Vec<LearnedForm> = Vec::new();
    let mut answered_right: Vec<(usize, HashSet<usize>)> = Vec::new();
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by(|one, other| groups[*other].2.len().cmp(&groups[*one].2.len()));
    for (form, closed_as_question, taught) in order.into_iter().map(|at| &groups[at]) {
        if taught.len() < LEAST_SUPPORT {
            continue;
        }
        let mut lesson_order = taught.clone();
        lesson_order.sort_unstable();
        let taught = &lesson_order;
        let known: Vec<usize> = taught.iter().copied().filter(|at| read[*at].is_some()).collect();
        if !known.is_empty() && answered_right.iter().any(|(count, right)| *count > known.len() && known.iter().all(|at| right.contains(at))) {
            continue;
        }
        let places = places_of(form).len();
        let form_text = library_writing(form);
        let lessons = lessons_of(taught);
        let held: Vec<&WordingLesson> = lessons.iter().take(FORM_SAMPLE).copied().collect();
        let print = sample_print(&held);
        let mut candidates: Vec<WordingCandidate> = Vec::new();
        let mut rows: Vec<Vec<bool>> = Vec::new();
        let rewritten = rewritten_forms(&taught.iter().filter_map(|at| Some((fit[*at], read[*at].as_ref()?.facts.as_ref()))).collect::<Vec<(&Case, &Facts)>>());
        for candidate in single_candidates(topic, form, places, &form_text, Reach { as_told: true, wide: false, timed: false }, &rewritten) {
            if let Some(row) = candidate_row(&candidate, &held, print, true) {
                candidates.push(candidate);
                rows.push(row);
            }
        }
        let broad: Vec<&WordingCandidate> = candidates.iter().zip(&rows).filter(|(candidate, row)| candidate.single.is_some() && row.iter().filter(|holds| **holds).count() * WHOLE_SHARE >= held.len() * BROAD_PERCENT).map(|(candidate, _)| candidate).collect();
        let compared = compared_candidates(&broad, form, &form_text);
        rows.extend(compared.iter().filter_map(|candidate| candidate_row(candidate, &held, print, false)));
        candidates.extend(compared);
        let mut filled: Vec<Option<Vec<bool>>> = vec![None; candidates.len()];
        let mut said_places: Vec<Option<String>> = vec![None; candidates.len()];
        let constants = if by_share.contains(form) { told_constants(form, &taught.iter().map(|at| &said[*at].told).collect::<Vec<&HashSet<String>>>()) } else { Vec::new() };
        let held_facts: Vec<&Facts> = held.iter().map(|lesson| lesson.facts.as_ref()).collect();
        let mut in_order: HashMap<Vec<WordId>, Vec<bool>> = HashMap::new();
        let valued = answers_held(&held);
        let made = if valued { made_facts(topic, &rewritten) } else { Vec::new() };
        let read_filled = filled_candidates(form, places, &form_text, (&made, &constants, &rewritten), (false, &mut |candidate| may_reply_rows(candidate.probe.as_ref(), &held_facts, &mut in_order).contains(&true)));
        let mut worked_seen: HashSet<String> = HashSet::new();
        let mut worked_question = FormReading::default();
        let sample_cases: Vec<&Case> = known.iter().take(held.len()).filter(|_| places == usize::default()).map(|at| fit[*at]).collect();
        let worked: Vec<(WordingCandidate, String)> = worked_conditions(form, &held).into_iter().chain(told_patterns(&sample_cases)).filter_map(|(condition, said)| filled_candidate(&form_text, &condition, said, (&mut worked_seen, &mut worked_question))).collect();
        for (candidate, said) in read_filled.into_iter().chain(worked) {
            let may = may_reply_rows(candidate.probe.as_ref(), &held_facts, &mut in_order);
            let (holds, right) = filled_row(&candidate, &held, &may);
            if right.iter().any(|right| *right) {
                rows.push(holds);
                filled.push(Some(right));
                said_places.push(Some(said));
                candidates.push(candidate);
            }
        }
        if !list_misses(&held, &rows, (&filled, &loose_conditions(&candidates, form))).is_empty() {
            let (asked, filled_asked) = asked_forms(form, &held, base);
            let known: HashSet<(String, Option<String>)> = candidates.iter().zip(&said_places).map(|(candidate, said)| (candidate.text.clone(), said.clone())).collect();
            for candidate in asked_candidates(form, &form_text, &asked).into_iter().filter(|candidate| !known.contains(&(candidate.text.clone(), None))) {
                if let Some(row) = candidate_row(&candidate, &held, print, true) {
                    candidates.push(candidate);
                    rows.push(row);
                    filled.push(None);
                    said_places.push(None);
                }
            }
            for (candidate, said) in asked_filled(form, places, &form_text, &filled_asked).into_iter().filter(|(candidate, said)| !known.contains(&(candidate.text.clone(), Some(said.clone())))) {
                let may = may_reply_rows(candidate.probe.as_ref(), &held_facts, &mut in_order);
                let (holds, right) = filled_row(&candidate, &held, &may);
                if right.iter().any(|right| *right) {
                    rows.push(holds);
                    filled.push(Some(right));
                    said_places.push(Some(said));
                    candidates.push(candidate);
                }
            }
        }
        let reach = Reach { as_told: true, wide: valued, timed: true };
        if !list_misses(&held, &rows, (&filled, &loose_conditions(&candidates, form))).is_empty() {
            let known: HashSet<(String, Option<String>)> = candidates.iter().zip(&said_places).map(|(candidate, said)| (candidate.text.clone(), said.clone())).collect();
            for candidate in single_candidates(topic, form, places, &form_text, reach, &rewritten).into_iter().filter(|candidate| !known.contains(&(candidate.text.clone(), None))) {
                if let Some(row) = candidate_row(&candidate, &held, print, true) {
                    candidates.push(candidate);
                    rows.push(row);
                    filled.push(None);
                    said_places.push(None);
                }
            }
            if valued {
                let mut seen: HashSet<String> = HashSet::new();
                let mut question = FormReading::default();
                let imprecise: Vec<(String, String)> = (0..candidates.len()).filter_map(|at| (rows[at] != *filled[at].as_ref()?).then(|| (format!("{LATEST_START}{}", candidates[at].text), said_places[at].clone().unwrap_or_default()))).collect();
                let mut more: Vec<(WordingCandidate, String)> = imprecise.into_iter().filter_map(|(condition, said)| filled_candidate(&form_text, &condition, said, (&mut seen, &mut question))).collect();
                let (sources, narrow) = source_patterns(topic, form, Reach { as_told: false, wide: true, timed: false }, &rewritten);
                more.extend(filled_candidates(form, places, &form_text, (&sources[narrow..], &[], &rewritten), (true, &mut |candidate| may_reply_rows(candidate.probe.as_ref(), &held_facts, &mut in_order).contains(&true))));
                for (candidate, said) in more.into_iter().filter(|(candidate, said)| !known.contains(&(candidate.text.clone(), Some(said.clone())))) {
                    let may = may_reply_rows(candidate.probe.as_ref(), &held_facts, &mut in_order);
                    let (holds, right) = filled_row(&candidate, &held, &may);
                    if right.iter().any(|right| *right) {
                        rows.push(holds);
                        filled.push(Some(right));
                        said_places.push(Some(said));
                        candidates.push(candidate);
                    }
                }
            }
        }
        if !valued {
            let label_misses: Vec<usize> = list_misses(&held, &rows, (&filled, &loose_conditions(&candidates, form))).into_iter().take(SCREEN_SAMPLE).collect();
            if !label_misses.is_empty() {
                let known: HashSet<String> = candidates.iter().map(|candidate| candidate.text.clone()).collect();
                for candidate in joined_labels(topic, form, &form_text, &rewritten, (&held, print), &label_misses).into_iter().filter(|candidate| !known.contains(&candidate.text)) {
                    if let Some(row) = candidate_row(&candidate, &held, print, false) {
                        candidates.push(candidate);
                        rows.push(row);
                        filled.push(None);
                        said_places.push(None);
                    }
                }
            }
        }
        let sample_cases: Vec<&Case> = taught.iter().filter(|at| read[**at].is_some()).take(FORM_SAMPLE).map(|at| fit[*at]).collect();
        let told = answers_told(&sample_cases);
        let told_answers = told >= LEAST_SUPPORT && told * WHOLE_SHARE >= sample_cases.len() * BROAD_PERCENT;
        let misses: Vec<usize> = if valued && told_answers { list_misses(&held, &rows, (&filled, &loose_conditions(&candidates, form))).into_iter().take(SCREEN_SAMPLE).collect() } else { Vec::new() };
        if !misses.is_empty() {
            let sources = source_patterns(topic, form, reach, &rewritten).0;
            let (seconds, lone) = joined_seconds(&sources, form, places);
            let firsts: Vec<usize> = (0..candidates.len()).filter(|at| candidates[*at].free && rows[*at].iter().filter(|holds| **holds).count() * WHOLE_SHARE >= held.len() * BROAD_PERCENT).collect();
            let gives_right = |candidate: &WordingCandidate, place: usize| reply(std::slice::from_ref(candidate.probe.as_ref()), &held[place].question, &Facts::given_again(&held[place].facts)).is_some_and(|answered| answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP)) == held[place].answer);
            let free_letter = place_letter(places);
            let mut seen: HashSet<String> = HashSet::new();
            let mut question = FormReading::default();
            let mut joined: Vec<(WordingCandidate, String)> = Vec::new();
            for second in lone {
                let Some((candidate, said)) = filled_candidate(&form_text, &second, free_letter.clone(), (&mut seen, &mut question)) else { continue };
                let may = may_reply_rows(candidate.probe.as_ref(), &held_facts, &mut in_order);
                if misses.iter().any(|place| may[*place] && gives_right(&candidate, *place)) {
                    joined.push((candidate, said));
                }
            }
            for first in &firsts {
                if !misses.iter().any(|place| rows[*first][*place]) {
                    continue;
                }
                for (second, answer) in &seconds {
                    let plain = format!("{}{CONDITIONS_JOINED}{second}", candidates[*first].text);
                    let may_hold = filled_probe(&form_text, (answer, &plain), &filled_line(&form_text, &plain, answer), &mut question).is_none_or(|probe| {
                        let words_may = may_reply_rows(&probe, &held_facts, &mut in_order);
                        misses.iter().any(|place| {
                            let facts = Facts::given_again(&held[*place].facts);
                            rows[*first][*place] && words_may[*place] && (reply(std::slice::from_ref(&probe), &held[*place].question, &facts).is_some() || facts.exhausted())
                        })
                    });
                    for timed in SECOND_TIMES {
                        let condition = format!("{}{CONDITIONS_JOINED}{timed}{second}", candidates[*first].text);
                        if !may_hold {
                            seen.insert(filled_line(&form_text, &condition, answer));
                            continue;
                        }
                        if let Some((candidate, said)) = filled_candidate(&form_text, &condition, answer.clone(), (&mut seen, &mut question))
                            && misses.iter().any(|place| rows[*first][*place] && gives_right(&candidate, *place))
                        {
                            joined.push((candidate, said));
                        }
                    }
                }
            }
            for (first, second) in event_joins(&sources, form, places) {
                let plain = format!("{first}{CONDITIONS_JOINED}{second}");
                let may_hold = filled_probe(&form_text, (&free_letter, &plain), &filled_line(&form_text, &plain, &free_letter), &mut question).is_some_and(|probe| {
                    let words_may = may_reply_rows(&probe, &held_facts, &mut in_order);
                    misses.iter().any(|place| words_may[*place] && reply(std::slice::from_ref(&probe), &held[*place].question, &Facts::given_again(&held[*place].facts)).is_some())
                });
                if !may_hold {
                    continue;
                }
                for timed in EVENT_TIMES {
                    let condition = format!("{first}{CONDITIONS_JOINED}{timed}{second}");
                    if let Some((candidate, said)) = filled_candidate(&form_text, &condition, free_letter.clone(), (&mut seen, &mut question))
                        && misses.iter().any(|place| gives_right(&candidate, *place))
                    {
                        joined.push((candidate, said));
                    }
                }
            }
            for (candidate, said) in joined {
                let may = may_reply_rows(candidate.probe.as_ref(), &held_facts, &mut in_order);
                let (holds, right) = filled_row(&candidate, &held, &may);
                rows.push(holds);
                filled.push(Some(right));
                said_places.push(Some(said));
                candidates.push(candidate);
            }
        }
        if !list_misses(&held, &rows, (&filled, &loose_conditions(&candidates, form))).is_empty() {
            let amounts = told_amounts(&sample_cases, form);
            let known: HashSet<String> = candidates.iter().map(|candidate| candidate.text.clone()).collect();
            let mut broad: Vec<usize> = Vec::new();
            for candidate in singles_of((&amounts, amounts.len()), (form, places, &form_text), false, &[]).into_iter().filter(|candidate| !known.contains(&candidate.text)) {
                if let Some(row) = candidate_row(&candidate, &held, print, true) {
                    if candidate.single.is_some() && row.iter().filter(|holds| **holds).count() * WHOLE_SHARE >= held.len() * BROAD_PERCENT {
                        broad.push(candidates.len());
                    }
                    candidates.push(candidate);
                    rows.push(row);
                    filled.push(None);
                    said_places.push(None);
                }
            }
            let compared = compared_candidates(&broad.iter().map(|at| &candidates[*at]).collect::<Vec<&WordingCandidate>>(), form, &form_text);
            for candidate in compared.into_iter().filter(|candidate| !known.contains(&candidate.text)) {
                if let Some(row) = candidate_row(&candidate, &held, print, false) {
                    candidates.push(candidate);
                    rows.push(row);
                    filled.push(None);
                    said_places.push(None);
                }
            }
        }
        if !list_misses(&held, &rows, (&filled, &loose_conditions(&candidates, form))).is_empty() {
            let relations = text_relations(topic);
            let firsts: Vec<String> = candidates
                .iter()
                .zip(&rows)
                .zip(&filled)
                .filter(|((candidate, row), filled)| filled.is_none() && candidate.single.is_some() && row.iter().filter(|holds| **holds).count() * WHOLE_SHARE >= held.len() * BROAD_PERCENT)
                .flat_map(|((candidate, _), _)| [candidate.text.clone(), format!("{LATEST_START}{}", candidate.text)])
                .collect();
            let known: HashSet<String> = candidates.iter().map(|candidate| candidate.text.clone()).collect();
            for text in related_conditions(&firsts, &relations, form).into_iter().filter(|text| !known.contains(text)) {
                if let Some(candidate) = probed(&form_text, &text, None, false)
                    && let Some(row) = candidate_row(&candidate, &held, print, false)
                {
                    candidates.push(candidate);
                    rows.push(row);
                    filled.push(None);
                    said_places.push(None);
                }
            }
        }
        if !list_misses(&held, &rows, (&filled, &loose_conditions(&candidates, form))).is_empty() {
            let mut known: HashSet<String> = candidates.iter().map(|candidate| candidate.text.clone()).collect();
            let holding: Vec<String> = candidates.iter().zip(&filled).filter(|(_, filled)| filled.is_none()).filter_map(|(candidate, _)| value_holding(candidate, places)).collect();
            for text in holding.into_iter().filter(|text| known.insert(text.clone())) {
                if let Some(candidate) = probed(&form_text, &text, None, false)
                    && let Some(row) = candidate_row(&candidate, &held, print, false)
                {
                    candidates.push(candidate);
                    rows.push(row);
                    filled.push(None);
                    said_places.push(None);
                }
            }
        }
        let lists = row_lists(&rows);
        let (distinct, keys) = answer_keys(&held);
        let chosen = decision_list(&keys, &lists, (&filled, &loose_conditions(&candidates, form)), FACT_LIST);
        let labels = list_answers(&keys, &lists, &filled, &chosen);
        let mut answers: Vec<(Vec<String>, String)> = Vec::new();
        let mut probes: Vec<AskedForm> = Vec::new();
        let mut fills: Vec<bool> = Vec::new();
        let mut latest_seen: HashSet<String> = HashSet::new();
        let mut latest_question = FormReading::default();
        for (step, at) in chosen.iter().enumerate() {
            let answer = match &said_places[*at] {
                Some(said) => vec![said.clone()],
                None => {
                    let Some(answer) = labels[step].map(|key| distinct[key].clone()) else { continue };
                    answer
                }
            };
            let held_latest = said_places[*at].as_ref().filter(|_| !candidates[*at].text.starts_with(LATEST_START)).and_then(|said| filled_candidate(&form_text, &format!("{LATEST_START}{}", candidates[*at].text), said.clone(), (&mut latest_seen, &mut latest_question))).filter(|(latest, _)| {
                let may = may_reply_rows(latest.probe.as_ref(), &held_facts, &mut in_order);
                let (holds, right) = filled_row(latest, &held, &may);
                holds == rows[*at] && filled[*at].as_ref() == Some(&right)
            });
            let chosen_one = held_latest.as_ref().map_or(&candidates[*at], |(latest, _)| latest);
            answers.push((answer, chosen_one.text.clone()));
            probes.push(chosen_one.probe.as_ref().clone());
            fills.push(said_places[*at].is_some());
        }
        if answers.is_empty() {
            continue;
        }
        let otherwise = majority_otherwise(&keys, &lists, &chosen, FACT_LIST).map(|key| distinct[key].clone());
        let mut learned = LearnedForm { question: form.clone(), closed_as_question: *closed_as_question, answers, lessons: lessons.len(), right: 0, otherwise, taught: taught.clone(), probes, fills };
        let right: HashSet<usize> = lessons.iter().enumerate().filter(|(_, lesson)| learned_answer(&learned, lesson).as_ref() == Some(&lesson.answer)).map(|(place, _)| known[place]).collect();
        learned.right = right.len();
        learned.taught.sort_unstable();
        answered_right.push((lessons.len(), right));
        forms.push(learned);
    }
    forms.sort_by(|one, other| other.lessons.cmp(&one.lessons).then_with(|| fixed_count(&other.question).cmp(&fixed_count(&one.question))));
    let mut kept: Vec<LearnedForm> = Vec::new();
    for form in forms {
        if !kept.iter().any(|known| known.taught == form.taught && fixed_count(&known.question) > fixed_count(&form.question)) {
            kept.push(form);
        }
    }
    (kept, text)
}
because!(learn_forms_grounded, LearnedWording, "learn_forms_with on a ground found before, with the words of the text found for the text (the forms, and the words of the text that their cuts used)");
because!(learn_forms_with, LearnedWording, "the learned forms of the questions of the first lessons for one text, from the facts of each lesson, each with the answers whose conditions the facts of the text make true, the answers that a step of arithmetic works out from numbers of the question, and for a question with no open place the answers made of the words of the one told sentence of each lesson; only when those miss some lessons of the sample, the same conditions held by the latest fact, and for a form whose answers stand in its facts the filled answers that miss held by the latest fact, the facts that the rules read with no fixed word of the form as a value, and, for a form whose answers the stories of half its lessons tell (a colour or a need told of someone, not true or unknown, the answers of a reading), answers of two facts, the dearest search of a form, for the lessons that the rest still misses; then, for the lessons still missed, the amounts that the stories tell as facts of one fact and their comparisons, and then the value of a broad fact (also held by the latest fact) joined with a relation of the text that ties it to a place of the question (latest the place of A is C and C holds B); a chosen filled answer is held by the latest fact that fits it when so held it answers the same lessons of the sample, as a value that a story may tell more than once (a thing moved twice) is the one told last, while a plain condition takes whichever fact it finds first; the forms with the most lessons first, the forms of worked examples for questions that no cut gives a form for (a sum among choices to pick from), a form passed over when a form with more lessons already answers all of its lessons right, as a question tries the form with the most lessons first; the lessons of a form in the order of the lessons, so that its sample is its first lessons and not those of the reading that found the form first (a form that the reading by values joins holds the colours of every reading); the forms that answer the same lessons kept once, with the most fixed words, and the words of the questions that cut a question into its form; a lesson whose facts run away under the bounds of a trial is left out of the learning, as one long story does not tell how the others are worded");

pub fn learn_wording(name: &str, fit: &[&Case], check: &[&Case], with_told: bool) -> Result<WordingReport, String> {
    let library = library()?;
    let place = library.topics.iter().position(|topic| topic.name == name).ok_or_else(|| format!("{NO_SUCH_TEXT} {name}"))?;
    let told = if with_told { learn_told(&library.topics[place], library.base(place), fit) } else { Vec::new() };
    let (learned, learned_base) = with_learned(&library.topics[place], library.base(place), &told);
    let topic = &learned;
    let base = &learned_base;
    let (kept, words) = learn_forms(topic, base, fit);
    let (learned_right, written_right) = checked_right(topic, base, &kept, &words, check);
    Ok(WordingReport { told, forms: kept, checked: check.len(), learned_right, written_right, seconds: (f64::default(), f64::default()), below_bar: Vec::new() })
}
because!(learn_wording, LearnedWording, "the wording of a kind learned for one named text of the library, with the share on the last lessons of the learned forms beside the written forms of the text");

pub(super) fn checked_right(topic: &Topic, base: &Arc<Facts>, kept: &[LearnedForm], words: &QuestionWords, check: &[&Case]) -> (usize, usize) {
    let facts: Vec<Option<Arc<Facts>>> = check.iter().map(|case| bounded_facts(case, base, topic, true).map(Arc::new)).collect();
    let said: Vec<SaidCase> = check.iter().map(|case| said_case(case)).collect();
    checked_right_with(topic, kept, words, (check, &said), &facts)
}
because!(checked_right, LearnedWording, "how many of the given lessons the learned forms and the written forms answer right, with the facts of each lesson found and kept for later reads");

pub(super) struct Checked {
    pub(super) learned: usize,
    pub(super) written: usize,
    pub(super) wrong: usize,
    pub(super) silent: usize,
}
because!(Checked, LearnedWording, "what the learned forms of a kind give on its last lessons: right, wrong and no answer, beside the right answers of the written forms of the text, so that a report tells a guess that is wrong from a lesson left open");

pub(super) fn checked_answers(topic: &Topic, base: &Arc<Facts>, kept: &[LearnedForm], words: &QuestionWords, check: &[&Case]) -> Checked {
    let facts: Vec<Option<Arc<Facts>>> = check.iter().map(|case| bounded_facts(case, base, topic, true).map(Arc::new)).collect();
    let said: Vec<SaidCase> = check.iter().map(|case| said_case(case)).collect();
    checked_tally(topic, kept, words, (check, &said), &facts)
}
because!(checked_answers, LearnedWording, "checked_right with the wrong answers and the lessons left with no answer of the learned forms counted apart");

fn checked_right_with(topic: &Topic, kept: &[LearnedForm], words: &QuestionWords, (check, said): (&[&Case], &[SaidCase]), check_facts: &[Option<Arc<Facts>>]) -> (usize, usize) {
    let checked = checked_tally(topic, kept, words, (check, said), check_facts);
    (checked.learned, checked.written)
}

fn checked_tally(topic: &Topic, kept: &[LearnedForm], words: &QuestionWords, (check, said): (&[&Case], &[SaidCase]), check_facts: &[Option<Arc<Facts>>]) -> Checked {
    checked_tally_on(topic, kept, (words, None), (check, said, None), check_facts)
}

fn checked_tally_on(topic: &Topic, kept: &[LearnedForm], (words, text_found): (&QuestionWords, Option<&TextWords>), (check, said, ground): (&[&Case], &[SaidCase], Option<&FormsGround>), check_facts: &[Option<Arc<Facts>>]) -> Checked {
    let (mut learned_right, mut written_right, mut learned_wrong, mut silent) = (0usize, 0usize, 0usize, 0usize);
    let own;
    let text = match text_found.or(words.text.as_ref()) {
        Some(text) => text,
        None => {
            own = rule_words(topic, check);
            &own
        }
    };
    for (at, ((case, said), facts)) in check.iter().zip(said).zip(check_facts).enumerate() {
        let Some(facts) = facts else {
            silent += 1;
            continue;
        };
        let expected = answer_words(&case.answer);
        let asked = ground.map_or_else(|| word_numbers(&said.question), |ground| ground.asked[at].clone());
        if reply(&topic.questions, &asked, &Facts::given_again(facts)).is_some_and(|answered| answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP)) == expected) {
            written_right += 1;
        }
        let lesson = lesson_of(asked, Arc::clone(facts), expected);
        if kept.is_empty() {
            silent += 1;
            continue;
        }
        let mut readings: Vec<&LearnedForm> = cuts(text, words).into_iter().filter_map(|cut| ground.map_or_else(|| form_in(said, words, cut), |ground| form_kept(ground, (said, at), words, cut))).filter_map(|form| kept.iter().find(|learned| learned.question == form)).collect();
        if readings.is_empty() {
            readings = kept.iter().filter(|learned| run_fits(&learned.question, &lesson.question)).collect();
        }
        readings.sort_by(|one, other| other.lessons.cmp(&one.lessons));
        match readings.iter().find_map(|learned| learned_answer(learned, &lesson)) {
            Some(answer) if answer == lesson.answer => learned_right += 1,
            Some(_) => learned_wrong += 1,
            None => silent += 1,
        }
    }
    Checked { learned: learned_right, written: written_right, wrong: learned_wrong, silent }
}
because!(checked_tally_on, LearnedWording, "checked_tally with the forms of the checked questions under the cuts that no text names, and the questions as numbers, kept before when a ground is given");
because!(checked_tally, LearnedWording, "the answers of the learned forms on the given lessons, right, wrong or none, and the right answers of the written forms, from the facts of each lesson; a lesson whose facts ran past their bound has no answer");
because!(checked_right_with, LearnedWording, "how many of the given lessons the learned forms answer right, and how many the written forms of the text answer right, from the facts of each lesson, each read on a new layer with its own count of the facts read; a lesson whose question gives no learned form reads by a learned run form that it fits");

pub fn form_line(form: &LearnedForm) -> String {
    written_form(&form.question, form.closed_as_question, &form.answers, form.otherwise.as_ref())
}
because!(form_line, LearnedWording, "a learned form as a line of the library, so that a person reads what was learned in the words of a written text");

pub(super) fn written_form(question: &Pattern, closed_as_question: bool, answers: &[(Vec<String>, String)], otherwise: Option<&Vec<String>>) -> String {
    let end = if closed_as_question { QUESTION_END } else { STATEMENT_END };
    let mut written: Vec<String> = answers.iter().map(|(answer, condition)| format!("{}{LEARNED_IF}{condition}{STATEMENT_END}", answer.join(FORM_WORD_GAP))).collect();
    written.extend(otherwise.iter().map(|answer| format!("{}{LEARNED_OTHERWISE}{STATEMENT_END}", answer.join(FORM_WORD_GAP))));
    format!("{LINE_START}{}{end}{LINE_MIDDLE}{}", library_writing(question), written.join(FORM_WORD_GAP))
}
because!(written_form, LearnedWording, "a question form with its answers and conditions in order and its answer otherwise, as a line of the library writes it");

pub(super) const RELEVANCE_SAMPLE: usize = 20;
because!(RELEVANCE_SAMPLE, LearnedWording, "the lessons on which a text must show that its rules read the stories of a kind before its wording is learned, so that a text that reads nothing of a kind costs little");

const TRIAL_FACTS: usize = 2_000;
because!(TRIAL_FACTS, LearnedWording, "the new facts that the rules of a text may make from one story of the sample: a text that reads a kind makes a few facts for each told sentence, and a text whose rules make far more runs away on that kind, as a rule whose made fact fits its own condition does");

const TRIAL_READS: usize = 20_000;
because!(TRIAL_READS, LearnedWording, "the facts that the engine may read while it chains one story of the sample for a text: a text that reads a kind needs far fewer, and a text whose rules join many facts with many on that kind runs away, as the board would show on every story of the kind");

const PROMISE_SUPPORT: usize = 2;

static PROMISES: OnceLock<Mutex<HashMap<(usize, u64), Worked<(usize, usize)>>>> = OnceLock::new();
because!(PROMISES, LearnedWording, "what each text promised for a sample of stories, kept for the run, by its single facts and, when they promise nothing, by its learned wording: the board learns a kind several times from parts of its lessons that start with the same stories, and a text that read nothing of them the first time is not chained again");

static SEARCHED: OnceLock<Mutex<HashMap<(usize, u64), Worked<Option<String>>>>> = OnceLock::new();
because!(SEARCHED, LearnedWording, "the wording that each text learned from the first lessons of a search, by the text and the lessons, kept for the run: the board learns a kind several times from parts of its lessons that start with the same lessons, so the search learns the same wording again; a text that ran away on them or learned no form is not chained again");

static TOLD_LEARNED: OnceLock<Mutex<HashMap<(usize, u64, u64), LearnedCell>>> = OnceLock::new();
because!(TOLD_LEARNED, LearnedWording, "the rules that read the told sentences which each text learned from some lessons, by the text, its rules and the lessons, kept for the run: the board learns a kind several times from parts of its lessons that start with the same lessons (the bar on the first part, the search, the run with few lessons), and the told learner judges many readings each time");

const LEARN_LESSONS: usize = 1_000;
because!(LEARN_LESSONS, LearnedWording, "the most lessons from which the wording of the chosen text is learned, as many as the board gives a kind");

const SEARCH_LESSONS: usize = 100;
because!(SEARCH_LESSONS, LearnedWording, "the first lessons from which the wording of each promising text is learned while the texts are compared, fewer than the board's own test of learning from few lessons gives, so that every learning of a kind on the board searches on the same lessons and a text that does not read a kind costs little; only the chosen text is learned again from all lessons");
because!(PROMISE_SUPPORT, LearnedWording, "the lessons of the sample on which a form and one of its conditions must stand before the wording of a text is learned on all lessons");

const LEAST_FIRED_PERCENT: usize = 50;
because!(LEAST_FIRED_PERCENT, LearnedWording, "the share of that sample on which the rules of a text must make a new fact");

const LEAST_LEARNED_PERCENT: usize = 50;
because!(LEAST_LEARNED_PERCENT, LearnedWording, "the share of the last lessons that a learned wording must answer right to be kept, the same trust as a written topic needs on its sample");

pub(super) const LEARNED_LINE_BREAK: &str = "\n";
because!(LEARNED_LINE_BREAK, LearnedWording, "the end of a line between the learned lines of a text");

const LEARNED_NOTE: &str = "learned wording for the text";
because!(LEARNED_NOTE, LearnedWording, "what the note of an answer says when a learned line gave it, before the name of the text and the line");

const LEARNED_NONE: &str = "no learned answer holds";
because!(LEARNED_NONE, LearnedWording, "the note of the learned wording when no learned answer holds for a question");

pub struct WordedModel {
    topic: Topic,
    base: Arc<Facts>,
    source: String,
    hand: bool,
    learned: Vec<String>,
    taught: Vec<usize>,
    named: (usize, usize),
    frames: Option<SeenFrames>,
}
because!(WordedModel, LearnedWording, "a model of the explain tool whose question forms were learned from the lessons of the kind: the rules and facts of one text of the library with the learned lines as its question forms, its facts before any story, the name of the text, and whether that text, or one it builds on, brings rules of a wording written by a person, so that the model reads mixed and not learned; the lines that it learned from the lessons (rules that read the told sentences, rules of a concept, lines that write a sentence again, then the question forms), as lines of the library; and the kinds before its kind in its set that taught some of those lines, by their places in the order of the set before the kind; and, for the wording learned again from all lessons, how many of its forms name a word that a form with fewer fixed words reads too, of all its forms; and for the model of a kind, the frames of the sentences of its lessons, by which it marks a guess");

impl WordedModel {
    pub fn framed(self, cases: &[Case]) -> WordedModel {
        let frames = SeenFrames::of(&[&self.topic], cases);
        WordedModel { frames: Some(frames), ..self }
    }
}

impl Learner for WordedModel {
    fn used_kinds_before(&self) -> bool {
        !self.taught.is_empty()
    }

    fn kinds_used_before(&self) -> Vec<usize> {
        self.taught.clone()
    }

    fn reading(&self) -> Option<Reading> {
        Some(if self.hand { Reading::Mixed } else { Reading::Learned })
    }

    fn missing(&self, sentences: &[String], question: &str, expected: &[String]) -> Vec<String> {
        missing_step(&self.topic, &self.base, sentences, question, expected).unwrap_or_default()
    }

    fn proof(&self, sentences: &[String], question: &str) -> Vec<String> {
        answer_steps(&self.topic, &self.base, &told_of(sentences, question, &self.topic.rules), &fact_numbers(question), false).map(|answered| proof_text(&self.source, &answered)).unwrap_or_default()
    }

    fn rules_used(&self, sentences: &[String], question: &str) -> Vec<(String, bool)> {
        answer_steps(&self.topic, &self.base, &told_of(sentences, question, &self.topic.rules), &fact_numbers(question), false).map(|answered| used_rules(&answered)).unwrap_or_default()
    }

    fn facts_used(&self, sentences: &[String], question: &str) -> Vec<(String, bool)> {
        answer_steps(&self.topic, &self.base, &told_of(sentences, question, &self.topic.rules), &fact_numbers(question), false).map(|answered| used_facts(&answered)).unwrap_or_default()
    }

    fn lines_from_lessons(&self) -> Vec<String> {
        self.learned.clone()
    }

    fn named_words(&self) -> Option<(usize, usize)> {
        (self.named.1 > usize::default()).then_some(self.named)
    }

    fn answer_counts(&self, question: &str, lessons: &[Case]) -> Vec<String> {
        form_counts(&self.topic, &self.base, question, lessons)
    }

    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let facts = chain_facts(&told_of(sentences, question, &self.topic.rules), &self.base, &self.topic.rules, self.topic.rules_key);
        match reply(&self.topic.questions, &fact_numbers(question), &facts) {
            Some(answered) => {
                let words = answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP));
                let mark = answer_mark(&words, answered.reply.text.ends_with(LEARNED_OTHERWISE), (self.frames.as_ref(), sentences, question));
                Said { words, note: format!("{LEARNED_NOTE} {}: {}", self.source, answered.reply.text), mark }
            }
            None => Said { words: Vec::new(), note: LEARNED_NONE.to_string(), mark: AnswerMark::NoAnswer },
        }
    }
}

fn form_counts(topic: &Topic, base: &Arc<Facts>, question: &str, lessons: &[Case]) -> Vec<String> {
    let fits = |form: &AskedForm, words: &[WordId]| {
        let mut found = Vec::new();
        fit_into(&form.question, words, &empty_filling(), &mut found);
        !found.is_empty()
    };
    let Some(form) = topic.questions.iter().find(|form| fits(form, &fact_numbers(question))) else { return Vec::new() };
    let mut counts: Vec<HashMap<String, usize>> = vec![HashMap::new(); form.replies.len() + 1];
    for case in lessons.iter().filter(|case| fits(form, &fact_numbers(&case.question))).take(FORM_SAMPLE) {
        let words = fact_numbers(&case.question);
        let facts = chain_facts(&told_of(&case.sentences, &case.question, &topic.rules), base, &topic.rules, topic.rules_key);
        let answer = answer_words(&case.answer).join(FORM_WORD_GAP);
        let held: Vec<usize> = (0..form.replies.len())
            .filter(|at| !form.replies[*at].conditions.is_empty())
            .filter(|at| reply(&[AskedForm { question: form.question.clone(), replies: vec![form.replies[*at].clone()] }], &words, &facts).is_some())
            .collect();
        for at in held.iter().copied().chain(held.is_empty().then_some(form.replies.len())) {
            *counts[at].entry(answer.clone()).or_default() += 1;
        }
    }
    let tally = |counted: &HashMap<String, usize>| {
        let mut ranked: Vec<(&String, &usize)> = counted.iter().collect();
        ranked.sort_by(|one, other| other.1.cmp(one.1).then_with(|| one.0.cmp(other.0)));
        if ranked.is_empty() { NO_LESSON.to_string() } else { ranked.iter().map(|(answer, count)| format!("{answer} {count}")).collect::<Vec<String>>().join(COUNTS_BETWEEN) }
    };
    let mut lines: Vec<String> = form.replies.iter().enumerate().filter(|(_, one)| !one.conditions.is_empty()).map(|(at, one)| format!("{}: {}", one.text, tally(&counts[at]))).collect();
    lines.push(format!("{NO_CONDITION_HOLDS}: {}", tally(&counts[form.replies.len()])));
    lines
}
because!(form_counts, LearnedWording, "for the first form of a text that fits a question, each answer with a condition and the lessons of the form, among its first lessons, where that condition holds, by the answers of those lessons, and the lessons where no condition holds, by their answers: a condition that holds on lessons of another answer, or lessons that no condition reads, show at once why the form answers some lessons wrong (maybe if the place of A is B or C held on six maybe, two no and one yes lessons of babi10)");

const NO_LESSON: &str = "no lesson";
because!(NO_LESSON, LearnedWording, "what the counts of an answer say when its condition holds on no lesson of the form");

const COUNTS_BETWEEN: &str = ", ";
because!(COUNTS_BETWEEN, LearnedWording, "the marks between the answers of the lessons where a condition holds, with their counts");

const NO_CONDITION_HOLDS: &str = "no condition holds";
because!(NO_CONDITION_HOLDS, LearnedWording, "the words before the lessons of a form where no answer with a condition holds, by their answers, which the answer otherwise of the form gives or nothing");

fn rewritten_facts(case: &Case, facts: &Facts) -> Vec<Vec<WordId>> {
    let told: Vec<Vec<WordId>> = case.sentences.iter().map(|sentence| fact_numbers(sentence)).chain([fact_numbers(&format!("{QUESTION_TOLD}{}", case.question))]).collect();
    facts.told_facts().into_iter().filter(|fact| !told.iter().any(|sentence| sentence.as_slice() == *fact)).map(<[WordId]>::to_vec).collect()
}
because!(rewritten_facts, LearnedWording, "the told sentences of a lesson that a line of the text wrote in a new form, as the facts that they became; a sentence that no line wrote again is left out");

fn rewritten_forms(lessons: &[(&Case, &Facts)]) -> Vec<Pattern> {
    let rewritten: Vec<Vec<Vec<WordId>>> = lessons.iter().map(|(case, facts)| rewritten_facts(case, facts)).filter(|facts| !facts.is_empty()).collect();
    sentence_forms(&rewritten, false, 1)
}
because!(rewritten_forms, LearnedWording, "the forms of the told sentences that the lines of a text wrote again in some lessons");

fn told_amounts(cases: &[&Case], form: &Pattern) -> Vec<Pattern> {
    let fixed: HashSet<WordId> = form.iter().filter_map(|logic_piece| match logic_piece { LogicPiece::Word(word) => Some(*word), _ => None }).collect();
    let told: Vec<Vec<Vec<WordId>>> = cases
        .iter()
        .map(|case| {
            let asked: HashSet<WordId> = fact_numbers(&case.question).into_iter().filter(|word| !fixed.contains(word) && !is_value(word_text(*word))).collect();
            case.sentences.iter().map(|sentence| fact_numbers(sentence)).filter(|words| words.iter().any(|word| is_value(word_text(*word))) && words.iter().any(|word| asked.contains(word))).collect::<Vec<Vec<WordId>>>()
        })
        .filter(|facts| !facts.is_empty())
        .collect();
    sentence_forms(&told, true, LEAST_SUPPORT).into_iter().filter(|pattern| fixed_count(pattern) > usize::default()).collect()
}
because!(told_amounts, LearnedWording, "the forms of the told sentences of the lessons of a form that hold a number and a word of a place of the question, each number a place of its own (a person has a number of things: A has B C), with a fixed word at least and in enough lessons: an amount that the story tells of what the question names, which a condition may compare with a number of the question or with another amount (entailment if A has D C and D is at least B); a number that names a clue of a list or a house is told of nothing that the question asks about in the same sentence, mostly, and its forms stay few");

fn text_relations(topic: &Topic) -> Vec<Pattern> {
    let facts = topic.rules.iter().flat_map(|rule| {
        rule.conditions
            .iter()
            .filter_map(|condition| match condition {
                Condition::Fact { pattern, .. } => Some(pattern),
                _ => None,
            })
            .chain([&rule.outcome])
    });
    let mut out: Vec<Pattern> = Vec::new();
    for pattern in facts {
        let fresh = places_of(pattern).into_iter().max().map_or(usize::default(), |most| most + 1);
        let mut joined: Pattern = Vec::new();
        let mut at = usize::default();
        while at < pattern.len() {
            if let (Some(LogicPiece::Open(_)), Some(LogicPiece::Word(word)), Some(LogicPiece::Open(_))) = (pattern.get(at), pattern.get(at + 1), pattern.get(at + RUN_REST))
                && RELATION_JOINS.contains(&word_text(*word))
            {
                joined.push(LogicPiece::Open(fresh));
                at += RUN_REST + 1;
                continue;
            }
            joined.push(pattern[at].clone());
            at += 1;
        }
        let from = places_of(&joined);
        if joined.len() == pattern.len() || from.len() != RELATION_PLACES {
            continue;
        }
        let relation = renamed(&joined, &from, &(0..from.len()).collect::<Vec<usize>>());
        if !out.contains(&relation) {
            out.push(relation);
        }
    }
    out
}
because!(text_relations, LearnedWording, "the relations of two values that the facts of a text make or read, where one value is a run of places joined by or or and (B or C holds B: the value B or C holds B), with that run read as one place (E holds B), so that a value of the story can be tied to a word of the question; only such facts, so that the relations stay few");

const RELATION_JOINS: &[&str] = &["or", "and"];
because!(RELATION_JOINS, LearnedWording, "the words that join two places into one value of a fact, as a person who is either in the B or the C is in the place B or C");

const RUN_REST: usize = 2;
because!(RUN_REST, LearnedWording, "the pieces after the first place of a run of places joined by a word: the word and the next place");

const RELATION_PLACES: usize = 2;
because!(RELATION_PLACES, LearnedWording, "the places of a relation of the text: the value of a fact of the story and a word of the question");

fn related_conditions(firsts: &[String], relations: &[Pattern], form: &Pattern) -> Vec<String> {
    let own = places_of(form);
    let free = own.len();
    let mut out: Vec<String> = Vec::new();
    for relation in relations {
        for place in &own {
            for to in [[free, *place], [*place, free]] {
                let tie = library_writing(&with_form_places(&renamed(relation, &[usize::default(), 1], &to), form));
                out.extend(firsts.iter().map(|first| format!("{first}{CONDITIONS_JOINED}{tie}")));
            }
        }
    }
    out
}
because!(related_conditions, LearnedWording, "the conditions that join a broad fact with a free value to a relation of the text between that value and a place of the question, either way round (the place of A is C and C holds B), so that a question is answered by what the latest fact of the story makes of a value and not by the value alone");

fn sentence_forms(sentences: &[Vec<Vec<WordId>>], numbers_apart: bool, least: usize) -> Vec<Pattern> {
    let mut shares: HashMap<WordId, usize> = HashMap::new();
    for facts in sentences {
        let words: HashSet<WordId> = facts.iter().flatten().copied().collect();
        for word in words {
            *shares.entry(word).or_default() += 1;
        }
    }
    let fixed = |word: WordId| !is_value(word_text(word)) && shares.get(&word).copied().unwrap_or_default() * WHOLE_SHARE >= sentences.len() * REWRITTEN_FIXED_PERCENT;
    let mut counted: Vec<(Pattern, usize)> = Vec::new();
    for fact in sentences.iter().flatten() {
        let mut pattern: Pattern = Vec::new();
        let mut used = (usize::default(), usize::default());
        let mut run: Vec<WordId> = Vec::new();
        let mut fits = true;
        for word in fact.iter().copied().chain([WordId::MAX]) {
            let open = word != WordId::MAX && !fixed(word);
            let apart = numbers_apart && open && run.last().is_some_and(|last| is_value(word_text(*last)) || is_value(word_text(word)));
            if open && !apart {
                run.push(word);
                continue;
            }
            if !run.is_empty() {
                let Some(place) = run_place(&run, &mut used) else {
                    fits = false;
                    break;
                };
                pattern.push(LogicPiece::Open(place));
                run.clear();
            }
            if apart {
                run.push(word);
            } else if word != WordId::MAX {
                pattern.push(LogicPiece::Word(word));
            }
        }
        if !fits {
            continue;
        }
        match counted.iter_mut().find(|(known, _)| *known == pattern) {
            Some((_, count)) => *count += 1,
            None => counted.push((pattern, 1)),
        }
    }
    counted.sort_by(|one, other| other.1.cmp(&one.1));
    counted.into_iter().filter(|(_, count)| *count >= least).take(MOST_REWRITTEN_FORMS).map(|(pattern, _)| pattern).collect()
}
because!(sentence_forms, LearnedWording, "the forms of told sentences in some lessons, the most common first: a word that stands in most of those lessons stays, a number never, and each run of other words becomes one open place, of words when the run holds no mark, else of any signs; when asked, each number a place of its own; a form that needs more places than there are is left out, and one of fewer sentences than asked");

fn run_place(run: &[WordId], (words_used, signs_used): &mut (usize, usize)) -> Option<usize> {
    if run.iter().all(|one| word_not_mark(word_text(*one))) {
        *words_used += 1;
        (*words_used <= PLAIN_PLACES).then(|| *words_used - 1)
    } else {
        *signs_used += 1;
        (ANY_FIRST + *signs_used <= OPEN_PLACES).then(|| ANY_FIRST + *signs_used - 1)
    }
}
because!(run_place, LearnedWording, "the open place of a run of words in the form of a told sentence: the next place of words when the run holds no mark, else the next place of any signs, and nothing when that kind has no place left");

fn sample_words(cases: &[&Case]) -> Vec<Vec<WordId>> {
    cases.iter().take(RELEVANCE_SAMPLE).flat_map(|case| told_of(&case.sentences, &case.question, &[]).iter().map(|sentence| fact_numbers(sentence)).collect::<Vec<Vec<WordId>>>()).collect()
}
because!(sample_words, LearnedWording, "the words of the told sentences of the sample of a kind (the question of a case that tells no sentence), read once for all the texts that a search tries on the sample");

fn may_fire(topic: &Topic, told: &[Vec<WordId>]) -> bool {
    topic.rules.iter().flat_map(|rule| &rule.conditions).any(|condition| match condition {
        Condition::Fact { pattern, .. } => told.iter().any(|sentence| holds_words(pattern, sentence)),
        _ => false,
    })
}
because!(may_fire, LearnedWording, "whether some condition of a rule of a text holds its fixed words in order in some told sentence of the sample (the question of a case that tells no sentence): when none does, no rule of the text can start on the stories of the kind, and the text is passed over before any chaining");

fn trial(topic: &Topic, base: &Arc<Facts>, (cases, told): (&[&Case], &[Vec<WordId>]), least_fired: usize) -> Option<Vec<Facts>> {
    if !may_fire(topic, told) {
        return None;
    }
    let mut out: Vec<Facts> = Vec::new();
    let mut fired = 0usize;
    for case in cases.iter().take(RELEVANCE_SAMPLE) {
        let facts = bounded_facts(case, base, topic, true)?;
        if facts.added_to(base) > told_of(&case.sentences, &case.question, &topic.rules).len() || !rewritten_facts(case, &facts).is_empty() {
            fired += 1;
        }
        out.push(facts);
    }
    (!out.is_empty() && fired * WHOLE_SHARE >= out.len() * least_fired).then_some(out)
}
because!(trial, LearnedWording, "the facts of a small sample of stories for one text, when its rules make a new fact from the given share of them, or write a told sentence of them again, and run away on none, so that the wording is learned only for a text that reads the kind and does not cost the bound of the engine on every story; the words of the told sentences of the sample come read once for all texts");

const CONCEPT_FIRED_PERCENT: usize = 25;
because!(CONCEPT_FIRED_PERCENT, LearnedWording, "the share of the sample on which the rules of a concept with question forms must make a new fact when no text reads the kind as a learned kind must: the stories of a kind of labels may name the relation of a concept in fewer than half of them (a person went with a relative or with a stranger), and its absence is then the answer");

const MOST_RELEARNED: usize = 3;
because!(MOST_RELEARNED, LearnedWording, "the most texts whose wording is learned again from all the first lessons when the search on fewer lessons found no text that reads the kind as a learned kind must: a text with many forms of questions learns few of them from a hundred lessons, while learning from all costs each text far more");

fn promise(topic: &Topic, base: &Facts, cases: &[&Case], facts: &[Facts], words: &QuestionWords, as_told: bool) -> usize {
    let text = rule_words(topic, cases);
    let nothing = Arc::new(Facts::default());
    let read: Vec<(WordingLesson, WordingLesson)> = cases
        .iter()
        .zip(facts)
        .map(|(case, facts)| (lesson_of(fact_numbers(&case.question), Arc::new(facts.clone()), answer_words(&case.answer)), lesson_of(fact_numbers(&case.question), Arc::new(chain_facts(&told_of(&case.sentences, &case.question, &[]), &nothing, &[], rules_key(&[]))), Vec::new())))
        .collect();
    let mut known: HashMap<(Pattern, Vec<usize>, bool), usize> = HashMap::new();
    let mut best = 0usize;
    for cut in cuts(&text, words) {
        best = best.max(reading_promise(topic, base, (cases, &read), words, (cut, as_told), &mut known));
    }
    best
}
because!(promise, LearnedWording, "how much a text promises a wording for a kind on the small sample, in the best of the readings of the words of the questions, each form with the same lessons judged once for all the readings; with the form of a question as a fact too when the facts were read by learned rules, which may read a told sentence into the fact that a question states");

fn learned_promise(topic: &Topic, base: &Facts, cases: &[&Case], facts: &[Facts]) -> usize {
    let lessons: Vec<WordingLesson> = cases.iter().zip(facts).map(|(case, facts)| lesson_of(fact_numbers(&case.question), Arc::new(facts.clone()), answer_words(&case.answer))).collect();
    let held: Vec<&WordingLesson> = lessons.iter().collect();
    if !answers_held(&held) && !answers_numbered(&held) && !patterns_told(cases) {
        return 0;
    }
    let kept: Vec<Option<Arc<Facts>>> = lessons.iter().map(|lesson| Some(Arc::clone(&lesson.facts))).collect();
    let said: Vec<SaidCase> = cases.iter().map(|case| said_case(case)).collect();
    learn_forms_with(topic, base, (cases, &said), &kept).0.iter().map(|form| form.right).sum()
}
because!(learned_promise, LearnedWording, "how much a text promises a wording for a kind whose answers are values of its facts, numbers that a step of arithmetic may work out, or made of the words of the one told sentence, when no single fact promises it: the lessons of the small sample that the wording learned on that sample answers right, as an answer held only by the latest fact or reached through a second fact is not a single fact with one answer");

fn reading_promise(topic: &Topic, base: &Facts, (cases, read): (&[&Case], &[(WordingLesson, WordingLesson)]), words: &QuestionWords, (cut, as_told): (Cut, bool), known: &mut HashMap<(Pattern, Vec<usize>, bool), usize>) -> usize {
    let mut groups: Vec<(Pattern, Vec<usize>)> = Vec::new();
    let mut formless: Vec<usize> = Vec::new();
    for (at, case) in cases.iter().enumerate().take(read.len()) {
        let Some(form) = form_of(case, words, cut) else {
            formless.push(at);
            continue;
        };
        match groups.iter_mut().find(|(known, _)| *known == form) {
            Some((_, members)) => members.push(at),
            None => groups.push((form, vec![at])),
        }
    }
    let (kept, small): (Vec<_>, Vec<_>) = groups.into_iter().partition(|(_, members)| members.len() >= PROMISE_SUPPORT);
    let mut groups = kept;
    formless.extend(small.into_iter().flat_map(|(_, members)| members));
    if formless.len() >= PROMISE_SUPPORT
        && let Some(form) = run_form(&formless.iter().map(|at| cases[*at]).collect::<Vec<&Case>>())
    {
        groups.push((form, formless));
    }
    let mut promised = 0usize;
    for (form, members) in groups.into_iter().filter(|(_, members)| members.len() >= PROMISE_SUPPORT) {
        let key = (form, members, read_by_share(cut));
        if let Some(best) = known.get(&key) {
            promised += best;
            continue;
        }
        let best = form_promise(topic, base, &key.0, &key.1.iter().map(|at| (&read[*at].0, &read[*at].1, cases[*at])).collect::<Vec<(&WordingLesson, &WordingLesson, &Case)>>(), (as_told, key.2));
        promised += best;
        known.insert(key, best);
    }
    promised
}
because!(reading_promise, LearnedWording, "how much a text promises a wording for a kind on the small sample in one reading of the words of the questions: the promises of its forms summed, so that a text that reads every question of a kind comes before a text that reads one by chance; a form with the same lessons as in a reading before is not judged again");

fn form_promise(topic: &Topic, base: &Facts, form: &Pattern, lessons: &[(&WordingLesson, &WordingLesson, &Case)], (as_told, by_share): (bool, bool)) -> usize {
    let places = places_of(form).len();
    let form_text = library_writing(form);
    let said: Vec<SaidCase> = if by_share { lessons.iter().map(|(_, _, case)| said_case(case)).collect() } else { Vec::new() };
    let constants = if by_share { told_constants(form, &said.iter().map(|one| &one.told).collect::<Vec<&HashSet<String>>>()) } else { Vec::new() };
    let rewritten = rewritten_forms(&lessons.iter().map(|(lesson, _, case)| (*case, lesson.facts.as_ref())).collect::<Vec<(&Case, &Facts)>>());
    let lesson_facts: Vec<&Facts> = lessons.iter().map(|(lesson, _, _)| lesson.facts.as_ref()).collect();
    let mut in_order: HashMap<Vec<WordId>, Vec<bool>> = HashMap::new();
    let filled_best = |candidates: Vec<(WordingCandidate, String)>, in_order: &mut HashMap<Vec<WordId>, Vec<bool>>| -> usize {
        let mut best = 0usize;
        for (candidate, _) in candidates {
            let mut held = 0usize;
            let mut right = 0usize;
            let may = may_reply_rows(candidate.probe.as_ref(), &lesson_facts, in_order);
            for ((lesson, told, _), may) in lessons.iter().zip(&may) {
                if !*may {
                    continue;
                }
                let Some(answered) = reply(std::slice::from_ref(candidate.probe.as_ref()), &lesson.question, &Facts::given_again(&lesson.facts)) else { continue };
                if reply(std::slice::from_ref(candidate.probe.as_ref()), &told.question, &Facts::given_again(&told.facts)).is_some() {
                    continue;
                }
                held += 1;
                right += usize::from(answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP)) == lesson.answer);
            }
            if held >= PROMISE_SUPPORT && right * WHOLE_SHARE >= held * LEAST_RIGHT_PERCENT {
                best = best.max(right);
            }
        }
        best
    };
    let single_best = |candidates: Vec<WordingCandidate>| -> usize {
        let mut best = 0usize;
        for candidate in candidates {
            let held: Vec<&WordingLesson> = lessons.iter().filter(|(lesson, told, _)| condition_holds(&candidate, lesson) && !condition_holds(&candidate, told)).map(|(lesson, _, _)| *lesson).collect();
            let mut by_answer: Vec<(&Vec<String>, usize)> = Vec::new();
            for lesson in &held {
                match by_answer.iter_mut().find(|(answer, _)| **answer == lesson.answer) {
                    Some((_, count)) => *count += 1,
                    None => by_answer.push((&lesson.answer, 1)),
                }
            }
            let right = by_answer.iter().map(|(_, count)| *count).max().unwrap_or_default();
            if held.len() >= PROMISE_SUPPORT && right * WHOLE_SHARE >= held.len() * LEAST_RIGHT_PERCENT {
                best = best.max(right);
            }
        }
        best
    };
    let read_filled = filled_candidates(form, places, &form_text, (&made_facts(topic, &rewritten), &constants, &rewritten), (false, &mut |candidate| may_reply_rows(candidate.probe.as_ref(), &lesson_facts, &mut in_order).contains(&true)));
    let mut best = filled_best(read_filled, &mut in_order);
    best = best.max(single_best(single_candidates(topic, form, places, &form_text, Reach { as_told, wide: false, timed: false }, &rewritten)));
    if best == usize::default() {
        let (asked, filled_asked) = asked_forms(form, &lessons.iter().map(|(lesson, _, _)| *lesson).collect::<Vec<&WordingLesson>>(), base);
        best = filled_best(asked_filled(form, places, &form_text, &filled_asked), &mut in_order).max(single_best(asked_candidates(form, &form_text, &asked)));
    }
    best
}
because!(form_promise, LearnedWording, "how much one form of the questions of the sample promises: the most lessons on which a fact that a rule of the text makes holds, where the told sentences alone do not make it hold, and gives one answer right as often as a learned answer must (a label, or the words that it fills, a fixed word of the form among them only for a form of a reading by a share); nothing when no condition holds on a few lessons");

pub fn worded_model(library: &Library, lessons: &Lessons) -> Option<WordedModel> {
    worded_search(library, lessons, true)
}
because!(worded_model, LearnedWording, "the explain tool for a kind that no written question form reads: the learned wording of a text of rules, with its told side when no wording alone reads the kind");

pub(crate) fn worded_search(library: &Library, lessons: &Lessons, with_told: bool) -> Option<WordedModel> {
    let (first, last) = inner_parts(lessons.cases);
    let fit: Vec<&Case> = first.iter().map(|at| &lessons.cases[*at]).collect();
    let check: Vec<&Case> = last.iter().map(|at| &lessons.cases[*at]).collect();
    if fit.is_empty() || check.is_empty() {
        return None;
    }
    let words = question_words(&fit);
    let sampled: Vec<&Case> = fit.iter().take(RELEVANCE_SAMPLE).copied().collect();
    let fit_words = sample_words(&fit);
    let mut hasher = DefaultHasher::new();
    for case in &sampled {
        case.sentences.hash(&mut hasher);
        case.question.hash(&mut hasher);
    }
    let sample_key = hasher.finish();
    let learned_on: Vec<&Case> = fit.iter().take(SEARCH_LESSONS).copied().collect();
    let search_started = Instant::now();
    let mut best: Option<(usize, usize)> = None;
    let mut best_defaults = usize::default();
    let mut searched: Vec<(usize, usize)> = Vec::new();
    let mut same_texts: HashSet<(u64, u64)> = HashSet::new();
    let mut told_best: Option<((usize, usize), usize, Vec<String>, WordedModel)> = None;
    let told_stories = sampled.iter().filter(|case| !case.sentences.is_empty()).count() * WHOLE_SHARE >= sampled.len() * RULE_GROUND_PERCENT;
    let mut before: Option<Vec<Vec<Case>>> = None;
    let mut bridges_tried = false;
    let mut searched_any = false;
    let mut promising = usize::default();
    let bridged = |before: &mut Option<Vec<Vec<Case>>>, known: Option<(usize, usize)>| -> Option<((usize, usize), usize, Vec<String>, WordedModel)> {
        let _bridges = cost_scope(Some(WAY_BRIDGES), None);
        let mut found: Option<((usize, usize), usize, Vec<String>, WordedModel)> = None;
        let mut people: Option<Vec<String>> = None;
        for place in grounded_texts(library, &sampled).into_iter().take(MOST_RULE_TEXTS) {
            if scope_spent() || found.as_ref().is_some_and(|((right, _), _, _, _)| *right == check.len()) {
                break;
            }
            let started = Instant::now();
            let kinds_before = lessons.before.map(|read| before.get_or_insert_with(read).as_slice());
            let (bridges, taught) = bridge_lines(state_bridges(library, place, (&fit, kinds_before)));
            let persons = people.get_or_insert_with(|| person_lines(&fit)).clone();
            for (lines, taught, way) in [(bridges, taught, STATE_BRIDGES_WAY), (persons, Vec::new(), PERSON_STATE_WAY)] {
                if lines.is_empty() {
                    continue;
                }
                let Some(model) = told_model(library, place, &lines, &fit) else { continue };
                let (right, defaults) = judged(&model, &check);
                path_note(lessons, &format!("{way} for {}: {} lines, {right} of {} last lessons: {}", library.topics[place].name, lines.len(), check.len(), lines.join(PATH_LINES_GAP)), started);
                if right * WHOLE_SHARE >= check.len() * LEAST_LEARNED_PERCENT && ahead((right, defaults), known) && ahead((right, defaults), found.as_ref().map(|(known, _, _, _)| *known)) {
                    found = Some(((right, defaults), place, lines, WordedModel { taught, ..model }));
                }
            }
        }
        found
    };
    'plain: for with_forms in [false, true] {
        if with_forms && best.is_some_and(|(right, _)| right * WHOLE_SHARE >= check.len() * TOLD_TRY_PERCENT) {
            break;
        }
        let mut promised: Vec<(bool, usize, usize)> = Vec::new();
        for (place, topic) in library.topics.iter().enumerate() {
            if topic.rules.is_empty() || topic.questions.is_empty() == with_forms || (with_forms && hand_rules(library, place)) {
                continue;
            }
            let mut hasher = DefaultHasher::new();
            topic.facts.hash(&mut hasher);
            if !same_texts.insert((topic.rules_key, hasher.finish())) {
                continue;
            }
            let known = PROMISES.get_or_init(Mutex::default).lock().ok().and_then(|promises| promises.get(&(place, sample_key)).cloned());
            let ((score, learned), fresh) = learned_within(known.as_ref(), || {
                let _promise = cost_scope(Some(WAY_PROMISE), Some(&topic.name));
                trial(topic, library.base(place), (&fit, &fit_words), if with_forms { CONCEPT_FIRED_PERCENT } else { LEAST_FIRED_PERCENT }).map_or((0, 0), |sample| {
                    let score = promise(topic, library.base(place), &sampled, &sample, &words, false);
                    (score, if score == 0 { learned_promise(topic, library.base(place), &sampled, &sample) } else { 0 })
                })
            });
            if let Some(fresh) = fresh
                && let Ok(mut promises) = PROMISES.get_or_init(Mutex::default).lock()
            {
                promises.insert((place, sample_key), fresh);
            }
            if score > 0 {
                promised.push((false, score, place));
            } else if learned > 0 {
                promised.push((true, learned, place));
            }
        }
        promised.sort_by(|one, other| one.0.cmp(&other.0).then_with(|| other.1.cmp(&one.1)).then_with(|| one.2.cmp(&other.2)));
        promising += promised.len();
        for (_, _, place) in promised {
            if scope_spent() {
                break;
            }
            if searched_any && with_told && told_stories && !bridges_tried && best.is_none_or(|(right, _)| right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT) {
                bridges_tried = true;
                told_best = bridged(&mut before, best.map(|(known, _)| (known, best_defaults)));
                if told_best.as_ref().is_some_and(|((right, _), _, _, _)| *right == check.len()) {
                    break 'plain;
                }
            }
            searched_any = true;
            let _forms = cost_scope(Some(WAY_FORMS), Some(&library.topics[place].name));
            let Some(model) = searched_from(library, place, &learned_on) else { continue };
            let (right, defaults) = judged(&model, &check);
            if right * WHOLE_SHARE >= check.len() * LEAST_LEARNED_PERCENT {
                searched.push((right, place));
            }
            if right * WHOLE_SHARE >= check.len() * LEAST_LEARNED_PERCENT && ahead((right, defaults), best.map(|(known, _)| (known, best_defaults))) {
                best = Some((right, place));
                best_defaults = defaults;
            }
            if best.is_some_and(|(known, _)| known == check.len()) && !by_defaults(best_defaults, &check) {
                break;
            }
        }
    }
    if fit.len() > learned_on.len() && best.is_some_and(|(right, _)| right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT) && told_best.as_ref().is_none_or(|((right, _), _, _, _)| *right < check.len()) {
        searched.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| one.1.cmp(&other.1)));
        for (_, place) in searched.into_iter().take(MOST_RELEARNED) {
            let _forms = cost_scope(Some(WAY_FORMS), Some(&library.topics[place].name));
            let Some(model) = worded_from(library, place, &fit) else { continue };
            let (right, defaults) = judged(&model, &check);
            if ahead((right, defaults), best.map(|(known, _)| (known, best_defaults))) {
                best = Some((right, place));
                best_defaults = defaults;
            }
        }
    }
    path_note(lessons, &match best {
        Some((right, place)) => format!("plain wording: {} reads {right} of {} last lessons", library.topics[place].name, check.len()),
        None => format!("plain wording: no text reads half of {} last lessons", check.len()),
    }, search_started);
    if told_best.as_ref().is_some_and(|(known, _, _, _)| !ahead(*known, best.map(|(right, _)| (right, best_defaults)))) {
        told_best = None;
    }
    if with_told && told_stories && !bridges_tried && best.is_none_or(|(right, _)| right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT) {
        told_best = bridged(&mut before, best.map(|(known, _)| (known, best_defaults)));
    }
    if with_told && best.is_none() && told_best.is_none() {
        let _facts = cost_scope(Some(WAY_FACTS), None);
        if let Some((model, true)) = written_facts_model(library, lessons) {
            return Some(model);
        }
    }
    let below_bar = best.is_some_and(|(right, _)| right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT);
    if with_told && (best.is_none() || below_bar) && told_stories && told_best.as_ref().is_none_or(|((right, _), _, _, _)| right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT) {
        for place in grounded_texts(library, &sampled).into_iter().take(MOST_RULE_TEXTS) {
            if scope_spent() {
                break;
            }
            if best.is_some() && (lessons.before.is_none() || !library.texts[place].rules.is_empty()) {
                continue;
            }
            let _rules = cost_scope(Some(WAY_RULES), Some(&library.topics[place].name));
            let started = Instant::now();
            let lines = if best.is_some() { Vec::new() } else { learned_once(&RULES_LEARNED, (library, place, &fit), &learn_rules) };
            let mut found = match told_model(library, place, &lines, &fit) {
                Some(model) => {
                    let (right, defaults) = judged(&model, &check);
                    path_note(lessons, &format!("rules of a concept for {}: {} lines, {right} of {} last lessons: {}", library.topics[place].name, lines.len(), check.len(), lines.join(PATH_LINES_GAP)), started);
                    Some(((right, defaults), lines, model))
                }
                None => {
                    path_note(lessons, &format!("rules of a concept for {}: {} lines, no wording: {}", library.topics[place].name, lines.len(), lines.join(PATH_LINES_GAP)), started);
                    None
                }
            };
            if found.as_ref().is_none_or(|((right, _), _, _)| right * WHOLE_SHARE < check.len() * LEAST_LEARNED_PERCENT)
                && let Some(read) = lessons.before
                && let Some(own) = asked_line(&library.topics[place], &sampled)
            {
                let started = Instant::now();
                let kinds_before = before.get_or_insert_with(read);
                let (taught, taught_base, earlier, earlier_kinds) = {
                    let _taught = cost_scope(Some(WAY_TAUGHT), Some(&library.topics[place].name));
                    taught_before(library, place, (kinds_before, own))
                };
                if !kinds_before.is_empty() {
                    learning_note(format!("curriculum over the {} kinds before it for {}: {} rules learned", kinds_before.len(), library.topics[place].name, earlier.len()));
                }
                if !earlier.is_empty() {
                    let (lines, bridge_kinds) = over_kinds_before(library, place, (&fit, kinds_before), (&taught, &taught_base, earlier));
                    if let Some(model) = told_model(library, place, &lines, &fit) {
                        let (right, defaults) = judged(&model, &check);
                        path_note(lessons, &format!("rules of a concept for {} over the kinds before it in its set: {} lines, {right} of {} last lessons: {}", library.topics[place].name, lines.len(), check.len(), lines.join(PATH_LINES_GAP)), started);
                        if ahead((right, defaults), found.as_ref().map(|(known, _, _)| *known)) {
                            let mut kinds: Vec<usize> = earlier_kinds.iter().chain(&bridge_kinds).copied().collect();
                            kinds.sort_unstable();
                            kinds.dedup();
                            found = Some(((right, defaults), lines, WordedModel { taught: kinds, ..model }));
                        }
                    }
                }
            }
            if found.as_ref().is_none_or(|((right, _), _, _)| right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT) {
                let bridges: Vec<String> = state_bridges(library, place, (&fit, None)).into_iter().map(|(line, _)| line).collect();
                let bridged: Vec<String> = if bridges.is_empty() {
                    Vec::new()
                } else {
                    let (over, over_base) = with_learned(&library.topics[place], library.base(place), &bridges);
                    bridges.iter().cloned().chain(learned_once_of(&RULES_LEARNED, (place, &over, &over_base, &fit), &learn_rules).into_iter().filter(|line| !bridges.contains(line))).collect()
                };
                match told_model(library, place, &bridged, &fit).filter(|_| !bridged.is_empty()) {
                    Some(model) => {
                        let (right, defaults) = judged(&model, &check);
                        path_note(lessons, &format!("rules of a concept for {} with the bridges of its states: {} lines, {right} of {} last lessons: {}", library.topics[place].name, bridged.len(), check.len(), bridged.join(PATH_LINES_GAP)), started);
                        if ahead((right, defaults), found.as_ref().map(|(known, _, _)| *known)) {
                            found = Some(((right, defaults), bridged, model));
                        }
                    }
                    None if !bridged.is_empty() => path_note(lessons, &format!("rules of a concept for {} with the bridges of its states: {} lines, no wording: {}", library.topics[place].name, bridged.len(), bridged.join(PATH_LINES_GAP)), started),
                    None => {}
                }
            }
            let Some((read, lines, model)) = found else { continue };
            if read.0 * WHOLE_SHARE >= check.len() * LEAST_LEARNED_PERCENT && ahead(read, told_best.as_ref().map(|(known, _, _, _)| *known)) && ahead(read, best.map(|(known, _)| (known, best_defaults))) {
                told_best = Some((read, place, lines, model));
                break;
            }
        }
    }
    if with_told && told_stories && told_best.is_none() && best.is_some_and(|(right, _)| right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT) {
        let mut formed: Vec<(usize, usize)> = library.topics.iter().enumerate().filter_map(|(place, topic)| formed_ways(topic, &sampled).iter().filter_map(|way| if let AnswerFact::Formed(at) = way { topic.questions.get(*at).map(|form| fixed_count(&form.question)) } else { None }).max().map(|fixed| (fixed, place))).collect();
        formed.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| one.1.cmp(&other.1)));
        let formed: Vec<usize> = formed.into_iter().map(|(_, place)| place).take(MOST_RULE_TEXTS).collect();
        for place in formed {
            if scope_spent() {
                break;
            }
            let _rules = cost_scope(Some(WAY_RULES), Some(&library.topics[place].name));
            let started = Instant::now();
            let lines = learned_once(&FORMED_LEARNED, (library, place, &fit), &learn_formed);
            let Some(model) = told_model(library, place, &lines, &fit) else {
                path_note(lessons, &format!("rules of a concept for {} toward its own question form: {} lines, no wording: {}", library.topics[place].name, lines.len(), lines.join(PATH_LINES_GAP)), started);
                continue;
            };
            let (right, defaults) = judged(&model, &check);
            path_note(lessons, &format!("rules of a concept for {} toward its own question form: {} lines, {right} of {} last lessons: {}", library.topics[place].name, lines.len(), check.len(), lines.join(PATH_LINES_GAP)), started);
            if ahead((right, defaults), best.map(|(known, _)| (known, best_defaults))) && ahead((right, defaults), told_best.as_ref().map(|(known, _, _, _)| *known)) {
                told_best = Some(((right, defaults), place, lines, model));
            }
        }
    }
    if with_told && told_best.is_none() && best.is_none_or(|(right, _)| right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT) && (1..=MOST_TOLD_FORMS).contains(&told_form_count(&learned_on)) {
        let filled = answers_told(&learned_on) * WHOLE_SHARE >= learned_on.len() * TOLD_FILLED_PERCENT;
        let mut ranked: Vec<(usize, usize)> = library.topics.iter().enumerate().filter(|(_, topic)| !topic.rules.is_empty() && topic.questions.is_empty()).map(|(place, topic)| (told_relevance(topic, if filled { &learned_on } else { &sampled }), place)).filter(|(relevance, _)| *relevance >= TOLD_RELEVANT_PERCENT || (filled && *relevance > usize::default())).collect();
        ranked.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| one.1.cmp(&other.1)));
        let mut seen: HashSet<u64> = HashSet::new();
        ranked.retain(|(_, place)| seen.insert(library.topics[*place].rules_key) && trial(&library.topics[*place], library.base(*place), (&fit, &fit_words), LEAST_FIRED_PERCENT).is_none());
        for (_, place) in ranked.into_iter().take(if filled { 1 } else { MOST_TOLD_TEXTS }) {
            if scope_spent() {
                break;
            }
            let _told = cost_scope(Some(WAY_TOLD), Some(&library.topics[place].name));
            let started = Instant::now();
            let lines = learned_once(&TOLD_LEARNED, (library, place, &fit), &learn_told);
            let Some(model) = told_model(library, place, &lines, &fit) else {
                path_note(lessons, &format!("told side for {}: {} lines, no wording: {}", library.topics[place].name, lines.len(), lines.join(PATH_LINES_GAP)), started);
                continue;
            };
            let (right, defaults) = judged(&model, &check);
            path_note(lessons, &format!("told side for {}: {} lines, {right} of {} last lessons: {}", library.topics[place].name, lines.len(), check.len(), lines.join(PATH_LINES_GAP)), started);
            let known = told_best.as_ref().map(|(known, _, _, _)| *known).or(best.map(|(known, _)| (known, best_defaults)));
            if right * WHOLE_SHARE >= check.len() * LEAST_LEARNED_PERCENT && ahead((right, defaults), known) {
                told_best = Some(((right, defaults), place, lines, model));
            }
            if right * WHOLE_SHARE >= check.len() * TOLD_TRY_PERCENT {
                break;
            }
        }
    }
    if with_told && best.is_none() && told_best.is_none() {
        let _facts = cost_scope(Some(WAY_ANSWERED), None);
        told_best = answered_facts(library, &sampled, (&fit, &check));
    }
    if with_told && told_stories && best.is_none() && told_best.is_none() && sampled.iter().all(|case| case.sentences.len() <= ALONE_STORY_SENTENCES) && labelled_statements(&sampled) {
        let _alone = cost_scope(Some(WAY_ALONE), None);
        let started = Instant::now();
        let (blank, blank_base) = blank_text();
        let learned = learned_once_of(&ALONE_LEARNED, (usize::default(), &blank, &blank_base, &fit), &learn_alone);
        let (lines, model) = cleaned_lines((&blank, &blank_base), learned, &fit);
        let found = model.map(|model| (judged(&model, &check), model));
        let said = found.as_ref().map_or_else(|| format!("{} lines, no wording", lines.len()), |((right, _), _)| format!("{} lines, {right} of {} last lessons", lines.len(), check.len()));
        path_note(lessons, &format!("{WAY_ALONE}: {said}: {}", lines.join(PATH_LINES_GAP)), started);
        match found {
            Some(((right, _), model)) if right * WHOLE_SHARE >= check.len() * LEAST_LEARNED_PERCENT => {
                let _final = cost_scope(Some(WAY_FINAL), None);
                let every: Vec<&Case> = lessons.cases.iter().take(LEARN_LESSONS).collect();
                return Some(match learned_model((&blank, &blank_base, false), &lines, &every) {
                    Some(again) if check.iter().filter(|case| again.answer(&case.sentences, &case.question).words == answer_words(&case.answer)).count() >= right => again,
                    _ => model,
                });
            }
            _ => learning_note(format!("{WAY_ALONE}: {said}")),
        }
    }
    if with_told && best.is_none() && told_best.is_none() && promising == usize::default() {
        let _blank = cost_scope(Some(WAY_ALONE), None);
        let started = Instant::now();
        let found = blank_wording(&fit).map(|model| (judged(&model, &check), model));
        path_note(lessons, &format!("{BLANK_WORDING}: {}", found.as_ref().map_or_else(|| NO_BLANK_FORM.to_string(), |((right, _), _)| format!("{right} of {} last lessons", check.len()))), started);
        if let Some(((right, _), model)) = found.filter(|((right, _), _)| right * WHOLE_SHARE >= check.len() * LEAST_LEARNED_PERCENT) {
            let every: Vec<&Case> = lessons.cases.iter().take(LEARN_LESSONS).collect();
            return Some(match blank_wording(&every) {
                Some(again) if check.iter().filter(|case| again.answer(&case.sentences, &case.question).words == answer_words(&case.answer)).count() >= right => again,
                _ => model,
            });
        }
    }
    if with_told && best.is_none() && told_best.is_none() {
        learning_note(passed_over(library, (&sampled, &learned_on), promising));
    }
    let _final = cost_scope(Some(WAY_FINAL), None);
    let every: Vec<&Case> = lessons.cases.iter().take(LEARN_LESSONS).collect();
    match told_best {
        Some(((right, _), place, lines, model)) => match told_model(library, place, &lines, &every) {
            Some(again) if check.iter().filter(|case| again.answer(&case.sentences, &case.question).words == answer_words(&case.answer)).count() >= right => Some(WordedModel { taught: model.taught, ..again }),
            _ => Some(model),
        },
        None => best.and_then(|(_, place)| worded_final(library, place, &every)),
    }
}
because!(worded_search, LearnedWording, "the learned wording of a kind: the texts of the library with rules and no question forms, each set of rules and known facts once (a wording text with forms only reads with the rules of the concept it builds on), and when none of them reads the kind as a learned kind must, the concepts with question forms, their forms set aside (a text of the wording of another kind is a written reading of that kind), whose rules read the stories of a sample of the kind and promise a wording, the most promising first, and after them the texts whose single facts promise nothing but whose wording learned on the sample answers some of it; for each, the wording learned from the first lessons and judged on the last, and when none reads the kind as a learned kind must, the few that read most of it learned again from all the first lessons and judged again; the text whose learned wording answers the most of them right (of those that answer as many, the one with fewer answers that hold otherwise, ahead, also among the told side and the rules of a concept, and the search then goes on past a wording that answers all by defaults), if it answers half, with its wording learned again from all the lessons, as the board learns a tool again from all lessons once it is chosen; the search stops at a wording that answers all. When asked (not for a kind that written question forms already read in part, as the told learner judges many readings of each sentence form), and none answers nearly all and the stories tell few sentence forms, the told side is learned too, with the texts whose rules speak most of the told sentences but do not yet read the stories (when the stories tell most answers, only the text whose rules speak most of them, though it speaks of only some, as the first reading by filled answers judges its readings cheaply, while a text that speaks of fewer sentences costs as much and reads nothing), the most speaking first (counted on all the first lessons that the search learns from when the stories tell most answers, as a kind told in several settings, books, dances, games, speaks of the words of a rule in some settings only, and a small sample can miss them), until a text answers nearly all; and before that, when nothing reads the kind and its lessons tell stories, the rules of a concept are learned (learn_rules) for the text whose reply lines answer the questions of the kind or whose known facts hold the things that its stories name, as a text that names the asked fact is the surer ground and its search is cheaper than the told side (the told side then runs only when no such rules read the kind), of those that ground it alike the one whose rules speak most of the told sentences first, and of those still alike the smallest (the fewest rules, then the fewest facts), as a text that builds on another holds all of its rules and more, so its search judges more rules and its model reads each story with more rules, and when those rules do not read the kind as a learned kind must and a reply line of the text answers its questions, again over the rules that the kinds before the kind in its set whose questions other reply lines of that text answer taught it, in the order of the set (taught_before), as a kind may ask about facts that an earlier kind's rules make; the lessons of those kinds are read only then, as most kinds learn alone; the rules that read the told sentences are learned once from the first lessons, kept only when they answer more of the last lessons than the wording alone, and the wording is learned again from all the lessons with those rules, unless the wording learned again answers fewer of the last lessons, which it learned from, than the wording of the first lessons did without them: then the first wording stays, as more lessons can push a form of the questions below the share that its answers need and drop it with all its lessons; and when a plain wording reads the kind below the bar and the kind has earlier kinds, the rules over the kinds before it for each grounded text with no rules of its own (a concept whose rules are to be learned), taken only when they read more than the plain wording, as the plain wording of another text may read the carrier of a thing where the question asks the thing; and when a plain wording reads the kind below the bar, the texts whose own question forms read facts that no rule makes and no story tells learn the rules toward those facts (learn_formed), taken only when they read more; and when nothing reads the kind and no text promised a wording, the question forms alone on a text with no line (blank_wording); and when nothing reads the kind, its lessons tell short stories (ALONE_STORY_SENTENCES) and ask statements of a few labels (labelled_statements), and no fact of knowledge answers it, the rules of a new concept learned from the lessons alone, on a text with no line (blank_text), as a kind may teach a concept that no text of the library holds, kept when it reads half of the last lessons");

fn passed_over(library: &Library, (sampled, learned_on): (&[&Case], &[&Case]), promising: usize) -> String {
    let nearest = |mut scores: Vec<(usize, usize)>| -> String {
        scores.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| one.1.cmp(&other.1)));
        scores.iter().take(PASSED_SHOWN).map(|(share, place)| format!("{} {share}", library.topics[*place].name)).collect::<Vec<String>>().join(PASSED_GAP)
    };
    let asked = asked_things(sampled);
    let grounds = nearest(library.topics.iter().enumerate().filter(|(_, topic)| !topic.rules.is_empty() || !topic.questions.is_empty()).map(|(place, topic)| (rule_ground(topic, library.base(place), (sampled, &asked)), place)).collect());
    let speaking = nearest(library.topics.iter().enumerate().filter(|(_, topic)| !topic.rules.is_empty() && topic.questions.is_empty()).map(|(place, topic)| (told_relevance(topic, sampled), place)).collect());
    let longest = sampled.iter().map(|case| case.sentences.len()).max().unwrap_or_default();
    let statements = if labelled_statements(sampled) { PASSED_STATEMENTS } else { PASSED_NOT_STATEMENTS };
    format!("{PASSED_HEAD}{promising} texts promised a plain wording and none read half of the last lessons; the rules of a concept need a rule ground of {RULE_GROUND_PERCENT} (nearest: {grounds}); the told side needs at most {MOST_TOLD_FORMS} told forms ({}) and texts that speak of {TOLD_RELEVANT_PERCENT} of the told sentences (nearest: {speaking}); the lessons alone need stories of at most {ALONE_STORY_SENTENCES} told sentences (up to {longest}) and {statements}", told_form_count(learned_on))
}
because!(passed_over, LearnedWording, "a line of what the learned wording passed over for a kind that nothing read, for the notes of the learning: how many texts promised a plain wording, and for each later way its bar beside the kind's own measure, with the nearest texts (in hundredths: the rule ground of the texts for the rules of a concept, the share of the told sentences that the rules of a text speak of for the told side), and the size of the stories and the shape of the questions for the rules from the lessons alone, so that a person sees which way came nearest and what it lacked without a run with the search paths");

const PASSED_HEAD: &str = "the learned wording passed over: ";
because!(PASSED_HEAD, LearnedWording, "the words that open the note of what the learned wording passed over for a kind that nothing read");

const PASSED_GAP: &str = ", ";
because!(PASSED_GAP, LearnedWording, "the marks between the nearest texts of a way in the note of what the learned wording passed over");

const PASSED_SHOWN: usize = 3;
because!(PASSED_SHOWN, LearnedWording, "the nearest texts that the note of what the learned wording passed over names for each way: a few show where a kind falls short, while the whole library would hide it");

const PASSED_STATEMENTS: &str = "statements of a few labels that the stories do not tell (as here)";
because!(PASSED_STATEMENTS, LearnedWording, "the words of the note of what the learned wording passed over for a kind whose questions are statements of a few labels that the stories do not tell");

const PASSED_NOT_STATEMENTS: &str = "statements of a few labels that the stories do not tell (not so here)";
because!(PASSED_NOT_STATEMENTS, LearnedWording, "the words of the note of what the learned wording passed over for a kind whose questions are not statements of a few labels that the stories do not tell");

fn answered_facts(library: &Library, sampled: &[&Case], (fit, check): (&[&Case], &[&Case])) -> Option<((usize, usize), usize, Vec<String>, WordedModel)> {
    let names = lesson_names(fit);
    let sample = spread_sample(fit, SEED_SAMPLE);
    for place in grounded_texts(library, sampled).into_iter().take(MOST_RULE_TEXTS) {
        if scope_spent() {
            break;
        }
        let candidates: Vec<String> = suggested_facts(&library.topics[place], library.base(place), &sample, (false, false, true)).into_iter().filter(|(_, values, _)| !values.iter().flatten().any(|word| names.contains(word))).map(|(line, _, _)| line).collect();
        if candidates.is_empty() {
            continue;
        }
        let Some(model) = told_model(library, place, &candidates, fit) else { continue };
        let (right, defaults) = judged(&model, check);
        if right * WHOLE_SHARE < check.len() * TOLD_TRY_PERCENT {
            continue;
        }
        let said = |lines: &[String]| -> Vec<Vec<String>> {
            let (learned, learned_base) = with_learned(&library.topics[place], library.base(place), lines);
            let base = timed_base(&learned, &model.topic.questions, &learned_base);
            let kept = WordedModel { topic: Topic { questions: model.topic.questions.clone(), ..learned }, base, source: model.source.clone(), hand: model.hand, learned: Vec::new(), taught: Vec::new(), named: model.named, frames: None };
            check.iter().map(|case| kept.answer(&case.sentences, &case.question).words).collect()
        };
        let all = said(&candidates);
        let mut kept: Vec<String> = candidates.clone();
        for line in &candidates {
            let without: Vec<String> = kept.iter().filter(|known| *known != line).cloned().collect();
            if said(&without) == all {
                kept = without;
            }
        }
        learning_note(format!("facts beside the learned wording of {}: {} of {} suggested facts kept, as the last lessons need them: {}", library.topics[place].name, kept.len(), candidates.len(), kept.join(PATH_LINES_GAP)));
        let model = told_model(library, place, &kept, fit)?;
        return Some(((right, defaults), place, kept, model));
    }
    None
}
because!(answered_facts, LearnedWording, "facts of knowledge for a kind that nothing reads, beside the learned wording of a text that its reply lines ground: the facts that a sample of its lessons suggest for the conditions of the text that only known facts fit, each filling that is not known and that holds an answer of the lessons (white is a colour, for a kind whose stories tell that Lily is white and ask what colour Bernhard is), all added at once and the wording learned over them; kept when the kind then reads as a learned kind must, each fact whose removal changes no answer of the last lessons dropped in one pass, so that only the facts that some answer needs are kept and counted; the answers that the filling must hold are those of the lessons alone");

fn judged(model: &WordedModel, check: &[&Case]) -> (usize, usize) {
    let said: Vec<Said> = check.iter().take_while(|_| !scope_spent()).map(|case| model.answer(&case.sentences, &case.question)).collect();
    let right = check.iter().zip(&said).filter(|(case, said)| said.words == answer_words(&case.answer)).count();
    let defaults = said.iter().filter(|said| said.mark == AnswerMark::Default).count();
    (right, if model.hand { usize::MAX } else { defaults })
}
because!(judged, LearnedWording, "how a learned model reads the last lessons of a kind: how many it answers right, and how many of its answers hold otherwise, with no condition that a fact proves; read one by one until the learning has done its bound of work, the lessons not read counted as wrong (a model whose rules run away reads the bound of the engine on every story, which took minutes over the last lessons of one kind, and a learning past its bound gives up); a model that reads with rules written by a person counts as answering all by default, so that at the same right answers a learned model never gives way to it (the task would read mixed, not learned)");

fn ahead((right, defaults): (usize, usize), known: Option<(usize, usize)>) -> bool {
    known.is_none_or(|(known_right, known_defaults)| right > known_right || (right == known_right && defaults < known_defaults))
}
because!(ahead, LearnedWording, "whether a model read so goes before the one chosen so far: it answers more of the same last lessons right, or as many with fewer answers that hold otherwise, as an answer that a proof rests on explains itself where a default does not (the board shows which rules an answer rests on, and the share of answers by default is reported); never with fewer right answers");

fn by_defaults(defaults: usize, check: &[&Case]) -> bool {
    defaults != usize::MAX && defaults * WHOLE_SHARE >= check.len() * DEFAULTS_PERCENT
}
because!(by_defaults, LearnedWording, "whether a learned model answers the last lessons mostly by default, so that the search for a wording goes on past it, though it answers all, for one whose answers rest on proofs");

const DEFAULTS_PERCENT: usize = 50;
because!(DEFAULTS_PERCENT, LearnedWording, "the least share of the last lessons answered by default for a wording that answers all to count as a reading by defaults, worth more texts tried: a model whose answers mostly hold otherwise explains little of them, while a few defaults beside proofs are the usual case and cost no search");

fn grounded_texts(library: &Library, sampled: &[&Case]) -> Vec<usize> {
    let asked = asked_things(sampled);
    let mut ranked: Vec<(usize, usize, usize)> = library.topics.iter().enumerate().filter(|(_, topic)| !topic.rules.is_empty() || !topic.questions.is_empty()).map(|(place, topic)| (rule_ground(topic, library.base(place), (sampled, &asked)), place)).filter(|(share, _)| *share >= RULE_GROUND_PERCENT).map(|(share, place)| (share, told_relevance(&library.topics[place], sampled), place)).collect();
    let size = |place: usize| (library.topics[place].rules.len(), library.topics[place].facts.len());
    ranked.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| other.1.cmp(&one.1)).then_with(|| size(one.2).cmp(&size(other.2))).then_with(|| one.2.cmp(&other.2)));
    ranked.into_iter().map(|(_, _, place)| place).collect()
}
because!(grounded_texts, LearnedWording, "the texts of rules, or of question forms whose rules are still to be learned (a concept whose given rules are taken out), that the rules of a concept could be learned for on a sample of lessons (rule_ground at least the bar), the best grounded first, then the most relevant to the told sentences, then the smallest (fewest rules, then fewest facts), as a smaller text that reads the kind holds fewer rules that a search could join by chance");

fn sentence_map(one: &Pattern, other: &Pattern) -> Option<HashMap<usize, usize>> {
    if one.len() != other.len() {
        return None;
    }
    let mut map: HashMap<usize, usize> = HashMap::new();
    for (mine, theirs) in one.iter().zip(other) {
        let (from, to) = match (mine, theirs) {
            (LogicPiece::Word(word), LogicPiece::Word(known)) if word == known => continue,
            (LogicPiece::Open(from), LogicPiece::Open(to)) => (*from, *to),
            (LogicPiece::OpenWithEnd(from, ending), LogicPiece::OpenWithEnd(to, known)) if ending == known => (*from, *to),
            _ => return None,
        };
        if *map.entry(from).or_insert(to) != to {
            return None;
        }
    }
    let mut seen: HashSet<usize> = HashSet::new();
    map.values().all(|to| seen.insert(*to)).then_some(map)
}
because!(sentence_map, LearnedWording, "how the places of one sentence stand for the places of another when the two are one sentence whatever their letters (the same words at the same spots, each place of one to one place of the other), or nothing when they differ");

fn told_readers(topic: &Topic, event: &Pattern) -> usize {
    topic.rules.iter().filter(|rule| rule.conditions.iter().any(|condition| matches!(condition, Condition::Fact { pattern, .. } if sentence_map(pattern, event).is_some()))).count()
}
because!(told_readers, LearnedWording, "how many rules of a text read a told event as it is told (a condition that is the event whatever the letters of its places)");

fn read_as_told(topic: &Topic, state: &StateCandidate) -> bool {
    state.starts.iter().chain(&state.ends).any(|(event, _, _)| told_readers(topic, event) > usize::default())
}
because!(read_as_told, LearnedWording, "whether some rule of a text reads some event of a state as it is told: then the text already reads such events (the verbs of going, when the rules of movement read them), and the state of the persons gives no bridges");

fn event_bridges(topic: &Topic, state: &StateCandidate) -> Vec<String> {
    let loose = |event: &Pattern| topic.rules.iter().filter(|rule| rule.conditions.iter().any(|condition| matches!(condition, Condition::Fact { pattern, .. } if takes_words(pattern, event)))).count();
    let read_exactly = read_as_told(topic, state);
    let read = |event: &Pattern| if read_exactly { told_readers(topic, event) } else { loose(event) };
    let mut out: Vec<String> = Vec::new();
    for events in [&state.starts, &state.ends] {
        let roles: Vec<(&Pattern, usize, usize)> = events
            .iter()
            .filter_map(|(event, made, _)| {
                let key = *places_of(made).first()?;
                let others: Vec<usize> = places_of(event).into_iter().filter(|place| *place != key).collect();
                let [other] = others.as_slice() else { return None };
                Some((event, key, *other))
            })
            .collect();
        let Some((anchor, anchor_key, anchor_other)) = roles.iter().filter(|(event, _, _)| read(event) > usize::default()).max_by_key(|(event, _, _)| read(event)).copied() else { continue };
        for (event, key, other) in &roles {
            if read(event) > usize::default() {
                continue;
            }
            let target = renamed(anchor, &[anchor_key, anchor_other], &[*key, *other]);
            let (source, target) = without_shared_end(event, &target, &read);
            let line = format!("{TOLD_IF}{}{TOLD_THEN}{}{STATEMENT_END}", library_writing(&source), library_writing(&target));
            if !out.contains(&line) {
                out.push(line);
            }
        }
    }
    out
}
because!(event_bridges, LearnedWording, "the bridges that an invented state gives a text: among the told events that start the state, the one that the most rules of the text read stands for the others that no rule reads (A took the B as A got the B), the thing of the state for its thing and the other person for the other person, and the same among the events that end it (A put down the B as A dropped the B); events with the same role in one state name the same event, so the rules of the text read the told events that only another word named, and the places keep their kinds by the roles of the state");

fn takes_words(condition: &Pattern, event: &Pattern) -> bool {
    condition.len() == event.len()
        && condition.iter().zip(event).all(|(mine, theirs)| match (mine, theirs) {
            (_, LogicPiece::Open(_) | LogicPiece::OpenWithEnd(..)) => true,
            (LogicPiece::Word(word), LogicPiece::Word(known)) => word == known,
            _ => false,
        })
}
because!(takes_words, LearnedWording, "whether a condition reads a told event once the places of the event take the words that the condition fixes there (he went to the W reads A went to the B, with he for A): the words of the event stand at the same spots of the condition, which may fix a word where the event has a place; the readers of the events of a state that no rule reads as they are, such as the rules of words that point back, which read a verb of going only after he or they");

fn without_shared_end(source: &Pattern, target: &Pattern, read: &dyn Fn(&Pattern) -> usize) -> (Pattern, Pattern) {
    let tail = |pattern: &Pattern| pattern.iter().rposition(|piece| !matches!(piece, LogicPiece::Word(_))).map_or(usize::default(), |at| pattern.len() - at - 1);
    let most = tail(source).min(tail(target));
    let common = (usize::default()..most).take_while(|back| source[source.len() - 1 - back] == target[target.len() - 1 - back]).count();
    let short_target = target[..target.len() - common].to_vec();
    let wide_last = matches!(source.iter().rev().find(|piece| !matches!(piece, LogicPiece::Word(_))), Some(LogicPiece::Open(place)) if *place < PLAIN_PLACES);
    if common > usize::default() && wide_last && read(&short_target) > usize::default() {
        return (source[..source.len() - common].to_vec(), short_target);
    }
    (source.clone(), target.clone())
}
because!(without_shared_end, LearnedWording, "a bridge without the words after the last place that its two events share, when a rule of the text reads the event without them (A picked up the B there as A got the B there becomes A picked up the B as A got the B): a word after the thing that both events hold (there) is no part of the name of the event, and a place of one or more words still takes it; a bridge whose last place takes one word keeps those words, as such a place cannot take them, and the rule of the text that reads the event with those words (A got the B there) reads the fact that the bridge makes");

static STATE_BRIDGES: OnceLock<Mutex<HashMap<(usize, u64, u64), Arc<Mutex<Option<Worked<Vec<String>>>>>>>> = OnceLock::new();
because!(STATE_BRIDGES, LearnedWording, "the bridges of the invented states of each text and lessons in a run, found once while other kinds that need them wait, as every later kind of a set reads the states of the same kinds before it; kept with their work under the bound of work of a kept learning (learned_within), so that a kind that takes them adds the work that they cost");

fn kind_bridges(library: &Library, place: usize, cases: &[&Case]) -> Vec<String> {
    let topic = &library.topics[place];
    let sample: Vec<&Case> = spread_sample(cases, SEED_SAMPLE);
    let key = (place, topic.rules_key, lessons_key(&sample));
    let cell = STATE_BRIDGES.get_or_init(Mutex::default).lock().map(|mut kept| Arc::clone(kept.entry(key).or_default())).unwrap_or_default();
    let mut kept = cell.lock().unwrap_or_else(PoisonError::into_inner);
    let (found, fresh) = learned_within(kept.as_ref(), || {
        let (states, _) = state_candidates(topic, library.base(place), &sample, &story_patterns(topic));
        let read = &cases[..cases.len().min(RULE_LESSONS)];
        let persons = person_state(read, (true, true)).filter(|state| !read_as_told(topic, state));
        let mut out: Vec<String> = Vec::new();
        for line in states.iter().chain(persons.iter()).flat_map(|state| event_bridges(topic, state)) {
            if !out.contains(&line) {
                out.push(line);
            }
        }
        out
    });
    if fresh.is_some() {
        *kept = fresh;
    }
    found
}
because!(kind_bridges, LearnedWording, "the bridges of the invented states that the stories of a sample of one kind's lessons, spread over them, suggest for a text (state_candidates, event_bridges), each once: the events of a state come in most stories of a kind whose questions rest on it, so a sample shows them at a third of the cost of the lessons of a rule search");

const STATE_BRIDGES_WAY: &str = "bridges of a state";
because!(STATE_BRIDGES_WAY, LearnedWording, "the name in the path notes of the lines of an invented state of things (state_bridges) tried with a text");

const PERSON_STATE_WAY: &str = "the state of the persons";
because!(PERSON_STATE_WAY, LearnedWording, "the name in the path notes of the lines of the state of the persons that the questions name (person_lines) tried with a text, so that the notes show which of the two states a kind took");

pub(super) fn person_lines(cases: &[&Case]) -> Vec<String> {
    person_state(&cases[..cases.len().min(RULE_LESSONS)], (true, false)).iter().flat_map(state_lines).collect()
}
because!(person_lines, LearnedWording, "the lines of the invented state of the persons that the questions of a kind name, read from the told sentences of as many lessons as a rule search reads, with no chain (person_state): a text whose rules are to be learned reads where a person is through them, a move starting the state and a sentence that says the person is no longer there ending it");

fn unread_told(topic: &Topic, cases: &[&Case]) -> bool {
    let readers: Vec<&Pattern> = topic
        .rules
        .iter()
        .flat_map(|rule| rule.conditions.iter())
        .chain(topic.questions.iter().flat_map(|form| form.replies.iter().flat_map(|reply| reply.conditions.iter())))
        .filter_map(|condition| match condition {
            Condition::Fact { pattern, .. } if event_word(pattern).is_some() => Some(pattern),
            _ => None,
        })
        .collect();
    let mut seen: HashSet<&str> = HashSet::new();
    cases.iter().take(SEED_SAMPLE).flat_map(|case| case.sentences.iter()).filter(|sentence| seen.insert(sentence.as_str())).any(|sentence| {
        let words = fact_numbers(sentence);
        !readers.iter().any(|pattern| {
            let mut fits = Vec::new();
            fit_into(pattern, &words, &empty_filling(), &mut fits);
            !fits.is_empty()
        })
    })
}
because!(unread_told, LearnedWording, "whether some told sentence of the first lessons of a kind is read by no condition of a rule or a question form of a text that names its event (a fixed word that is no article: V B the C reads every sentence of its shape and names none): only such a sentence can be read through a bridge of a state, so a kind whose told sentences the text reads all needs no search for states");

fn state_bridges(library: &Library, place: usize, (fit, before): (&[&Case], Option<&[Vec<Case>]>)) -> Vec<(String, Option<usize>)> {
    if !unread_told(&library.topics[place], fit) {
        return Vec::new();
    }
    let mut out: Vec<(String, Option<usize>)> = kind_bridges(library, place, fit).into_iter().map(|line| (line, None)).collect();
    for (from, kind) in before.unwrap_or_default().iter().enumerate().rev() {
        let (first, _) = inner_parts(kind);
        let cases: Vec<&Case> = first.iter().map(|at| &kind[*at]).collect();
        for line in kind_bridges(library, place, &cases) {
            if !out.iter().any(|(known, _)| *known == line) {
                out.push((line, Some(from)));
            }
        }
    }
    out
}
because!(state_bridges, LearnedWording, "the bridges of the invented states of a kind and of the kinds before it in its set, the nearest first, each with the place of the kind before it that gave it in the order of the set (none for the kind's own): a kind whose questions name the thing that changes hands (where is the football) shows the state that the kinds after it need, while their own questions name only the person (how many objects is Mary carrying)");

fn bridge_lines(bridges: Vec<(String, Option<usize>)>) -> (Vec<String>, Vec<usize>) {
    let mut kinds: Vec<usize> = bridges.iter().filter_map(|(_, from)| *from).collect();
    kinds.sort_unstable();
    kinds.dedup();
    (bridges.into_iter().map(|(line, _)| line).collect(), kinds)
}
because!(bridge_lines, LearnedWording, "the lines of the bridges of a kind, and the places of the kinds before it that gave some of them, each once in the order of the set");

fn written_with(library: &Library, place: usize, (lines, taught): (&[String], Vec<usize>)) -> WordedModel {
    let topic = &library.topics[place];
    let (learned, learned_base) = with_learned(topic, library.base(place), lines);
    let base = timed_base(&learned, &learned.questions, &learned_base);
    WordedModel { topic: learned, base, source: topic.name.clone(), hand: true, learned: lines.to_vec(), taught, named: (usize::default(), usize::default()), frames: None }
}
because!(written_with, LearnedWording, "the model of a text with its written question forms and lines learned for it from lessons (the bridges of invented states, facts of knowledge), and the places of the kinds before it that gave some of them");

fn bridged_model(library: &Library, place: usize, (bridges, taught): (&[String], Vec<usize>)) -> Option<WordedModel> {
    (!bridges.is_empty()).then(|| written_with(library, place, (bridges, taught)))
}
because!(bridged_model, LearnedWording, "the model of a text with its written question forms and the bridges of the invented states learned for it; nothing without a bridge");

fn right_count(model: &WordedModel, cases: &[&Case]) -> (usize, usize) {
    let judged: Vec<(bool, bool)> = on_lanes(cases.len(), hardware_lanes(), &|at| {
        let said = model.answer(&cases[at].sentences, &cases[at].question);
        let right = said.words == answer_words(&cases[at].answer);
        (right, !right && !said.words.is_empty())
    });
    (judged.iter().filter(|(right, _)| *right).count(), judged.iter().filter(|(_, wrong)| *wrong).count())
}
because!(right_count, LearnedWording, "how many lessons a model answers right, and how many it answers with a wrong answer, a lesson that it gives no answer not counted as such, the lessons on several lanes");

fn lesson_names(lessons: &[&Case]) -> HashSet<WordId> {
    let mut capital: HashSet<String> = HashSet::new();
    let mut small: HashSet<String> = HashSet::new();
    for case in lessons {
        for written in case.sentences.iter().chain(std::iter::once(&case.question)).flat_map(|sentence| text_words(sentence)) {
            let word = written.to_lowercase();
            if written.chars().next().is_some_and(char::is_uppercase) {
                capital.insert(word);
            } else {
                small.insert(word);
            }
        }
    }
    capital.difference(&small).map(|word| word_number(word)).collect()
}
because!(lesson_names, LearnedWording, "the names of some lessons: the words that their stories and questions write only with a capital letter (Ashley, Esel), never in small letters (yesterday after its sentence's first word, der), as a name is a value of one story, not knowledge");

fn written_facts(library: &Library, place: usize, lessons: &[&Case]) -> Vec<String> {
    let topic = &library.topics[place];
    let names = lesson_names(lessons);
    let pool: Vec<&Case> = lessons.iter().take(READ_TOLD_LESSONS).copied().collect();
    let suggested = |cases: &[&Case], known: &[String]| -> Vec<(String, Vec<Vec<WordId>>, usize)> { suggested_facts(topic, library.base(place), cases, (true, true, false)).into_iter().filter(|(line, values, _)| !known.contains(line) && !values.iter().flatten().any(|word| names.contains(word))).collect() };
    let early = suggested(&pool[..pool.len().min(RULE_LESSONS)], &[]);
    learning_note(format!("the fact search for {} judged {} candidates suggested by the first lessons: {}", topic.name, early.len(), early.iter().take(SHOWN_CANDIDATES).map(|(line, _, count)| format!("{line} ({count})")).collect::<Vec<String>>().join(PATH_LINES_GAP)));
    let first = chosen_facts(library, place, (&pool, &early), &[]);
    if first.is_empty() || pool.len() <= RULE_LESSONS {
        return first;
    }
    let sample: Vec<&Case> = spread_sample(&pool, RULE_LESSONS);
    if right_count(&written_with(library, place, (&first, Vec::new())), &sample).0 == sample.len() {
        return first;
    }
    chosen_facts(library, place, (&pool, &suggested(&pool, &first)), &first)
}
because!(written_facts, LearnedWording, "facts of knowledge that a text with written question forms lacks, learned from the lessons of a kind: first from the candidates that the first lessons suggest (chosen_facts), then, when those gave a fact and still leave a lesson of a spread sample wrong, from the candidates that all the lessons suggest on top of them, as a rare word comes too seldom in the first lessons (the first German questions ask with der less than three times), while a kind whose first lessons give no fact pays for no more");

const SHOWN_CANDIDATES: usize = 4;
because!(SHOWN_CANDIDATES, LearnedWording, "the candidates of a fact search named in the notes of the learning, the most suggested first, with how many lessons suggest each: enough to see whether a fact that should come stands among them");

fn chosen_facts(library: &Library, place: usize, (pool, candidates): (&[&Case], &[(String, Vec<Vec<WordId>>, usize)]), start: &[String]) -> Vec<String> {
    if candidates.is_empty() {
        return start.to_vec();
    }
    let words_of: Vec<HashSet<WordId>> = pool.iter().map(|case| fact_numbers(&case.question).into_iter().chain(case.sentences.iter().flat_map(|sentence| fact_numbers(sentence))).collect()).collect();
    let relevant: Vec<Vec<usize>> = candidates.iter().map(|(_, values, _)| spread_sample(&(0..pool.len()).filter(|at| values.iter().flatten().any(|word| words_of[*at].contains(word))).collect::<Vec<usize>>(), RULE_LESSONS)).collect();
    let marked = |model: &WordedModel, case: &Case| -> (bool, bool) {
        let said = model.answer(&case.sentences, &case.question);
        let right = said.words == answer_words(&case.answer);
        (right, !right && !said.words.is_empty())
    };
    let judged = |lines: &[String], places: &[usize]| -> Vec<(bool, bool)> {
        let model = written_with(library, place, (lines, Vec::new()));
        places.iter().map(|at| marked(&model, pool[*at])).collect()
    };
    let summed = |marks: &[(bool, bool)]| (marks.iter().filter(|(right, _)| *right).count(), marks.iter().filter(|(_, wrong)| *wrong).count());
    let mut chosen: Vec<String> = start.to_vec();
    let mut left: Vec<usize> = (0..candidates.len()).collect();
    loop {
        let mut wanted: Vec<usize> = left.iter().flat_map(|at| relevant[*at].iter().copied()).collect();
        wanted.sort_unstable();
        wanted.dedup();
        let base: HashMap<usize, (bool, bool)> = wanted.iter().copied().zip(judged(&chosen, &wanted)).collect();
        let gain = |added: &[usize]| -> Option<(usize, usize)> {
            let mut places: Vec<usize> = added.iter().flat_map(|at| relevant[*at].iter().copied()).collect();
            places.sort_unstable();
            places.dedup();
            let before = summed(&places.iter().filter_map(|at| base.get(at).copied()).collect::<Vec<(bool, bool)>>());
            let lines: Vec<String> = chosen.iter().cloned().chain(added.iter().map(|at| candidates[*at].0.clone())).collect();
            let model = written_with(library, place, (&lines, Vec::new()));
            let after = summed(&places.iter().map(|at| marked(&model, pool[*at])).collect::<Vec<(bool, bool)>>());
            (after.0 > before.0 && after.1 <= before.1).then(|| (after.0 - before.0, added.iter().map(|at| candidates[*at].2).sum()))
        };
        let best_of = |tries: Vec<Vec<usize>>| -> Option<(Vec<usize>, (usize, usize))> {
            let gains: Vec<Option<(usize, usize)>> = on_lanes(tries.len(), hardware_lanes(), &|at| gain(&tries[at]));
            tries.into_iter().zip(gains).filter_map(|(added, counts)| counts.map(|counts| (added, counts))).max_by(|one, other| (one.1).0.cmp(&(other.1).0).then_with(|| (one.1).1.cmp(&(other.1).1)).then_with(|| other.0.cmp(&one.0)))
        };
        let top: Vec<usize> = left.iter().copied().take(MOST_PAIRED_FACTS).collect();
        let found = best_of(left.iter().map(|at| vec![*at]).collect()).or_else(|| best_of((0..top.len()).flat_map(|one| (one + 1..top.len()).map(move |other| (one, other))).map(|(one, other)| vec![top[one], top[other]]).collect()));
        let Some((added, _)) = found else { break };
        chosen.extend(added.iter().map(|at| candidates[*at].0.clone()));
        left.retain(|at| !added.contains(at));
    }
    chosen
}
because!(chosen_facts, LearnedWording, "the facts chosen from candidates on top of those chosen before: the candidates are the facts that a rule or a question form of the text needs and that no story nor known fact holds, filled from the stories and the questions (suggested_facts), with no name of the lessons; each candidate is judged on the lessons whose story or question holds its words (up to a sample of them), against the facts chosen so far on the same lessons, so that a fact of a rare word (unsere) is judged where it matters; at each step the candidate that adds the most right answers with no new wrong answer is kept, and every step judges each candidate again (a fact may answer wrong while others are unknown); when no fact alone adds a lesson, the best pair of the most suggested candidates is kept (a determiner and a helping verb of one German question); a judgment reads its few dozen lessons on the thread that asks, as the search makes thousands of judgments, and a set of threads started for each of them waits for its slowest thread at each judgment when the machine is busy (the German question kinds held the board for many times their work)");

fn question_frame(cases: &[&Case], percent: usize) -> HashSet<WordId> {
    let mut uses: HashMap<WordId, usize> = HashMap::new();
    for case in cases {
        for word in fact_numbers(&case.question).into_iter().collect::<HashSet<WordId>>() {
            *uses.entry(word).or_default() += 1;
        }
    }
    uses.into_iter().filter(|(word, count)| word_not_mark(word_text(*word)) && count * WHOLE_SHARE >= cases.len() * percent).map(|(word, _)| word).collect()
}
because!(question_frame, LearnedWording, "the words of the frame of the questions of some lessons: the words that at least the given share of them hold (where, was, before, lives, in), no marks");

const FIXED_FRAME_PERCENT: usize = 90;
because!(FIXED_FRAME_PERCENT, LearnedWording, "the least share of the questions of a kind that hold a word for a rule to keep it as a fixed word where the answer leaves it open: a word of the frame stands in nearly every question (lives, in), while a value stands in a part of them, and in a few lessons a value may stand in half of them (the apple of the first questions of a kind)");

const FORM_FRAME_PERCENT: usize = 50;
because!(FORM_FRAME_PERCENT, LearnedWording, "the least share of the questions of a kind that hold a word for it to be a word of their frame (where, was, before), which a written question form must hold to answer the kind as it asks: a form that asks any question (Is A?) holds none of them, and a search for facts through it would judge nothing that the kind asks");

fn asking_texts(library: &Library, fit: &[&Case]) -> Vec<usize> {
    let questions: Vec<Vec<WordId>> = fit.iter().map(|case| fact_numbers(&case.question)).collect();
    let frame = question_frame(fit, FORM_FRAME_PERCENT);
    let holds_frame = |form: &AskedForm| frame.iter().all(|word| form.question.contains(&LogicPiece::Word(*word)));
    let mut asking: Vec<(usize, usize, usize)> = library
        .topics
        .iter()
        .enumerate()
        .filter_map(|(place, topic)| {
            let fixed: Vec<usize> = questions.iter().filter_map(|words| topic.questions.iter().filter(|form| (holds_frame(form) || fixed_count(&form.question) * WHOLE_SHARE >= words.len() * FORM_FRAME_PERCENT) && asks(form, words)).map(|form| fixed_count(&form.question)).max()).collect();
            (!fixed.is_empty() && fixed.len() * WHOLE_SHARE >= questions.len() * LEAST_LEARNED_PERCENT).then(|| (fixed.iter().copied().max().unwrap_or_default(), fixed.len(), place))
        })
        .collect();
    asking.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| other.1.cmp(&one.1)).then_with(|| one.2.cmp(&other.2)));
    asking.into_iter().map(|(_, _, place)| place).take(MOST_RULE_TEXTS).collect()
}
because!(asking_texts, LearnedWording, "the texts whose written question forms that hold the frame words of the questions of some lessons, or fix at least half of the words of a question (a form of each kind of question of a set whose questions share few words), ask at least half of them, those whose asking forms hold the most fixed words first (Where was A before the B before a form that asks any question), then those that ask the most questions, a few of them: the texts that a search for the facts that the lessons show tries");

const MOST_PAIRED_FACTS: usize = 16;
because!(MOST_PAIRED_FACTS, LearnedWording, "the facts suggested by the most lessons that the search for facts judges in pairs when no fact alone adds a lesson: two facts of different kinds may pay off only together (a determiner and a helping verb of one question), and the pairs of a few dozen candidates are a few hundred readings of the sample");

pub(super) fn written_facts_model(library: &Library, lessons: &Lessons) -> Option<(WordedModel, bool)> {
    let (first, last) = inner_parts(lessons.cases);
    let fit: Vec<&Case> = first.iter().map(|at| &lessons.cases[*at]).collect();
    let check: Vec<&Case> = last.iter().map(|at| &lessons.cases[*at]).collect();
    for place in asking_texts(library, &fit) {
        let facts = written_facts(library, place, &fit);
        if facts.is_empty() {
            continue;
        }
        let model = written_with(library, place, (&facts, Vec::new()));
        let (right, _) = right_count(&model, &check);
        if right * WHOLE_SHARE >= check.len() * LEAST_LEARNED_PERCENT {
            learning_note(format!("no written text read the kind, and the text {} reads {right} of {} last lessons with facts of knowledge learned from its lessons: {}", library.topics[place].name, check.len(), facts.join(PATH_LINES_GAP)));
            return Some((model, right * WHOLE_SHARE >= check.len() * TOLD_TRY_PERCENT));
        }
    }
    None
}
because!(written_facts_model, LearnedWording, "when no written text reads a kind (tried in the learned wording before the search for the rules of a concept when no plain wording reads the kind, as that search judges hundreds of candidates and finds nothing when a text only lacks its facts, and kept there only when it reads the kind as a learned kind must), a text that asks its questions (asking_texts) with the facts of knowledge that its lessons show and that it lacks (written_facts), when it then answers at least half of the last lessons");

pub(super) fn facts_beside_written(library: &Library, (first, last): (&[&Case], &[Case]), written: usize) -> Option<WordedModel> {
    let _facts = cost_scope(Some(WAY_FACTS), None);
    let mut best: Option<(usize, WordedModel, Vec<String>, usize)> = None;
    for place in asking_texts(library, first) {
        if scope_spent() {
            break;
        }
        let facts = written_facts(library, place, first);
        if facts.is_empty() {
            continue;
        }
        let model = written_with(library, place, (&facts, Vec::new()));
        let right = last.iter().take_while(|_| !scope_spent()).filter(|case| model.answer(&case.sentences, &case.question).words == answer_words(&case.answer)).count();
        if right > written && best.as_ref().is_none_or(|(known, _, _, _)| right > *known) {
            best = Some((right, model, facts, place));
        }
    }
    let (right, model, facts, place) = best?;
    learning_note(format!("the written texts read {written} of {} last lessons, below the bar, and {right} with facts of knowledge that the text {} lacks, learned from the lessons: {}", last.len(), library.topics[place].name, facts.join(PATH_LINES_GAP)));
    Some(model)
}
because!(facts_beside_written, LearnedWording, "a text that asks the questions of a kind (asking_texts) with the facts of knowledge that it lacks and the first lessons show (written_facts), in place of the written topics, when it answers more of the last lessons than they do, the text that answers most; nothing when no fact is found or none answers more");

pub(super) fn bridged_written(library: &Library, place: usize, lessons: &Lessons, (first, last): (&[&Case], &[Case]), written: usize) -> Option<WordedModel> {
    let _bridges = cost_scope(Some(WAY_BRIDGES), Some(&library.topics[place].name));
    let before = lessons.before.map(|read| read());
    let (bridges, taught) = bridge_lines(state_bridges(library, place, (first, before.as_deref())));
    let model = bridged_model(library, place, (&bridges, taught))?;
    let right = last.iter().filter(|case| model.answer(&case.sentences, &case.question).words == answer_words(&case.answer)).count();
    if right <= written {
        return None;
    }
    learning_note(format!("the written texts read {written} of {} last lessons, below the bar, and {} with the bridges of the invented states of the kind and the kinds before it: {}", last.len(), right, bridges.join(PATH_LINES_GAP)));
    let every: Vec<&Case> = lessons.cases.iter().collect();
    let (bridges, taught) = bridge_lines(state_bridges(library, place, (&every, before.as_deref())));
    bridged_model(library, place, (&bridges, taught))
}
because!(bridged_written, LearnedWording, "the written topic that reads a kind best, with the bridges of the invented states of the kind and the kinds before it, in place of the written topics, when it answers more of the last lessons than they do with the bridges learned from the first lessons, then the bridges learned again from all lessons; nothing when it answers no more");

pub(super) fn rule_ground(topic: &Topic, base: &Arc<Facts>, (cases, asked): (&[&Case], &[(Vec<WordId>, Vec<WordId>)])) -> usize {
    if topic.rules.is_empty() {
        return if aligned_ways(topic, cases).is_empty() { usize::default() } else { WHOLE_SHARE };
    }
    let holders = known_holders(base);
    let grounded = cases
        .iter()
        .zip(asked)
        .filter(|(case, (things, values))| {
            reply_target(topic, case).is_some()
                || things.iter().any(|word| holders.get(word).is_some_and(|places| places.iter().take(MOST_KNOWN_JOINS).any(|place| values.iter().any(|value| base.fact(*place).contains(value)))))
        })
        .count();
    grounded * WHOLE_SHARE / cases.len().max(1)
}
because!(rule_ground, LearnedWording, "the share of the lessons of a sample for which a text could learn the rules of a concept: for a text with no rules of its own (a concept whose given rules are taken out), all of them when a reply line of the text answers the question of the lessons in another form with as many places (aligned_ways), as the text then names the asked fact, and none otherwise, as a text of question forms alone names no other asked fact, and the reply lines of texts with rules aligned with the questions of most kinds of a few places, searches that found nothing and cost much of the board's time; for a text with rules, the lessons where a reply line of the text says the answer to its question, or its question names a thing that its story does not tell (not a word that most questions of the kind hold, as where and is) in a known fact of the text that also holds a thing that the story names (the country and the capital in one fact), as a rule then joins what the story tells to what the text knows");

pub(super) fn asked_things(cases: &[&Case]) -> Vec<(Vec<WordId>, Vec<WordId>)> {
    let mut asked: HashMap<WordId, usize> = HashMap::new();
    let mut told: HashMap<WordId, usize> = HashMap::new();
    for case in cases {
        for word in stated(&case.question).into_iter().collect::<HashSet<WordId>>() {
            *asked.entry(word).or_default() += 1;
        }
        for word in case.sentences.iter().flat_map(|sentence| fact_numbers(sentence)).collect::<HashSet<WordId>>() {
            *told.entry(word).or_default() += 1;
        }
    }
    let seldom = |counts: &HashMap<WordId, usize>, word: &WordId| counts.get(word).is_some_and(|count| count * WHOLE_SHARE < cases.len() * BROAD_PERCENT);
    cases
        .iter()
        .map(|case| {
            let story: HashSet<WordId> = case.sentences.iter().flat_map(|sentence| fact_numbers(sentence)).collect();
            let things: Vec<WordId> = stated(&case.question).into_iter().filter(|word| word_not_mark(word_text(*word)) && seldom(&asked, word) && !story.contains(word)).collect();
            let values: Vec<WordId> = story.into_iter().filter(|word| word_not_mark(word_text(*word)) && seldom(&told, word)).collect();
            (things, values)
        })
        .collect()
}
because!(asked_things, LearnedWording, "for each lesson of a sample, the things that its question names and its story does not tell (words that most questions of the kind do not hold), and the things that its story names (words that most stories do not hold), found once for all the texts that may ground the kind");

pub fn known_holders(base: &Arc<Facts>) -> Arc<HashMap<WordId, Vec<usize>>> {
    let key = base.id();
    if let Some(known) = KNOWN_HOLDERS.get_or_init(Mutex::default).lock().ok().and_then(|held| held.get(&key).map(|(_, holders)| Arc::clone(holders))) {
        return known;
    }
    let mut holders: HashMap<WordId, Vec<usize>> = HashMap::new();
    for place in 0..base.added_to(&Facts::default()) {
        for word in base.fact(place) {
            let places = holders.entry(*word).or_default();
            if places.last() != Some(&place) {
                places.push(place);
            }
        }
    }
    let holders = Arc::new(holders);
    if let Ok(mut held) = KNOWN_HOLDERS.get_or_init(Mutex::default).lock() {
        held.retain(|_, (set, _)| set.strong_count() > 0);
        held.insert(key, (Arc::downgrade(base), Arc::clone(&holders)));
    }
    holders
}
because!(known_holders, LearnedWording, "for the known facts of a text, the facts that hold each word, made once for each set of known facts in a run and found by the number of the set (FactsId), as a set made after another is let go may take its place in memory and has other facts");

static KNOWN_HOLDERS: OnceLock<Mutex<HashMap<u64, (Weak<Facts>, Arc<HashMap<WordId, Vec<usize>>>)>>> = OnceLock::new();
because!(KNOWN_HOLDERS, LearnedWording, "the facts that hold each word, for each set of known facts of the library, by the number of the set, with a weak reference that lets the entry go with its set: every kind that nothing reads asks which texts ground it, and the known facts do not change while the program runs");

pub(super) const RULE_GROUND_PERCENT: usize = 50;
because!(RULE_GROUND_PERCENT, LearnedWording, "the least share of the lessons of a sample that a text must ground before the board learns the rules of its concept for a kind: a concept that a kind asks grounds most lessons (a reply line for each question; a known fact for each country asked in the lessons whose label says the question holds, which are about half of a kind of two labels), while a text that grounds a few lessons by chance would cost a search of rules for nothing");

const MOST_RULE_TEXTS: usize = 2;
because!(MOST_RULE_TEXTS, LearnedWording, "the texts for which the board learns a concept's rules for a kind that nothing else reads, the most grounded first, until one gives a model: a text of known facts alone (the capitals) grounds a kind as well as the concept that builds on it (geography) but has no sentence that reads the stories, and its search ends at once with no rule");

const MOST_TOLD_TEXTS: usize = 2;
because!(MOST_TOLD_TEXTS, LearnedWording, "the texts with rules and no question forms whose rules do not yet make facts from most stories of a sample (a text whose rules read the stories gets its wording without a told side), those whose rules speak most of the told sentences first and each set of rules once, with which the board tries to learn the told side of a kind that no learned wording of a text reads wholly: the told learner judges each way to read each sentence form with the whole wording, so it is tried on the few texts whose rules the stories speak of most");

const TOLD_TRY_PERCENT: usize = 95;
because!(TOLD_TRY_PERCENT, LearnedWording, "the share of the last lessons below which the board tries to learn the told side of a kind: a learned wording that answers this share already reads the stories as a learned kind must, and the told learner, which judges many readings, would cost minutes for a few lessons more");

const TOLD_FILLED_PERCENT: usize = 90;
because!(TOLD_FILLED_PERCENT, LearnedWording, "the least share of the first lessons whose answer their story tells word for word, for the board to try the told side with the text whose rules speak most of the told sentences though they speak of only some: a kind whose answers are the things that its stories name (who holds which book) has an answer to fill in nearly every lesson, and its first reading by filled answers is cheap and sure, while a kind that answers none in about half its lessons (the slots of a request) gives the first reading little to go on and would cost its whole search for nothing");

const MOST_TOLD_FORMS: usize = 64;
because!(MOST_TOLD_FORMS, LearnedWording, "the most sentence forms of the told sentences of a kind for which the board learns the told side: each form is judged with each of its readings, so a kind told in hundreds of forms (the words of people) would cost minutes, while a kind that a program wrote tells far fewer, though a story in many settings (books, balls, dances) tells some forms in each, and fewer lessons cut its sentences into more forms, as fewer words are seen as words of the frame");

fn told_relevance(topic: &Topic, cases: &[&Case]) -> usize {
    let conditions: Vec<&Pattern> = topic.rules.iter().flat_map(|rule| &rule.conditions).filter_map(|condition| match condition {
        Condition::Fact { pattern, .. } if pattern.iter().filter(|piece| matches!(piece, LogicPiece::Word(word) if word_not_mark(word_text(*word)))).count() >= TOLD_MATCH_WORDS => Some(pattern),
        _ => None,
    }).collect();
    let told: Vec<Vec<WordId>> = cases.iter().flat_map(|case| case.sentences.iter().map(|sentence| fact_numbers(sentence))).collect();
    let held = told.iter().filter(|words| conditions.iter().any(|pattern| holds_words(pattern, words))).count();
    held * WHOLE_SHARE / told.len().max(1)
}
because!(told_relevance, LearnedWording, "how much the rules of a text speak of what the stories tell: the share in hundredths of the told sentences of the sample that hold in their order the words (not marks) of a condition of a rule that names at least two words, so that a text whose conditions name what the stories say ('says ... lies', 'tells the truth') counts and a condition that holds any sentence (a mark, a number, one common word) does not");

const TOLD_MATCH_WORDS: usize = 2;
because!(TOLD_MATCH_WORDS, LearnedWording, "the fewest words (not marks) of a condition of a rule that a told sentence must hold for the rule to speak of it: one common word such as 'is' stands in almost any sentence");

fn answers_told(cases: &[&Case]) -> usize {
    cases
        .iter()
        .filter(|case| {
            let answer = fact_words(&case.answer);
            !answer.is_empty() && case.sentences.iter().any(|sentence| fact_words(sentence).windows(answer.len()).any(|run| run == answer.as_slice()))
        })
        .count()
}
because!(answers_told, LearnedWording, "how many of the given lessons have an answer that their story tells word for word, as a thing that it names");

const TOLD_RELEVANT_PERCENT: usize = 75;
because!(TOLD_RELEVANT_PERCENT, LearnedWording, "the share of the told sentences of the sample that the conditions of a text must speak of before the board learns the told side with that text: the told learner reads every sentence form, and a text that speaks of a few sentences cannot read the stories, while its search costs as much");

pub(super) fn told_frame_and_forms(cases: &[&Case]) -> (Vec<String>, Vec<(String, usize)>, usize) {
    let lessons: Vec<&Case> = cases.iter().take(READ_TOLD_LESSONS).copied().collect();
    let frame = frame_words(&lessons);
    let mut counted: Vec<(Pattern, usize)> = Vec::new();
    for case in &lessons {
        let mut seen: Vec<Pattern> = Vec::new();
        for sentence in &case.sentences {
            let Some((form, _)) = sentence_form(&fact_words(sentence), &frame, &HashSet::new(), MOST_TOLD_PLACES) else { continue };
            if seen.contains(&form) {
                continue;
            }
            seen.push(form.clone());
            match counted.iter_mut().find(|(known, _)| *known == form) {
                Some((_, count)) => *count += 1,
                None => counted.push((form, 1)),
            }
        }
    }
    counted.sort_by(|one, other| other.1.cmp(&one.1));
    let mut words: Vec<String> = frame.into_iter().filter(|word| word_not_mark(word)).collect();
    words.sort();
    (words, counted.into_iter().map(|(form, count)| (library_writing(&form), count)).collect(), lessons.len())
}
because!(told_frame_and_forms, LearnedWording, "the frame words of the told sentences of a kind (frame_words, marks aside, in the order of the alphabet) and the forms that they cut the told sentences into, each with the lessons that tell it, the most told first, over as many lessons as the told learner reads: a word of a value taken as a frame word splits the forms of one sentence (A moved the B to C from D room beside A moved the B to C from D)");

fn told_form_count(cases: &[&Case]) -> usize {
    let lessons: Vec<&Case> = cases.iter().take(READ_TOLD_LESSONS).copied().collect();
    let frame = frame_words(&lessons);
    let mut forms: HashMap<Pattern, usize> = HashMap::new();
    for case in &lessons {
        for sentence in &case.sentences {
            if let Some((form, _)) = sentence_form(&fact_words(sentence), &frame, &HashSet::new(), MOST_TOLD_PLACES) {
                *forms.entry(form).or_default() += 1;
            }
        }
    }
    forms.values().filter(|uses| **uses >= LEAST_SUPPORT).count()
}
because!(told_form_count, LearnedWording, "the sentence forms with enough uses that the told learner would read in the given lessons, as it cuts the told sentences");

type LearnedCell = Arc<Mutex<Option<Worked<Vec<String>>>>>;
because!(LearnedCell, LearnedWording, "the place of one learning in a memo of learnings: empty until the learning is made, and held by the kind that makes it, so that a kind that asks for the same learning meanwhile waits for it and takes it, as the kinds of a set that share their lessons ask at the same time");

fn over_kinds_before(library: &Library, place: usize, (fit, kinds_before): (&[&Case], &[Vec<Case>]), (taught, taught_base, earlier): (&Topic, &Arc<Facts>, Vec<String>)) -> (Vec<String>, Vec<usize>) {
    let bridges = state_bridges(library, place, (fit, Some(kinds_before)));
    let mut lines: Vec<String> = earlier.into_iter().chain(learn_rules(taught, taught_base, fit)).collect();
    let mut kinds: Vec<usize> = Vec::new();
    for (line, from) in bridges {
        if !lines.contains(&line) {
            lines.push(line);
            kinds.extend(from);
        }
    }
    kinds.sort_unstable();
    kinds.dedup();
    (lines, kinds)
}
because!(over_kinds_before, LearnedWording, "the lines of a concept learned over the kinds before it in its set, with the places of the kinds before it whose bridges joined them: the rules that they teach, the rules learned on top of them from the kind's own lessons, and the kind's own bridges of a state, which read the verbs of its stories into the events of those rules (picked up as got, for the state that babi02 taught, which babi02's lessons tell only with there)");

fn learned_once(kept: &OnceLock<Mutex<HashMap<(usize, u64, u64), LearnedCell>>>, (library, place, cases): (&Library, usize, &[&Case]), learn: &dyn Fn(&Topic, &Arc<Facts>, &[&Case]) -> Vec<String>) -> Vec<String> {
    learned_once_of(kept, (place, &library.topics[place], library.base(place), cases), learn)
}

fn learned_once_of(kept: &OnceLock<Mutex<HashMap<(usize, u64, u64), LearnedCell>>>, (place, topic, base, cases): (usize, &Topic, &Arc<Facts>, &[&Case]), learn: &dyn Fn(&Topic, &Arc<Facts>, &[&Case]) -> Vec<String>) -> Vec<String> {
    let key = (place, rules_with_store(topic), lessons_key(cases));
    let cell = kept.get_or_init(Mutex::default).lock().map(|mut learned| Arc::clone(learned.entry(key).or_default())).unwrap_or_default();
    let mut known = cell.lock().unwrap_or_else(PoisonError::into_inner);
    let (lines, fresh) = learned_within(known.as_ref(), || learn(topic, base, cases));
    if fresh.is_some() {
        *known = fresh;
    }
    lines
}
because!(learned_once, LearnedWording, "what one learner learns for one text of the library from the given lessons (learned_once_of)");

pub(super) fn rules_with_store(topic: &Topic) -> u64 {
    let mut hasher = DefaultHasher::new();
    (topic.rules_key, store_key(&topic.name)).hash(&mut hasher);
    hasher.finish()
}
because!(rules_with_store, LearnedWording, "the key of a text for what is learned once for it: its rules and its stored lines, as the store changes what its rule search takes");
because!(learned_once_of, LearnedWording, "what one learner learns for one text from the given lessons (the rules of a concept, or the rules that read the told sentences), learned once for each text and lessons in a run, also when it is nothing, in the memo of that learner, under the bound of work of a kept learning (learned_within); a kind that asks while another kind makes it waits for it (the nine CLUTRR graph kinds learn the rules of family from the same lessons)");

static RULES_LEARNED: OnceLock<Mutex<HashMap<(usize, u64, u64), LearnedCell>>> = OnceLock::new();

pub(super) fn searched_nothing(library: &Library, place: usize, (topic, fit): (&Topic, &[&Case])) -> Option<Worked<Vec<String>>> {
    if topic.rules_key != library.topics[place].rules_key {
        return None;
    }
    let left = work_left();
    let cell = RULES_LEARNED.get()?.lock().ok()?.get(&(place, topic.rules_key, lessons_key(fit))).cloned()?;
    let known = cell.lock().unwrap_or_else(PoisonError::into_inner);
    known.as_ref().filter(|worked| worked.value.is_empty() && !worked.stopped && worked.work <= left).cloned()
}
because!(searched_nothing, LearnedWording, "the search for the rules of a concept that a kind ran for a text in its own learning, on the same text and lessons, when it found nothing (and did not stop at the bound of its work), so that the curriculum of a later kind skips that kind as a step; given only when its work is within the work left of the learning that asks, as the memo of a kept learning is taken (learned_within), so that the later kind adds that work as its own and counts the same whether the search ran before it or not");
because!(RULES_LEARNED, LearnedWording, "the rules of a concept that each text learned from some lessons, by the text, its rules and the lessons, kept for the run: the board learns a kind from the same first lessons when it judges the tool on the first part and again when it sets the bar of the written forms on all lessons, and a search that finds no rule costs as much as one that finds some");

fn told_model(library: &Library, place: usize, lines: &[String], cases: &[&Case]) -> Option<WordedModel> {
    learned_model((&library.topics[place], library.base(place), hand_rules(library, place)), lines, cases)
}

fn learned_model((topic, base, hand): (&Topic, &Arc<Facts>, bool), lines: &[String], cases: &[&Case]) -> Option<WordedModel> {
    if lines.is_empty() {
        return None;
    }
    let (learned, learned_base) = with_learned(topic, base, lines);
    let (forms, _) = learn_forms(&learned, &learned_base, cases);
    let aligned = if topic.rules.is_empty() { aligned_form(&learned, &learned_base, cases) } else { None };
    if forms.is_empty() && aligned.is_none() {
        return None;
    }
    let read = read_topic(&topic.name, &forms.iter().map(form_line).collect::<Vec<String>>().join(LEARNED_LINE_BREAK)).ok()?;
    let mut questions = read.questions;
    if let Some(kept) = aligned {
        questions.insert(usize::default(), kept);
    }
    let base = timed_base(&learned, &questions, &learned_base);
    Some(WordedModel { topic: Topic { questions, ..learned }, base, source: topic.name.clone(), hand, learned: lines.iter().cloned().chain(forms.iter().map(form_line)).collect(), taught: Vec::new(), named: (named_forms(&forms, cases).into_iter().filter(|named| *named).count(), forms.len()), frames: None })
}

fn aligned_form(topic: &Topic, base: &Arc<Facts>, cases: &[&Case]) -> Option<AskedForm> {
    let ways = aligned_ways(topic, cases);
    if ways.is_empty() {
        return None;
    }
    let words = question_words(cases);
    let mut shapes: Vec<(Pattern, usize)> = Vec::new();
    for case in cases {
        let Some(shape) = form_of(case, &words, Cut::Share(FIXED_SHARE_PERCENT)) else { continue };
        match shapes.iter_mut().find(|(known, _)| *known == shape) {
            Some((_, count)) => *count += 1,
            None => shapes.push((shape, 1)),
        }
    }
    let (shape, _) = shapes.into_iter().max_by(|one, other| one.1.cmp(&other.1))?;
    let facts: Vec<Option<Facts>> = cases.iter().map(|case| bounded_facts(case, base, topic, false)).collect();
    let right = |form: &AskedForm| cases.iter().zip(&facts).filter(|(case, found)| found.as_ref().and_then(|found| reply(std::slice::from_ref(form), &fact_numbers(&case.question), found)).is_some_and(|answered| answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP)) == answer_words(&case.answer))).count();
    ways.iter().filter_map(|way| if let AnswerFact::Aligned(line, order) = way { kept_probe(topic, (*line, order), &shape) } else { None }).map(|form| (right(&form), form)).filter(|(count, _)| *count > usize::default()).max_by(|one, other| one.0.cmp(&other.0)).map(|(_, form)| form)
}
because!(aligned_form, LearnedWording, "the text's own question form read for the question of a kind of another form (kept_probe), first among the model's forms, in the order of places that answers the most of the lessons right, when a reply line of the text aligns with the kind's question: the learned rules make the facts that the text's form reads (a parent, a sex), and the form gives the answer from them as the text gives it for its own question; nothing when no order answers a lesson; only for a text with no rules of its own, as rule_ground grounds no other text in this way");
because!(told_model, LearnedWording, "the model of one text of the library with learned rules (learned_model), mixed when the text or one it builds on brings rules of a wording written by a person");
because!(learned_model, LearnedWording, "the model of one text with learned rules that read the told sentences, and the wording learned with them from the given lessons, after the text's own question form read for the kind's question of another form when a reply line aligns (aligned_form), also when no form of the kind's own is learned (a family document whose answers only the text's own form reads), as the wording command learns them with its told option; nothing when no rule is learned, or no form of either kind");

fn learned_lines(library: &Library, place: usize, cases: &[&Case]) -> Option<String> {
    lines_of(&learn_forms(&library.topics[place], library.base(place), cases).0)
}
because!(learned_lines, LearnedWording, "the lines of the wording that one text learns from the given lessons, or nothing when it learns no form");

fn lines_of(forms: &[LearnedForm]) -> Option<String> {
    (!forms.is_empty()).then(|| forms.iter().map(form_line).collect::<Vec<String>>().join(LEARNED_LINE_BREAK))
}
because!(lines_of, LearnedWording, "learned forms as lines of the library, or nothing when there is no form");

pub fn named_forms(forms: &[LearnedForm], fit: &[&Case]) -> Vec<bool> {
    forms
        .iter()
        .enumerate()
        .map(|(at, form)| {
            let questions: Vec<Vec<WordId>> = form.taught.iter().filter_map(|lesson| fit.get(*lesson)).map(|case| fact_numbers(&case.question)).collect();
            !questions.is_empty()
                && forms.iter().enumerate().any(|(other_at, other)| {
                    other_at != at
                        && fixed_count(&other.question) < fixed_count(&form.question)
                        && questions.iter().all(|question| {
                            let mut found = Vec::new();
                            fit_into(&other.question, question, &empty_filling(), &mut found);
                            !found.is_empty()
                        })
                })
        })
        .collect()
}
because!(named_forms, LearnedWording, "for each learned form, whether it names a word that another learned form leaves open: a form with fewer fixed words fits every question of its lessons, so the form keeps a value as a fixed word (one form for each colour beside the form of any colour), a sign that the wording learns the values of its lessons one by one and not the frame of its questions; the forms index the lessons that they were learned from");

fn worded_final(library: &Library, place: usize, cases: &[&Case]) -> Option<WordedModel> {
    let (forms, _) = learn_forms(&library.topics[place], library.base(place), cases);
    let named = named_forms(&forms, cases).into_iter().filter(|named| *named).count();
    let model = worded_by(library, place, &lines_of(&forms)?)?;
    Some(WordedModel { named: (named, forms.len()), ..model })
}
because!(worded_final, LearnedWording, "the model of the chosen text with the wording learned again from all the lessons, with how many of its forms name a word that a form with fewer fixed words reads too, of all its forms");

fn timed_base(topic: &Topic, questions: &[AskedForm], base: &Arc<Facts>) -> Arc<Facts> {
    if base.has_history() || !keeps_history(&topic.rules, questions) {
        return Arc::clone(base);
    }
    Arc::new(knowledge_base(&topic.facts, &topic.rules, true))
}
because!(timed_base, LearnedWording, "the facts before any story that a model with learned question forms reads: the base of its text, made again with the history of the facts when a learned form reads a fact at the time of a clause before it and the base keeps no history, as a fact that came twice holds at both times (where a person was before a place that the person went to twice)");

fn worded_by(library: &Library, place: usize, lines: &str) -> Option<WordedModel> {
    let topic = &library.topics[place];
    let read = read_topic(&topic.name, lines).ok()?;
    let base = timed_base(topic, &read.questions, library.base(place));
    Some(WordedModel { topic: Topic { questions: read.questions, ..topic.clone() }, base, source: topic.name.clone(), hand: hand_rules(library, place), learned: lines.split(LEARNED_LINE_BREAK).filter(|line| !line.is_empty()).map(str::to_string).collect(), taught: Vec::new(), named: (usize::default(), usize::default()), frames: None })
}
because!(worded_by, LearnedWording, "the model of one text with learned lines of wording, read as lines of the library");

fn worded_from(library: &Library, place: usize, cases: &[&Case]) -> Option<WordedModel> {
    worded_by(library, place, &learned_lines(library, place, cases)?)
}
because!(worded_from, LearnedWording, "the model of one text with the wording learned from the given lessons");

pub(super) fn lessons_key(cases: &[&Case]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for case in cases {
        case.sentences.hash(&mut hasher);
        case.question.hash(&mut hasher);
        case.answer.hash(&mut hasher);
    }
    hasher.finish()
}
because!(lessons_key, LearnedWording, "the key of some lessons in their order, by their stories, questions and answers, under which what a text learned from them is kept for the run");

fn searched_from(library: &Library, place: usize, cases: &[&Case]) -> Option<WordedModel> {
    let key = (place, lessons_key(cases));
    let known = SEARCHED.get_or_init(Mutex::default).lock().ok().and_then(|searched| searched.get(&key).cloned());
    let (lines, fresh) = learned_within(known.as_ref(), || learned_lines(library, place, cases));
    if let Some(fresh) = fresh
        && let Ok(mut searched) = SEARCHED.get_or_init(Mutex::default).lock()
    {
        searched.insert(key, fresh);
    }
    worded_by(library, place, &lines?)
}
because!(searched_from, LearnedWording, "the model of one text with the wording learned from the first lessons of a search, learned once for each text and lessons in a run");

const READ_TOLD_LESSONS: usize = 1_000;
because!(READ_TOLD_LESSONS, LearnedWording, "the most lessons whose told sentences are cut into forms and read by learned rules, as many as the board gives a kind, since a kind tells the same thing in many forms and each form needs a few uses");

const TOLD_FIXED_PERCENT: usize = 10;
because!(TOLD_FIXED_PERCENT, LearnedWording, "a word of a told sentence is a word of its form when it stands in at least this share of the told sentences: a kind tells a few kinds of sentence, and the words of each kind come again in all of them, while the names of things, places and subjects change");

const MOST_TOLD_PLACES: usize = 10;
because!(MOST_TOLD_PLACES, LearnedWording, "a told sentence form has at most this many open places: a sentence tells a few things with a few values each, or pairs a short list of things with a list of values");

const TOLD_SAMPLE: usize = 20;
because!(TOLD_SAMPLE, LearnedWording, "the lessons of a sentence form on which each way to read it is judged");

const RELEVANT_PERCENT: usize = 50;
because!(RELEVANT_PERCENT, LearnedWording, "a way to read a sentence form into a fact of a text is judged only when the words it gives to the fact stand in the question in at least this share of the lessons of the form, since a reading that the questions never ask about cannot answer them");

const SMALL_FORM_PLACES: usize = 2;
because!(SMALL_FORM_PLACES, LearnedWording, "a sentence form with at most this many open places has few ways to read it into a fact, so every way is judged, whether or not the questions name its words");

const MOST_UNFILTERED: usize = 24;
because!(MOST_UNFILTERED, LearnedWording, "the most ways to read a form into one fact that are judged without asking that the questions name their words: a list of facts (one edge of a graph in each sentence) is seldom named by the question, which asks about two of its things, and its few readings must still be tried, while a long sentence has too many");

pub(super) const TOLD_IF: &str = "If ";
because!(TOLD_IF, LearnedWording, "the start of a learned rule as a line of the library");

pub(super) const TOLD_THEN: &str = " then ";
because!(TOLD_THEN, LearnedWording, "the word between the condition and the outcome of a learned rule");

const TOLD_NAME: &str = "told";
because!(TOLD_NAME, LearnedWording, "the name under which the learned rules are read by the reader of the library");

pub(crate) fn with_learned(topic: &Topic, base: &Arc<Facts>, lines: &[String]) -> (Topic, Arc<Facts>) {
    let mut out = with_rules(topic, lines);
    let Ok(read) = read_topic(TOLD_NAME, &lines.join(LEARNED_LINE_BREAK)) else { return (out, Arc::clone(base)) };
    let history = keeps_history(&out.rules, &out.questions);
    if read.facts.is_empty() && (base.has_history() || !history) {
        return (out, Arc::clone(base));
    }
    out.facts.extend(read.facts);
    let learned_base = Arc::new(knowledge_base(&out.facts, &out.rules, history));
    (out, learned_base)
}
because!(with_learned, LearnedWording, "a text with learned rules and learned told facts, and its facts before any story: the base of the text when nothing was told, else a base made again with the learned facts, and made again also when a learned rule reads a fact at the time of a clause before it and the base keeps no history, as a fact told twice holds at both times");

fn with_rules(topic: &Topic, lines: &[String]) -> Topic {
    let mut out = topic.clone();
    if let Ok(read) = read_topic(TOLD_NAME, &lines.join(LEARNED_LINE_BREAK)) {
        let mut rules = read.rules;
        for rule in &mut rules {
            rule.learned = true;
        }
        rules.extend(out.rules);
        out.rules = rules;
        out.rules_key = rules_key(&out.rules);
    }
    out
}
because!(with_rules, LearnedWording, "a text with learned rules read before its own rules, so that the facts that they make from the told sentences reach the rules of the text; each learned rule is marked as learned, so that the engine fills its places with different things");


const VALUE_MARK: &str = "#";
because!(VALUE_MARK, LearnedWording, "how a number stands beside a word when the words around a word are counted: every number is a value, so a word after a time has the same neighbour whatever the time");

const MOST_FRAME_FILLERS: usize = 4;
because!(MOST_FRAME_FILLERS, LearnedWording, "a word of the frame of a told sentence or of a question stands between neighbours where at most this many words stand, as one of a few choices (am, pm or noon after a time, then or after between two things), while a place of a name, a thing or a place holds many words between the same neighbours");

const FRAME_PLACE_PERCENT: usize = 50;

const VALUE_RUN: usize = 2;
because!(VALUE_RUN, LearnedWording, "the length of the least run of a value of several words that a question and a told sentence of one lesson share (store room): a word of such a value stands beside another word of it");
because!(FRAME_PLACE_PERCENT, LearnedWording, "a word is a word of the frame when at least this share of its uses stand between neighbours that few words share");

const ASKED_PERCENT: usize = 50;
because!(ASKED_PERCENT, LearnedWording, "a word that the question of its lesson holds in at least this share of its uses is a value that the question asks about, unless it is common in the told sentences, as the name of a thing is asked about and a word of the frame such as after or pm is not");

fn context_word(word: &str) -> String {
    if is_value(word) { VALUE_MARK.to_string() } else { word.to_string() }
}
because!(context_word, LearnedWording, "a neighbour of a word as it is counted: a number as the mark of a value, a word as itself");

fn is_value(word: &str) -> bool {
    word.chars().all(|letter| letter.is_ascii_digit())
}
because!(is_value, LearnedWording, "whether a word of a told sentence is a number, which is always a value of its form, as the engine reckons with numbers");

pub(super) fn frame_words(lessons: &[&Case]) -> HashSet<String> {
    let mut uses: HashMap<String, (usize, usize)> = HashMap::new();
    let mut fillers: HashMap<(String, String), HashSet<String>> = HashMap::new();
    let mut stands: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut named: HashMap<String, (usize, usize)> = HashMap::new();
    let mut sentences = 0usize;
    let mut asked_in: HashMap<String, usize> = HashMap::new();
    for case in lessons {
        for word in fact_words(&case.question).into_iter().collect::<HashSet<String>>() {
            *asked_in.entry(word).or_default() += 1;
        }
    }
    let varying = |word: &String| word_not_mark(word) && asked_in.get(word).is_some_and(|count| count * WHOLE_SHARE < lessons.len() * FORM_FRAME_PERCENT);
    let mut valued: HashMap<String, usize> = HashMap::new();
    for case in lessons {
        let asked = fact_words(&case.question);
        let told: Vec<Vec<String>> = case.sentences.iter().map(|sentence| fact_words(sentence)).collect();
        let mut shared: HashSet<&String> = HashSet::new();
        for pair in asked.windows(VALUE_RUN) {
            if varying(&pair[0]) && varying(&pair[1]) && told.iter().any(|words| words.windows(VALUE_RUN).any(|told_pair| told_pair == pair)) {
                shared.extend(pair);
            }
        }
        for word in shared {
            *valued.entry(word.clone()).or_default() += 1;
        }
    }
    for case in lessons {
        let question: HashSet<String> = fact_words(&case.question).into_iter().collect();
        capitals_inside(&case.question, &mut named);
        for sentence in &case.sentences {
            sentences += 1;
            let words = fact_words(sentence);
            capitals_inside(sentence, &mut named);
            let seen: HashSet<&String> = words.iter().collect();
            for word in seen {
                let entry = uses.entry(word.clone()).or_default();
                entry.0 += 1;
                entry.1 += usize::from(question.contains(word));
            }
            for (at, word) in words.iter().enumerate() {
                let before = at.checked_sub(1).map_or_else(String::new, |earlier| context_word(&words[earlier]));
                let after = words.get(at + 1).map_or_else(String::new, |later| context_word(later));
                fillers.entry((before.clone(), after.clone())).or_default().insert(word.clone());
                stands.entry(word.clone()).or_default().push((before, after));
            }
        }
    }
    let mut out: HashSet<String> = HashSet::new();
    for (word, (held, asked)) in &uses {
        if !word_not_mark(word) {
            out.insert(word.clone());
            continue;
        }
        if is_value(word) || *held < LEAST_SUPPORT {
            continue;
        }
        if let Some((inside, capital)) = named.get(word)
            && *inside > usize::default()
            && capital * WHOLE_SHARE >= inside * NAMED_PERCENT
        {
            continue;
        }
        let places = stands.get(word).map(Vec::as_slice).unwrap_or_default();
        let few = places.iter().filter(|context| fillers.get(*context).map_or(0, HashSet::len) <= MOST_FRAME_FILLERS).count();
        let common = held * WHOLE_SHARE >= sentences * TOLD_FIXED_PERCENT;
        let unasked = asked * WHOLE_SHARE < held * ASKED_PERCENT;
        let in_value = valued.get(word).is_some_and(|count| count * WHOLE_SHARE >= asked_in.get(word).copied().unwrap_or_default() * FORM_FRAME_PERCENT);
        if few * WHOLE_SHARE >= places.len() * FRAME_PLACE_PERCENT && (common || unasked) && !in_value {
            out.insert(word.clone());
        }
    }
    out
}
because!(frame_words, LearnedWording, "the words of the frames of the told sentences of a kind: marks always, numbers never, a word of a value never (a word that, in most of the questions that hold it, stands beside another word in a pair that the question shares with a told sentence of its lesson, both words held by fewer than half the questions, as store room is one place of the question The pin is now in store room and of the told sentence Thomas moved the pin to kitchen from store room), a name never (a word that the told sentences or the questions write with a capital inside a sentence, however few names a kind has: the stories of bAbI start each sentence with its name, and only the questions show a name inside a sentence), and a word that stands between neighbours that few other words share, when it is common in the told sentences or when the questions of its lessons seldom hold it; a name or a thing stands between neighbours that many words share, and the questions ask about it, while a word of the frame (has, class, then, after, am, pm) is one of a few choices at its place");

fn sentence_form(words: &[String], frame: &HashSet<String>, values: &HashSet<Vec<String>>, most: usize) -> Option<(Pattern, Vec<Vec<String>>)> {
    let mut pattern: Pattern = Vec::new();
    let mut runs: Vec<Vec<String>> = Vec::new();
    let longest = values.iter().map(Vec::len).max().unwrap_or_default();
    let mut at = usize::default();
    while at < words.len() {
        let whole = (1..=longest.min(words.len() - at)).rev().find(|len| *len > 1 && values.contains(&words[at..at + *len]));
        if let Some(len) = whole {
            if runs.len() == most {
                return None;
            }
            pattern.push(LogicPiece::Open(runs.len()));
            runs.push(words[at..at + len].to_vec());
            at += len;
            continue;
        }
        let word = &words[at];
        at += 1;
        if !is_value(word) && frame.contains(word) {
            pattern.push(LogicPiece::Word(word_number(word)));
            continue;
        }
        if let Some(LogicPiece::Open(_)) = pattern.last()
            && let Some(run) = runs.last_mut()
            && run.last().is_some_and(|last| is_value(last) == is_value(word))
        {
            run.push(word.clone());
            continue;
        }
        if runs.len() == most {
            return None;
        }
        pattern.push(LogicPiece::Open(runs.len()));
        runs.push(vec![word.clone()]);
    }
    pattern.iter().any(|logic_piece| matches!(logic_piece, LogicPiece::Word(_))).then_some((pattern, runs))
}
because!(sentence_form, LearnedWording, "the form of a told sentence: a run that answers some lesson is one open place, even when it holds words of the frame (the tin kettle, a green jar), since a thing that a question asks about is one value; the other words of the frame stay, and each run of other words becomes an open place, numbers apart from words, since the engine computes with numbers; with the words of each run; nothing for a sentence with no word of the frame, since a form of open places alone fits every sentence of its length (five places for five words) and would read them all alike");

fn told_line(form: &Pattern, target: &Pattern, to: &[usize]) -> String {
    format!("{TOLD_IF}{}{TOLD_THEN}{}{STATEMENT_END}", library_writing(form), library_writing(&renamed(target, &places_of(target), to)))
}
because!(told_line, LearnedWording, "a learned rule as a line of the library: a sentence form read into one fact of a text, with the places of the fact given to places of the form");

fn told_promise(topic: &Topic, base: &Arc<Facts>, sample: &[&Case], lines: &[String], words: &QuestionWords) -> usize {
    let read = with_rules(topic, lines);
    let facts: Option<Vec<Facts>> = sample.iter().map(|case| bounded_facts(case, base, &read, false)).collect();
    facts.map_or(0, |facts| promise(&read, base, sample, &facts, words, true))
}
because!(told_promise, LearnedWording, "how much a reading of the told sentences promises: the promise of the text with the learned rules on a sample of lessons, where the fact that a question states counts as a condition, nothing when the rules run away");

fn told_values_opened(form: &Pattern, told: &HashSet<String>, frame: &HashSet<String>) -> Pattern {
    let mut out: Pattern = Vec::new();
    let mut places = 0usize;
    for logic_piece in form {
        let value = match logic_piece {
            LogicPiece::Word(word) => {
                let text = word_text(*word);
                word_not_mark(text) && told.contains(text) && !frame.contains(text)
            }
            _ => true,
        };
        if !value {
            out.push(logic_piece.clone());
            continue;
        }
        if let Some(LogicPiece::Open(_)) = out.last() {
            continue;
        }
        out.push(LogicPiece::Open(places));
        places += 1;
    }
    out
}
because!(told_values_opened, LearnedWording, "the form of a question as a fact that a told sentence may be read into: a word that the told sentences of the lessons hold as a value, not as a word of their frame, is an open place, as the name of a place or of a person is a value in the told sentence and so in the fact that it tells, though many questions name the same few places or people, and also in a lesson whose story does not name it");

struct ToldTarget {
    pattern: Pattern,
    numbers: Vec<bool>,
}
because!(ToldTarget, LearnedWording, "a fact of a text that a told sentence may be read into: its pattern, and for each of its open places in order, whether a rule of the text reckons with it or compares it, so that only a number may fill it");

fn told_targets(topic: &Topic) -> Vec<ToldTarget> {
    let mut out: Vec<ToldTarget> = Vec::new();
    let asked = topic.questions.iter().flat_map(|form| &form.replies).map(|reply| &reply.conditions);
    for conditions in topic.rules.iter().map(|rule| &rule.conditions).chain(asked) {
        let mut reckoned: HashSet<usize> = HashSet::new();
        for condition in conditions {
            let pieces: Vec<&LogicPiece> = match condition {
                Condition::Compare(_, one, other) => vec![one, other],
                Condition::Reckon(_, one, other, result) => vec![one, other, result],
                _ => Vec::new(),
            };
            reckoned.extend(pieces.into_iter().filter_map(|logic_piece| if let LogicPiece::Open(place) = logic_piece { Some(*place) } else { None }));
        }
        for condition in conditions {
            let pattern = match condition {
                Condition::Fact { pattern, .. } | Condition::Count { pattern, .. } => pattern,
                _ => continue,
            };
            let places = places_of(pattern);
            if fixed_count(pattern) == 0 || places.is_empty() {
                continue;
            }
            let numbers: Vec<bool> = places.iter().map(|place| reckoned.contains(place)).collect();
            match out.iter_mut().find(|known| known.pattern == *pattern) {
                Some(known) => known.numbers.iter_mut().zip(&numbers).for_each(|(known_number, number)| *known_number |= *number),
                None => out.push(ToldTarget { pattern: pattern.clone(), numbers }),
            }
        }
    }
    out
}
because!(told_targets, LearnedWording, "the facts that the rules of a text start from and that the answers of its written questions read, each once, with the places that a rule or an answer reckons with or compares");

pub fn learn_told(topic: &Topic, base: &Arc<Facts>, fit: &[&Case]) -> Vec<String> {
    kept_learning(LEARNED_TOLD, (topic, base, fit), &told_learning)
}
because!(learn_told, LearnedWording, "the rules that read the told sentences of a kind (told_learning), read back from the store between runs when an earlier run of the same code learned them for the same text, known facts and lessons");

fn told_learning(topic: &Topic, base: &Arc<Facts>, fit: &[&Case]) -> Vec<String> {
    let lines = learn_told_traced(topic, base, fit, (None, None));
    search_ended(lines.is_empty());
    lines
}
because!(told_learning, LearnedWording, "the rules that read the told sentences of a kind, learned from its lessons, with no report of the search, and the search counted in the costs of the learning with whether it found nothing");

pub(super) fn learn_told_traced(topic: &Topic, base: &Arc<Facts>, fit: &[&Case], (mut trace, joins): (Option<&mut Vec<SearchStep>>, Option<&mut Vec<FormJoin>>)) -> Vec<String> {
    let lessons: Vec<&Case> = fit.iter().take(READ_TOLD_LESSONS).copied().collect();
    let frame = frame_words(&lessons);
    let filled = answers_told(&lessons) * WHOLE_SHARE >= lessons.len() * SEED_TOLD_PERCENT;
    let values: HashSet<Vec<String>> = if filled { lessons.iter().map(|case| fact_words(&case.answer)).filter(|answer| answer.len() > 1 && answer.iter().all(|word| word_not_mark(word))).collect() } else { HashSet::new() };
    let names = told_names(&lessons);
    let valued_frame: HashSet<String> = lessons.iter().flat_map(|case| case.sentences.iter().flat_map(|sentence| fact_words(sentence))).filter(|word| !names.contains(word) && !is_value(word)).collect();
    let cut_frames: Vec<&HashSet<String>> = if filled { vec![&frame, &valued_frame] } else { vec![&frame] };
    let mut forms: Vec<(Pattern, Vec<(usize, Vec<Vec<String>>)>)> = Vec::new();
    for (at, case) in lessons.iter().enumerate() {
        for sentence in &case.sentences {
            let words = fact_words(sentence);
            let cuts: Vec<Option<(Pattern, Vec<Vec<String>>)>> = cut_frames.iter().map(|cut_frame| sentence_form(&words, cut_frame, &values, MOST_TOLD_PLACES)).collect();
            for (form, runs) in cuts.into_iter().flatten() {
                match forms.iter_mut().find(|(known, _)| *known == form) {
                    Some((_, uses)) => {
                        if uses.last().is_none_or(|(lesson, known)| *lesson != at || *known != runs) {
                            uses.push((at, runs));
                        }
                    }
                    None => forms.push((form, vec![(at, runs)])),
                }
            }
        }
    }
    let mut forms = if filled { merged_told_forms(forms, (&|opened: &Pattern, place: usize| text_events_at(topic, &lessons, (opened, place)), &|opened: &Pattern, place: usize| words_at(&lessons, (opened, place))), joins) } else { forms };
    forms.retain(|(_, uses)| uses.len() >= LEAST_SUPPORT);
    forms.sort_by(|one, other| other.1.len().cmp(&one.1.len()));
    let covered: HashSet<Pattern> = forms.iter().map(|(form, _)| form.clone()).collect();
    forms.extend(sentence_windows(&lessons, (&frame, &values), &covered));
    let mut edges: Vec<Vec<ItemEdge>> = vec![Vec::new(); forms.len()];
    let mut joined_values: Vec<Vec<(Pattern, usize)>> = vec![Vec::new(); forms.len()];
    for cut_frame in cut_frames.iter().filter(|_| filled) {
        for (item, uses, sides, variants) in list_items(&lessons, (cut_frame, &values)) {
            if !forms.iter().any(|(known, _)| *known == item) {
                forms.push((item, uses));
                edges.push(sides);
                joined_values.push(variants);
            }
        }
    }
    let fixed: Vec<bool> = forms.iter().map(|(form, _)| fixed_count(form) > usize::default()).collect();
    let mut kept = fixed.iter();
    forms.retain(|_| kept.next().copied().unwrap_or(true));
    let mut kept = fixed.iter();
    edges.retain(|_| kept.next().copied().unwrap_or(true));
    let mut kept = fixed.iter();
    joined_values.retain(|_| kept.next().copied().unwrap_or(true));
    let words = question_words(&lessons);
    let mut targets = told_targets(topic);
    let told_words: HashSet<String> = lessons.iter().flat_map(|case| case.sentences.iter().flat_map(|sentence| fact_words(sentence))).collect();
    for told_least in TOLD_FIXED_SHARES {
        let mut counted: Vec<(Pattern, usize)> = Vec::new();
        for case in &lessons {
            let Some(form) = form_of(case, &words, Cut::Share(*told_least)) else { continue };
            let form = told_values_opened(&form, &told_words, &frame);
            match counted.iter_mut().find(|(known, _)| *known == form) {
                Some((_, count)) => *count += 1,
                None => counted.push((form, 1)),
            }
        }
        for (pattern, count) in counted {
            let places = places_of(&pattern).len();
            if count >= LEAST_SUPPORT && places > 0 && fixed_count(&pattern) > 0 && !targets.iter().any(|known| known.pattern == pattern) {
                targets.push(ToldTarget { pattern, numbers: vec![false; places] });
            }
        }
    }
    let lanes = hardware_lanes();
    let judged_from = lessons.len() * JUDGE_LEARN_PERCENT / WHOLE_SHARE;
    let (learn_part, judge_part) = lessons.split_at(judged_from);
    let mut slots: Vec<Vec<String>> = vec![Vec::new(); forms.len()];
    let mut choices: Vec<Vec<ToldChoice>> = Vec::new();
    let companions: Vec<Vec<String>> = (0..forms.len()).map(|at| if edges[at].is_empty() { companion_conditions(&forms, at) } else { Vec::new() }).collect();
    let seed_sample: Vec<&Case> = spread_sample(&lessons, SEED_SAMPLE);
    let seeding = answers_told(&seed_sample) * WHOLE_SHARE >= seed_sample.len() * SEED_TOLD_PERCENT;
    let rule_facts = told_targets(topic).len();
    for (form_at, (form, uses)) in forms.iter().enumerate() {
        let places = own_places(form).len();
        let numbers: Vec<bool> = (0..places).map(|place| uses.iter().all(|(_, runs)| runs[place].iter().all(|word| is_value(word)))).collect();
        let mut sample: Vec<&Case> = Vec::new();
        for (at, _) in uses {
            if !sample.iter().any(|known| std::ptr::eq(*known, lessons[*at])) && sample.len() < TOLD_SAMPLE {
                sample.push(lessons[*at]);
            }
        }
        let asked_words: Vec<HashSet<String>> = lessons.iter().map(|case| fact_words(&case.question).into_iter().collect()).collect();
        let named = uses.iter().filter(|(at, runs)| runs.iter().any(|run| run.iter().all(|word| asked_words[*at].contains(word)))).count();
        let seldom_asked = named * WHOLE_SHARE < uses.len() * RELEVANT_PERCENT;
        let mut singles: Vec<(String, usize, bool, usize, String)> = Vec::new();
        let mut seen_lines: HashSet<String> = HashSet::new();
        for (kind, target) in targets.iter().enumerate() {
            let shared = target.pattern.iter().filter(|piece| matches!(piece, LogicPiece::Word(_)) && form.contains(piece)).count();
            let target_places = target.numbers.len();
            if target_places > places {
                continue;
            }
            let typed: Vec<Vec<usize>> = place_choices(target_places, places, places).into_iter().filter(|to| !to.contains(&places)).filter(|to| to.iter().zip(&target.numbers).all(|(place, number)| !number || numbers[*place])).collect();
            let unfiltered = (places <= SMALL_FORM_PLACES || seldom_asked) && typed.len() <= MOST_UNFILTERED;
            let mut relevant: Vec<Vec<usize>> = typed.into_iter().filter(|to| {
                let asked = uses.iter().filter(|(at, runs)| to.iter().any(|place| runs[*place].iter().all(|word| asked_words[*at].contains(word))));
                unfiltered || asked.count() * WHOLE_SHARE >= uses.len() * RELEVANT_PERCENT
            }).collect();
            relevant.sort_by_key(|to| !to.is_sorted());
            relevant.truncate(MOST_TARGET_WAYS);
            for to in relevant {
                let line = if edges[form_at].is_empty() {
                    told_line(form, &target.pattern, &to)
                } else {
                    edges[form_at]
                        .iter()
                        .map(|edge| told_line(&item_window(form, edge), &target.pattern, &to))
                        .chain(joined_values[form_at].iter().flat_map(|(variant, place)| edges[form_at].iter().map(|edge| stretched_line(&item_window(variant, edge), (variant, *place), &target.pattern, &to))))
                        .collect::<Vec<String>>()
                        .join(LEARNED_LINE_BREAK)
                };
                if seen_lines.insert(line.clone()) {
                    singles.push((line, shared, to.is_sorted(), kind, library_writing(&renamed(&target.pattern, &places_of(&target.pattern), &to))));
                }
            }
        }
        if seeding {
            choices.push(singles.iter().filter(|single| single.3 < rule_facts).map(|single| ToldChoice { line: single.0.clone(), narrow: None, fact: single.3, among_fact: usize::default(), promising: true, shared: single.1, in_order: single.2 }).collect());
            continue;
        }
        let earlier: Vec<String> = slots.concat();
        let promised: Vec<usize> = on_lanes(singles.len(), lanes, &|at| {
            let mut lines = earlier.clone();
            lines.push(singles[at].0.clone());
            told_promise(topic, base, &sample, &lines, &words)
        });
        let mut ranked: Vec<usize> = (0..singles.len()).collect();
        ranked.sort_by(|one, other| promised[*other].cmp(&promised[*one]).then_with(|| singles[*other].1.cmp(&singles[*one].1)).then_with(|| singles[*other].2.cmp(&singles[*one].2)).then_with(|| one.cmp(other)));
        let mut taken: Vec<usize> = vec![0; targets.len()];
        let mut kept: Vec<ToldChoice> = Vec::new();
        for (place, at) in ranked.into_iter().enumerate() {
            let fact = singles[at].3;
            if place < JOINT_CHOICES || taken[fact] < JOINT_CHOICES {
                kept.push(ToldChoice { line: singles[at].0.clone(), narrow: None, fact, among_fact: taken[fact], promising: place < JOINT_CHOICES, shared: singles[at].1, in_order: singles[at].2 });
            }
            taken[fact] += 1;
        }
        let first_ways: Vec<(String, usize)> = kept.iter().filter(|choice| choice.promising).take(JOINED_WAYS).map(|choice| (choice.line.clone(), choice.fact)).collect();
        for (line, fact) in first_ways {
            let Some(outcome) = singles.iter().find(|single| single.0 == line).map(|single| single.4.clone()) else { continue };
            for companion in &companions[form_at] {
                kept.push(ToldChoice { line: format!("{TOLD_IF}{}{CONDITIONS_JOINED}{companion}{TOLD_THEN}{outcome}{STATEMENT_END}", library_writing(form)), narrow: None, fact, among_fact: 0, promising: true, shared: usize::default(), in_order: true });
            }
        }
        choices.push(kept);
    }
    let joined = |slots: &[Vec<String>], at: usize, unit: &[String]| -> Vec<String> { slots.iter().enumerate().flat_map(|(place, chosen)| if place == at { unit.to_vec() } else { chosen.clone() }).collect() };
    let memo: Mutex<HashMap<Vec<String>, Worked<usize>>> = Mutex::new(HashMap::new());
    let joint = joint_ground((learn_part, judge_part));
    let judged = |lines: Vec<String>, before: Option<&JudgedReading>| -> usize {
        if let Some(right) = memo.lock().ok().and_then(|held| held.get(&lines).cloned()) {
            return right.again();
        }
        let right = in_work_scope(MOST_LEARNING_WORK, || joint_right(topic, base, &lines, (learn_part, judge_part, &joint), before).0);
        if let Ok(mut held) = memo.lock() {
            held.insert(lines, right.clone());
        }
        right.value
    };
    if seeding {
        let sampled_at: HashSet<usize> = spread_sample(&(0..lessons.len()).collect::<Vec<usize>>(), SEED_SAMPLE).into_iter().collect();
        let form_uses = |picked: &HashSet<usize>| -> Vec<usize> { forms.iter().map(|(_, uses)| if uses.iter().any(|(lesson, _)| picked.contains(lesson)) { uses.len() } else { usize::default() }).collect() };
        let first_slots = seeded_slots(topic, base, &seed_sample, (&choices, rule_facts, &form_uses(&sampled_at)), (lanes, Vec::new(), None, trace.as_mut().map(|steps| &mut **steps)));
        if first_slots.iter().all(Vec::is_empty) {
            return Vec::new();
        }
        if scope_spent() {
            return first_slots.concat();
        }
        let everyone = SeedGround::of(topic, &lessons);
        let Some(judged) = everyone.reading(topic, base, &first_slots.concat()) else { return first_slots.concat() };
        let wrong = everyone.wrong(&judged);
        let missed: Vec<usize> = (0..lessons.len()).filter(|at| wrong[*at] && !sampled_at.contains(at)).collect();
        if missed.is_empty() || scope_spent() {
            return first_slots.concat();
        }
        let picked: HashSet<usize> = sampled_at.iter().copied().chain(spread_sample(&missed, SEED_SAMPLE)).collect();
        let second: Vec<&Case> = (0..lessons.len()).filter(|at| picked.contains(at)).map(|at| lessons[at]).collect();
        return seeded_slots(topic, base, &second, (&choices, rule_facts, &form_uses(&picked)), (lanes, first_slots, None, trace)).concat();
    }
    let mut now = judged(slots.concat(), None);
    for pass in 0..JOINT_PASSES {
        let mut changed = false;
        for at in 0..slots.len() {
            if scope_spent() {
                break;
            }
            if pass > 0 && at >= MOST_REPASSED && slots[at].is_empty() {
                continue;
            }
            let mut chosen: Vec<String> = Vec::new();
            let (mut best_here, mut before) = joint_right(topic, base, &joined(&slots, at, &chosen), (learn_part, judge_part, &joint), None);
            for _ in 0..MOST_FORM_LINES {
                if scope_spent() {
                    break;
                }
                let facts_chosen: Vec<usize> = choices[at].iter().filter(|choice| chosen.contains(&choice.line)).map(|choice| choice.fact).collect();
                let tried: Vec<&String> = choices[at].iter().filter(|choice| (choice.promising || choice.among_fact == 0 || facts_chosen.contains(&choice.fact)) && !chosen.contains(&choice.line)).map(|choice| &choice.line).collect();
                let rights: Vec<usize> = on_lanes(tried.len(), lanes, &|unit| {
                    let mut lines = chosen.clone();
                    lines.push(tried[unit].clone());
                    judged(joined(&slots, at, &lines), before.as_ref())
                });
                let mut order: Vec<usize> = (0..tried.len()).collect();
                order.sort_by(|one, other| rights[*other].cmp(&rights[*one]).then_with(|| one.cmp(other)));
                let Some(&first) = order.first() else { break };
                let mut added: Vec<String> = vec![tried[first].clone()];
                let mut right = rights[first];
                if chosen.len() + 1 < MOST_FORM_LINES {
                    let paired: Vec<usize> = order.iter().take(PAIRED_CHOICES).copied().collect();
                    let pairs: Vec<(usize, usize)> = paired.iter().enumerate().flat_map(|(place, one)| paired[place + 1..].iter().map(move |other| (*one, *other))).collect();
                    let pair_rights: Vec<usize> = on_lanes(pairs.len(), lanes, &|pair| {
                        let mut lines = chosen.clone();
                        lines.push(tried[pairs[pair].0].clone());
                        lines.push(tried[pairs[pair].1].clone());
                        judged(joined(&slots, at, &lines), before.as_ref())
                    });
                    let best_pair = pair_rights.iter().enumerate().max_by(|one, other| one.1.cmp(other.1).then_with(|| other.0.cmp(&one.0)));
                    if let Some((pair, pair_right)) = best_pair
                        && *pair_right > right
                    {
                        right = *pair_right;
                        added = vec![tried[pairs[pair].0].clone(), tried[pairs[pair].1].clone()];
                    }
                }
                if right <= best_here {
                    break;
                }
                best_here = right;
                chosen.extend(added);
                before = joint_right(topic, base, &joined(&slots, at, &chosen), (learn_part, judge_part, &joint), before.as_ref()).1;
            }
            if best_here > now || (best_here == now && chosen.len() < slots[at].len()) {
                changed |= chosen != slots[at];
                now = best_here;
                slots[at] = chosen;
            }
        }
        if !changed {
            break;
        }
    }
    slots.concat()
}
because!(learn_told_traced, LearnedWording, "rules that read the told sentences of a kind, learned from its lessons, with the steps of its first reading kept for the report of the discovery when the caller asks: each told sentence is cut into a form by the words of its frame; when the stories tell most answers (the things that the questions ask about), a sentence is also cut with only its names, numbers and the runs that answer some lesson as values, forms that differ in one fixed word are joined, the items of its lists are taken one by one, and the reading is the first reading by filled answers, judged again on all the lessons and, when it leaves some of them wrong, taken on with those lessons added to its sample (a sample can miss a setting whose stories it does not hold), unless the first reading learned nothing, as then the text reads none of the sample and a second sample would cost as much for the same, with no judge of the whole wording after it, as the judge gained a few lessons of the hundreds that the first reading made right for many times its work; else, for each form with enough uses, the ways to read it into a fact whose places it fills (a fact that a rule of the text starts from, or a fact that a form of the questions states, as the questions show which facts matter) (a number only where a rule reckons) are ranked by the promise of the text on the lessons of the form; then form by form, with the readings of the other forms kept, the readings of a form are added one by one while the wording learned with them answers more of the lessons that it did not learn from, in passes until no form changes, so that a reading is chosen by what the whole wording gains and not by a few lessons of its own form; a form that holds no fixed word is left out, as a reading of open words alone fits every fact that the rules make, its own outcomes too, in thousands of ways, so its chains only grow until a bound refuses them");

struct SeedForm {
    lessons: Vec<usize>,
    probes: Vec<Arc<AskedForm>>,
    masks: Vec<Option<Vec<u64>>>,
}
because!(SeedForm, LearnedWording, "a form of the questions of the sample of a first reading: the lessons whose questions take it, the single filled answers that the facts of the rules of the text give it, and the masks of the sentences that each of them reads (reply_masks), made once with the form, as every judged reading of a step reads them again");

impl SeedForm {
    fn of(lessons: Vec<usize>, probes: Vec<Arc<AskedForm>>) -> SeedForm {
        let masks = probes.iter().map(|probe| reply_masks(probe)).collect();
        SeedForm { lessons, probes, masks }
    }
}

struct SeedReading {
    rules: Vec<ToldRule>,
    facts: Vec<Arc<Facts>>,
    rights: Vec<Vec<Vec<Vec<bool>>>>,
    answered: Vec<Vec<Vec<Vec<bool>>>>,
    bounds: Vec<Vec<Vec<Vec<Option<usize>>>>>,
}
because!(SeedReading, LearnedWording, "a first reading of the told sentences as judged: its rules with those of the text, the facts of each lesson of the sample under them, and for each reading of the words of the questions, each of its forms and each filled answer of a form, on which lessons of the form the answer is right, with the bound of the reads of each reply of an answer whose masks are known (reply_masks), as a later step keeps only such answers, when it did not reach the read bound of its chain; score is how many lessons the best filled answer of each form makes right, summed over the forms, in the better reading of the words, with no bar on the share that it answers right: a reading that pays off only with others still makes a few answers right by itself, where a learned answer would need nearly all");

impl SeedReading {
    fn score(&self) -> usize {
        self.rights.iter().map(|forms| forms.iter().map(|probes| probes.iter().map(|right| right.iter().filter(|right| **right).count()).max().unwrap_or_default()).sum::<usize>()).max().unwrap_or_default()
    }

    fn share_wrongs(&self, share: usize) -> usize {
        judged_tally(&self.rights[share..=share], &self.answered[share..=share]).1
    }
}

fn sample_told(sample: &[&Case]) -> Vec<(Vec<Vec<WordId>>, Vec<WordId>)> {
    sample.iter().map(|case| (case.sentences.iter().map(|sentence| fact_numbers(sentence)).collect(), fact_numbers(&format!("{QUESTION_TOLD}{}", case.question)))).collect()
}
because!(sample_told, LearnedWording, "the told sentences of each lesson of a sample read into words once, and its question as the sentence that tells it, which the trials of a first reading chain under many readings: told_of in words, the question told when there is no story or a rule reads it");

fn judged_tally(rights: &[Vec<Vec<Vec<bool>>>], answered: &[Vec<Vec<Vec<bool>>>]) -> (usize, usize, usize) {
    rights
        .iter()
        .zip(answered)
        .enumerate()
        .map(|(share, (forms, said))| {
            let counted = forms.iter().zip(said).fold((usize::default(), usize::default()), |(right_sum, wrong_sum), (probes, said)| {
                let best = probes.iter().zip(said).map(|(right, said)| (right.iter().filter(|right| **right).count(), right.iter().zip(said).filter(|(right, said)| **said && !**right).count())).enumerate().max_by(|one, other| (one.1).0.cmp(&(other.1).0).then_with(|| other.0.cmp(&one.0))).map(|(_, counts)| counts).unwrap_or_default();
                (right_sum + best.0, wrong_sum + best.1)
            });
            (counted.0, counted.1, share)
        })
        .max_by(|one, other| one.0.cmp(&other.0))
        .unwrap_or_default()
}
because!(judged_tally, LearnedWording, "how many lessons the best filled answer of each form makes right and how many it answers wrong, summed over the forms, in the reading of the words of the questions with the most right, and which reading that is: a reading whose answers are wrong where it answers at all reads its stories wrong, however many it makes right");

struct SeedGround<'a> {
    sample: &'a [&'a Case],
    told: Vec<(Vec<Vec<WordId>>, Vec<WordId>)>,
    questions: Vec<Vec<WordId>>,
    answers: Vec<Vec<String>>,
    shares: Vec<Vec<SeedForm>>,
}
const SAID_GAP: &str = " said: ";
because!(WANTED_GAP, LearnedWording, "the words between the answer that a refused rule gave and the answer of the lesson, in the report of the discovery");

const WANTED_GAP: &str = ", wanted: ";
because!(SAID_GAP, LearnedWording, "the words between a question and the answer that a refused rule gave to it, in the report of the discovery");

because!(SeedGround, LearnedWording, "what each step of a first reading is judged on, made once, as it does not change with the reading of the told sentences: the lessons of the sample with their questions and answers, and for each reading of the words of the questions its forms with their lessons and their single filled answers (of the facts that the rules of the text make, or of the facts that learned rules may make, also held by the latest fact and naming a fixed word of the form that the stories tell, as a question about the football asks where the football is). reading judges a reading on every lesson of the sample, nothing when the facts of some lesson run away; score_with judges a reading that adds a rule or a pair of rules to a judged one on every lesson of the sample, or on the lessons asked for, and a lesson on which the added rules hold no fact keeps the facts and the rights of the judged reading, as its chain is the same, so that a step costs the lessons that a new reading touches and not the whole sample; wrong tells the lessons of the sample that the best filled answer of their form does not answer under a judged reading; new_wrongs tells the lessons that a reading answers wrong in its share where the reading before did not, each with the answer and its proof, for the report of the discovery; and touched how many of them the rules that a reading adds read at all; a judgment adds the facts that its reply read to the work of the learning, as on the told side of some kinds the replies cost as much as the chains (logic_grid_puzzle at a hundred lessons); score_bound is the most lessons that a reading that adds rules could make right, every lesson that the added rules touch made right and every other lesson as it was; score_with also gives the chain of each lesson that the added rules touched (LessonChain), when asked, and takes the answers of a lesson from a chain kept for fewer of the rules when the rules added to them are idle on it (KeptChains)");

impl<'a> SeedGround<'a> {
    fn of(topic: &Topic, sample: &'a [&'a Case]) -> SeedGround<'a> {
        SeedGround::of_sources(sample, (&made_facts(topic, &[]), true, false))
    }

    fn of_answers(topic: &Topic, sample: &'a [&'a Case], way: &AnswerFact) -> SeedGround<'a> {
        let (wanted, other) = match way {
            AnswerFact::Reply => return SeedGround::of_replies(topic, sample),
            AnswerFact::Formed(at) => {
                let read: Vec<SeedForm> = topic.questions.get(*at).map(|form| SeedForm::of((0..sample.len()).filter(|lesson| asks(form, &fact_numbers(&sample[*lesson].question))).collect(), vec![Arc::new(form.clone())])).into_iter().collect();
                return SeedGround { sample, told: sample_told(sample), questions: sample.iter().map(|case| fact_numbers(&case.question)).collect(), answers: sample.iter().map(|case| answer_words(&case.answer)).collect(), shares: vec![read] };
            }
            AnswerFact::Label(label) => {
                let mut counted: Vec<(String, usize)> = Vec::new();
                for case in sample.iter().filter(|case| answer_words(&case.answer) != answer_words(label)) {
                    match counted.iter_mut().find(|(known, _)| *known == case.answer) {
                        Some((_, count)) => *count += 1,
                        None => counted.push((case.answer.clone(), 1)),
                    }
                }
                (Some(label.clone()), counted.into_iter().max_by(|one, other| one.1.cmp(&other.1)).map(|(answer, _)| answer))
            }
            AnswerFact::Spoken | AnswerFact::Aligned(..) => (None, None),
        };
        let words = question_words(sample);
        let mut forms: Vec<(Pattern, Vec<usize>)> = Vec::new();
        for (at, case) in sample.iter().enumerate() {
            let Some(form) = form_of(case, &words, Cut::Share(FIXED_SHARE_PERCENT)) else { continue };
            match forms.iter_mut().find(|(known, _)| *known == form) {
                Some((_, lessons)) => lessons.push(at),
                None => forms.push((form, vec![at])),
            }
        }
        let read: Vec<SeedForm> = forms
            .into_iter()
            .filter_map(|(form, lessons)| {
                let mut fact = form.clone();
                while fact.last().is_some_and(|piece| matches!(piece, LogicPiece::Word(word) if !word_not_mark(word_text(*word)))) {
                    fact.pop();
                }
                let form_text = library_writing(&form);
                let line = match (&wanted, &other) {
                    (Some(label), Some(other)) => format!("{LINE_START}{form_text}{LINE_MIDDLE}{label}{LEARNED_IF}{}{STATEMENT_END}{FORM_WORD_GAP}{other}{LEARNED_OTHERWISE}{STATEMENT_END}", library_writing(&fact)),
                    (Some(label), None) => format!("{LINE_START}{form_text}{LINE_MIDDLE}{label}{LEARNED_IF}{}{STATEMENT_END}", library_writing(&fact)),
                    _ if matches!(way, AnswerFact::Aligned(..)) => {
                        let AnswerFact::Aligned(at, order) = way else { return None };
                        let reply = topic.reply_lines.get(*at)?;
                        let [LogicPiece::Open(answer)] = reply.answer.as_slice() else { return None };
                        let places = places_of(&form);
                        if places.len() != order.len() {
                            return None;
                        }
                        let free = places.len();
                        let from: Vec<usize> = order.iter().copied().chain([*answer]).collect();
                        let to: Vec<usize> = places.iter().copied().chain([free]).collect();
                        format!("{LINE_START}{form_text}{LINE_MIDDLE}{}{LEARNED_IF}{LATEST_START}{}{STATEMENT_END}", place_letter(free), library_writing(&renamed(&reply.said, &from, &to)))
                    }
                    _ => {
                        let free = place_letter(places_of(&form).len());
                        format!("{LINE_START}{form_text}{LINE_MIDDLE}{free}{LEARNED_IF}{LATEST_START}{}{FORM_WORD_GAP}{free}{STATEMENT_END}", library_writing(&fact))
                    }
                };
                let probe = read_topic(PROBE, &line).ok()?.questions.into_iter().next()?;
                let mut probes = vec![Arc::new(probe)];
                if let AnswerFact::Aligned(at, order) = way
                    && let Some(kept) = kept_probe(topic, (*at, order), &form)
                {
                    probes.push(Arc::new(kept));
                }
                Some(SeedForm::of(lessons, probes))
            })
            .collect();
        SeedGround { sample, told: sample_told(sample), questions: sample.iter().map(|case| fact_numbers(&case.question)).collect(), answers: sample.iter().map(|case| answer_words(&case.answer)).collect(), shares: vec![read] }
    }

    fn of_replies(topic: &Topic, sample: &'a [&'a Case]) -> SeedGround<'a> {
        let mut forms: Vec<(usize, Vec<usize>)> = Vec::new();
        for (at, case) in sample.iter().enumerate() {
            let asked = || {
                let question = fact_numbers(&case.question);
                topic.reply_lines.iter().position(|line| {
                    let mut fits = Vec::new();
                    fit_into(&line.question, &question, &empty_filling(), &mut fits);
                    !fits.is_empty()
                })
            };
            let Some(line) = reply_target(topic, case).map(|(_, _, line, _)| line).or_else(asked) else { continue };
            match forms.iter_mut().find(|(known, _)| *known == line) {
                Some((_, lessons)) => lessons.push(at),
                None => forms.push((line, vec![at])),
            }
        }
        let read: Vec<SeedForm> = forms.into_iter().filter_map(|(line, lessons)| Some(SeedForm::of(lessons, vec![Arc::new(reply_probe(&topic.reply_lines[line])?)]))).collect();
        let mut shares = vec![read];
        shares.extend(SeedGround::of_sources(sample, (&made_facts(topic, &[]), true, true)).shares);
        SeedGround { sample, told: sample_told(sample), questions: sample.iter().map(|case| fact_numbers(&case.question)).collect(), answers: sample.iter().map(|case| answer_words(&case.answer)).collect(), shares }
    }

    fn of_sources(sample: &'a [&'a Case], (sources, timed, named): (&[Pattern], bool, bool)) -> SeedGround<'a> {
        let words = question_words(sample);
        let shares = TOLD_FIXED_SHARES
            .iter()
            .map(|told_least| {
                let mut forms: Vec<(Pattern, Vec<usize>)> = Vec::new();
                for (at, case) in sample.iter().enumerate() {
                    let Some(form) = form_of(case, &words, Cut::Share(*told_least)) else { continue };
                    match forms.iter_mut().find(|(known, _)| *known == form) {
                        Some((_, lessons)) => lessons.push(at),
                        None => forms.push((form, vec![at])),
                    }
                }
                forms
                    .into_iter()
                    .map(|(form, lessons)| {
                        let form_text = library_writing(&form);
                        let told: Vec<HashSet<String>> = lessons.iter().map(|at| sample[*at].sentences.iter().flat_map(|sentence| fact_words(sentence)).collect()).collect();
                        let constants = if named { told_constants(&form, &told.iter().collect::<Vec<&HashSet<String>>>()) } else { Vec::new() };
                        let probes = filled_candidates(&form, places_of(&form).len(), &form_text, (sources, &constants, &[]), (timed, &mut |_| true)).into_iter().map(|(candidate, _)| candidate.probe).collect();
                        SeedForm::of(lessons, probes)
                    })
                    .collect()
            })
            .collect();
        SeedGround { sample, told: sample_told(sample), questions: sample.iter().map(|case| fact_numbers(&case.question)).collect(), answers: sample.iter().map(|case| answer_words(&case.answer)).collect(), shares }
    }

    fn wrong_probes(&self, (after, before): (&SeedReading, &SeedReading)) -> (usize, Vec<(usize, usize)>) {
        let share = judged_tally(&after.rights, &after.answered).2;
        let wrong_in = |reading: &SeedReading| -> Vec<Option<usize>> {
            let mut out = vec![None; self.sample.len()];
            for ((form, rights), said) in self.shares[share].iter().zip(&reading.rights[share]).zip(&reading.answered[share]) {
                let Some(best) = (0..form.probes.len()).max_by_key(|probe| rights[*probe].iter().filter(|right| **right).count()) else { continue };
                for (place, at) in form.lessons.iter().enumerate() {
                    if said[best][place] && !rights[best][place] {
                        out[*at] = Some(best);
                    }
                }
            }
            out
        };
        let was = wrong_in(before);
        let now = wrong_in(after);
        (share, (0..self.sample.len()).filter_map(|at| match (now[at], was[at]) {
            (Some(probe_at), None) => Some((at, probe_at)),
            _ => None,
        }).collect())
    }

    fn new_wrongs(&self, base: &Arc<Facts>, after: &SeedReading, before: &SeedReading) -> Vec<WrongLesson> {
        let (share, wrong) = self.wrong_probes((after, before));
        let mut out: Vec<WrongLesson> = Vec::new();
        for (at, probe_at) in wrong {
            let Some(form) = self.shares[share].iter().find(|form| form.lessons.contains(&at)) else { continue };
            let probe = &form.probes[probe_at];
            let facts = Facts::given_again(&after.facts[at]);
            let Some(answered) = reply(std::slice::from_ref(probe.as_ref()), &self.questions[at], &facts) else { continue };
            let steps = answer_proof(&after.rules, &facts, base.added_to(&Facts::default()), &answered.reply.conditions, &answered.reply.absent, &answered.filling);
            out.push(WrongLesson { lesson: at, said: word_texts(&answered.words).join(FORM_WORD_GAP), proof: proof_lines(&steps) });
        }
        out
    }

    fn bounded(&self, at: usize, base: &Arc<Facts>, (topic, asks): (&Topic, bool)) -> Option<Facts> {
        let (sentences, question) = &self.told[at];
        if self.sample[at].sentences.is_empty() {
            return bounded_words(self.sample[at], std::slice::from_ref(question), base, topic);
        }
        if asks {
            return bounded_words(self.sample[at], &[sentences.as_slice(), std::slice::from_ref(question)].concat(), base, topic);
        }
        bounded_words(self.sample[at], sentences, base, topic)
    }

    fn judge(&self, probe: &AskedForm, at: usize, (facts, bounded): (&Arc<Facts>, bool)) -> (bool, bool, Option<usize>) {
        let layer = Facts::given_again(facts);
        let left = layer.reads_left();
        let answered = reply(std::slice::from_ref(probe), &self.questions[at], &layer);
        let reads = left - layer.reads_left();
        add_scope_work(reads, false);
        let bound = (bounded && !layer.exhausted()).then(|| reply_read_bound(probe, &self.questions[at], reads));
        match answered {
            Some(answered) => (true, answer_words(&word_texts(&answered.words).join(FORM_WORD_GAP)) == self.answers[at], bound),
            None => (false, false, bound),
        }
    }

    fn reading(&self, topic: &Topic, base: &Arc<Facts>, lines: &[String]) -> Option<SeedReading> {
        let read = with_rules(topic, lines);
        let asks = reads_question(&read.rules);
        let facts: Vec<Arc<Facts>> = (0..self.sample.len()).map(|at| self.bounded(at, base, (&read, asks)).map(Arc::new)).collect::<Option<Vec<Arc<Facts>>>>()?;
        let judged: Vec<Vec<Vec<Vec<(bool, bool, Option<usize>)>>>> = self.shares.iter().map(|forms| forms.iter().map(|form| form.probes.iter().zip(&form.masks).map(|(probe, masks)| form.lessons.iter().map(|at| self.judge(probe, *at, (&facts[*at], masks.is_some()))).collect()).collect()).collect()).collect();
        let rights = judged.iter().map(|forms| forms.iter().map(|probes| probes.iter().map(|lessons| lessons.iter().map(|(_, right, _)| *right).collect()).collect()).collect()).collect();
        let answered = judged.iter().map(|forms| forms.iter().map(|probes| probes.iter().map(|lessons| lessons.iter().map(|(said, _, _)| *said).collect()).collect()).collect()).collect();
        let bounds = judged.iter().map(|forms| forms.iter().map(|probes| probes.iter().map(|lessons| lessons.iter().map(|(_, _, bound)| *bound).collect()).collect()).collect()).collect();
        Some(SeedReading { rules: read.rules, facts, rights, answered, bounds })
    }

    fn lesson_marks(&self, reading: &SeedReading) -> Vec<(bool, bool)> {
        let mut out = vec![(false, false); self.sample.len()];
        let (_, _, share) = judged_tally(&reading.rights, &reading.answered);
        let count = |right: &Vec<bool>| right.iter().filter(|right| **right).count();
        for ((form, probes), said) in self.shares[share].iter().zip(&reading.rights[share]).zip(&reading.answered[share]) {
            let Some(best) = probes.iter().enumerate().max_by(|one, other| count(one.1).cmp(&count(other.1)).then_with(|| other.0.cmp(&one.0))).map(|(at, _)| at) else { continue };
            for (place, at) in form.lessons.iter().enumerate() {
                out[*at] = (probes[best][place], said[best][place] && !probes[best][place]);
            }
        }
        out
    }

    fn wrong(&self, before: &SeedReading) -> Vec<bool> {
        let mut out = vec![false; self.sample.len()];
        let best = self.shares.iter().zip(&before.rights).max_by_key(|(_, rights)| rights.iter().map(|probes| probes.iter().map(|right| right.iter().filter(|right| **right).count()).max().unwrap_or_default()).sum::<usize>());
        let Some((forms, rights)) = best else { return out };
        for (form, probes) in forms.iter().zip(rights) {
            let held = probes.iter().max_by_key(|right| right.iter().filter(|right| **right).count());
            for (place, at) in form.lessons.iter().enumerate() {
                out[*at] = held.is_none_or(|right| !right[place]);
            }
        }
        out
    }

    fn score_bound(&self, topic: &Topic, lines: &[String], before: &SeedReading) -> usize {
        let read = with_rules(topic, lines);
        let idle = idle_sentences(&before.rules, &read.rules);
        let open: Vec<bool> = (0..self.sample.len()).map(|at| !idle.as_ref().is_some_and(|sentences| before.facts[at].holds_none(sentences))).collect();
        self.shares
            .iter()
            .zip(&before.rights)
            .map(|(forms, rights)| {
                forms
                    .iter()
                    .zip(rights)
                    .filter(|(form, _)| !form.probes.is_empty())
                    .map(|(form, probes)| probes.iter().map(|right| form.lessons.iter().zip(right).filter(|(at, right)| **right && !open[**at]).count()).max().unwrap_or_default() + form.lessons.iter().filter(|at| open[**at]).count())
                    .sum::<usize>()
            })
            .max()
            .unwrap_or_default()
    }

    fn touched(&self, topic: &Topic, lines: &[String], before: &SeedReading, wrong: &[bool]) -> usize {
        let read = with_rules(topic, lines);
        let idle = idle_sentences(&before.rules, &read.rules);
        (0..self.sample.len()).filter(|at| wrong[*at] && !idle.as_ref().is_some_and(|sentences| before.facts[*at].holds_none(sentences))).count()
    }

    fn score_with(&self, topic: &Topic, base: &Arc<Facts>, lines: &[String], (before, judged): (&SeedReading, Option<&[bool]>), (kept, keep, base_held): (&[KeptChains], bool, Option<&BaseHeld>)) -> Option<((usize, usize, usize), Vec<Option<Arc<LessonChain>>>)> {
        let read = with_rules(topic, lines);
        let idle = idle_sentences(&before.rules, &read.rules);
        let counted = |at: usize| judged.is_none_or(|judged| judged[at]);
        let mut reusable: Vec<Option<&Arc<LessonChain>>> = vec![None; self.sample.len()];
        for kept in kept {
            let Some(patterns) = idle_sentences(&with_rules(topic, &kept.lines).rules, &read.rules) else { continue };
            if base_held.is_none_or(|held| held.holds_any(base, &patterns)) {
                continue;
            }
            for (at, chain) in kept.chains.iter().enumerate() {
                if reusable[at].is_none() && chain.as_ref().is_some_and(|chain| chain.holds_none(&patterns)) {
                    reusable[at] = chain.as_ref();
                }
            }
        }
        let asks = reads_question(&read.rules);
        let mut facts: Vec<Option<Arc<Facts>>> = Vec::with_capacity(self.sample.len());
        let mut reused: Vec<Option<&Arc<LessonChain>>> = Vec::with_capacity(self.sample.len());
        for at in 0..self.sample.len() {
            let kept = !counted(at) || idle.as_ref().is_some_and(|sentences| before.facts[at].holds_none(sentences));
            reused.push(reusable[at].filter(|_| !kept));
            facts.push(if kept || reused[at].is_some() { None } else { Some(Arc::new(self.bounded(at, base, (&read, asks))?)) });
        }
        let changed: Vec<Option<Vec<(u64, &[WordId])>>> = facts.iter().zip(&before.facts).map(|(found, known)| found.as_ref().and_then(|found| found.changed_facts(known))).collect();
        let left: Vec<usize> = facts.iter().map(|found| found.as_ref().map_or(usize::default(), |found| found.reads_left())).collect();
        let mut marks: Vec<Option<Vec<Vec<(bool, bool)>>>> = facts.iter().map(|found| (keep && found.is_some()).then(|| vec![Vec::new(); self.shares.len()])).collect();
        let mut rights: Vec<Vec<Vec<Vec<bool>>>> = Vec::with_capacity(self.shares.len());
        let mut answered: Vec<Vec<Vec<Vec<bool>>>> = Vec::with_capacity(self.shares.len());
        for (share, (((forms, was_right), was_said), was_bound)) in self.shares.iter().zip(&before.rights).zip(&before.answered).zip(&before.bounds).enumerate() {
            let mut share_rights = Vec::with_capacity(forms.len());
            let mut share_said = Vec::with_capacity(forms.len());
            for (((form, probes_right), probes_said), probes_bound) in forms.iter().zip(was_right).zip(was_said).zip(was_bound) {
                let mut form_rights = Vec::with_capacity(form.probes.len());
                let mut form_said = Vec::with_capacity(form.probes.len());
                for (probe_at, ((((probe, masks), right), said), bound)) in form.probes.iter().zip(&form.masks).zip(probes_right).zip(probes_said).zip(probes_bound).enumerate() {
                    let untouched = |place: usize, at: usize| {
                        masks.as_ref().zip(changed[at].as_ref()).zip(bound[place]).is_some_and(|((masks, changed), bound)| left[at] >= bound * (changed.len() + 1) && changed.iter().all(|fact| reads_none(masks.iter().copied().zip(reply_patterns(probe)), *fact)))
                    };
                    let mut judged: Vec<(bool, bool)> = Vec::with_capacity(form.lessons.len());
                    for (place, at) in form.lessons.iter().enumerate() {
                        let mark = if !counted(*at) {
                            (false, false)
                        } else if let Some(chain) = reused[*at] {
                            chain.marks[share][probe_at]
                        } else {
                            facts[*at].as_ref().filter(|_| !untouched(place, *at)).map_or((said[place], right[place]), |found| {
                                let (said, right, _) = self.judge(probe, *at, (found, false));
                                (said, right)
                            })
                        };
                        if let Some(lesson) = marks[*at].as_mut() {
                            lesson[share].push(mark);
                        }
                        judged.push(mark);
                    }
                    form_rights.push(judged.iter().map(|(_, right)| *right).collect());
                    form_said.push(judged.iter().map(|(said, _)| *said).collect());
                }
                share_rights.push(form_rights);
                share_said.push(form_said);
            }
            rights.push(share_rights);
            answered.push(share_said);
        }
        let chains: Vec<Option<Arc<LessonChain>>> = if keep {
            facts.iter().zip(marks).zip(&reused).map(|((found, marks), chain)| match (found, marks) {
                (Some(found), Some(marks)) => Some(Arc::new(LessonChain::of(found, base, marks))),
                _ => chain.cloned(),
            }).collect()
        } else {
            Vec::new()
        };
        Some((judged_tally(&rights, &answered), chains))
    }
}

type Scored = ((usize, usize, usize), Vec<Option<Arc<LessonChain>>>);
because!(Scored, LearnedWording, "what the judge of a reading on the sample gives: its right answers, wrong answers and reading of the words of the questions, with the chains of the lessons that its added rules touched, kept for the step after");

struct LessonChain {
    words: Vec<WordId>,
    ends: Vec<usize>,
    marks: Vec<Vec<(bool, bool)>>,
}
because!(LessonChain, LearnedWording, "what the chain of one lesson gave when a step of a first reading judged a rule on it: the words of each fact that the chain added to the facts known before any story, and for each reading of the words of the questions whether each filled answer of the form of the lesson answered and answered right; a later step that adds rules which no fact of this chain holds has the same chain for that rule, and takes these answers without making the chain again");

impl LessonChain {
    fn of(facts: &Facts, base: &Facts, marks: Vec<Vec<(bool, bool)>>) -> LessonChain {
        let first = base.added_to(&Facts::default());
        let mut words: Vec<WordId> = Vec::new();
        let mut ends: Vec<usize> = Vec::new();
        for place in first..first + facts.added_to(base) {
            words.extend_from_slice(facts.fact(place));
            ends.push(words.len());
        }
        LessonChain { words, ends, marks }
    }

    fn holds_none(&self, sentences: &[Pattern]) -> bool {
        let mut start = usize::default();
        self.ends.iter().all(|end| {
            let fact = &self.words[start..*end];
            start = *end;
            sentences.iter().all(|sentence| !holds_words(sentence, fact))
        })
    }
}

struct KeptChains<'a> {
    lines: Vec<String>,
    chains: &'a [Option<Arc<LessonChain>>],
}
because!(KeptChains, LearnedWording, "the chains that a step of a first reading kept for one rule on each lesson, with the lines that they were made under (the lines learned before the step and the rule): a reading whose lines hold these lines and add rules to them has the same chain on a lesson whose kept chain holds none of the added rules, nor any fact known before a story, as the added rules are idle on it (idle_sentences): the step after takes the answers of a rule from its chains of the step before, and a pair of rules from the chains of either rule alone");

struct BaseHeld(Mutex<HashMap<Vec<Pattern>, bool>>);
because!(BaseHeld, LearnedWording, "for the sentences of the rules that a reading adds, whether some fact known before any story holds one of them, worked out once for each set of sentences in a search, as these facts are the same in every lesson and a kept chain holds only the facts that the chain added");

impl BaseHeld {
    fn holds_any(&self, base: &Facts, patterns: &[Pattern]) -> bool {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner).entry(patterns.to_vec()).or_insert_with(|| !base.holds_none(patterns))
    }
}

const MOST_UNLESS_TRIED: usize = 4;
because!(MOST_UNLESS_TRIED, LearnedWording, "the rules refused at a step for their wrong answers, the most right first, that are judged again with a condition that must not hold: a rule that is wrong only where something else is true (a gorb is no wrade) makes many lessons right, and the few with the most right are the ones worth a second judgment");

const MOST_UNLESS: usize = 4;
because!(MOST_UNLESS, LearnedWording, "the conditions that must not hold tried for one refused rule, those that hold in the most of its wrong lessons first");

fn unless_lines(topic: &Topic, line: &str, wrong: &[&Case]) -> Vec<String> {
    let Some(rule) = read_topic(TOLD_LINE_TOPIC, line).ok().and_then(|read| read.rules.into_iter().next()) else { return Vec::new() };
    let Some(then) = line.find(TOLD_THEN) else { return Vec::new() };
    let read: Vec<&Pattern> = rule.conditions.iter().filter_map(|condition| match condition {
        Condition::Fact { pattern, .. } => Some(pattern),
        _ => None,
    }).collect();
    let mut counted: Vec<(Pattern, usize)> = Vec::new();
    for case in wrong {
        let question = fact_numbers(&case.question);
        let Some(asked) = topic.reply_lines.iter().find_map(|reply| {
            let mut fits = Vec::new();
            fit_into(&reply.question, &question, &empty_filling(), &mut fits);
            fits.into_iter().find_map(|filling| fill(&reply.said, &filling))
        }) else { continue };
        let mut fillings = Vec::new();
        fit_into(&rule.outcome, &asked, &empty_filling(), &mut fillings);
        let Some(filling) = fillings.into_iter().next() else { continue };
        let values: Vec<(usize, Vec<WordId>)> = places_of(&rule.outcome).into_iter().filter_map(|place| filling.get(place).filter(|value| !value.is_empty()).map(|value| (place, value.to_vec()))).collect();
        let mut seen: Vec<Pattern> = Vec::new();
        for sentence in &case.sentences {
            let words = fact_numbers(sentence);
            let mut pattern: Pattern = Vec::new();
            let mut at = usize::default();
            while at < words.len() {
                match values.iter().filter(|(_, value)| words[at..].starts_with(value)).max_by_key(|(_, value)| value.len()) {
                    Some((place, value)) => {
                        pattern.push(LogicPiece::Open(*place));
                        at += value.len();
                    }
                    None => {
                        pattern.push(LogicPiece::Word(words[at]));
                        at += 1;
                    }
                }
            }
            let plain = |sentence: &Pattern| -> Pattern { sentence.iter().map(plain_piece).collect() };
            let own = read.iter().any(|known| sentence_map(&plain(known), &plain(&pattern)).is_some());
            if !places_of(&pattern).is_empty() && !own && !seen.contains(&pattern) {
                seen.push(pattern);
            }
        }
        for pattern in seen {
            match counted.iter_mut().find(|(known, _)| *known == pattern) {
                Some((_, count)) => *count += 1,
                None => counted.push((pattern, 1)),
            }
        }
    }
    counted.retain(|(_, count)| count * WHOLE_SHARE >= wrong.len() * FORM_FRAME_PERCENT);
    counted.sort_by(|one, other| other.1.cmp(&one.1).then_with(|| library_writing(&one.0).cmp(&library_writing(&other.0))));
    counted.into_iter().take(MOST_UNLESS).map(|(pattern, _)| format!("{}{CONDITIONS_JOINED}{NOT_START}{}{}", &line[..then], library_writing(&pattern), &line[then..])).collect()
}
because!(unless_lines, LearnedWording, "a refused rule again with a condition that must not hold: a fact that the told sentences of its wrong lessons hold about the things of its outcome (as the question of each wrong lesson fills the outcome through the reply line: Niz is a gorb, for is Niz the wrade of Gop: no), in at least half of them, the most held first, not one of the rule's own conditions; the judge then keeps it only when it adds no wrong answer, as the fact must not hold in the lessons that the rule makes right (If A is the flen of B and not A is a gorb then A is the wrade of B)");

fn told_reading(line: &str, told: &[Vec<WordId>]) -> (bool, usize) {
    let Some(rule) = read_topic(TOLD_LINE_TOPIC, line).ok().and_then(|topic| topic.rules.into_iter().next()) else { return (false, usize::default()) };
    let read = rule.conditions.iter().all(|condition| match condition {
        Condition::Fact { pattern, .. } => told.iter().any(|sentence| {
            let mut found = Vec::new();
            fit_into(pattern, sentence, &empty_filling(), &mut found);
            !found.is_empty()
        }),
        _ => true,
    });
    (read, rule.conditions.len())
}
because!(told_reading, LearnedWording, "whether every fact that a suggested rule reads is told by some sentence of the sample, and how many conditions it has: of two rules that answer as many lessons, the one that reads the stories comes before one whose condition only the question's own statement makes (C didn't come to the B as a condition of C came to the B), and then the one of fewer conditions");

const TOLD_LINE_TOPIC: &str = "suggested";
because!(TOLD_LINE_TOPIC, LearnedWording, "the name under which one suggested rule is read as a line of the library to see what facts it reads");

fn seeded_slots(topic: &Topic, base: &Arc<Facts>, sample: &[&Case], (choices, rule_facts, uses): (&[Vec<ToldChoice>], usize, &[usize]), (lanes, start, judged_by, mut trace): (usize, Vec<Vec<String>>, Option<&AnswerFact>, Option<&mut Vec<SearchStep>>)) -> Vec<Vec<String>> {
    let mut slots: Vec<Vec<String>> = if start.len() == choices.len() { start } else { vec![Vec::new(); choices.len()] };
    let ground = match judged_by {
        Some(way) => SeedGround::of_answers(topic, sample, way),
        None => SeedGround::of(topic, sample),
    };
    if scope_spent() {
        return slots;
    }
    let Some(mut before) = ground.reading(topic, base, &slots.concat()) else { return slots };
    let mut now = before.score();
    let precise = judged_by.is_some();
    let told_sentences: Vec<Vec<WordId>> = sample.iter().flat_map(|case| case.sentences.iter().map(|sentence| fact_numbers(sentence))).collect();
    let mut first_step = true;
    let mut kept: Option<(Vec<String>, HashMap<String, Vec<Option<Arc<LessonChain>>>>)> = None;
    let base_held = BaseHeld(Mutex::default());
    let mut last_scores: HashMap<String, usize> = HashMap::new();
    while now < sample.len() {
        if scope_spent() {
            break;
        }
        let found: Vec<(usize, String, Option<String>, (usize, bool))> = choices
            .iter()
            .enumerate()
            .flat_map(|(at, kept)| kept.iter().filter(|choice| choice.fact < rule_facts).map(move |choice| (at, choice.line.clone(), choice.narrow.clone(), (choice.shared, choice.in_order))))
            .filter(|(at, _, _, _)| uses[*at] > usize::default())
            .filter(|(at, line, _, _)| slots[*at].is_empty() && !slots[*at].contains(line))
            .collect();
        let earlier = slots.concat();
        let with_line = |line: &str| -> Vec<String> {
            let mut lines = earlier.clone();
            lines.push(line.to_string());
            lines
        };
        let wrong = ground.wrong(&before);
        let allowed = |judged: (usize, usize, usize)| -> Option<usize> { (!precise || judged.1 <= before.share_wrongs(judged.2)).then_some(judged.0) };
        let chains_of = |line: &str| -> Vec<KeptChains> {
            kept.as_ref().and_then(|(old, chains)| chains.get(line).map(|chains| KeptChains { lines: old.iter().cloned().chain(std::iter::once(line.to_string())).collect(), chains })).into_iter().collect()
        };
        let bounded = !first_step && found.len() > SEED_BOUND_FIRST;
        let bounds: Vec<usize> = if bounded { on_lanes(found.len(), lanes, &|place| ground.score_bound(topic, &with_line(&found[place].1), &before)) } else { Vec::new() };
        let mut order: Vec<usize> = (0..found.len()).collect();
        if bounded {
            let last = |place: usize| last_scores.get(&found[place].1).copied().unwrap_or_default();
            order.sort_by(|one, other| last(*other).cmp(&last(*one)).then_with(|| bounds[*other].cmp(&bounds[*one])).then_with(|| one.cmp(other)));
        }
        let mut unjudged: Vec<(usize, usize, usize)> = Vec::new();
        let mut single: Vec<Option<Scored>> = (0..found.len()).map(|_| None).collect();
        let mut narrowed: Vec<Option<Scored>> = (0..found.len()).map(|_| None).collect();
        let judge_places = |places: &[usize], single: &mut Vec<Option<Scored>>, narrowed: &mut Vec<Option<Scored>>| {
            let wide = on_lanes(places.len(), lanes, &|at| ground.score_with(topic, base, &with_line(&found[places[at]].1), (&before, None), (&chains_of(&found[places[at]].1), true, Some(&base_held))));
            let refused: Vec<usize> = places.iter().zip(&wide).filter(|(place, scored)| found[**place].2.is_some() && scored.as_ref().is_some_and(|(tally, _)| allowed(*tally).is_none())).map(|(place, _)| *place).collect();
            let narrow = on_lanes(refused.len(), lanes, &|at| {
                let line = found[refused[at]].2.as_deref()?;
                ground.score_with(topic, base, &with_line(line), (&before, None), (&chains_of(line), true, Some(&base_held)))
            });
            for (place, scored) in places.iter().zip(wide) {
                single[*place] = scored;
            }
            for (place, scored) in refused.iter().zip(narrow) {
                narrowed[*place] = scored;
            }
        };
        let first = if bounded { SEED_BOUND_FIRST } else { order.len() };
        judge_places(&order[..first], &mut single, &mut narrowed);
        if scope_spent() {
            break;
        }
        let effective = |place: usize, single: &[Option<Scored>], narrowed: &[Option<Scored>]| -> Option<usize> {
            let narrowed_at = found[place].2.is_some() && single[place].as_ref().is_some_and(|(tally, _)| allowed(*tally).is_none());
            if narrowed_at { narrowed[place].as_ref() } else { single[place].as_ref() }.and_then(|(tally, _)| allowed(*tally))
        };
        if bounded {
            let mut best: Vec<usize> = order[..first].iter().filter_map(|place| effective(*place, &single, &narrowed)).collect();
            best.sort_unstable_by(|one, other| other.cmp(one));
            let least = best.first().copied().filter(|top| *top > now).unwrap_or_default();
            let rest: Vec<usize> = order[first..].iter().filter(|place| bounds[**place] >= least).copied().collect();
            unjudged = order[first..].iter().filter(|place| bounds[**place] < least).map(|place| (*place, bounds[*place], least)).collect();
            judge_places(&rest, &mut single, &mut narrowed);
            if scope_spent() {
                break;
            }
        }
        last_scores = (0..found.len()).filter_map(|place| effective(place, &single, &narrowed).map(|score| (found[place].1.clone(), score))).collect();
        let mut judged: Vec<Option<(usize, usize, usize)>> = single.iter().map(|scored| scored.as_ref().map(|(tally, _)| *tally)).collect();
        let refused: Vec<usize> = (0..found.len()).filter(|place| found[*place].2.is_some() && judged[*place].is_some_and(|judged| allowed(judged).is_none())).collect();
        let narrow_judged: Vec<Option<(usize, usize, usize)>> = refused.iter().map(|place| narrowed[*place].as_ref().map(|(tally, _)| *tally)).collect();
        let mut next_chains: HashMap<String, Vec<Option<Arc<LessonChain>>>> = HashMap::new();
        for (place, scored) in single.into_iter().enumerate() {
            if let Some((_, chains)) = scored {
                next_chains.insert(found[place].1.clone(), chains);
            }
        }
        for (place, scored) in narrowed.into_iter().enumerate() {
            if let (Some((_, chains)), Some(line)) = (scored, &found[place].2) {
                next_chains.insert(line.clone(), chains);
            }
        }
        let mut tried: Vec<(usize, String)> = found.iter().map(|(at, line, _, _)| (*at, line.clone())).collect();
        for (at, place) in refused.iter().enumerate() {
            if let Some(narrow) = &found[*place].2 {
                tried[*place].1 = narrow.clone();
                judged[*place] = narrow_judged[at];
            }
        }
        if precise {
            let mut still: Vec<usize> = (0..tried.len()).filter(|place| judged[*place].is_some_and(|counts| allowed(counts).is_none())).collect();
            still.sort_by(|one, other| judged[*other].map(|counts| counts.0).cmp(&judged[*one].map(|counts| counts.0)).then_with(|| one.cmp(other)));
            for place in still.into_iter().take(MOST_UNLESS_TRIED) {
                let Some(after) = ground.reading(topic, base, &with_line(&tried[place].1)) else { continue };
                let wrong: Vec<&Case> = ground.wrong_probes((&after, &before)).1.into_iter().map(|(at, _)| sample[at]).collect();
                for unless in unless_lines(topic, &tried[place].1, &wrong) {
                    let Some((counts, _)) = ground.score_with(topic, base, &with_line(&unless), (&before, None), (&[], false, Some(&base_held))) else { continue };
                    if allowed(counts).is_some() && counts.0 > now {
                        tried[place].1 = unless;
                        judged[place] = Some(counts);
                        break;
                    }
                }
            }
        }
        let scores: Vec<Option<(usize, usize)>> = on_lanes(tried.len(), lanes, &|place| Some((allowed(judged[place]?)?, ground.touched(topic, &with_line(&tried[place].1), &before, &wrong))));
        let refused_best = (0..tried.len()).filter(|place| judged[*place].is_some_and(|counts| allowed(counts).is_none())).max_by_key(|place| judged[*place].map_or(usize::default(), |counts| counts.0));
        let refused_wrong = trace.is_some().then(|| refused_best.and_then(|place| {
            let after = ground.reading(topic, base, &with_line(&tried[place].1))?;
            Some((tried[place].1.clone(), ground.new_wrongs(base, &after, &before).into_iter().map(|wrong| WrongLesson { lesson: wrong.lesson, said: format!("{}{SAID_GAP}{}{WANTED_GAP}{}", sample[wrong.lesson].question, wrong.said, sample[wrong.lesson].answer), proof: wrong.proof }).collect::<Vec<WrongLesson>>()))
        })).flatten();
        let mut step = trace.is_some().then(|| SearchStep {
            refused: refused_wrong.clone(),
            before: now,
            judged: (0..tried.len()).filter_map(|place| judged[place].map(|(right, wrongs, share)| Judged { line: tried[place].1.clone(), right, wrong: wrongs, wrong_before: before.share_wrongs(share), allowed: allowed((right, wrongs, share)).is_some(), narrow: refused.contains(&place), touched: scores[place].map_or(usize::default(), |(_, touched)| touched), uses: uses[tried[place].0], bound: None, marks: Vec::new() })).chain(unjudged.iter().map(|(place, most, least)| Judged { line: tried[*place].1.clone(), right: usize::default(), wrong: usize::default(), wrong_before: usize::default(), allowed: false, narrow: false, touched: usize::default(), uses: uses[tried[*place].0], bound: Some((*most, *least)), marks: Vec::new() })).collect(),
            pair: None,
            added: Vec::new(),
            reached: now,
            stop: None,
            sample: sample.len(),
        });
        if let Some(step) = step.as_mut() {
            let mut marked: Vec<usize> = (0..step.judged.len()).filter(|at| step.judged[*at].bound.is_none()).collect();
            marked.sort_by(|one, other| step.judged[*other].right.cmp(&step.judged[*one].right).then_with(|| one.cmp(other)));
            for at in marked.into_iter().take(MOST_MARKED) {
                let lines = with_line(&step.judged[at].line);
                if let Some(reading) = with_work_scope(None, || ground.reading(topic, base, &lines)) {
                    step.judged[at].marks = sample.iter().zip(ground.lesson_marks(&reading)).map(|(case, (right, wrong))| (case_place(case), right, wrong)).collect();
                }
            }
        }
        let mut ranked: Vec<(usize, usize)> = scores.iter().enumerate().filter_map(|(place, score)| score.map(|(score, _)| (score, place))).collect();
        let read_by_told: Vec<(bool, usize)> = tried.iter().map(|(_, line)| told_reading(line, &told_sentences)).collect();
        let all_told = |place: usize| read_by_told[place].0;
        let conditions = |place: usize| read_by_told[place].1;
        let touched = |place: usize| scores[place].map_or(usize::default(), |(_, touched)| touched);
        let length = |place: usize| if precise { tried[place].1.split_whitespace().count() } else { usize::default() };
        let shared = |place: usize| found[place].3;
        ranked.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| all_told(other.1).cmp(&all_told(one.1))).then_with(|| conditions(one.1).cmp(&conditions(other.1))).then_with(|| touched(other.1).cmp(&touched(one.1))).then_with(|| shared(other.1).cmp(&shared(one.1))).then_with(|| length(one.1).cmp(&length(other.1))).then_with(|| uses[tried[other.1].0].cmp(&uses[tried[one.1].0])).then_with(|| one.1.cmp(&other.1)));
        let Some(&(score, place)) = ranked.first() else {
            noted(&mut trace, step, Some(STOP_NO_RULE));
            break;
        };
        let mut partners: Vec<usize> = (0..tried.len()).collect();
        partners.sort_by(|one, other| uses[tried[*other].0].cmp(&uses[tried[*one].0]).then_with(|| one.cmp(other)));
        let common: HashSet<usize> = partners.iter().map(|unit| tried[*unit].0).collect::<Vec<usize>>().into_iter().fold(Vec::new(), |mut kept: Vec<usize>, form| { if !kept.contains(&form) && kept.len() < SEED_PARTNER_FORMS { kept.push(form); } kept }).into_iter().collect();
        let paired = first_step || score <= now;
        first_step = false;
        let pairs: Vec<(usize, usize)> = ranked.iter().take(SEED_PAIRED).filter(|_| paired && score < sample.len()).flat_map(|(_, one)| (0..tried.len()).filter(|other| tried[*other].0 != tried[*one].0 && common.contains(&tried[*other].0)).take(SEED_PAIR_PARTNERS).map(move |other| (*one, other))).collect();
        let pair_lines = |pair: usize| -> Vec<String> {
            let mut lines = earlier.clone();
            lines.push(tried[pairs[pair].0].1.clone());
            lines.push(tried[pairs[pair].1].1.clone());
            lines
        };
        let few_wrong = wrong.iter().filter(|wrong| **wrong).count() * SEED_SCREEN_STRIDE <= sample.len();
        let screen: Vec<bool> = (0..sample.len()).map(|at| (few_wrong && wrong[at]) || at % SEED_SCREEN_STRIDE == usize::default()).collect();
        let alone = |pair: usize| -> Vec<KeptChains> { [pairs[pair].0, pairs[pair].1].into_iter().filter_map(|unit| next_chains.get(&tried[unit].1).map(|chains| KeptChains { lines: with_line(&tried[unit].1), chains })).collect() };
        let screened: Vec<Option<usize>> = on_lanes(pairs.len(), lanes, &|pair| ground.score_with(topic, base, &pair_lines(pair), (&before, Some(&screen)), (&alone(pair), false, Some(&base_held))).map(|((right, _, _), _)| right));
        if scope_spent() {
            break;
        }
        let mut finalists: Vec<usize> = (0..pairs.len()).filter(|pair| screened[*pair].is_some()).collect();
        finalists.sort_by(|one, other| screened[*other].cmp(&screened[*one]).then_with(|| one.cmp(other)));
        finalists.truncate(SEED_PAIR_FINALISTS);
        let pair_scores: Vec<Option<usize>> = on_lanes(finalists.len(), lanes, &|at| allowed(ground.score_with(topic, base, &pair_lines(finalists[at]), (&before, None), (&alone(finalists[at]), false, Some(&base_held)))?.0));
        let best_pair = pair_scores.iter().zip(&finalists).filter_map(|(score, pair)| score.map(|score| (score, *pair))).max_by(|one, other| one.0.cmp(&other.0).then_with(|| other.1.cmp(&one.1)));
        let mut added: Vec<usize> = vec![place];
        let mut reached = score;
        if let (Some(step), Some((pair_score, pair))) = (step.as_mut(), best_pair) {
            step.pair = Some((tried[pairs[pair].0].1.clone(), tried[pairs[pair].1].1.clone(), pair_score));
        }
        if let Some((pair_score, pair)) = best_pair
            && pair_score > score
        {
            added = vec![pairs[pair].0, pairs[pair].1];
            reached = pair_score;
        }
        if reached <= now {
            noted(&mut trace, step, Some(STOP_NO_GAIN));
            break;
        }
        now = reached;
        if let Some(step) = step.as_mut() {
            step.added = added.iter().map(|unit| tried[*unit].1.clone()).collect();
            step.reached = reached;
        }
        for unit in added {
            let (at, line) = tried[unit].clone();
            slots[at].push(line);
        }
        let Some(next) = ground.reading(topic, base, &slots.concat()) else {
            noted(&mut trace, step, Some(STOP_RAN_AWAY));
            break;
        };
        noted(&mut trace, step, (now >= sample.len()).then_some(STOP_ALL_RIGHT));
        before = next;
        kept = Some((earlier, next_chains));
    }
    slots
}

fn noted(trace: &mut Option<&mut Vec<SearchStep>>, step: Option<SearchStep>, stop: Option<&'static str>) {
    if let (Some(steps), Some(mut step)) = (trace.as_mut(), step) {
        step.stop = stop;
        progress(&format!("step {} of a search: {} of {} right before it, {} rules judged{}", steps.len() + 1, step.before, step.sample, step.judged.iter().filter(|judged| judged.bound.is_none()).count(), stop.map_or(String::new(), |why| format!(", stopped: {why}"))));
        steps.push(step);
    }
}
because!(noted, LearnedWording, "a step of a search for rules kept for the report of the discovery, with why the search stopped at it, if it did, when the caller asked for the steps, and a progress line of the discovery for it"); 
because!(seeded_slots, LearnedWording, "a first reading of the told sentences before the judge of the wording: of the ways to read each sentence form that the sample tells into a fact that a rule of the text starts from, one way for each form at most, the one that makes the most lessons of a sample right under the best single filled answers, with no bar on their share, is added, one at a time, while that number grows and some lesson of the sample is still wrong; from the readings given at the start, if any; a reading of the start of a story (who holds what) makes some answers right by itself, and the readings of the later steps (who swaps with whom) make more right only after it, so that readings that pay off only together are found together: pairs of ways are judged at the first step, and later only when no way alone makes more lessons right, as each step after the start of a story shows its gain alone; of ways that make as many lessons right, the one that reads more of the lessons still wrong comes first, then the one whose sentence form holds more words of the frame of its fact and gives its places in the order of the sentence, and of learned rules the shorter, as a rule with fewer fixed words holds in more stories (got the thing, not only got the thing there), also among the ways paired, as the start of a story in a setting not yet read (who dances with whom) gains little alone and much with the steps of that setting; a learned rule that adds a wrong answer is judged again with its places of one word, and that form takes its place in the step, alone and in pairs (a rule that is wrong alone may be right beside its partner, and its narrow form makes only facts that it makes), so that the narrow form costs a judge only where the wide one fails; each step, with every rule that it judged, is kept for the report of the discovery when the caller asks; a step keeps the chain of each lesson that each single rule touched, and the step after it takes the answers of a lesson from that chain when no fact of it holds the rules that the step added, as the chain is then the same, so that a step after the first makes again only the chains that the new rules reach (the readings of the same setting of a story), with the same judgment; after the first step, a few dozen single ways are judged first, those that made the most lessons right at the step before, and when the best of them makes more lessons right than the step began with, only the other ways whose bound (score_bound) reaches it are judged, as no other way could reach the way that the step keeps and no pair follows such a step; else every way is judged, as the pairs that then follow start from the eight best ways; a traced search cuts the same ways, so that its work and any stop at the budget are those of the board, and its report names each way not judged with its bound");

pub(super) fn reply_target(topic: &Topic, case: &Case) -> Option<(Vec<WordId>, usize, usize, Vec<bool>)> {
    let question = fact_numbers(&case.question);
    let answer = fact_numbers(&case.answer);
    for (at, line) in topic.reply_lines.iter().enumerate() {
        let mut asked = Vec::new();
        fit_into(&line.question, &question, &empty_filling(), &mut asked);
        for filling in &asked {
            let mut said = Vec::new();
            fit_into(&line.answer, &answer, filling, &mut said);
            if let Some(whole) = said.iter().find(|whole| fill(&line.said, whole).is_some())
                && let Some(words) = fill(&line.said, whole)
            {
                let open: Vec<bool> = line.said.iter().flat_map(|logic_piece| match logic_piece {
                    LogicPiece::Word(_) => vec![false],
                    LogicPiece::Open(place) => vec![true; whole.get(*place).map_or(usize::default(), <[WordId]>::len)],
                    LogicPiece::OpenWithEnd(..) => vec![true],
                }).collect();
                return Some((words, places_of(&line.said).len(), at, open));
            }
        }
    }
    None
}
because!(reply_target, LearnedWording, "the fact that answers the question of a lesson, as the reply line of its text says the answer (the football is in the garden), with the number of places of that line, its place in the text, and which of its words the places of the line gave: the fact that a concept's rules would make, named by the text that asks it even when those rules are gone");

#[derive(Clone, PartialEq)]
enum AnswerFact {
    Reply,
    Label(String),
    Spoken,
    Aligned(usize, Vec<usize>),
    Formed(usize),
}
because!(AnswerFact, LearnedWording, "where the fact that answers a lesson comes from: the reply line of the text, which says the answer in a sentence; or, when the text has none, the question itself as a statement for the lessons of one label (Arthur lives in Australia, for the lessons answered entailment); or the words of the question followed by the answer, a name made from the lessons when nothing else names the fact; or a reply line of the text whose question has as many places as the question of the lesson, of another form, with the lesson's values in its places in the order given (('Ashley', 'Nicholas') with son as Nicholas is the son of Ashley)");

fn answer_fact(topic: &Topic, case: &Case, (way, words): (&AnswerFact, &QuestionWords)) -> Option<(Vec<WordId>, usize, usize, Vec<bool>)> {
    match way {
        AnswerFact::Reply => reply_target(topic, case),
        AnswerFact::Label(label) => {
            let (said, open) = question_opened(case, words)?;
            (answer_words(&case.answer) == answer_words(label)).then(|| (said, 1, usize::default(), open))
        }
        AnswerFact::Spoken => {
            let (mut said, mut open) = question_opened(case, words)?;
            let answer = fact_numbers(&case.answer);
            open.extend(vec![true; answer.len()]);
            said.extend(answer);
            Some((said, 1, usize::default(), open))
        }
        AnswerFact::Aligned(line, order) => aligned_target(topic, case, words, (*line, order, true)),
        AnswerFact::Formed(at) => formed_target(topic, case, *at),
    }
}

fn formed_ways(topic: &Topic, sample: &[&Case]) -> Vec<AnswerFact> {
    let made: Vec<Vec<WordId>> = topic.rules.iter().filter(|rule| adds(rule) || matches!(rule.kind, LineKind::Replaces | LineKind::ReplacesInside)).map(|rule| frame_of(&rule.outcome)).collect();
    let told: Vec<Vec<WordId>> = sample.iter().flat_map(|case| case.sentences.iter().map(|sentence| fact_numbers(sentence))).chain(topic.facts.iter().cloned()).collect();
    topic
        .questions
        .iter()
        .enumerate()
        .filter(|(_, form)| fixed_count(&form.question) > usize::default() && sample.iter().filter(|case| asks(form, &fact_numbers(&case.question))).count() * WHOLE_SHARE >= sample.len() * RULE_GROUND_PERCENT)
        .filter(|(_, form)| {
            form.replies.iter().filter(|reply| reply.words.iter().all(|piece| matches!(piece, LogicPiece::Word(_)))).any(|reply| {
                reply.conditions.iter().any(|condition| {
                    let Condition::Fact { pattern, .. } = condition else { return false };
                    let frame = frame_of(pattern);
                    !frame.is_empty() && !made.contains(&frame) && !told.iter().any(|words| {
                        let mut fits = Vec::new();
                        fit_into(pattern, words, &empty_filling(), &mut fits);
                        !fits.is_empty()
                    })
                })
            })
        })
        .map(|(at, _)| AnswerFact::Formed(at))
        .collect()
}
because!(formed_ways, LearnedWording, "the question forms of a text that name the facts its rules must make for the lessons of a kind: a form whose question holds a fixed word (a form of open places alone asks every question) and asks most questions of the sample, with an answer of fixed words (a label) whose condition reads a sentence that no rule or line of the text that writes a sentence again makes, and that no told sentence of the sample and no fact that the text knows is (the place of the A is B, with location's rules out, where the stories say that someone moved the thing), so that the rules are learned toward that sentence and judged by the form itself");

fn frame_of(pattern: &Pattern) -> Vec<WordId> {
    pattern.iter().filter_map(|piece| if let LogicPiece::Word(word) = piece { Some(*word) } else { None }).collect()
}
because!(frame_of, LearnedWording, "the fixed words of a sentence in their order: two sentences with the same fixed words in the same order name the same kind of fact, whatever fills their places");

fn formed_target(topic: &Topic, case: &Case, at: usize) -> Option<(Vec<WordId>, usize, usize, Vec<bool>)> {
    let form = topic.questions.get(at)?;
    let mut fits = Vec::new();
    fit_into(&form.question, &fact_numbers(&case.question), &empty_filling(), &mut fits);
    let filling = fits.into_iter().next()?;
    let answer = fact_numbers(&case.answer);
    form.replies.iter().filter(|reply| fill(&reply.words, &filling).is_some_and(|words| words == answer)).find_map(|reply| {
        reply.conditions.iter().find_map(|condition| {
            let Condition::Fact { pattern, .. } = condition else { return None };
            let said = fill(pattern, &filling)?;
            let open: Vec<bool> = pattern.iter().flat_map(|piece| match piece {
                LogicPiece::Word(_) => vec![false],
                LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => vec![true; filling.get(*place).map_or(usize::default(), <[WordId]>::len)],
            }).collect();
            (open.len() == said.len()).then(|| (said, places_of(pattern).len(), usize::default(), open))
        })
    })
}
because!(formed_target, LearnedWording, "the fact that answers a lesson as the text's own question form reads it: the first sentence of a condition of the answer that the lesson gives, filled from its question (the place of the potato is green envelope, for entailment); nothing for an answer that holds otherwise, which names no fact");

fn aligned_target(topic: &Topic, case: &Case, words: &QuestionWords, (line, order, answer_open): (usize, &[usize], bool)) -> Option<(Vec<WordId>, usize, usize, Vec<bool>)> {
    let reply = topic.reply_lines.get(line)?;
    let [LogicPiece::Open(answer)] = reply.answer.as_slice() else { return None };
    let mut form = form_of(case, words, Cut::Share(FIXED_SHARE_PERCENT))?;
    while form.last().is_some_and(|piece| matches!(piece, LogicPiece::Word(word) if !word_not_mark(word_text(*word)))) {
        form.pop();
    }
    let places = places_of(&form);
    if places.len() != order.len() {
        return None;
    }
    let mut fitted = Vec::new();
    fit_into(&form, &stated(&case.question), &empty_filling(), &mut fitted);
    let whole = fitted.into_iter().next()?;
    let mut filling = empty_filling();
    for (place, to) in places.iter().zip(order) {
        filling.set(*to, whole.get(*place)?.to_vec());
    }
    filling.set(*answer, fact_numbers(&case.answer));
    let said = fill(&reply.said, &filling)?;
    let open: Vec<bool> = reply.said.iter().flat_map(|piece| match piece {
        LogicPiece::Word(_) => vec![false],
        LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => vec![answer_open || place != answer; filling.get(*place).map_or(usize::default(), <[WordId]>::len)],
    }).collect();
    (open.len() == said.len()).then(|| (said, order.len(), line, open))
}
because!(aligned_target, LearnedWording, "the fact that answers a lesson as a reply line of the text says it when the question of the lesson has another form with as many places: the values of the question's form go to the places of the reply line's question in the order given, the answer to its answer place, and the sentence of the reply line is filled with them, with its places open (the answer's place too, or not, as the caller asks: a rule may give the answer a place, while the answer is no value that the question asks about), and the places of the question as the fewest that a rule's outcome must hold, as an answer may stay a word of the outcome (son); nothing when the form of the question has another number of places or the reply line's answer is more than one place");

fn abduced_rounds(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case], (line, order): (usize, &[usize])) -> Vec<String> {
    let Some(form) = kept_form(topic, line) else { return Vec::new() };
    let words = question_words(lessons);
    let valued: Vec<(&Case, Vec<Vec<WordId>>, Vec<String>)> = lessons.iter().filter_map(|case| {
        let (values, written) = written_values(case, &words)?;
        (values.len() == order.len()).then_some((*case, values, written))
    }).collect();
    let answers = told_answers(base, form, lessons);
    let steps = one_step_lessons(&valued.iter().map(|(case, _, written)| (*case, written.clone())).collect::<Vec<(&Case, Vec<String>)>>(), &answers);
    let names = capital_words(lessons);
    let step_values: Vec<(&Case, Vec<Vec<WordId>>)> = steps.iter().filter_map(|case| question_values(case, &words).map(|values| (case, values))).collect();
    let first_round = stated_lessons(base, (form, order, &names), &step_values);
    let first = held_by_sentence(topic, base, &first_round, &names);
    let (taught, taught_base) = with_learned(topic, base, &first);
    let second_round: Vec<(Pattern, Case)> = valued.iter().flat_map(|(case, values, _)| {
        let statements = stated_lessons(base, (form, order, &names), &[(*case, values.clone())]);
        let made = bounded_facts(case, &taught_base, &taught, false);
        let new: Vec<(Pattern, Case)> = statements.into_iter().filter(|(_, lesson)| lesson.answer != STATED_HOLDS || made.as_ref().is_none_or(|facts| !facts.holds(&stated(&lesson.question)))).collect();
        if new.iter().any(|(_, lesson)| lesson.answer == STATED_HOLDS) { new } else { Vec::new() }
    }).collect();
    let second = held_by_sentence(&taught, &taught_base, &second_round, &names);
    [first, second].concat()
}
because!(abduced_rounds, LearnedWording, "rules learned in two rounds from the facts that the answers of the lessons need under the text's own question form (abduced_facts), with the question's values in the places of its reply line in the order of the way: first from the lessons of one step that the told sentences tell (Ashley daughter Lillian, for what Lillian is to Ashley), as statements of the facts held and ruled out, which the rules that read a told sentence into the text's facts make; then, on top of those rules, from the statements of the lessons themselves, which rules that join the facts of several steps make (a parent of a sibling is a parent); in each round the statements of each sentence of the form are learned apart (held_by_sentence); the rules of both rounds, which the text's own form then reads");

fn kept_probe(topic: &Topic, (line, order): (usize, &[usize]), form: &Pattern) -> Option<AskedForm> {
    let reply = topic.reply_lines.get(line)?;
    let kept = topic.questions.iter().find(|known| sentence_map(&known.question, &reply.question).is_some())?;
    let asked = places_of(form);
    if asked.len() != order.len() {
        return None;
    }
    let mut from: Vec<usize> = order.to_vec();
    let mut to: Vec<usize> = asked.clone();
    let mut next = asked.iter().max().map_or(usize::default(), |most| most + 1);
    let others = kept.replies.iter().flat_map(|answer| std::iter::once(&answer.words).chain(answer.conditions.iter().chain(&answer.absent).filter_map(|condition| if let Condition::Fact { pattern, .. } = condition { Some(pattern) } else { None })));
    for pattern in others {
        for place in places_of(pattern) {
            if !from.contains(&place) {
                from.push(place);
                to.push(next);
                next += 1;
            }
        }
    }
    if next > PLAIN_PLACES {
        return None;
    }
    let fact = |condition: &Condition| match condition {
        Condition::Fact { pattern, when } => Some(Condition::Fact { pattern: renamed(pattern, &from, &to), when: *when }),
        _ => None,
    };
    let replies = kept.replies.iter().map(|answer| Some(Reply { words: renamed(&answer.words, &from, &to), conditions: answer.conditions.iter().map(fact).collect::<Option<Vec<Condition>>>()?, absent: answer.absent.iter().map(fact).collect::<Option<Vec<Condition>>>()?, text: answer.text.clone(), every: answer.every })).collect::<Option<Vec<Reply>>>()?;
    Some(AskedForm { question: form.clone(), replies })
}
because!(kept_probe, LearnedWording, "the text's own question form whose reply line a way reads, asked with the question of the lesson's form: its answers with the places of its question given to the places of the lesson's form in the order of the way, its other places after them, so that a lesson is judged right when the text answers it from the facts that its rules make (a parent and a sex give a son), as the reply line's sentence itself is made by no rule");

fn kept_form(topic: &Topic, line: usize) -> Option<&AskedForm> {
    let reply = topic.reply_lines.get(line)?;
    topic.questions.iter().find(|form| sentence_map(&form.question, &reply.question).is_some())
}
because!(kept_form, LearnedWording, "the text's own question form whose question is the question of a reply line, whatever the letters of its places: the form that says which facts give each answer of that question");

fn question_values(case: &Case, words: &QuestionWords) -> Option<Vec<Vec<WordId>>> {
    let mut shape = form_of(case, words, Cut::Share(FIXED_SHARE_PERCENT))?;
    while shape.last().is_some_and(|piece| matches!(piece, LogicPiece::Word(word) if !word_not_mark(word_text(*word)))) {
        shape.pop();
    }
    let mut fitted = Vec::new();
    fit_into(&shape, &stated(&case.question), &empty_filling(), &mut fitted);
    let whole = fitted.into_iter().next()?;
    places_of(&shape).iter().map(|place| whole.get(*place).map(<[WordId]>::to_vec)).collect()
}
because!(question_values, LearnedWording, "the values of the question of a lesson, place by place, in the form that the questions of its kind share (the marks at the end of the form left out, as a value may stand before them)");

fn written_values(case: &Case, words: &QuestionWords) -> Option<(Vec<Vec<WordId>>, Vec<String>)> {
    let values = question_values(case, words)?;
    let written: Vec<String> = values.iter().map(|value| text_words(&case.question).into_iter().find(|word| word_ids(std::slice::from_ref(word)) == *value)).collect::<Option<Vec<String>>>()?;
    Some((values, written))
}
because!(written_values, LearnedWording, "the values of the question of a lesson with each written as the question writes it (a name with its capital letter), for values of one word, so that the lessons drawn from its told sentences name the people as the story does");

fn told_answers(base: &Facts, form: &AskedForm, lessons: &[&Case]) -> HashSet<String> {
    let mut out: HashSet<String> = lessons.iter().map(|case| case.answer.to_lowercase()).filter(|answer| answer.split_whitespace().count() == 1).collect();
    out.extend(form_answers(base, form));
    out
}
because!(told_answers, LearnedWording, "the words that a told sentence of one step may give as its answer, in small letters: the answers of one word that the lessons give, and the words that the text's own form can answer from what it knows (form_answers)");

fn held_by_sentence(topic: &Topic, base: &Arc<Facts>, statements: &[(Pattern, Case)], names: &HashSet<String>) -> Vec<String> {
    let mut groups: Vec<(Pattern, Vec<&Case>)> = Vec::new();
    for (pattern, lesson) in statements {
        let shape = places_in_order(pattern);
        match groups.iter_mut().find(|(known, _)| *known == shape) {
            Some((_, lessons)) => lessons.push(lesson),
            None => groups.push((shape, vec![lesson])),
        }
    }
    let mut out: Vec<String> = Vec::new();
    for (_, lessons) in groups.iter().filter(|(_, lessons)| lessons.iter().any(|lesson| lesson.answer == STATED_HOLDS)) {
        for line in held_rules(topic, base, lessons, names) {
            if !out.contains(&line) {
                out.push(line);
            }
        }
    }
    out
}
because!(held_by_sentence, LearnedWording, "the rules that make the stated facts, learned for each sentence of the form in a search of its own on the statements of that sentence: a rule makes facts of one sentence, so the statements of the others do not judge it, and a sentence that few lessons state (a spouse, a grandchild beside many parents) gets a sample of its own, where its word is a word of the form and no value; one shared search judged every rule on every statement and found neither a spouse nor an in-law");

fn places_in_order(pattern: &Pattern) -> Pattern {
    let mut seen: Vec<usize> = Vec::new();
    let mut place_of = |place: usize| seen.iter().position(|known| *known == place).unwrap_or_else(|| {
        seen.push(place);
        seen.len() - 1
    });
    pattern.iter().map(|piece| match piece {
        LogicPiece::Open(place) => LogicPiece::Open(place_of(*place)),
        LogicPiece::OpenWithEnd(place, ending) => LogicPiece::OpenWithEnd(place_of(*place), ending.clone()),
        other => other.clone(),
    }).collect()
}
because!(places_in_order, LearnedWording, "a sentence with its places named in the order in which they stand, so that one sentence with other letters (A is a parent of B, B is a parent of A) is one sentence");

fn stated_lessons(base: &Arc<Facts>, (form, order, names): (&AskedForm, &[usize], &HashSet<String>), cases: &[(&Case, Vec<Vec<WordId>>)]) -> Vec<(Pattern, Case)> {
    cases.iter().flat_map(|(case, values)| {
        let (held, ruled_out) = abduced_facts(base, form, (values, &fact_numbers(&case.answer)), order);
        statement_lessons(&case.sentences, (&held, &ruled_out), names)
    }).collect()
}
because!(stated_lessons, LearnedWording, "the lessons that state the facts that the answers of some lessons need under a question form, each with the sentence of the form that it states, with the values of each question in the places of the form in the given order: each fact held (yes) and each fact that the answer rules out (no), on the story of its lesson");

fn told_contradictions(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case], (line, order): (usize, &[usize])) -> usize {
    let Some(form) = kept_form(topic, line) else { return usize::default() };
    let words = question_words(lessons);
    let answers = told_answers(base, form, lessons);
    let names = capital_words(lessons);
    lessons
        .iter()
        .filter_map(|case| written_values(case, &words).filter(|(values, _)| values.len() == order.len()).map(|valued| (*case, valued)))
        .map(|(case, (values, written))| {
            let steps = one_step_lessons(&[(case, written)], &answers);
            let step_values: Vec<(&Case, Vec<Vec<WordId>>)> = steps.iter().filter_map(|step| question_values(step, &words).map(|values| (step, values))).collect();
            let mut said: HashMap<String, (bool, bool)> = HashMap::new();
            for (_, lesson) in stated_lessons(base, (form, order, &names), &step_values).into_iter().chain(stated_lessons(base, (form, order, &names), &[(case, values)])) {
                let entry = said.entry(lesson.question.to_lowercase()).or_default();
                if lesson.answer == STATED_HOLDS {
                    entry.0 = true;
                } else {
                    entry.1 = true;
                }
            }
            said.values().filter(|(held, ruled_out)| *held && *ruled_out).count()
        })
        .sum()
}
because!(told_contradictions, LearnedWording, "how many facts an order of the places of a reply line both holds and rules out within one story, over the lessons: the facts that the answer of each told sentence of one step needs and the facts that the answer of the lesson needs, under the text's own question form; in the right order they agree, while in a wrong one the sex of the asked person goes to the other person and a person of two told sentences gets both (Ashley as a son in one and as a daughter in the other)");

fn consistent_ways(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case], ways: Vec<AnswerFact>) -> Vec<AnswerFact> {
    let counted: Vec<(AnswerFact, usize)> = ways
        .into_iter()
        .map(|way| {
            let wrong = match &way {
                AnswerFact::Aligned(line, order) => told_contradictions(topic, base, lessons, (*line, order)),
                _ => usize::default(),
            };
            (way, wrong)
        })
        .collect();
    let line_of = |way: &AnswerFact| if let AnswerFact::Aligned(line, _) = way { Some(*line) } else { None };
    counted
        .iter()
        .filter(|(way, wrong)| counted.iter().filter(|(other, _)| line_of(other) == line_of(way)).all(|(_, again)| wrong <= again))
        .map(|(way, _)| way.clone())
        .collect()
}
because!(consistent_ways, LearnedWording, "the orders of the places of each reply line that are worth the rounds of rules: those whose facts contradict each other in the fewest stories (told_contradictions), all of them when they tie, as the rounds of a wrong order cost more work than the right one and its facts make the answers wrong");

fn held_rules(topic: &Topic, base: &Arc<Facts>, fit: &[&Case], names: &HashSet<String>) -> Vec<String> {
    let timed = if base.has_history() { Arc::clone(base) } else { Arc::new(knowledge_base(&topic.facts, &topic.rules, true)) };
    let lessons: Vec<&Case> = fit.iter().take(READ_TOLD_LESSONS).copied().collect();
    let sample: Vec<&Case> = spread_sample(&lessons, STATED_SAMPLE);
    let way = AnswerFact::Label(STATED_HOLDS.to_string());
    let (lines, _, _) = learned_rules_by(topic, &timed, (&lessons, &sample, RULE_LESSONS * STATED_PER_LESSON), &way, false);
    let named: HashSet<WordId> = names.iter().map(|name| word_number(&name.to_lowercase())).collect();
    let lines: Vec<String> = lines.into_iter().filter(|line| read_topic(PROBE, line).ok().and_then(|read| read.rules.into_iter().next()).is_none_or(|rule| !rule.outcome.iter().any(|piece| matches!(piece, LogicPiece::Word(word) if named.contains(word))))).collect();
    let ground = SeedGround::of_answers(topic, &sample, &way);
    let lines = untimed(topic, &timed, &ground, lines);
    linked_made_one(topic, &timed, &ground, lines)
}

fn linked_merge(one: &ToldRule, other: &ToldRule) -> Option<String> {
    if one.conditions.len() != other.conditions.len() || !one.absent.is_empty() || !other.absent.is_empty() || one.outcome.len() != other.outcome.len() {
        return None;
    }
    let pieces = |rule: &ToldRule| -> Option<Vec<Pattern>> { rule.conditions.iter().map(|condition| if let Condition::Fact { pattern, when: Moment::Any } = condition { Some(pattern.clone()) } else { None }).chain(std::iter::once(Some(rule.outcome.clone()))).collect() };
    let (mine, theirs) = (pieces(one)?, pieces(other)?);
    let place = mine.iter().flat_map(|pattern| places_of(pattern)).max().map_or(usize::default(), |most| most + 1);
    let mut pair: Option<(WordId, WordId)> = None;
    let mut merged: Vec<Pattern> = Vec::new();
    let mut in_outcome = false;
    let mut in_condition = false;
    for (at, (left, right)) in mine.iter().zip(&theirs).enumerate() {
        if left.len() != right.len() {
            return None;
        }
        let mut sentence: Pattern = Vec::new();
        for (piece, again) in left.iter().zip(right) {
            match (piece, again) {
                (LogicPiece::Word(word), LogicPiece::Word(known)) if word != known => {
                    if pair.is_some_and(|pair| pair != (*word, *known)) {
                        return None;
                    }
                    pair = Some((*word, *known));
                    if at + 1 == mine.len() { in_outcome = true } else { in_condition = true }
                    sentence.push(LogicPiece::Open(place));
                }
                _ if piece == again => sentence.push(piece.clone()),
                _ => return None,
            }
        }
        merged.push(sentence);
    }
    if !(in_outcome && in_condition) || place >= PLAIN_PLACES {
        return None;
    }
    let outcome = merged.pop()?;
    let conditions: Vec<Condition> = merged.into_iter().map(|pattern| Condition::Fact { pattern, when: Moment::Any }).collect();
    Some(rule_text_line(&conditions, &outcome))
}
because!(linked_merge, LearnedWording, "two rules that differ only in one word, the same in a condition and in the outcome (C means a female D then B is female; C means a male D then B is male), as one rule with a place there in both (C means a E D then B is E), which says what both say; nothing for rules that differ otherwise");

fn linked_made_one(topic: &Topic, base: &Arc<Facts>, ground: &SeedGround, lines: Vec<String>) -> Vec<String> {
    let tally = |lines: &[String]| ground.reading(topic, base, lines).map(|reading| judged_tally(&reading.rights, &reading.answered));
    let Some(mut now) = tally(&lines) else { return lines };
    let mut out = lines;
    loop {
        let rules: Vec<Option<ToldRule>> = out.iter().map(|line| read_topic(PROBE, line).ok().and_then(|read| read.rules.into_iter().next())).collect();
        let found = (0..out.len()).flat_map(|one| ((one + 1)..out.len()).map(move |other| (one, other))).find_map(|(one, other)| {
            let merged = linked_merge(rules[one].as_ref()?, rules[other].as_ref()?)?;
            let mut tried: Vec<String> = out.iter().enumerate().filter(|(at, _)| *at != one && *at != other).map(|(_, line)| line.clone()).collect();
            tried.insert(one, merged);
            let judged = tally(&tried)?;
            (judged.0 >= now.0 && judged.1 <= now.1).then_some((tried, judged))
        });
        let Some((tried, judged)) = found else { return out };
        out = tried;
        now = judged;
    }
}
because!(linked_made_one, LearnedWording, "the rules learned over stated facts with each two that differ in one word of a condition and the outcome made one (linked_merge), when the one rule answers the sample as well with no more wrong answers, again while two do");

fn untimed(topic: &Topic, base: &Arc<Facts>, ground: &SeedGround, lines: Vec<String>) -> Vec<String> {
    let tally = |lines: &[String]| ground.reading(topic, base, lines).map(|reading| judged_tally(&reading.rights, &reading.answered));
    let Some(mut now) = tally(&lines) else { return lines };
    let mut out = lines.clone();
    for line in &lines {
        let Some(rule) = read_topic(PROBE, line).ok().and_then(|read| read.rules.into_iter().next()) else { continue };
        let plain: Vec<Condition> = rule.conditions.iter().enumerate().filter(|(at, condition)| match condition {
            Condition::Fact { pattern, when: Moment::AtThatTime | Moment::BeforeThatTime } => !rule.conditions[..*at].iter().any(|before| matches!(before, Condition::Fact { pattern: earlier, .. } if earlier == pattern)),
            _ => true,
        }).map(|(_, condition)| match condition {
            Condition::Fact { pattern, .. } => Condition::Fact { pattern: pattern.clone(), when: Moment::Any },
            other => other.clone(),
        }).collect();
        if plain.len() == rule.conditions.len() && rule.conditions.iter().all(|condition| !matches!(condition, Condition::Fact { when: Moment::AtThatTime | Moment::BeforeThatTime, .. })) {
            continue;
        }
        let Some(at) = out.iter().position(|known| known == line) else { continue };
        let simple = rule_text_line(&plain, &rule.outcome);
        let mut tried = out.clone();
        tried[at] = simple;
        if let Some(judged) = tally(&tried)
            && judged.0 >= now.0
            && judged.1 <= now.1
        {
            out = tried;
            now = judged;
        }
    }
    out
}
because!(untimed, LearnedWording, "the rules learned over stated facts with their conditions read at a time made plain, when they answer the sample as well with no more wrong answers: a condition at the time of an earlier one that repeats it goes, and the others read any fact, as facts that a story states once (who is whose parent) hold at every time, and a rule that reads them at a time says no more than the plain one");

fn rule_text_line(conditions: &[Condition], outcome: &Pattern) -> String {
    let joined: Vec<String> = conditions.iter().filter_map(|condition| if let Condition::Fact { pattern, .. } = condition { Some(library_writing(pattern)) } else { None }).collect();
    format!("{TOLD_IF}{}{TOLD_THEN}{}{STATEMENT_END}", joined.join(CONDITIONS_JOINED), library_writing(outcome))
}
because!(rule_text_line, LearnedWording, "a rule of fact conditions read at any time as a line of the library");
const STATED_PER_LESSON: usize = 4;
because!(STATED_PER_LESSON, LearnedWording, "about how many stated facts one lesson gives (a fact held of each of two sentences, and the fact of each ruled out with its places turned or its value changed): the lessons whose stories suggest the rules of a search over stated facts are as many lessons of the kind as a search over the kind itself reads");

const STATED_SAMPLE: usize = 60;
because!(STATED_SAMPLE, LearnedWording, "the lessons of the sample of a search over stated facts: each lesson of the kind states a few facts of several sentences (a parent, a sibling, a grandparent), so a sample of the usual size holds too few of each sentence for its rules to pay off on it; half as many again is enough, as a sample twice that size kept the same rules on the lessons of kinship at twice the work, past the bound of the work of a learning");

because!(held_rules, LearnedWording, "rules learned from lessons that state facts, for the facts that hold alone: a stated fact that is ruled out is a fact that no rule may make, not one that rules should make, so the search names its asked fact by the lessons answered yes only; with no search for states, as stated facts hold at every time; a rule whose outcome names a person of the stories is left out, as it names that one person; and each rule's conditions at a time made plain where the sample allows (untimed)");

fn aligned_ways(topic: &Topic, sample: &[&Case]) -> Vec<AnswerFact> {
    let replies: Vec<(usize, &ReplyLine, Vec<usize>)> = topic
        .reply_lines
        .iter()
        .enumerate()
        .filter_map(|(line, reply)| {
            let asked = places_of(&reply.question);
            let [LogicPiece::Open(answer)] = reply.answer.as_slice() else { return None };
            let own = topic.questions.iter().filter(|form| sentence_map(&form.question, &reply.question).is_some()).any(|form| form.replies.iter().any(fact_named));
            (own && !asked.contains(answer) && (1..=MOST_BRIDGE_PLACES).contains(&asked.len())).then_some((line, reply, asked))
        })
        .collect();
    if replies.is_empty() {
        return Vec::new();
    }
    let words = question_words(sample);
    let mut runs: HashMap<usize, usize> = HashMap::new();
    for case in sample {
        if let Some(form) = form_of(case, &words, Cut::Share(FIXED_SHARE_PERCENT)) {
            *runs.entry(places_of(&form).len()).or_default() += 1;
        }
    }
    let Some(most) = runs.into_iter().max_by(|one, other| one.1.cmp(&other.1).then_with(|| other.0.cmp(&one.0))).map(|(count, _)| count) else { return Vec::new() };
    let mut out: Vec<AnswerFact> = Vec::new();
    for (line, reply, asked) in replies {
        let same_form = sample.iter().any(|case| {
            let mut fits = Vec::new();
            fit_into(&reply.question, &fact_numbers(&case.question), &empty_filling(), &mut fits);
            !fits.is_empty()
        });
        if same_form || asked.len() != most {
            continue;
        }
        for order in orders(most) {
            out.push(AnswerFact::Aligned(line, order.iter().map(|at| asked[*at]).collect()));
        }
    }
    out
}
because!(aligned_ways, LearnedWording, "the ways to read a reply line of the text for the lessons of a question of another form: each reply line of the text's own question forms (not one of a text that it builds on) whose question fits no question of the sample (a question of the same form that its reply line does not answer gains nothing from being read again), whose form has an answer that a sentence of facts names (not a number that the engine works out, as a flow is: no lesson of facts can be drawn back from it), with a one-place answer whose question has as many places as most questions of the sample have runs (at most as many as a bridge reaches, as each order is a search of its own and their number grows as the factorial of the places), in every order of its places, as the lessons and not a guess tell which value goes where (who is asked about and who is the answer's other person); the reply lines are looked at before the questions of the sample, as most texts have none of them and the forms of the sample cost a reading of each question");

fn fact_named(reply: &Reply) -> bool {
    let [LogicPiece::Open(answer)] = reply.words.as_slice() else { return false };
    reply.conditions.iter().any(|condition| matches!(condition, Condition::Fact { pattern, .. } if pattern.iter().any(|piece| matches!(piece, LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) if place == answer))))
}
because!(fact_named, LearnedWording, "whether an answer of a question form is a word that a sentence of facts holds, so that the facts that the form needs can be drawn back from the answer of a lesson (a parent and a sex from son), and not a value that the engine works out");

fn form_opened(case: &Case, words: &QuestionWords) -> Option<(Vec<WordId>, Vec<bool>)> {
    let said = stated(&case.question);
    let mut form = form_of(case, words, Cut::Share(FIXED_SHARE_PERCENT))?;
    while form.last().is_some_and(|piece| matches!(piece, LogicPiece::Word(word) if !word_not_mark(word_text(*word)))) {
        form.pop();
    }
    let mut fitted = Vec::new();
    fit_into(&form, &said, &empty_filling(), &mut fitted);
    let whole = fitted.into_iter().next()?;
    let open: Vec<bool> = form.iter().flat_map(|logic_piece| match logic_piece {
        LogicPiece::Word(_) => vec![false],
        LogicPiece::Open(place) => vec![true; whole.get(*place).map_or(usize::default(), <[WordId]>::len)],
        LogicPiece::OpenWithEnd(..) => vec![true],
    }).collect();
    (open.len() == said.len()).then_some((said, open))
}
because!(form_opened, LearnedWording, "the words of a question as a statement, with the words that its form leaves open (a name, a country that few questions share), as the values that the question asks about");

fn question_opened(case: &Case, words: &QuestionWords) -> Option<(Vec<WordId>, Vec<bool>)> {
    let (said, open) = form_opened(case, words)?;
    let told: HashSet<String> = case.sentences.iter().flat_map(|sentence| fact_words(sentence)).collect();
    let open: Vec<bool> = open.iter().zip(&said).map(|(open, word)| *open || (word_not_mark(word_text(*word)) && told.contains(word_text(*word)))).collect();
    (open.len() == said.len()).then_some((said, open))
}
because!(question_opened, LearnedWording, "the words of a question as a statement, with which of them a rule may give places: the words that its form leaves open (form_opened), and the words of the question that its story tells (the football that a question asks about, though most questions name it), while the other words of its frame stay as they are (is, where)");
because!(answer_fact, LearnedWording, "the fact that answers one lesson in the way asked for, with the fewest places that a rule's outcome must hold, the reply line it comes from, and which of its words a place may take (the words that the reply line or the form of the question leaves open, and the answer); nothing for a lesson of another label, which a rule must not make true");

fn stated(question: &str) -> Vec<WordId> {
    let mut words = fact_numbers(question);
    while words.last().is_some_and(|word| !word_not_mark(word_text(*word))) {
        words.pop();
    }
    words
}
because!(stated, LearnedWording, "the words of a question as a statement: its fact words without the marks at its end");

fn reply_probe(line: &ReplyLine) -> Option<AskedForm> {
    let text = format!("{LINE_START}{}{QUESTION_END}{LINE_MIDDLE}{}{LEARNED_IF}{LATEST_START}{}{STATEMENT_END}", library_writing(&line.question), library_writing(&line.answer), library_writing(&line.said));
    read_topic(PROBE, &text).ok()?.questions.into_iter().next()
}
because!(reply_probe, LearnedWording, "the question form that a reply line makes: its question, answered by its answer when the latest fact holds the sentence that says the answer, so that a learned rule is judged by the fact that the reply line names, in its words and their order, and not by any fact that a filled answer might read; beside it, the filled answers of the facts that the text's own rules make, so that a line that lets those rules read more stories (a sentence written again without its word of sequence) is judged by them, while a fact that only a learned rule names is judged by the reply line alone");

pub(super) fn fact_values(fact: &[WordId], patterns: &[Pattern]) -> Option<(Pattern, Vec<Vec<WordId>>)> {
    patterns
        .iter()
        .filter_map(|pattern| {
            let mut out = Vec::new();
            fit_into(pattern, fact, &empty_filling(), &mut out);
            let filling = out.into_iter().next()?;
            let values = places_of(pattern).iter().map(|place| filling.get(*place).map(<[WordId]>::to_vec)).collect::<Option<Vec<Vec<WordId>>>>()?;
            Some((pattern.clone(), values))
        })
        .max_by_key(|(pattern, _)| fixed_count(pattern))
}
because!(fact_values, LearnedWording, "a fact of a story read by the most fixed of the sentences that the rules of the text read or make, with the words that fill each of its places: the values of the fact, as the rules show where a fact holds its things");

fn generalized(pattern: &Pattern, values: &[Vec<WordId>], named: &mut Vec<Vec<WordId>>) -> Pattern {
    let places = places_of(pattern);
    let mut place_of = |place: usize| -> usize {
        let value = places.iter().position(|known| *known == place).map(|at| values[at].clone()).unwrap_or_default();
        match named.iter().position(|known| *known == value) {
            Some(at) => at,
            None => {
                named.push(value);
                named.len() - 1
            }
        }
    };
    pattern
        .iter()
        .map(|logic_piece| match logic_piece {
            LogicPiece::Open(place) => LogicPiece::Open(place_of(*place)),
            LogicPiece::OpenWithEnd(place, ending) => LogicPiece::OpenWithEnd(place_of(*place), ending.clone()),
            LogicPiece::Word(word) => LogicPiece::Word(*word),
        })
        .collect()
}
because!(generalized, LearnedWording, "a sentence of a rule from a fact of a story: each place of the sentence that read the fact gets the place of its value, the same value the same place over all the facts of one rule, as two facts that name the same person join on that person");

fn generalized_outcome((target, open): (&[WordId], &[bool]), named: &[Vec<WordId>]) -> Pattern {
    let mut out: Pattern = Vec::new();
    let mut at = usize::default();
    while at < target.len() {
        match named.iter().enumerate().filter(|(_, value)| !value.is_empty() && target[at..].starts_with(value) && open[at..at + value.len()].iter().all(|open| *open)).max_by_key(|(_, value)| value.len()) {
            Some((place, value)) => {
                out.push(LogicPiece::Open(place));
                at += value.len();
            }
            None => {
                out.push(LogicPiece::Word(target[at]));
                at += 1;
            }
        }
    }
    out
}
because!(generalized_outcome, LearnedWording, "the outcome of a rule from the fact that answers a lesson: each run of its open words that is a value of a condition becomes the place of that value, the longest first, and the other words stay, so that a word of the frame of the question (is, where) never becomes a place");

pub(super) fn story_patterns(topic: &Topic) -> Vec<Pattern> {
    let mut patterns: Vec<Pattern> = Vec::new();
    for pattern in made_facts(topic, &[]).into_iter().chain(told_targets(topic).into_iter().map(|target| target.pattern)) {
        if !places_of(&pattern).is_empty() && !patterns.contains(&pattern) {
            patterns.push(pattern);
        }
    }
    patterns
}
because!(story_patterns, LearnedWording, "the sentences by which the facts of a story are read for learned rules: every fact that a rule of the text makes and every fact that a told sentence may be read into, each once, with a place");

fn rule_candidates(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case], way: &AnswerFact) -> (Vec<(String, Option<String>, usize)>, Vec<(String, usize)>, bool) {
    let (found, dropped) = suggested_rules(topic, base, lessons, way, (false, false));
    if !found.is_empty() {
        return (found, dropped, false);
    }
    let (found, dropped) = suggested_rules(topic, base, lessons, way, (true, false));
    if !found.is_empty() {
        return (found, dropped, true);
    }
    let (found, dropped) = suggested_rules(topic, base, lessons, way, (true, true));
    (found, dropped, true)
}
because!(rule_candidates, LearnedWording, "the rules that the stories of the lessons suggest (suggested_rules), read through the sentences of the text; only when those suggest no rule that enough lessons share, read again in the forms of the told sentences themselves, with rules that differ in one word made one and a free place scoped by known facts, kept only when a known fact scopes one of its places: a text whose rules are gone may read a told sentence only as one place beside is (B is D), so that each suggested rule names one person, while a text that reads its stories keeps the rules that it found; and only when those forms suggest no rule either, with chains of three facts too, as a concept that joins three facts (A is the flen of B, B is the morp of C, C is the trep of D) is suggested by no fact alone and by no two");

fn second_round(topic: &Topic, base: &Arc<Facts>, (lessons, sample): (&[&Case], &[&Case]), (choices, uses, way): (&[Vec<ToldChoice>], &[usize], &AnswerFact), (slots, trace): (Vec<Vec<String>>, Option<&mut Vec<SearchStep>>)) -> Vec<Vec<String>> {
    let ground = SeedGround::of_answers(topic, lessons, way);
    let Some(reading) = ground.reading(topic, base, &slots.concat()) else { return slots };
    let missed: Vec<&Case> = lessons.iter().zip(ground.wrong(&reading)).filter(|(case, wrong)| *wrong && !sample.iter().any(|known| std::ptr::eq(*known, **case))).map(|(case, _)| *case).take(SEED_SAMPLE).collect();
    if missed.is_empty() {
        return slots;
    }
    let wider: Vec<&Case> = sample.iter().copied().chain(missed).collect();
    seeded_slots(topic, base, &wider, (choices, 1, uses), (hardware_lanes(), slots, Some(way), trace))
}
because!(second_round, LearnedWording, "a second round of the search for rules learned from the forms of the told sentences: the lessons whose stories suggested the rules that the chosen rules still answer wrong, as many as the sample at most, join the sample, and the search goes on from the rules chosen, so that a question that the sample happened not to hold (A is sibling of B) gets its rule, judged with no wrong answer on the sample and on those lessons; nothing more when the rules answer all those lessons");

fn split_through_kept(topic: &Topic, base: &Arc<Facts>, ground: &SeedGround, lines: Vec<String>) -> (Vec<String>, Vec<(String, String, String)>) {
    let tally = |lines: &[String]| ground.reading(topic, base, lines).map(|reading| judged_tally(&reading.rights, &reading.answered));
    let Some(mut now) = tally(&lines) else { return (lines, Vec::new()) };
    let mut out = lines.clone();
    let mut splits: Vec<(String, String, String)> = Vec::new();
    for line in &lines {
        let Some(rule) = read_topic(PROBE, line).ok().and_then(|read| read.rules.into_iter().next()) else { continue };
        let Some(at) = out.iter().position(|known| known == line) else { continue };
        for (bridge, step) in kept_splits(topic, &rule) {
            let mut tried = out.clone();
            tried.splice(at..=at, [bridge.clone(), step.clone()]);
            if let Some(judged) = tally(&tried)
                && judged.0 >= now.0
                && judged.1 <= now.1
            {
                out = tried;
                now = judged;
                splits.push((line.clone(), bridge, step));
                break;
            }
        }
    }
    (out, splits)
}
because!(split_through_kept, LearnedWording, "the rules chosen from the forms of the told sentences, each split through a sentence that the question forms of the text read, with each split made, when the bridge to it and the rule from it answer the sample as well as the rule did, with no more wrong answers (kept_splits: If A is B of C and B means a D parent then A is a parent of C, and If A is a parent of C then C is child of A): the fact in between is in the text's own words, so its other rules and questions read it too, while the rule learned in the words of the kind reads it for that kind alone");

fn asked_values(topic: &Topic, case: &Case, (way, words): (&AnswerFact, &QuestionWords)) -> Option<Vec<bool>> {
    match way {
        AnswerFact::Reply => reply_target(topic, case).map(|(_, _, _, open)| open),
        AnswerFact::Label(_) => form_opened(case, words).map(|(_, open)| open),
        AnswerFact::Spoken => form_opened(case, words).map(|(_, mut open)| {
            open.extend(vec![false; fact_numbers(&case.answer).len()]);
            open
        }),
        AnswerFact::Aligned(line, order) => aligned_target(topic, case, words, (*line, order, false)).map(|(_, _, _, open)| open),
        AnswerFact::Formed(at) => formed_target(topic, case, *at).map(|(_, _, _, open)| open),
    }
}
because!(asked_values, LearnedWording, "for the fact that answers a lesson, which of its words are values that the question asks about: the places of the reply line, or the words that the form of the question leaves open, while the answer of a question that names its fact with its answer is a label, not a value");

fn keeps_open((target, open, asked): (&[WordId], &[bool], &[bool]), named: &[Vec<WordId>]) -> bool {
    let mut at = usize::default();
    while at < target.len() {
        match named.iter().filter(|value| !value.is_empty() && target[at..].starts_with(value) && open[at..at + value.len()].iter().all(|open| *open)).map(Vec::len).max() {
            Some(length) => at += length,
            None if asked[at] => return true,
            None => at += 1,
        }
    }
    false
}
because!(keeps_open, LearnedWording, "whether the outcome that a lesson suggests keeps a word that its question asks about (asked_values), as no value of the rule's conditions stands for it where the outcome is made (generalized_outcome) (a country that the question names, a name that it asks about): such a rule names that one thing (If R lives in S and S is the capital of C then R lives in argentina), true by chance on the few lessons that ask it, so the forms of the told sentences suggest it to no search");

fn told_sentence_forms(lessons: &[&Case], frame: &HashSet<String>) -> Vec<Pattern> {
    let mut out: Vec<Pattern> = Vec::new();
    for case in lessons {
        for sentence in &case.sentences {
            if let Some((form, _)) = sentence_form(&fact_words(sentence), frame, &HashSet::new(), MOST_TOLD_PLACES)
                && !out.contains(&form)
            {
                out.push(form);
            }
        }
    }
    out
}
because!(told_sentence_forms, LearnedWording, "the forms of the told sentences of some lessons, each once: the words of their frame stay and each run of other words is a place (Mike is mother of Ron as A is B of C), as the sentences of the stories show where their things stand");

fn suggested_rules(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case], way: &AnswerFact, (own_forms, chains): (bool, bool)) -> (Vec<(String, Option<String>, usize)>, Vec<(String, usize)>) {
    let mut patterns = story_patterns(topic);
    let frame = if own_forms { frame_words(lessons) } else { HashSet::new() };
    if own_forms {
        for form in told_sentence_forms(lessons, &frame) {
            if !patterns.contains(&form) {
                patterns.push(form);
            }
        }
    }
    let first = base.added_to(&Facts::default());
    let mut known: HashMap<WordId, Vec<usize>> = HashMap::new();
    for place in 0..first {
        for word in base.fact(place) {
            let places = known.entry(*word).or_default();
            if places.last() != Some(&place) {
                places.push(place);
            }
        }
    }
    let mut counted: HashMap<String, SuggestedRule> = HashMap::new();
    let words = question_words(lessons);
    let mut question_words_frame = question_frame(lessons, FIXED_FRAME_PERCENT);
    question_words_frame.extend(untold_frame(lessons));
    for (lesson, case) in lessons.iter().enumerate() {
        let Some((target, target_places, _, open)) = answer_fact(topic, case, (way, &words)) else { continue };
        let asked = if own_forms { asked_values(topic, case, (way, &words)).filter(|asked| asked.len() == target.len()) } else { None };
        let Some(facts) = bounded_facts(case, base, topic, false) else { continue };
        let story: Vec<(Pattern, Vec<Vec<WordId>>)> = (first..first + facts.added_to(base)).filter_map(|place| fact_values(facts.fact(place), &patterns)).collect();
        let mut count = |conditions: Vec<(String, Pattern)>, outcome: Pattern, named: &[Vec<WordId>]| {
            if own_forms && asked.as_ref().is_none_or(|asked| keeps_open((&target, &open, asked), named)) {
                return;
            }
            let carried: Vec<WordId> = conditions.iter().flat_map(|(_, condition)| condition.iter().filter_map(|logic_piece| match logic_piece {
                LogicPiece::Word(word) => Some(*word),
                _ => None,
            })).collect();
            let fixed = value_fixed((&target, &open), (named, &question_words_frame, &carried));
            let entry = counted.entry(rule_line(&conditions, &outcome)).or_insert_with(|| SuggestedRule { fixed_value: false, first_values: named.to_vec(), varied: vec![false; named.len()], conditions, outcome, lessons: usize::default(), last: usize::MAX, one_word: vec![true; named.len()], values: if own_forms { vec![Vec::new(); named.len()] } else { Vec::new() }, suggested_by: Vec::new() });
            if entry.last != lesson {
                entry.lessons += 1;
                entry.last = lesson;
                if own_forms {
                    entry.suggested_by.push(lesson);
                }
            }
            for (one_word, value) in entry.one_word.iter_mut().zip(named) {
                *one_word &= value.len() == 1;
            }
            entry.fixed_value |= fixed;
            for ((first, varied), value) in entry.first_values.iter().zip(entry.varied.iter_mut()).zip(named) {
                *varied |= first != value;
            }
            if own_forms {
                for (seen, value) in entry.values.iter_mut().zip(named) {
                    if seen.len() < MOST_SEEN_VALUES && !seen.contains(value) {
                        seen.push(value.clone());
                    }
                }
            }
        };
        let near = |values: &Vec<Vec<WordId>>| values.iter().any(|value| !value.is_empty() && target.windows(value.len()).any(|run| run == value.as_slice()));
        for (one, (first_pattern, first_values)) in story.iter().enumerate() {
            let mut named: Vec<Vec<WordId>> = Vec::new();
            let condition = generalized(first_pattern, first_values, &mut named);
            let outcome = generalized_outcome((&target, &open), &named);
            if places_of(&outcome).len() >= target_places && outcome != condition {
                count(vec![(String::new(), condition)], outcome, &named);
            }
            let target_words: Vec<WordId> = target.iter().copied().filter(|word| !first_pattern.contains(&LogicPiece::Word(*word))).collect();
            let mut held: Vec<usize> = first_values.iter().filter_map(|value| value.first()).flat_map(|word| known.get(word).cloned().unwrap_or_default()).collect();
            held.sort_unstable();
            held.dedup();
            for place in held.into_iter().take(MOST_KNOWN_JOINS) {
                let fact = base.fact(place);
                let Some((second_pattern, second_values)) = known_values(fact, first_values, &target_words) else { continue };
                let mut named: Vec<Vec<WordId>> = Vec::new();
                let condition = generalized(first_pattern, first_values, &mut named);
                let joined = named.len();
                let second = generalized(&second_pattern, &second_values, &mut named);
                let outcome = generalized_outcome((&target, &open), &named);
                let outcome_places = places_of(&outcome);
                if outcome_places.len() >= target_places && outcome_places.iter().any(|place| *place >= joined) && outcome != condition {
                    count(vec![(String::new(), condition), (CONDITIONS_JOINED.to_string(), second)], outcome, &named);
                }
            }
            for (other, (second_pattern, second_values)) in story.iter().enumerate().filter(|_| chains) {
                if other == one || !first_values.iter().any(|value| second_values.contains(value)) || !near(first_values) {
                    continue;
                }
                for (last, (third_pattern, third_values)) in story.iter().enumerate() {
                    if last == one || last == other || !second_values.iter().any(|value| third_values.contains(value)) || !near(third_values) {
                        continue;
                    }
                    let mut named: Vec<Vec<WordId>> = Vec::new();
                    let condition = generalized(first_pattern, first_values, &mut named);
                    let joined = named.len();
                    let middle = generalized(second_pattern, second_values, &mut named);
                    let middle_end = named.len();
                    let third = generalized(third_pattern, third_values, &mut named);
                    let outcome = generalized_outcome((&target, &open), &named);
                    let outcome_places = places_of(&outcome);
                    if outcome_places.len() >= target_places && outcome_places.iter().any(|place| *place < joined) && outcome_places.iter().any(|place| *place >= middle_end) {
                        count(vec![(String::new(), condition), (CONDITIONS_JOINED.to_string(), middle), (CONDITIONS_JOINED.to_string(), third)], outcome, &named);
                    }
                }
            }
            for (other, (second_pattern, second_values)) in story.iter().enumerate() {
                if other == one || !first_values.iter().any(|value| second_values.contains(value)) || !(near(first_values) || near(second_values)) {
                    continue;
                }
                let mut named: Vec<Vec<WordId>> = Vec::new();
                let condition = generalized(first_pattern, first_values, &mut named);
                let joined = named.len();
                let second = generalized(second_pattern, second_values, &mut named);
                let outcome = generalized_outcome((&target, &open), &named);
                let outcome_places = places_of(&outcome);
                if outcome_places.len() < target_places || !outcome_places.iter().any(|place| *place < joined) || !outcome_places.iter().any(|place| *place >= joined) {
                    continue;
                }
                let bound = places_of(&condition);
                let unbound = places_of(&second).iter().any(|place| !bound.contains(place));
                for timed in RULE_TIMES {
                    if timed.is_empty() || !unbound {
                        count(vec![(String::new(), condition.clone()), (format!("{CONDITIONS_JOINED}{timed}"), second.clone())], outcome.clone(), &named);
                    }
                    if !timed.is_empty() {
                        count(vec![(String::new(), second.clone()), (CONDITIONS_JOINED.to_string(), condition.clone()), (format!("{CONDITIONS_JOINED}{timed}"), second.clone())], outcome.clone(), &named);
                    }
                }
            }
        }
    }
    let mut listed: Vec<(String, SuggestedRule)> = counted.into_iter().collect();
    if own_forms {
        listed.sort_by(|one, other| one.0.cmp(&other.0));
        let merged = rules_made_one(&listed, &|condition: &Pattern, place: usize| text_events_at(topic, lessons, (condition, place)));
        listed.extend(merged);
    }
    let listed = with_words_of_one_value(listed);
    let (listed, chance): (Vec<(String, SuggestedRule)>, Vec<(String, SuggestedRule)>) = listed.into_iter().partition(|(_, rule)| !rule.fixed_value && rule.varied.iter().all(|varied| *varied));
    let (kept, mut dropped): (Vec<(String, SuggestedRule)>, Vec<(String, SuggestedRule)>) = listed.into_iter().partition(|(_, rule)| rule.lessons >= LEAST_SUPPORT);
    constants_left_out(chance.iter().filter(|(_, rule)| rule.lessons >= LEAST_SUPPORT).count());
    dropped.extend(chance);
    let readings_here = || -> Option<HashSet<WordId>> { told_side_refuses(topic, base, lessons).map(|_| varying_question_words(lessons)) };
    let mut kept: Vec<(String, SuggestedRule)> = if own_forms { scoped_rules(kept, base, (&known, &readings_here)) } else { kept };
    kept.sort_by(|one, other| other.1.lessons.cmp(&one.1.lessons).then_with(|| one.0.cmp(&other.0)));
    dropped.extend(kept.split_off(kept.len().min(MOST_RULE_CANDIDATES)));
    let out = kept
        .into_iter()
        .map(|(line, rule)| {
            let narrow: Vec<bool> = rule.one_word.iter().enumerate().map(|(place, one_word)| *one_word && place < MARKED_FIRST - PLAIN_PLACES).collect();
            let conditions: Vec<(String, Pattern)> = rule.conditions.iter().map(|(joined, condition)| (joined.clone(), narrowed(condition, &narrow))).collect();
            (line, narrow.contains(&true).then(|| rule_line(&conditions, &narrowed(&rule.outcome, &narrow))), rule.lessons)
        })
        .collect();
    (out, dropped.into_iter().map(|(line, rule)| (line, rule.lessons)).collect())
}

fn with_words_of_one_value(listed: Vec<(String, SuggestedRule)>) -> Vec<(String, SuggestedRule)> {
    let mut out: Vec<(String, SuggestedRule)> = Vec::new();
    for (line, mut rule) in listed {
        let one_value: Vec<bool> = rule.varied.iter().map(|varied| !*varied).collect();
        let ended = |pattern: &Pattern| pattern.iter().any(|logic_piece| matches!(logic_piece, LogicPiece::OpenWithEnd(place, _) if one_value.get(*place).copied().unwrap_or_default()));
        if !one_value.contains(&true) || !one_value.contains(&false) || ended(&rule.outcome) || rule.conditions.iter().any(|(_, condition)| ended(condition)) {
            out.push((line, rule));
            continue;
        }
        let worded = |pattern: &Pattern| -> Pattern {
            pattern.iter().flat_map(|logic_piece| match logic_piece {
                LogicPiece::Open(place) if one_value.get(*place).copied().unwrap_or_default() => rule.first_values[*place].iter().map(|word| LogicPiece::Word(*word)).collect::<Vec<LogicPiece>>(),
                other => vec![other.clone()],
            }).collect()
        };
        let conditions: Vec<(String, Pattern)> = rule.conditions.iter().map(|(joined, condition)| (joined.clone(), worded(condition))).collect();
        let outcome = worded(&rule.outcome);
        let worded_line = rule_line(&conditions, &outcome);
        if out.iter().any(|(known, _)| *known == worded_line) {
            continue;
        }
        rule.conditions = conditions;
        rule.outcome = outcome;
        rule.varied = vec![true; rule.varied.len()];
        out.push((worded_line, rule));
    }
    out
}
because!(with_words_of_one_value, LearnedWording, "the suggested rules with each place that took one value in every lesson that suggests the rule written as that value, when some other place took several (a word of the form of the told sentences that the cut made a place, as subjects and scholarship in If A passes all the subjects, D will get the scholarship): the rule is then judged, not dropped as a rule of one thing; a rule whose places all took one value stays one thing, and a value that the outcome keeps while no condition holds it is still refused as a value of the answer");

struct SuggestedRule {
    fixed_value: bool,
    first_values: Vec<Vec<WordId>>,
    varied: Vec<bool>,
    conditions: Vec<(String, Pattern)>,
    outcome: Pattern,
    lessons: usize,
    last: usize,
    one_word: Vec<bool>,
    values: Vec<Vec<Vec<WordId>>>,
    suggested_by: Vec<usize>,
}
because!(SuggestedRule, LearnedWording, "a rule that the stories suggest, as counted: whether its outcome keeps a value of the answer as a fixed word (a constant, which the search never judges: a value that no condition names is right only by the chance of a few lessons, as A lives in australia for any B), the value of each place in the first lesson that suggests it and whether another lesson gave the place another value (a rule whose place took one value only is suggested by one thing, as A lives in canberra, and says no more than a fact of that thing, so the search never judges it either), its conditions, each with the words that join it to the conditions before it, its outcome, how many lessons suggest it and the last of them, for each of its places whether every lesson that suggests it gave that place one word, and, in a reading of the forms of the told sentences, the words that the lessons gave each place and the lessons that suggest it (which only that reading makes rules one and scopes by)");

fn value_fixed((target, open): (&[WordId], &[bool]), (named, frame, carried): (&[Vec<WordId>], &HashSet<WordId>, &[WordId])) -> bool {
    let mut at = usize::default();
    while at < target.len() {
        match named.iter().filter(|value| !value.is_empty() && target[at..].starts_with(value) && open[at..at + value.len()].iter().all(|open| *open)).map(Vec::len).max() {
            Some(length) => at += length,
            None if open[at] && word_not_mark(word_text(target[at])) && !frame.contains(&target[at]) && !carried.contains(&target[at]) => return true,
            None => at += 1,
        }
    }
    false
}
because!(value_fixed, LearnedWording, "whether the outcome of a rule made from the fact that answers a lesson keeps one of its open words as a fixed word (the answer, or a word that the question leaves open), as generalized_outcome makes it, when that word is no word of the frame of the questions (lives and in are told and stay, australia or garden is a value) and no condition of the rule holds it as a fixed word (came to the, carried from the told fact A came to the B): a value that no condition of the rule names, so that the rule says that value whatever its places hold; the frame holds the words of the frame that no story tells too (didn't), as a kind of several question forms gives no word of one form most of its questions");

const MOST_SEEN_VALUES: usize = 64;
because!(MOST_SEEN_VALUES, LearnedWording, "the most different words kept for a place of a suggested rule: the words of one kind that a place takes (the words of a relation, the people of the stories) are a few dozen at most, and a place that takes more takes any word");

fn rules_made_one(listed: &[(String, SuggestedRule)], events_at: &dyn Fn(&Pattern, usize) -> usize) -> Vec<(String, SuggestedRule)> {
    let shapes: Vec<(&[(String, Pattern)], &Pattern)> = listed.iter().map(|(_, rule)| (rule.conditions.as_slice(), &rule.outcome)).collect();
    let mut out: Vec<(String, SuggestedRule)> = Vec::new();
    for merge in merged_rules(&shapes) {
        let joined = merge.conditions.iter().find(|(_, condition)| places_of(condition).contains(&merge.place));
        if joined.is_some_and(|(_, condition)| events_at(condition, merge.place) > 1) {
            continue;
        }
        let members: Vec<&SuggestedRule> = merge.members.iter().map(|at| &listed[*at].1).collect();
        let Some(outcome) = members.first().map(|first| first.outcome.clone()) else { continue };
        let line = rule_line(&merge.conditions, &outcome);
        if listed.iter().chain(out.iter()).any(|(known, _)| *known == line) {
            continue;
        }
        let mut one_word = vec![true; merge.place + 1];
        let mut values: Vec<Vec<Vec<WordId>>> = vec![Vec::new(); merge.place + 1];
        for member in &members {
            for (place, (one, seen)) in member.one_word.iter().zip(&member.values).enumerate() {
                one_word[place] &= *one;
                for value in seen {
                    if values[place].len() < MOST_SEEN_VALUES && !values[place].contains(value) {
                        values[place].push(value.clone());
                    }
                }
            }
        }
        for word in &merge.words {
            if !values[merge.place].contains(&vec![*word]) {
                values[merge.place].push(vec![*word]);
            }
        }
        let mut suggested_by: Vec<usize> = members.iter().flat_map(|member| member.suggested_by.iter().copied()).collect();
        suggested_by.sort_unstable();
        suggested_by.dedup();
        out.push((line, SuggestedRule { fixed_value: members.iter().any(|member| member.fixed_value), first_values: Vec::new(), varied: Vec::new(), conditions: merge.conditions, outcome, lessons: suggested_by.len(), last: usize::MAX, one_word, values, suggested_by }));
    }
    out
}
because!(rules_made_one, LearnedWording, "the suggested rules that differ in one word of a condition made one (merged_rules: A And his B went to C and A And her B went to C as A And D B went to C), each suggested by every lesson that suggests one of them and counted once for each, with the words that each place took, the new place taking the words in which they differ; a rule already suggested is not made again, nor one whose new place would read, in the told sentences of the lessons, two or more words that the rules of the text read as events of their own (text_events_at: a take and a drop), as told forms are not joined there either");

fn scoped_rules(kept: Vec<(String, SuggestedRule)>, base: &Facts, (known, readings_here): (&HashMap<WordId, Vec<usize>>, &dyn Fn() -> Option<HashSet<WordId>>)) -> Vec<(String, SuggestedRule)> {
    let mut out: Vec<(String, SuggestedRule)> = Vec::new();
    let readings: std::cell::OnceCell<Option<HashSet<WordId>>> = std::cell::OnceCell::new();
    for (line, mut rule) in kept {
        let mut uses: HashMap<usize, usize> = HashMap::new();
        for piece in rule.conditions.iter().flat_map(|(_, condition)| condition.iter()) {
            if let LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) = piece {
                *uses.entry(*place).or_default() += 1;
            }
        }
        let said = places_of(&rule.outcome);
        let mut free: Vec<usize> = uses.iter().filter(|(place, count)| **count == 1 && !said.contains(place)).map(|(place, _)| *place).collect();
        free.sort_unstable();
        if joins_facts(&rule, &said) {
            match out.iter_mut().find(|(known_line, _)| *known_line == line) {
                Some((_, known_rule)) => known_rule.lessons = known_rule.lessons.max(rule.lessons),
                None => out.push((line, rule)),
            }
            continue;
        }
        let mut scoped = false;
        let mut joined_known = false;
        let mut one_thing = false;
        for place in free {
            if let Some([value]) = rule.values.get(place).map(Vec::as_slice) {
                let words: Pattern = value.iter().map(|word| LogicPiece::Word(*word)).collect();
                for (_, condition) in &mut rule.conditions {
                    *condition = condition.iter().flat_map(|piece| if *piece == LogicPiece::Open(place) { words.clone() } else { vec![piece.clone()] }).collect();
                }
                scoped = true;
                continue;
            }
            let Some(shape) = rule.values.get(place).and_then(|values| known_scope(base, known, values)) else { continue };
            let given = shape_values(base, known, &shape);
            one_thing |= rule.outcome.iter().any(|piece| matches!(piece, LogicPiece::Word(word) if given.contains(word)));
            let first_new = rule.values.len();
            let from = places_of(&shape);
            let to: Vec<usize> = from.iter().map(|old| if *old == usize::default() { place } else { first_new + old - 1 }).collect();
            if to.iter().any(|new| *new >= PLAIN_PLACES) {
                continue;
            }
            rule.conditions.push((CONDITIONS_JOINED.to_string(), renamed(&shape, &from, &to)));
            rule.one_word.extend(vec![false; from.len() - 1]);
            rule.values.extend(vec![Vec::new(); from.len() - 1]);
            scoped = true;
            joined_known = true;
        }
        let reading = !joined_known && !scoped && rule.conditions.len() == 1 && readings.get_or_init(readings_here).as_ref().is_some_and(|asked| !rule.outcome.iter().any(|piece| matches!(piece, LogicPiece::Word(word) if asked.contains(word))));
        if (!joined_known && !reading) || one_thing {
            continue;
        }
        let line = if scoped { rule_line(&rule.conditions, &rule.outcome) } else { line };
        match out.iter_mut().find(|(known_line, _)| *known_line == line) {
            Some((_, known_rule)) => known_rule.lessons = known_rule.lessons.max(rule.lessons),
            None => out.push((line, rule)),
        }
    }
    out
}
fn varying_question_words(lessons: &[&Case]) -> HashSet<WordId> {
    let mut uses: HashMap<WordId, usize> = HashMap::new();
    for case in lessons {
        for word in fact_numbers(&case.question).into_iter().collect::<HashSet<WordId>>() {
            *uses.entry(word).or_default() += 1;
        }
    }
    uses.into_iter().filter(|(_, count)| count * WHOLE_SHARE < lessons.len() * FORM_FRAME_PERCENT).map(|(word, _)| word).collect()
}
because!(varying_question_words, LearnedWording, "the words that some questions of the lessons hold and most do not, as an asked value does (the country that a question names): a reading of one told sentence whose outcome holds such a word states one value of the answer whatever its places hold");

fn joins_facts(rule: &SuggestedRule, said: &[usize]) -> bool {
    let read: Vec<Vec<usize>> = rule.conditions.iter().map(|(_, condition)| places_of(condition)).collect();
    read.len() > 1 && read.iter().all(|places| !places.is_empty()) && read.iter().enumerate().all(|(at, places)| places.iter().all(|place| said.contains(place) || read.iter().enumerate().any(|(other, more)| other != at && more.contains(place))))
}
because!(joins_facts, LearnedWording, "whether a suggested rule joins two or more facts through the values that they share: every place of each condition is named by the outcome or by another condition, so the rule reads no place that only one fact gives (If A came to the B and C will come to the B if A comes then C came to the B); such a rule is a rule of the concept and needs no known fact to scope a place, while a rule of one condition is a reading of one told sentence, the told learner's");

because!(scoped_rules, LearnedWording, "the suggested rules with each place that one condition reads and the outcome does not name scoped by what the text knows (known_scope: If A is B of C then C is child of A, for B father or mother, reads B means a D parent too), as such a place takes any word while the lessons show the rule only for the words that they gave it; a place that the lessons gave one word only is that word again (If A is mother of C), the least general rule of one case; a rule kept only when a known fact scopes one of its places, as a bridge joins a told form to what the text knows, or when it joins two or more facts through the values that they share (joins_facts), while a reading of one told sentence alone is the told learner's (its readings come first where one fits), and left out when its outcome keeps a word that the known facts of its scope give a place (shape_values: R lives in Argentina beside S is the capital of C), as it names that one thing; two rules that become one keep the most lessons of the two");

fn rule_line(conditions: &[(String, Pattern)], outcome: &Pattern) -> String {
    let joined: String = conditions.iter().map(|(joined, condition)| format!("{joined}{}", library_writing(condition))).collect();
    format!("{TOLD_IF}{joined}{TOLD_THEN}{}{STATEMENT_END}", library_writing(outcome))
}
because!(rule_line, LearnedWording, "a suggested rule as a line of the library: its conditions with the words that join them, and its outcome");

pub(super) fn narrowed(pattern: &Pattern, narrow: &[bool]) -> Pattern {
    let narrow_at = |place: usize| place < MARKED_FIRST - PLAIN_PLACES && narrow.get(place).copied().unwrap_or_default();
    pattern
        .iter()
        .map(|logic_piece| match logic_piece {
            LogicPiece::Open(place) if narrow_at(*place) => LogicPiece::Open(PLAIN_PLACES + *place),
            LogicPiece::OpenWithEnd(place, ending) if ending.is_empty() && narrow_at(*place) => LogicPiece::Open(PLAIN_PLACES + *place),
            other => other.clone(),
        })
        .collect()
}
because!(narrowed, LearnedWording, "a sentence of a suggested rule or of an invented state with each place that every lesson gave one word made a place of one word, as the library writes a name or a thing of one word, while a place with no letter of one word left (past Z, where the letters of places that may hold marks begin) stays as it is: two places side by side (the thing and then its place, in where is the football garden) cannot then split their words in another way, as places of more words also read where is the football there garden as the football in the place there garden");
because!(suggested_rules, LearnedWording, "the rules that the stories of the lessons suggest for the facts that answer their questions: for each lesson, the fact of its answer in the way asked for, and every fact of its story, also joined to a fact that the text knows before any story (the capital of a country) that holds one of its values, that a sentence of the text reads, alone or joined to a second fact that shares a value with it when one of the two names a value of the answer fact (at any time, at the time of the first, or before it); the values of the facts become places, and the outcome is the fact of the answer with those places in it, kept when it holds as many places as the reply line and draws on both facts; a second fact read at a time is also read first at any time, so that its things are named before its state at that time is read (the holder of a thing was the person when the person went somewhere, not only some holder of something), and when it holds a thing that the first fact does not name it is read so only, as a fact read at a time with a place still open is the newest fact of its sentence whatever fills that place, not the state of one thing (a dropped thing would still move with its last holder); each rule counted once for each lesson that suggests it, the most suggested first, each with the same rule with the places that all its lessons gave one word made places of one word, if any; and the rules suggested but not judged (by fewer lessons than the least, or below the bound of the most suggested), each with the lessons that suggest it, for the report of the discovery; when asked, the stories are also read in the forms of their own told sentences (told_sentence_forms), a rule whose outcome keeps a word that the question leaves open is left out (keeps_open), and the rules are made one (rules_made_one) and scoped (scoped_rules); when asked for chains, also three facts in a row, each sharing a value with the next, the first and the last naming values of the answer fact, the outcome drawing on both ends");

fn known_values(fact: &[WordId], values: &[Vec<WordId>], target_words: &[WordId]) -> Option<(Pattern, Vec<Vec<WordId>>)> {
    let mut pattern: Pattern = Vec::new();
    let mut found: Vec<Vec<WordId>> = Vec::new();
    let mut at = usize::default();
    while at < fact.len() {
        if let Some(value) = values.iter().filter(|value| !value.is_empty() && fact[at..].starts_with(value)).max_by_key(|value| value.len()) {
            pattern.push(LogicPiece::Open(found.len()));
            found.push(value.clone());
            at += value.len();
            continue;
        }
        if target_words.contains(&fact[at]) && word_not_mark(word_text(fact[at])) {
            pattern.push(LogicPiece::Open(found.len()));
            found.push(vec![fact[at]]);
        } else {
            pattern.push(LogicPiece::Word(fact[at]));
        }
        at += 1;
    }
    (found.len() > 1).then_some((pattern, found))
}
because!(known_values, LearnedWording, "a fact that the text knows read with places: each run of its words that is a value of the story fact it joins, and each word that the answer fact names beyond the words of that story fact, becomes a place, the other words stay; nothing when it shares fewer than a value and a word of the answer, as it would add nothing to the rule");

const MOST_KNOWN_JOINS: usize = 32;
because!(MOST_KNOWN_JOINS, LearnedWording, "the most facts that the text knows before any story which one fact of a story is joined to: a value that a story names (a city) holds in a few known facts, while a common word would join every known fact");

const RULE_TIMES: &[&str] = &["", "at that time ", "before that time "];
because!(RULE_TIMES, LearnedWording, "the ways a second fact of a learned rule may stand in time to the first: at any time, the latest up to the time of the first (where the holder was when the thing was got), or the latest before it, as the library writes the conditions of its rules");

fn write_candidates(topic: &Topic, lessons: &[&Case]) -> Vec<(String, Option<String>, usize)> {
    let frame = frame_words(lessons);
    let patterns = story_patterns(topic);
    let fillings = |pattern: &Pattern, words: &[String]| -> Vec<Filling> {
        let numbers: Vec<WordId> = words.iter().map(|word| word_number(word)).collect();
        let mut found = Vec::new();
        fit_into(pattern, &numbers, &empty_filling(), &mut found);
        found
    };
    let read = |rest: &[String]| patterns.iter().any(|pattern| !fillings(pattern, rest).is_empty());
    let cuts_in_front = |whole: &[String], length: usize| -> bool {
        let read_whole: Vec<(&Pattern, Vec<Filling>)> = patterns.iter().map(|pattern| (pattern, fillings(pattern, whole))).filter(|(_, found)| !found.is_empty()).collect();
        let Some(most) = read_whole.iter().map(|(pattern, _)| fixed_count(pattern)).max() else { return true };
        read_whole.iter().filter(|(pattern, _)| fixed_count(pattern) == most).any(|(pattern, found)| match pattern.first() {
            Some(LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _)) => found.iter().any(|filling| filling.get(*place).is_some_and(|value| value.len() > length)),
            _ => true,
        })
    };
    let mut counted: HashMap<String, (usize, usize)> = HashMap::new();
    for (lesson, case) in lessons.iter().enumerate() {
        for sentence in &case.sentences {
            let words = fact_words(sentence);
            for length in 1..=MOST_WRITE_WORDS {
                if words.len() <= length || !words[..length].iter().all(|word| word_not_mark(word) && frame.contains(word)) {
                    break;
                }
                if !read(&words[length..]) || !cuts_in_front(&words, length) {
                    continue;
                }
                let mut opening: Pattern = words[..length].iter().map(|word| LogicPiece::Word(word_number(word))).collect();
                opening.push(LogicPiece::Open(usize::default()));
                let line = format!("{LEARNED_WRITE}{}{LEARNED_WRITE_AS}{}{STATEMENT_END}", library_writing(&opening), library_writing(&vec![LogicPiece::Open(usize::default())]));
                let entry = counted.entry(line).or_insert((usize::default(), usize::MAX));
                if entry.1 != lesson {
                    entry.0 += 1;
                    entry.1 = lesson;
                }
            }
        }
    }
    let mut out: Vec<(String, Option<String>, usize)> = counted.into_iter().filter(|(_, (count, _))| *count >= LEAST_SUPPORT).map(|(line, (count, _))| (line, None, count)).collect();
    out.sort_by(|one, other| other.2.cmp(&one.2).then_with(|| one.0.cmp(&other.0)));
    out.truncate(MOST_WRITE_CANDIDATES);
    out
}
because!(write_candidates, LearnedWording, "the lines that write a told sentence again without the words of the frame that open it (then, after that), as a sentence that starts with a word of sequence tells its event as the plain sentence does: each opening of one or a few words of the frame that enough lessons tell where the rest of the sentence is a sentence that the rules of the text read (he went to the hallway, after then), counted once for each lesson, the most told first; an opening whose rest no rule reads takes away a part of the event (Mary journeyed, leaving to the kitchen), as the name of a story that starts each sentence stands where a word of the frame stands; and the most fixed sentences of the rules that read the whole told sentence must hold the opening inside the value of their first place, before more words of it (Afterwards he moved to the hallway), as a sentence with few fixed words reads Mary picked up the football with Mary as a value of its own, which the opening would take away (a Write line of Mary left babi03 reading the carrier of a thing where its question asks the thing)");

const LEARNED_WRITE: &str = "Write ";
because!(LEARNED_WRITE, LearnedWording, "the word that opens a learned line that writes a told sentence in a new form, as the library writes such lines");

const LEARNED_WRITE_AS: &str = " as ";
because!(LEARNED_WRITE_AS, LearnedWording, "the words between the old form and the new form of a learned line that writes a sentence again");

const MOST_WRITE_WORDS: usize = 2;
because!(MOST_WRITE_WORDS, LearnedWording, "the most words of the frame that open a told sentence which a learned line may take away: a word of sequence is a word or a short phrase (then, after that, following that), and a longer opening is a sentence of its own");

const MOST_WRITE_CANDIDATES: usize = 16;
because!(MOST_WRITE_CANDIDATES, LearnedWording, "the openings of told sentences, the most told first, whose lines a first reading judges: a few words of sequence open the sentences of a kind");

const RULE_LESSONS: usize = 120;
because!(RULE_LESSONS, LearnedWording, "the first lessons whose stories suggest the rules of a concept: enough for a rule that holds in many stories to be suggested by several, while each story costs the pairs of its facts");

const MOST_RULE_CANDIDATES: usize = 256;
because!(MOST_RULE_CANDIDATES, LearnedWording, "the rules suggested by the most lessons that a first reading judges: each is judged with the whole sample, so the rare ones that a few stories suggest by chance are left out");

pub fn learn_rules(topic: &Topic, base: &Arc<Facts>, fit: &[&Case]) -> Vec<String> {
    kept_learning(LEARNED_RULES, (topic, base, fit), &|topic, base, fit| {
        let lines = learn_rules_traced(topic, base, fit, None).0;
        search_ended(lines.is_empty());
        lines
    })
}
because!(learn_rules, LearnedWording, "rules of a concept learned from the lessons of a kind, with no report of the search, read back from the store between runs when an earlier run of the same code discovered them for the same text, known facts and lessons; a search that runs is counted in the costs of the learning with whether it found nothing");

fn learn_formed(topic: &Topic, base: &Arc<Facts>, fit: &[&Case]) -> Vec<String> {
    let timed = if base.has_history() { Arc::clone(base) } else { Arc::new(knowledge_base(&topic.facts, &topic.rules, true)) };
    let lessons: Vec<&Case> = fit.iter().take(READ_TOLD_LESSONS).copied().collect();
    let sample: Vec<&Case> = spread_sample(&lessons, SEED_SAMPLE);
    let found: Vec<(Vec<String>, usize, Option<RuleSearch>)> = formed_ways(topic, &sample).iter().map(|way| with_states(topic, &timed, (&lessons, &sample, RULE_LESSONS), way, false)).collect();
    found.into_iter().enumerate().max_by(|one, other| (one.1).1.cmp(&(other.1).1).then_with(|| other.0.cmp(&one.0))).map(|(_, (lines, _, _))| lines).unwrap_or_default()
}
because!(learn_formed, LearnedWording, "the rules of a concept learned toward the facts that the text's own question forms read when no rule makes them and no story tells them (formed_ways), judged by the form itself on the lessons, the form whose rules answer the sample best kept: for a kind that a plain wording reads below the bar; the texts whose asking form holds the most fixed words are tried first, as the form most of the kind's own");

const ALONE_STORY_SENTENCES: usize = 1;
because!(ALONE_STORY_SENTENCES, LearnedWording, "the most told sentences of each story of the sample for which the board learns the rules of a new concept from the lessons alone: a text with no line reads a story only in the forms of its told sentences, and on the longer stories of the kinds that nothing reads (StepGame, the kinship stories of CLUTRR, BoardgameQA, FLD, the logic grids) the search over their facts and the readings of their sentences ran for minutes on each kind and found nothing, while the premise of a pair of sentences, told alone, costs seconds");

fn labelled_statements(cases: &[&Case]) -> bool {
    let mut labels: Vec<Vec<String>> = Vec::new();
    for case in cases {
        let label = answer_words(&case.answer);
        if !labels.contains(&label) {
            labels.push(label);
        }
    }
    labels.len() <= MOST_RULE_LABELS && answers_told(cases) * WHOLE_SHARE < cases.len() * SEED_TOLD_PERCENT
}
because!(labelled_statements, LearnedWording, "whether some lessons are of a few labels that their stories do not tell, so that the rules of a concept are learned for the question as a statement (learn_rules_traced): the way in which a text with no line can name the asked fact, as a question whose answer is a word of the story, or one of many labels (the relations of StepGame, which a text with no line searched for half a minute and found nothing for), needs a fact that a text names");

const ALONE_TEXT: &str = "of the lessons alone";
because!(ALONE_TEXT, LearnedWording, "the name of the text of a concept learned from the lessons of a kind alone, which holds no line of the library, as the notes and proofs of its answers name it");

fn blank_text() -> (Topic, Arc<Facts>) {
    let topic = Topic { name: ALONE_TEXT.to_string(), rules: Vec::new(), questions: Vec::new(), reply_lines: Vec::new(), facts: Vec::new(), builds_on: Vec::new(), lines: usize::default(), rules_key: rules_key(&[]) };
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    (topic, base)
}
because!(blank_text, LearnedWording, "a text with no rule, fact, question form or reply line, and its facts before any story, kept as the library keeps those of a text (the learners of rules read the stories on facts that keep their history, and the model keeps it when a learned rule needs it): the ground of the rules of a concept that a kind's lessons teach alone");

const BLANK_WORDING: &str = "the wording on a text with no line";
because!(BLANK_WORDING, LearnedWording, "the name in the path notes of the question forms learned from a kind's lessons on a text with no line, with no rule under them");

const NO_BLANK_FORM: &str = "no form";
because!(NO_BLANK_FORM, LearnedWording, "what the path note says when no question form is learned on a text with no line");

fn blank_wording(cases: &[&Case]) -> Option<WordedModel> {
    let (blank, blank_base) = blank_text();
    let (forms, _) = learn_forms(&blank, &blank_base, cases);
    if forms.is_empty() {
        return None;
    }
    let read = read_topic(&blank.name, &forms.iter().map(form_line).collect::<Vec<String>>().join(LEARNED_LINE_BREAK)).ok()?;
    let base = timed_base(&blank, &read.questions, &blank_base);
    let named = (named_forms(&forms, cases).into_iter().filter(|named| *named).count(), forms.len());
    Some(WordedModel { topic: Topic { questions: read.questions, ..blank.clone() }, base, source: blank.name.clone(), hand: false, learned: forms.iter().map(form_line).collect(), taught: Vec::new(), named, frames: None })
}
because!(blank_wording, LearnedWording, "the question forms of a kind learned from its lessons on a text with no line (blank_text), with no rule under them: a kind whose answers the told sentences name, or the reckoning of the engine works out, needs no line of the library, only a text to hold its forms, and a text of rules would only add rules that it does not use; tried for a kind that no text promised a wording for, as where texts promised one and none read it, the forms alone read no more (the logic grids paid seconds for nothing)");

fn learn_alone(topic: &Topic, base: &Arc<Facts>, fit: &[&Case]) -> Vec<String> {
    rules_and_readings(topic, base, fit, (None, None, None)).lines
}
because!(learn_alone, LearnedWording, "the lines that a kind's lessons teach a text with no line (rules_and_readings): the rules of a concept, and when they leave some lesson of the sample wrong, the readings of the told sentences with the rules learned again on top of them, as the curriculum teaches a text");

fn proved_rules(model: &WordedModel, cases: &[&Case]) -> ((usize, usize), Vec<HashSet<String>>) {
    let mut right = usize::default();
    let mut wrong = usize::default();
    let mut proofs: Vec<HashSet<String>> = Vec::new();
    for case in cases {
        let said = model.answer(&case.sentences, &case.question);
        if said.words.is_empty() {
            continue;
        }
        if said.words != answer_words(&case.answer) {
            wrong += 1;
            continue;
        }
        right += 1;
        proofs.push(model.rules_used(&case.sentences, &case.question).into_iter().map(|(rule, _)| rule).collect());
    }
    ((right, wrong), proofs)
}
because!(proved_rules, LearnedWording, "how many lessons a learned model answers right and wrong, and for each right answer the rules that its proof rests on");

fn answer_tally(model: &WordedModel, cases: &[&Case]) -> (usize, usize) {
    let said: Vec<Said> = cases.iter().map(|case| model.answer(&case.sentences, &case.question)).collect();
    let right = cases.iter().zip(&said).filter(|(case, said)| said.words == answer_words(&case.answer)).count();
    (right, said.iter().filter(|said| !said.words.is_empty()).count() - right)
}
because!(answer_tally, LearnedWording, "how many lessons a learned model answers right and how many wrong (an empty answer is neither), with no proofs");

fn rule_of_line(line: &str) -> Option<String> {
    let read = read_topic(PROBE, line).ok()?;
    (read.rules.len() == 1 && read.facts.is_empty() && read.questions.is_empty()).then(|| rule_text(&read.rules[0]))
}
because!(rule_of_line, LearnedWording, "a learned line as a proof names its rule, when the line is one rule (not a fact, a question form or a line that writes a sentence again)");

fn rule_parts(line: &str) -> Option<(&str, &str)> {
    line.strip_prefix(TOLD_IF)?.strip_suffix(STATEMENT_END)?.split_once(TOLD_THEN)
}
because!(rule_parts, LearnedWording, "the conditions and the outcome of a learned rule as the words of its line");

fn is_place_letter(word: &str) -> bool {
    let mut letters = word.chars();
    letters.next().is_some_and(|letter| letter.is_ascii_uppercase()) && letters.next().is_none()
}
because!(is_place_letter, LearnedWording, "whether a word of a learned line is an open place, written as one capital letter");

fn composed_line(first: &str, second: &str) -> Option<String> {
    let (first_if, first_then) = rule_parts(first)?;
    let (second_if, second_then) = rule_parts(second)?;
    if second_if.contains(CONDITIONS_JOINED) {
        return None;
    }
    let made: Vec<&str> = first_then.split(FORM_WORD_GAP).collect();
    let read: Vec<&str> = second_if.split(FORM_WORD_GAP).collect();
    if made.len() != read.len() {
        return None;
    }
    let mut places: HashMap<&str, &str> = HashMap::new();
    for (word, made_word) in read.iter().zip(&made) {
        if is_place_letter(word) != is_place_letter(made_word) || (!is_place_letter(word) && word != made_word) || *places.entry(word).or_insert(made_word) != *made_word {
            return None;
        }
    }
    let outcome: Vec<&str> = second_then.split(FORM_WORD_GAP).map(|word| if is_place_letter(word) { places.get(word).copied() } else { Some(word) }).collect::<Option<Vec<&str>>>()?;
    Some(format!("{TOLD_IF}{first_if}{TOLD_THEN}{}{STATEMENT_END}", outcome.join(FORM_WORD_GAP)))
}
because!(composed_line, LearnedWording, "two learned rules made one when the second reads, in its one condition, the very fact that the first makes (the same words, its places in the same order): the conditions of the first with the outcome of the second, so that a fact in between that no told sentence or question names (A the B from C, read off a question of another form) leaves the proof");

fn cleaned_lines((topic, base): (&Topic, &Arc<Facts>), lines: Vec<String>, fit: &[&Case]) -> (Vec<String>, Option<WordedModel>) {
    let Some(model) = learned_model((topic, base, false), &lines, fit) else { return (lines, None) };
    let (score, proofs) = proved_rules(&model, fit);
    let used: HashSet<&String> = proofs.iter().flatten().collect();
    let kept: Vec<String> = lines.iter().filter(|line| rule_of_line(line).is_none_or(|rule| used.contains(&rule))).cloned().collect();
    if kept.len() == lines.len() && !kept.iter().any(|first| kept.iter().any(|second| first != second && composed_line(first, second).is_some())) {
        return (lines, Some(model));
    }
    let mut composed: Vec<String> = Vec::new();
    let mut gone: HashSet<&String> = HashSet::new();
    for first in &kept {
        for second in kept.iter().filter(|second| *second != first) {
            let (Some(line), Some(first_rule), Some(second_rule)) = (composed_line(first, second), rule_of_line(first), rule_of_line(second)) else { continue };
            if !proofs.iter().any(|proof| proof.contains(&first_rule) && proof.contains(&second_rule)) {
                continue;
            }
            if !composed.contains(&line) && !kept.contains(&line) {
                composed.push(line);
            }
            gone.insert(first);
            gone.insert(second);
        }
    }
    let ahead_of = |(right, wrong): (usize, usize)| right >= score.0 && wrong <= score.1;
    for tried in [kept.iter().filter(|line| !gone.contains(line)).cloned().chain(composed.iter().cloned()).collect::<Vec<String>>(), kept.clone()] {
        if tried.len() == lines.len() && tried.iter().all(|line| lines.contains(line)) {
            continue;
        }
        if let Some(cleaner) = learned_model((topic, base, false), &tried, fit)
            && ahead_of(answer_tally(&cleaner, fit))
        {
            return (tried, Some(cleaner));
        }
    }
    (lines, Some(model))
}
because!(cleaned_lines, LearnedWording, "the lines learned from the lessons alone with what no proof needs taken out, and the model learned with them: the rules in no proof of a right lesson answer leave, and two rules that stand together in such a proof, the second reading the fact that the first makes, are made one (composed_line), so that the proofs rest on the told sentences directly; the wording is learned once again on the lines left, and kept only when it answers as many lessons right and no more wrong, else the lines with only the rules in no proof taken out, else the lines as they were");

static ALONE_LEARNED: OnceLock<Mutex<HashMap<(usize, u64, u64), LearnedCell>>> = OnceLock::new();
because!(ALONE_LEARNED, LearnedWording, "the rules learned from some lessons alone, by the lessons, kept for the run, as the board learns a kind more than once from the same first lessons");

static FORMED_LEARNED: OnceLock<Mutex<HashMap<(usize, u64, u64), LearnedCell>>> = OnceLock::new();
because!(FORMED_LEARNED, LearnedWording, "the rules learned toward a text's own question form from some lessons, by the text, its rules and the lessons, kept for the run, as the board learns a kind more than once from the same first lessons");

pub(super) fn learn_rules_traced(topic: &Topic, base: &Arc<Facts>, fit: &[&Case], searches: Option<&mut Vec<RuleSearch>>) -> (Vec<String>, usize, usize) {
    let timed = if base.has_history() { Arc::clone(base) } else { Arc::new(knowledge_base(&topic.facts, &topic.rules, true)) };
    let base = &timed;
    let lessons: Vec<&Case> = fit.iter().take(READ_TOLD_LESSONS).copied().collect();
    let sample: Vec<&Case> = spread_sample(&lessons, SEED_SAMPLE);
    let traced = searches.is_some();
    let kept = |found: Vec<(Vec<String>, usize, Option<RuleSearch>)>| -> (Vec<String>, usize, usize) {
        let best = found.iter().enumerate().max_by(|one, other| (one.1).1.cmp(&(other.1).1).then_with(|| other.0.cmp(&one.0))).map(|(at, _)| at);
        let lines = best.map(|at| found[at].0.clone()).unwrap_or_default();
        let score = best.map_or(usize::default(), |at| found[at].1);
        if let Some(searches) = searches {
            searches.extend(found.into_iter().enumerate().filter_map(|(at, (_, _, search))| search.map(|search| RuleSearch { kept: Some(at) == best, ..search })));
        }
        (lines, score, sample.len())
    };
    if sample.iter().any(|case| reply_target(topic, case).is_some()) {
        return kept(vec![with_states(topic, base, (&lessons, &sample, RULE_LESSONS), &AnswerFact::Reply, traced)]);
    }
    let aligned = if topic.rules.is_empty() { consistent_ways(topic, base, &lessons, aligned_ways(topic, &sample)) } else { Vec::new() };
    let mut labels: Vec<(String, usize)> = Vec::new();
    for case in &lessons {
        match labels.iter_mut().find(|(known, _)| answer_words(known) == answer_words(&case.answer)) {
            Some((_, count)) => *count += 1,
            None => labels.push((case.answer.clone(), 1)),
        }
    }
    if labels.len() > MOST_RULE_LABELS || answers_told(&lessons) * WHOLE_SHARE >= lessons.len() * SEED_TOLD_PERCENT {
        let ways: Vec<AnswerFact> = aligned.into_iter().chain([AnswerFact::Spoken]).collect();
        return kept(ways_until_whole(topic, base, (&lessons, &sample), &ways, traced));
    }
    labels.sort_by(|one, other| other.1.cmp(&one.1));
    let ways: Vec<AnswerFact> = aligned.into_iter().chain(labels.into_iter().map(|(label, _)| AnswerFact::Label(label))).collect();
    kept(ways_until_whole(topic, base, (&lessons, &sample), &ways, traced))
}

fn stored_first(topic: &Topic, base: &Arc<Facts>, sample: &[&Case], way: &AnswerFact) -> Option<(Vec<String>, usize, usize)> {
    let stored = stored_lines(&topic.name);
    if stored.is_empty() {
        return None;
    }
    let (source, kept, right) = stored_first_of(topic, base, (sample, way), &stored_groups(stored))?;
    note_store_taken(source_kind(source));
    Some((kept, right, stored.len()))
}

fn stored_first_of<'a>(topic: &Topic, base: &Arc<Facts>, (sample, way): (&[&Case], &AnswerFact), groups: &[(&'a str, Vec<String>)]) -> Option<(&'a str, Vec<String>, usize)> {
    groups.iter().find_map(|(source, lines)| {
        let (taught, taught_base) = with_learned(topic, base, lines);
        let (right, wrong, _) = SeedGround::of_answers(&taught, sample, way).reading(&taught, &taught_base, &[]).map(|reading| judged_tally(&reading.rights, &reading.answered))?;
        (wrong == usize::default() && right >= sample.len()).then(|| (*source, lines.clone(), right))
    })
}
because!(stored_first, LearnedWording, "the lines that the known-good store holds for a text, judged on the sample of a kind before its rule search (stored_first_of), the kind of the group taken kept by the scope of the learning (note_store_taken), so that the board names it");

pub fn kept_of_stored(topic: &Topic, base: &Arc<Facts>, fit: &[&Case], groups: &[(&str, Vec<String>)]) -> Option<(String, Vec<String>)> {
    let timed = if base.has_history() { Arc::clone(base) } else { Arc::new(knowledge_base(&topic.facts, &topic.rules, true)) };
    let lessons: Vec<&Case> = fit.iter().take(READ_TOLD_LESSONS).copied().collect();
    stored_first_of(topic, &timed, (&spread_sample(&lessons, SEED_SAMPLE), &AnswerFact::Reply), groups).map(|(source, kept, _)| (source.to_string(), kept))
}
because!(kept_of_stored, LearnedWording, "the group of stored lines that a kind of a text with a reply line takes, with its source, judged on its sample as its rule search would judge them before it runs, or nothing when every group leaves some lesson of the sample wrong");
because!(stored_first_of, LearnedWording, "the stored lines of a text (the known-good store) judged on the sample of a kind like the rules of its search, before the search: group by group in the order of the store, the first group whose lines answer every lesson of the sample right and none wrong is taken whole, as lines that one kind's lessons taught together hold together and a line taken out of its group may lean on the others; a group that leaves a lesson wrong is passed over, and with no such group the search runs");

fn stored_search(topic: &Topic, (lessons, way): (&[&Case], &AnswerFact), (kept, right, offered): (&[String], usize, usize), sample: usize) -> RuleSearch {
    RuleSearch { way: way_name(topic, way), target: way_target(topic, lessons, way), answer_targets: answer_targets(topic, lessons, way), states: Vec::new(), candidates: Vec::new(), dropped: Vec::new(), steps: Vec::new(), chosen: kept.to_vec(), score: right, sample, kept: false, tried_states: Vec::new(), state_counts: None, splits: Vec::new(), stored: Some((kept.len(), offered)) }
}
because!(stored_search, LearnedWording, "the report of a search that took the stored lines of its text: how many of them it kept and how many the store offered, with no step");

fn ways_until_whole(topic: &Topic, base: &Arc<Facts>, (lessons, sample): (&[&Case], &[&Case]), ways: &[AnswerFact], traced: bool) -> Vec<(Vec<String>, usize, Option<RuleSearch>)> {
    let mut found = Vec::new();
    for way in ways {
        let one = way_rules(topic, base, (lessons, sample), way, traced);
        let whole = one.1 >= sample.len();
        found.push(one);
        if whole && !traced {
            break;
        }
    }
    found
}
because!(ways_until_whole, LearnedWording, "the rules of each way in order until one answers every lesson of the sample, when no report asks for the others: a later way can at most tie, and a tie goes to the earlier way");

fn way_rules(topic: &Topic, base: &Arc<Facts>, (lessons, sample): (&[&Case], &[&Case]), way: &AnswerFact, traced: bool) -> (Vec<String>, usize, Option<RuleSearch>) {
    let AnswerFact::Aligned(line, order) = way else { return with_states(topic, base, (lessons, sample, RULE_LESSONS), way, traced) };
    if let Some((kept, right, offered)) = stored_first(topic, base, sample, way) {
        let search = traced.then(|| stored_search(topic, (lessons, way), (&kept, right, offered), sample.len()));
        return (kept, right, search);
    }
    let first = abduced_rounds(topic, base, lessons, (*line, order));
    if first.is_empty() {
        return searched_states(topic, base, (lessons, sample, RULE_LESSONS), way, traced);
    }
    let (taught, taught_base) = with_learned(topic, base, &first);
    let score = SeedGround::of_answers(&taught, sample, way).reading(&taught, &taught_base, &[]).map_or(usize::default(), |reading| reading.score());
    let search = traced.then(|| RuleSearch { way: way_name(topic, way), target: way_target(topic, lessons, way), answer_targets: answer_targets(topic, lessons, way), states: Vec::new(), candidates: Vec::new(), dropped: Vec::new(), steps: Vec::new(), chosen: first.clone(), score, sample: sample.len(), kept: false, tried_states: Vec::new(), state_counts: None, splits: Vec::new(), stored: None });
    (first, score, search)
}
because!(way_rules, LearnedWording, "the rules of a concept learned for facts named in one way (with_states); for a reply line of the text read for a question of another form, the rules of the rounds of the facts that the answers need (abduced_rounds), judged on the sample as they stand: the asked fact itself is read from their facts by the text's own form (aligned_form), so no rule that makes it is searched (that search found none and cost three times the work of the rounds, with invented states that never read more of the sample); the plain search when the rounds give no rule");
because!(learn_rules_traced, LearnedWording, "rules of a concept learned from the lessons of a kind, when the text holds no rule that makes the facts that its questions ask, on facts that keep their history, as a learned rule may read a fact at the time of a clause before it, with each search kept for the report of the discovery when the caller asks, and how many lessons of the sample the rules make right, of how many: with the facts named by the reply lines of the text when it has them; else, for a kind of a few labels that its stories do not tell, the question as a statement for the lessons of each label in turn, kept for the label whose rules answer the sample best; else the question with its answer as the name of the fact; in both cases, for a text with no rules of its own, the facts that a reply line of the text names for a question of another form, in the order that the lessons tell, are tried first, so that at the same score the name of the reply line wins, as it is the text's own name of the asked fact");

fn with_states(topic: &Topic, base: &Arc<Facts>, (lessons, sample, reach): (&[&Case], &[&Case], usize), way: &AnswerFact, traced: bool) -> (Vec<String>, usize, Option<RuleSearch>) {
    if let Some((kept, right, offered)) = stored_first(topic, base, sample, way) {
        let search = traced.then(|| stored_search(topic, (lessons, way), (&kept, right, offered), sample.len()));
        return (kept, right, search);
    }
    searched_states(topic, base, (lessons, sample, reach), way, traced)
}

fn searched_states(topic: &Topic, base: &Arc<Facts>, (lessons, sample, reach): (&[&Case], &[&Case], usize), way: &AnswerFact, traced: bool) -> (Vec<String>, usize, Option<RuleSearch>) {
    let mut best = learned_rules_by(topic, base, (lessons, sample, reach), way, traced);
    if best.1 >= sample.len() {
        return best;
    }
    let (states, counts) = state_candidates(topic, base, &lessons[..lessons.len().min(reach)], &story_patterns(topic));
    let mut tried: Vec<(Vec<String>, usize, Vec<SearchStep>)> = Vec::new();
    let mut best_state: Vec<String> = Vec::new();
    for state in states {
        if scope_spent() {
            break;
        }
        let lines = state_lines(&state);
        let (with_state, state_base) = with_learned(topic, base, &lines);
        let (rules, score, search) = learned_rules_by(&with_state, &state_base, (lessons, sample, reach), way, traced);
        if traced {
            tried.push((lines.clone(), score, search.as_ref().map(|search| search.steps.clone()).unwrap_or_default()));
        }
        if score > best.1 {
            best_state = lines.clone();
            best = ([lines.clone(), rules].concat(), score, search.map(|search| RuleSearch { states: lines, ..search }));
        }
        if best.1 >= sample.len() {
            break;
        }
    }
    if best.1 < sample.len()
        && !scope_spent()
        && let Some(persons) = person_state(&lessons[..lessons.len().min(reach)], (true, true))
    {
        let lines: Vec<String> = best_state.iter().cloned().chain(state_lines(&persons)).collect();
        let (with_state, state_base) = with_learned(topic, base, &lines);
        let (rules, score, search) = learned_rules_by(&with_state, &state_base, (lessons, sample, reach), way, traced);
        if traced {
            tried.push((lines.clone(), score, search.as_ref().map(|search| search.steps.clone()).unwrap_or_default()));
        }
        if score > best.1 {
            best = ([lines.clone(), rules].concat(), score, search.map(|search| RuleSearch { states: lines, ..search }));
        }
    }
    let (lines, score, search) = best;
    (lines, score, search.map(|search| RuleSearch { tried_states: tried, state_counts: Some(counts), ..search }))
}
because!(searched_states, LearnedWording, "the rule search of with_states after the store: a caller that judged the store already runs it alone, as the store would give the same again for nothing");
because!(with_states, LearnedWording, "rules of a concept learned for facts named in one way (the stored lines first, then searched_states), with how many lessons of the sample they answer right, and the search that found them when the caller asks; when they leave some wrong, the rules are learned again over each state that the stories suggest (who holds a thing from the time that it was got until it was dropped), and a state is kept with the rules over it when they answer more of the sample, the search ending at a state whose rules answer all of it, its trace with every state tried and its score and the counts of what the search for states read: a fact that the answers rest on may hold between two events and be named by no sentence of the text, so the learner invents it");

fn fact_lines(topic: &Topic, base: &Arc<Facts>, (lessons, sample): (&[&Case], &[&Case]), way: &AnswerFact, found: &[String]) -> Vec<String> {
    let (learned, learned_base) = with_learned(topic, base, found);
    let ground = SeedGround::of_answers(&learned, sample, way);
    let Some(before) = ground.reading(&learned, &learned_base, &[]) else { return Vec::new() };
    let candidates = fact_candidates(&learned, &learned_base, &lessons[..lessons.len().min(RULE_LESSONS)]);
    let tally = |lines: &[String]| -> Option<(usize, usize, usize)> {
        let (with_fact, fact_base) = with_learned(&learned, &learned_base, lines);
        let reading = ground.reading(&with_fact, &fact_base, &[])?;
        Some(judged_tally(&reading.rights, &reading.answered))
    };
    let now = before.score();
    let alone: Vec<Option<(usize, usize, usize)>> = on_lanes(candidates.len(), hardware_lanes(), &|at| tally(std::slice::from_ref(&candidates[at].0)));
    let mut gaining: Vec<usize> = (0..candidates.len()).filter(|at| alone[*at].is_some_and(|(right, wrong, share)| right > now && wrong <= before.share_wrongs(share))).collect();
    gaining.sort_by(|one, other| alone[*other].map(|judged| judged.0).cmp(&alone[*one].map(|judged| judged.0)).then_with(|| candidates[*other].2.cmp(&candidates[*one].2)).then_with(|| one.cmp(other)));
    let mut chosen: Vec<String> = Vec::new();
    let mut taken: Vec<&Vec<Vec<WordId>>> = Vec::new();
    let mut score = now;
    for at in gaining {
        if taken.contains(&&candidates[at].1) {
            continue;
        }
        let mut lines = chosen.clone();
        lines.push(candidates[at].0.clone());
        let Some((right, wrong, share)) = tally(&lines) else { continue };
        if right > score && wrong <= before.share_wrongs(share) {
            score = right;
            chosen = lines;
            taken.push(&candidates[at].1);
        }
    }
    chosen
}
because!(fact_lines, LearnedWording, "facts of knowledge learned on top of the lines found so far: a fact is a rule with no condition, and the candidates are the facts that a rule of the text needs and that no story nor known fact holds (a name that is a female name), filled with the things that the stories give its other conditions; each is judged alone, and those that make more lessons right with no wrong answer are added, the best first, while each still adds right lessons with no wrong answer, and of facts about the same things (a name that is male and a name that is female) only the first that pays off");

fn fact_candidates(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case]) -> Vec<(String, Vec<Vec<WordId>>, usize)> {
    suggested_facts(topic, base, lessons, (false, false, false))
}

fn suggested_facts(topic: &Topic, base: &Arc<Facts>, lessons: &[&Case], (asked, paired, answered): (bool, bool, bool)) -> Vec<(String, Vec<Vec<WordId>>, usize)> {
    let answers: HashSet<Vec<WordId>> = if answered { lessons.iter().map(|case| fact_numbers(&case.answer)).collect() } else { HashSet::new() };
    let first = base.added_to(&Facts::default());
    let stories: Vec<(Vec<WordId>, Facts)> = lessons.iter().filter_map(|case| Some((fact_numbers(&case.question), bounded_facts(case, base, topic, false)?))).collect();
    let fits_at = |pattern: &Pattern, facts: &Facts, place: usize| {
        let mut fits = Vec::new();
        fit_into(pattern, facts.fact(place), &empty_filling(), &mut fits);
        !fits.is_empty()
    };
    let holds = |pattern: &Pattern| stories.iter().any(|(_, facts)| (usize::default()..first + facts.added_to(base)).any(|place| fits_at(pattern, facts, place)));
    let told_holds = |pattern: &Pattern| stories.iter().any(|(_, facts)| (first..first + facts.added_to(base)).any(|place| fits_at(pattern, facts, place)));
    let joined = |(question, join): (Option<&Pattern>, &[&Pattern]), (asked_words, facts): (&[WordId], &Facts)| -> Vec<Filling> {
        let mut out = vec![empty_filling()];
        if let Some(question) = question {
            out = Vec::new();
            fit_into(question, asked_words, &empty_filling(), &mut out);
        }
        for pattern in join {
            let mut next: Vec<Filling> = Vec::new();
            for filling in &out {
                for place in first..first + facts.added_to(base) {
                    fit_into(pattern, facts.fact(place), filling, &mut next);
                }
            }
            out = next;
        }
        out
    };
    let forms = topic.questions.iter().filter(|_| asked).flat_map(|form| form.replies.iter().map(move |reply| (Some(&form.question), &reply.conditions)));
    let mut counted: HashMap<Vec<WordId>, (Vec<Vec<WordId>>, usize, usize)> = HashMap::new();
    for (question, conditions) in topic.rules.iter().map(|rule| (None, &rule.conditions)).chain(forms) {
        let asked_places = question.map(places_of).unwrap_or_default();
        let conditions: Vec<&Pattern> = conditions.iter().filter_map(|condition| match condition {
            Condition::Fact { pattern, .. } => Some(pattern),
            _ => None,
        }).collect();
        for (at, missing) in conditions.iter().enumerate() {
            let places = places_of(missing);
            let known_only = (asked || answered) && !told_holds(missing);
            if places.is_empty() || (holds(missing) && !known_only) {
                continue;
            }
            let others: Vec<&Pattern> = conditions.iter().enumerate().filter(|(place, _)| *place != at).map(|(_, other)| *other).collect();
            let covers = |join: &[&Pattern]| places.iter().all(|wanted| asked_places.contains(wanted) || join.iter().any(|other| places_of(other).contains(wanted)));
            let mut joins: Vec<Vec<&Pattern>> = if question.is_some() && covers(&[]) { vec![Vec::new()] } else { Vec::new() };
            joins.extend(others.iter().filter(|other| covers(std::slice::from_ref(*other))).map(|other| vec![*other]));
            for (one, first_other) in others.iter().enumerate().filter(|_| paired) {
                for second_other in &others[one + 1..] {
                    let shared = places_of(first_other).iter().any(|place| places_of(second_other).contains(place));
                    if shared && !covers(std::slice::from_ref(first_other)) && !covers(std::slice::from_ref(second_other)) && covers(&[*first_other, *second_other]) {
                        joins.push(vec![*first_other, *second_other]);
                    }
                }
            }
            for join in &joins {
                for (lesson, (asked_words, facts)) in stories.iter().enumerate() {
                    for filling in joined((question, join), (asked_words, facts)) {
                        let Some(words) = fill(missing, &filling) else { continue };
                        if known_only && facts.holds(&words) {
                            continue;
                        }
                        let values: Vec<Vec<WordId>> = places.iter().filter_map(|wanted| filling.get(*wanted).map(<[WordId]>::to_vec)).collect();
                        if answered && !values.iter().any(|value| answers.contains(value)) {
                            continue;
                        }
                        let entry = counted.entry(words).or_insert((values, usize::default(), usize::MAX));
                        if entry.2 != lesson {
                            entry.1 += 1;
                            entry.2 = lesson;
                        }
                    }
                }
            }
        }
    }
    let mut out: Vec<(String, Vec<Vec<WordId>>, usize)> = counted.into_iter().filter(|(_, (_, count, _))| *count >= LEAST_SUPPORT).map(|(words, (values, count, _))| (format!("{}{STATEMENT_END}", library_writing(&words.into_iter().map(LogicPiece::Word).collect::<Pattern>())), values, count)).collect();
    out.sort_by(|one, other| other.2.cmp(&one.2).then_with(|| one.0.cmp(&other.0)));
    out.truncate(MOST_FACT_CANDIDATES);
    out
}
because!(suggested_facts, LearnedWording, "the facts of knowledge that the stories suggest for the rules of a text (and its question forms when asked), filled from one other condition of a line, or from two that share a place when asked (paired): E is before G from A was in the B, E and A was in the B, G in one story; for a question form, from the places that its question gives too (A is a German determiner, for quest: der Esel kann ...), alone or with the other conditions; when asked, a condition that only known facts fit (no story fact) is missing for each filling whose fact is not known, as a text may know some facts of a kind (gold is a colour) and lack others (green is a colour), while a condition that story facts fit gives no candidate; when asked for the answers (answered), the same for each filling of a condition that only known facts fit, and only the facts that hold an answer of the lessons as a value (white is a colour, for what colour is Bernhard: white), as a kind that nothing reads lacks the facts that its answers rest on");
because!(fact_candidates, LearnedWording, "the facts of knowledge that the stories suggest: for each condition of a rule of the text that no fact of the stories nor any known fact fits (the gender of a name once the list of names is gone), filled with the things that a fact of a story gives the other conditions of the same rule that hold all its places, each fact counted once for each lesson that suggests it, with the things that fill it, the most suggested first");

const MOST_FACT_CANDIDATES: usize = 128;
because!(MOST_FACT_CANDIDATES, LearnedWording, "the facts of knowledge suggested by the most lessons that the search judges: the names of a kind of stories are a few dozen, each with a fact for each of a few kinds (a male name, a female name)");

fn state_lines(state: &StateCandidate) -> Vec<String> {
    state.starts.iter().chain(&state.ends).map(|(event, made, narrow)| format!("{TOLD_IF}{}{TOLD_THEN}{}{STATEMENT_END}", library_writing(&narrowed(event, narrow)), library_writing(&narrowed(made, narrow)))).collect()
}
because!(state_lines, LearnedWording, "an invented state as rules of the library: each event that starts it makes the state, and each event that ends it makes the state with no value, with each place that every lesson gave one word written as a place of one word (narrowed), in the event and in the state alike, so that a sentence with a word after its thing (Lucy took the kettle there) starts the state of the thing alone");

const MOST_RULE_LABELS: usize = 4;
because!(MOST_RULE_LABELS, LearnedWording, "the most different answers of a kind whose answers are labels (entailment, neutral, contradiction; yes, no, maybe): each label is tried as the one whose lessons make the question true");

fn way_name(topic: &Topic, way: &AnswerFact) -> String {
    match way {
        AnswerFact::Reply => WAY_REPLY.to_string(),
        AnswerFact::Label(label) => format!("{WAY_LABEL}{label}"),
        AnswerFact::Spoken => WAY_SPOKEN.to_string(),
        AnswerFact::Formed(at) => format!("{WAY_FORMED}\"{}\"", topic.questions.get(*at).map(|form| library_writing(&form.question)).unwrap_or_default()),
        AnswerFact::Aligned(at, order) => format!("{WAY_ALIGNED}\"{}\", its places in the order {}", topic.reply_lines.get(*at).map(|reply| reply.sentence.clone()).unwrap_or_default(), order.iter().map(|place| place_letter(*place)).collect::<Vec<String>>().join(FORM_WORD_GAP)),
    }
}
because!(way_name, LearnedWording, "the words of the report for a way of naming the asked fact: a reply line, a label, the question with its answer, or a reply line read for a question of another form with the order of its places");

fn way_target(topic: &Topic, lessons: &[&Case], way: &AnswerFact) -> String {
    let words = question_words(lessons);
    lessons.iter().find_map(|case| answer_fact(topic, case, (way, &words))).map(|(words, _, _, _)| word_texts(&words).join(FORM_WORD_GAP)).unwrap_or_default()
}
because!(way_target, LearnedWording, "the asked fact of the first lesson that a way names, as the report shows it (nicholas is the son of ashley)");

fn answer_targets(topic: &Topic, lessons: &[&Case], way: &AnswerFact) -> Vec<(String, String)> {
    let words = question_words(lessons);
    let mut lengths: Vec<usize> = Vec::new();
    let mut out: Vec<(String, String)> = Vec::new();
    for case in lessons {
        let length = fact_words(&case.answer).len();
        if lengths.contains(&length) {
            continue;
        }
        lengths.push(length);
        if lengths.len() > 1 {
            let target = answer_fact(topic, case, (way, &words)).map_or_else(|| NO_ASKED_FACT.to_string(), |(target, _, _, _)| word_texts(&target).join(FORM_WORD_GAP));
            out.push((case.answer.clone(), target));
        }
        if out.len() >= SHOWN_CANDIDATES {
            break;
        }
    }
    out
}
const NO_ASKED_FACT: &str = "none: this way names no asked fact for the answer";
because!(NO_ASKED_FACT, LearnedWording, "what the report says for a lesson whose answer the way of naming the asked fact cannot parse (a reply line of one thing, for an answer of two things)");

because!(answer_targets, LearnedWording, "the asked fact of the first lesson of each length of answer after the length of the first lesson, with its answer, or that the way names none for it, a few at most, as the report shows them: an answer of several words (the cellar no more, the attic or the porch) is parsed into the asked fact in a way that the first lesson does not show");

fn learned_rules_by(topic: &Topic, base: &Arc<Facts>, (lessons, sample, reach): (&[&Case], &[&Case], usize), way: &AnswerFact, traced: bool) -> (Vec<String>, usize, Option<RuleSearch>) {
    let (mut candidates, dropped, own_forms) = rule_candidates(topic, base, &lessons[..lessons.len().min(reach)], way);
    let rules = candidates.len();
    candidates.extend(write_candidates(topic, &lessons[..lessons.len().min(reach)]));
    let choices: Vec<Vec<ToldChoice>> = candidates.iter().map(|(line, narrow, _)| vec![ToldChoice { line: line.clone(), narrow: narrow.clone(), fact: usize::default(), among_fact: usize::default(), promising: true, shared: usize::default(), in_order: true }]).collect();
    let uses: Vec<usize> = candidates.iter().map(|(_, _, count)| *count).collect();
    let written_uses: Vec<usize> = uses.iter().enumerate().map(|(at, count)| if at < rules { usize::default() } else { *count }).collect();
    let mut steps: Vec<SearchStep> = Vec::new();
    let ground = SeedGround::of_answers(topic, sample, way);
    let written = seeded_slots(topic, base, sample, (&choices, 1, &written_uses), (hardware_lanes(), Vec::new(), Some(way), traced.then_some(&mut steps)));
    let written_lines = written.concat();
    let written_score = ground.reading(topic, base, &written_lines).map_or(usize::default(), |reading| reading.score());
    let facts = if written_score >= sample.len() || scope_spent() { Vec::new() } else { fact_lines(topic, base, (lessons, sample), way, &written_lines) };
    let (fact_topic, fact_base) = with_learned(topic, base, &facts);
    let fact_score = if facts.is_empty() { written_score } else { ground.reading(&fact_topic, &fact_base, &written_lines).map_or(usize::default(), |reading| reading.score()) };
    let searched = fact_score < sample.len() && (rules > usize::default() || !facts.is_empty());
    let (lines, splits) = if searched {
        let slots = seeded_slots(&fact_topic, &fact_base, sample, (&choices, 1, &uses), (hardware_lanes(), written, Some(way), traced.then_some(&mut steps)));
        let slots = if own_forms { second_round(&fact_topic, &fact_base, (&lessons[..lessons.len().min(reach)], sample), (&choices, &uses, way), (slots, traced.then_some(&mut steps))) } else { slots };
        if own_forms { split_through_kept(&fact_topic, &fact_base, &ground, slots.concat()) } else { (slots.concat(), Vec::new()) }
    } else {
        (written_lines, Vec::new())
    };
    let score = if searched { ground.reading(&fact_topic, &fact_base, &lines).map_or(usize::default(), |reading| reading.score()) } else { fact_score };
    let lines = [facts, lines].concat();
    let search = traced.then(|| RuleSearch { way: way_name(topic, way), target: way_target(topic, lessons, way), answer_targets: answer_targets(topic, lessons, way), states: Vec::new(), candidates: candidates.iter().map(|(line, _, count)| (line.clone(), *count)).collect(), dropped, steps, chosen: lines.clone(), score, sample: sample.len(), kept: false, tried_states: Vec::new(), state_counts: None, splits, stored: None });
    (lines, score, search)
}
because!(learned_rules_by, LearnedWording, "rules of a concept learned from the lessons of a kind for facts named in one way, with how many lessons of the sample they answer right: the lines that write a told sentence again are searched alone first, then the facts of knowledge that the rules of the text miss (fact_lines), and the rules only when those leave some lesson of the sample wrong and the stories suggest some rule, from the lines and facts found (with no rule suggested and no fact found, a search of all the candidates again would judge the same lines that write a sentence again on the same text), as a line or a fact that lets the rules of the text read the stories invents no rule (a search of new rules first takes rules that are right on a few stories by chance, which the facts then cannot mend), as such a line invents no fact but lets the rules of the text read a sentence that a word of sequence opens (then, after that), and a search over a few lines costs a small part of a search over all the rules that the stories suggest; the rules that the stories suggest for the facts of their answers, added one at a time (pairs at the first step) while the best single filled answers of those facts make more lessons of a sample right, as the told side is first read, and only a rule that adds no wrong answer in the reading of the questions where it makes its lessons right, as FOIL keeps a rule only when it covers no negative example: a broad rule that is right on most stories (the thing goes where its last taker goes) would block the precise rules that pay off together (where the taker was then, where the holder went while holding it); when the rules came from the forms of the told sentences, a second round on the lessons that they still answer wrong (second_round), and each rule chosen split through a sentence of the text (split_through_kept), with each split for the report");

pub fn learn_curriculum(name: &str, kinds: &[(&[&Case], &[&Case])]) -> Result<Vec<Worked<WordingReport>>, String> {
    let library = library()?;
    let place = library.topics.iter().position(|topic| topic.name == name).ok_or_else(|| format!("{NO_SUCH_TEXT} {name}"))?;
    let mut topic = library.topics[place].clone();
    let mut base = Arc::clone(library.base(place));
    let mut out: Vec<Worked<WordingReport>> = Vec::new();
    for (fit, check) in kinds {
        let taught = in_work_scope(MOST_LEARNING_WORK, || rules_and_readings(&topic, &base, fit, (None, None, None)));
        let KindLines { lines, rule_seconds, reading_seconds, .. } = taught.value;
        let seconds = (rule_seconds, reading_seconds);
        let (learned, learned_base) = with_learned(&topic, &base, &lines);
        let (kept, words) = learn_forms(&learned, &learned_base, fit);
        let (learned_right, written_right) = checked_right(&learned, &learned_base, &kept, &words, check);
        let below_bar = told_forms_below_bar(&fit[..fit.len().min(READ_TOLD_LESSONS).min(RULE_LESSONS)]).into_iter().map(|(form, count, of)| (library_writing(&form), count, of)).collect();
        out.push(Worked { value: WordingReport { told: lines, forms: kept, checked: check.len(), learned_right, written_right, seconds, below_bar }, work: taught.work, stopped: taught.stopped, taken: taught.taken });
        topic = learned;
        base = learned_base;
    }
    Ok(out)
}
because!(learn_curriculum, LearnedWording, "the rules of a concept learned kind after kind, each kind on top of the rules that the kinds before it learned, as a lesson book teaches a step only after the steps that it needs: a kind whose answer fact rests on facts that only an earlier kind's rules make (where a thing was before a place rests on where it is) finds them made; for each kind its rules and readings (rules_and_readings), sought under the bound of work of one learning as the board seeks them, with that work, and its wording, judged on its last lessons, and the told sentences that too few of the lessons of its search for states hold for a state to read them (told_forms_below_bar)");

pub(super) struct KindLines {
    pub(super) lines: Vec<String>,
    pub(super) readings: Vec<String>,
    pub(super) dropped: Vec<String>,
    pub(super) rule_seconds: f64,
    pub(super) reading_seconds: f64,
    pub(super) forms_not_read: Option<usize>,
}
because!(KindLines, LearnedWording, "what a kind teaches a text: the lines (the readings kept, then the rules), the readings kept, the readings learned and dropped because the rules over them answered no more of the sample than the rules alone, and the seconds of the searches for rules and of the search for readings apart, for the report, and, when no readings were sought as the board's told side would not read the kind with the text (told_side_refuses), how many sentence forms the stories tell");

fn told_side_refuses(topic: &Topic, base: &Arc<Facts>, fit: &[&Case]) -> Option<usize> {
    if topic.rules.is_empty() && topic.questions.is_empty() && topic.reply_lines.is_empty() && topic.facts.is_empty() {
        return None;
    }
    let learned_on: Vec<&Case> = fit.iter().take(SEARCH_LESSONS).copied().collect();
    let forms = told_form_count(&learned_on);
    if !(1..=MOST_TOLD_FORMS).contains(&forms) || topic.rules.is_empty() || !topic.questions.is_empty() {
        return Some(forms);
    }
    let sampled: Vec<&Case> = fit.iter().take(RELEVANCE_SAMPLE).copied().collect();
    let filled = answers_told(&learned_on) * WHOLE_SHARE >= learned_on.len() * TOLD_FILLED_PERCENT;
    let relevant = told_relevance(topic, if filled { &learned_on } else { &sampled }) >= TOLD_RELEVANT_PERCENT;
    (!relevant || trial(topic, base, (fit, &sample_words(fit)), LEAST_FIRED_PERCENT).is_some()).then_some(forms)
}
because!(told_side_refuses, LearnedWording, "whether the board's own told side would read the told sentences of a kind with a text, and if not, how many sentence forms its first lessons tell: as the board learns readings only for a text of rules and no question forms whose rules speak of the told sentences but do not yet read the stories, and only for a kind told in a few dozen forms, the report of a discovery and a curriculum seek readings only there, so that they learn each kind as the board does; a text with no lines at all (the step that learns a new concept from the lessons alone) is not refused, as the board's own step seeks readings there");

pub(super) fn rules_and_readings(topic: &Topic, base: &Arc<Facts>, fit: &[&Case], (mut searches, steps, joins): (Option<&mut Vec<RuleSearch>>, Option<&mut Vec<SearchStep>>, Option<&mut Vec<FormJoin>>)) -> KindLines {
    let started = Instant::now();
    let (rules, score, sample) = learn_rules_traced(topic, base, fit, searches.as_mut().map(|kept| &mut **kept));
    let mut taught = KindLines { lines: rules, readings: Vec::new(), dropped: Vec::new(), rule_seconds: started.elapsed().as_secs_f64(), reading_seconds: f64::default(), forms_not_read: None };
    if score + LEAST_SUPPORT > sample {
        return taught;
    }
    if let Some(forms) = told_side_refuses(topic, base, fit) {
        taught.forms_not_read = Some(forms);
        return taught;
    }
    let started = Instant::now();
    let readings = learn_told_traced(topic, base, fit, (steps, joins));
    taught.reading_seconds = started.elapsed().as_secs_f64();
    if readings.is_empty() {
        return taught;
    }
    let started = Instant::now();
    let (read, read_base) = with_learned(topic, base, &readings);
    let (more, read_score, _) = learn_rules_traced(&read, &read_base, fit, searches);
    taught.rule_seconds += started.elapsed().as_secs_f64();
    if read_score <= score {
        taught.dropped = readings;
        return taught;
    }
    taught.lines = readings.iter().cloned().chain(more).collect();
    taught.readings = readings;
    taught
}
because!(rules_and_readings, LearnedWording, "the lines that a kind teaches a text: the rules of its concept (lines that write a sentence again, facts, rules, states), and only when they leave some lesson of the sample wrong, the readings of its told sentences into the facts that the rules of the text start from (moved to as went to), with the rules learned again on top of them, kept when they answer more of the sample than the rules alone, and not sought when fewer lessons are left wrong than a reading needs to be suggested by (ProofWriter's rules left two of the sample wrong, and the search for readings on its long stories took minutes for them), as a reading is wanted where no rule of the text reads a sentence, while a search for readings on a kind that its rules already read takes readings that are right by chance (babi03 over the rules that babi01 taught lost lessons of its own to three such readings); with the readings apart, for the report");

pub fn store_discovery(name: &str, fit: &[&Case]) -> Result<Worked<Vec<String>>, String> {
    let library = library()?;
    let place = library.topics.iter().position(|topic| topic.name == name).ok_or_else(|| format!("{NO_SUCH_TEXT} {name}"))?;
    let base = Arc::clone(library.base(place));
    Ok(in_work_scope_offering(MOST_LEARNING_WORK, false, || rules_and_readings(&library.topics[place], &base, fit, (None, None, None)).lines))
}
because!(store_discovery, LearnedWording, "the lines that one kind's lessons teach a text, discovered for the known-good store as the discovery report finds them (rules_and_readings under the bound of the work of one learning), with the store of the text set aside, so that the store holds only what discovery found on the kind's own lessons");

pub fn learn_rule_wording(name: &str, fit: &[&Case], check: &[&Case]) -> Result<WordingReport, String> {
    let library = library()?;
    let place = library.topics.iter().position(|topic| topic.name == name).ok_or_else(|| format!("{NO_SUCH_TEXT} {name}"))?;
    let rules = learn_rules(&library.topics[place], library.base(place), fit);
    let (learned, learned_base) = with_learned(&library.topics[place], library.base(place), &rules);
    let (kept, words) = learn_forms(&learned, &learned_base, fit);
    let (learned_right, written_right) = checked_right(&learned, &learned_base, &kept, &words, check);
    Ok(WordingReport { told: rules, forms: kept, checked: check.len(), learned_right, written_right, seconds: (f64::default(), f64::default()), below_bar: Vec::new() })
}
because!(learn_rule_wording, LearnedWording, "the wording of a kind learned for one named text with the rules of its concept learned first from the lessons, with the share of the learned forms on the last lessons");

const SEED_TOLD_PERCENT: usize = 50;
because!(SEED_TOLD_PERCENT, LearnedWording, "the least share of the lessons of a sample whose answer their story tells, for a first reading by filled answers: an answer that the story names (a book, a partner) can be filled from a fact, while a label (true, neutral) cannot, and a first reading would only cost time; the board asks for a far larger share before it tries the told side with a text whose rules speak of only some of the told sentences");

const MOST_MARKED: usize = 8;
because!(MOST_MARKED, LearnedWording, "the rules of a step of a traced search, the most right first, whose answers on each lesson of the sample are kept for the report, judged again with no work counted: as many as a report shows beside the rules added, so that it can tell right and wrong by the source of each lesson");

pub(super) fn case_place(case: &Case) -> usize {
    std::ptr::from_ref::<Case>(case) as usize
}
because!(case_place, LearnedWording, "the place of a case in memory, which names it for the run: a sample holds the same cases as the lessons of its kind, so the report finds each sample lesson's source by it");

const SEED_SCREEN_STRIDE: usize = 2;
because!(SEED_SCREEN_STRIDE, LearnedWording, "the lessons of the sample of a first reading on which every pair of ways is judged first, beside every lesson still wrong once few are (where a pair gains, if it gains at all; while most are wrong, every second lesson holds many of them): every second one of the others, spread over all the lessons as the sample is, where a pair can lose what the reading already makes right, so that the many pairs cost fewer chains and only the best of them are judged on the whole sample");

const SEED_PAIR_FINALISTS: usize = 16;
because!(SEED_PAIR_FINALISTS, LearnedWording, "the pairs of ways with the most right lessons on part of the sample that are judged again on the whole sample, where the pair is chosen: a pair that pays off makes many lessons right on any spread part, and a few more than the pairs that a step adds keep a near tie from being lost to the chance of the part");

const SEED_BOUND_FIRST: usize = 32;
because!(SEED_BOUND_FIRST, LearnedWording, "the single ways to read that a step of a first reading after the first judges first, those that made the most lessons right at the step before: the best of them sets the score that another way must be able to reach to be judged at all, as a way that touches only lessons that are right already, or few that are wrong, cannot reach the best of a step; a few dozen come near the best way of a step in the searches of the board, all the others are then judged in one run of the threads, and their number does not change with the threads, so neither does the work of a search");

const SEED_PAIRED: usize = 8;
because!(SEED_PAIRED, LearnedWording, "the best ways to read of a step of a first reading that are also judged together with each other way to read another form: the start of a story and its steps may each make few answers right alone and many together");

const SEED_PAIR_PARTNERS: usize = 128;
because!(SEED_PAIR_PARTNERS, LearnedWording, "the most ways to read other forms that each of the best ways of a step is paired with, those of the most told forms first: the ways to read the forms of a kind told in a few settings are some dozens (who swaps with whom, who dances with whom), while the many forms of a story of pronouns and moves read into the facts of a text in hundreds of ways, and pairing each best way with all of them costs thousands of judges for a pair that the first few hundred hold if any does");

const SEED_PARTNER_FORMS: usize = 12;
because!(SEED_PARTNER_FORMS, LearnedWording, "the sentence forms with the most uses in the sample whose ways to read are judged together with the best ways of a step of a first reading: the steps of a story come in every story, so their forms are the most told");

pub(super) const SEED_SAMPLE: usize = 40;
because!(SEED_SAMPLE, LearnedWording, "the lessons on which a first reading is judged: a few stories show which readings make answers right");

pub(super) fn spread_sample<T: Copy>(items: &[T], most: usize) -> Vec<T> {
    let taken = items.len().min(most);
    (0..taken).map(|at| items[at * items.len() / taken]).collect()
}
because!(spread_sample, LearnedWording, "at most the given number of items, spread evenly over all of them (the item at each equal share of the length), and all of them when they are fewer: a stride of the length divided by that number, rounded down, reaches only the items up to its last whole stride, so the sample of a list of fewer than twice the number would hold its first items only and leave its end out");

fn companion_conditions(forms: &[(Pattern, Vec<(usize, Vec<Vec<String>>)>)], at: usize) -> Vec<String> {
    let (form, uses) = &forms[at];
    let places = places_of(form).len();
    let mut out: Vec<String> = Vec::new();
    if own_places(form).len() < places {
        return out;
    }
    for (other_at, (other, other_uses)) in forms.iter().enumerate() {
        if other_at == at || own_places(other).len() < places_of(other).len() {
            continue;
        }
        let together: Vec<(&Vec<Vec<String>>, &Vec<Vec<String>>)> = uses.iter().filter_map(|(lesson, runs)| other_uses.iter().find(|(other_lesson, _)| other_lesson == lesson).map(|(_, other_runs)| (runs, other_runs))).collect();
        if together.len() < LEAST_SUPPORT {
            continue;
        }
        let other_places = places_of(other);
        let mut fresh = places;
        let to: Vec<usize> = other_places
            .iter()
            .map(|place| {
                let same = (0..places).find(|mine| together.iter().filter(|(runs, other_runs)| runs[*mine] == other_runs[*place]).count() * WHOLE_SHARE >= together.len() * RELEVANT_PERCENT);
                same.unwrap_or_else(|| {
                    fresh += 1;
                    fresh - 1
                })
            })
            .collect();
        if to.iter().any(|place| *place < places) && fresh < MARKED_FIRST_PLACE {
            out.push(library_writing(&renamed(other, &other_places, &to)));
        }
    }
    out
}
because!(companion_conditions, LearnedWording, "the other sentence forms that a few lessons of a form tell beside it (a sentence that someone came, or that someone did not come), each with its places given to the places of the form whose words it repeats in most of those lessons, and new places for the rest, as a condition that a learned rule may need besides its sentence: a sentence that says what happens if someone comes tells a fact only together with the sentence that says that the person came");

const MOST_TARGET_WAYS: usize = 720;
because!(MOST_TARGET_WAYS, LearnedWording, "the most ways to read one sentence form into one fact, those that keep the order of the sentence first: a long form and a fact of many places give thousands of ways, each judged on the lessons of the form, which fills the memory and the time of a run while nearly all of them are wrong");

const JOINED_WAYS: usize = 8;
because!(JOINED_WAYS, LearnedWording, "the most promising ways to read a form that are tried again with each companion sentence as a second condition");

const MARKED_FIRST_PLACE: usize = PLAIN_PLACES;
because!(MARKED_FIRST_PLACE, LearnedWording, "a learned rule names at most the open places of one or more words, the letters from A to P");

struct ToldChoice {
    line: String,
    narrow: Option<String>,
    fact: usize,
    among_fact: usize,
    promising: bool,
    shared: usize,
    in_order: bool,
}
because!(ToldChoice, LearnedWording, "a way to read a sentence form that the whole wording may judge: the learned rule, the same rule with the places that every lesson gave one word made places of one word (judged only when the rule adds a wrong answer, as the narrow rule makes only facts that the rule makes), the fact that it reads into, its place among the ways to read into that fact by promise, whether it is among the most promising of all, how many words of the frame of the fact the sentence form holds too, as a reading that keeps the words of its sentence says what the sentence says (moved to the kitchen as went to the kitchen, not as the kitchen got the person), and whether it gives the places of the fact in the order in which the sentence holds them (the person who went, then the place, not the place first)");

struct JudgedReading {
    base: Arc<Facts>,
    rules: Vec<ToldRule>,
    facts: Vec<Option<Arc<Facts>>>,
}
because!(JudgedReading, LearnedWording, "a reading of the told sentences that the whole wording judged: the facts of the text before any story, its rules with those of the text, and the facts of each lesson of both halves under them; the words of each lesson are in the ground of the judge, the same under every reading");

const JOINT_HALVES: usize = 2;
because!(JOINT_HALVES, LearnedWording, "the halves of the lessons of the joint judge of the readings: the forms are learned from each and checked on the other");

pub(super) struct JointGround {
    said: Arc<Vec<SaidCase>>,
    every_words: QuestionWords,
    halves: [(FormsGround, FormsGround); JOINT_HALVES],
}
because!(JointGround, LearnedWording, "what the joint judge of the readings rests on whatever the rules judged: the words of every lesson of both halves, the words of all their questions, and for each half the ground of the forms learned from it with the ground of the other half checked under its words");

pub(super) fn joint_ground((learn_part, judge_part): (&[&Case], &[&Case])) -> JointGround {
    let every: Vec<&Case> = learn_part.iter().chain(judge_part).copied().collect();
    let said: Vec<SaidCase> = every.iter().map(|case| said_case(case)).collect();
    let (learn_said, judge_said) = said.split_at(learn_part.len());
    let learn_ground = forms_ground(learn_part, learn_said);
    let judge_ground = forms_ground(judge_part, judge_said);
    let learn_checked = checked_ground(judge_said, &learn_ground.words);
    let judge_checked = checked_ground(learn_said, &judge_ground.words);
    JointGround { every_words: question_words(&every), halves: [(learn_ground, learn_checked), (judge_ground, judge_checked)], said: Arc::new(said) }
}
because!(joint_ground, LearnedWording, "the ground of the joint judge of the readings of two halves of the lessons, found once for every reading that it judges");

fn joint_right(topic: &Topic, base: &Arc<Facts>, lines: &[String], (learn_part, judge_part, ground): (&[&Case], &[&Case], &JointGround), before: Option<&JudgedReading>) -> (usize, Option<JudgedReading>) {
    let read = with_rules(topic, lines);
    let idle = before.filter(|known| Arc::ptr_eq(&known.base, base)).and_then(|known| Some((known, idle_sentences(&known.rules, &read.rules)?)));
    let mut facts: Vec<Option<Arc<Facts>>> = Vec::with_capacity(learn_part.len() + judge_part.len());
    for (at, case) in learn_part.iter().chain(judge_part).enumerate() {
        let same = idle.as_ref().and_then(|(known, sentences)| known.facts[at].as_ref().filter(|found| found.holds_none(sentences)));
        match same {
            Some(found) => facts.push(Some(Arc::clone(found))),
            None => {
                let Some(found) = bounded_facts(case, base, &read, false) else { return (0, None) };
                facts.push(Some(Arc::new(found)));
            }
        }
    }
    let every: Vec<&Case> = learn_part.iter().chain(judge_part).copied().collect();
    let said = Arc::clone(&ground.said);
    if !read.questions.is_empty() {
        let right = checked_tally_on(&read, &[], (&ground.every_words, None), (&every, &said, None), &facts).written;
        return (right, Some(JudgedReading { base: Arc::clone(base), rules: read.rules, facts }));
    }
    let (learn_facts, judge_facts) = facts.split_at(learn_part.len());
    let (learn_said, judge_said) = said.split_at(learn_part.len());
    let right = [(learn_part, learn_said, learn_facts, judge_part, judge_said, judge_facts), (judge_part, judge_said, judge_facts, learn_part, learn_said, learn_facts)]
        .iter()
        .zip(&ground.halves)
        .map(|((learned_on, learned_said, learned_facts, judged_on, judged_said, judged_facts), (learned_ground, checked))| {
            let (kept, text) = learn_forms_grounded(&read, base, (learned_on, learned_said, learned_ground), learned_facts);
            checked_tally_on(&read, &kept, (&learned_ground.words, Some(&text)), (judged_on, judged_said, Some(checked)), judged_facts).learned
        })
        .sum();
    (right, Some(JudgedReading { base: Arc::clone(base), rules: read.rules, facts }))
}
because!(joint_right, LearnedWording, "how well a whole reading of the told sentences serves the wording: the forms of the questions are learned with it from one half of the lessons and judged on the other, and again the other way round, all forms at once, so that every lesson judges and a reading is not chosen by the chance of a small part; so that a reading of one sentence form that answers its own few lessons by chance but breaks a question form that other sentence forms share loses; nothing for a reading under which the facts of some lesson run away, as a reading that makes a fact fit its own sentence again and again reads nothing; when the text has written question forms, the reading is judged by them on all the lessons, as nothing of the questions is learned then; a lesson keeps its facts from the reading judged just before, on the same facts of the text, when the rules that this reading adds hold no fact of them, as its chain would be the same; and the words of the lessons, which no reading changes, from the reading judged before");

const JOINT_CHOICES: usize = 24;
because!(JOINT_CHOICES, LearnedWording, "the most promising ways to read one sentence form on the lessons of the form, which the whole wording judges; beside them the best way to read it into each other fact, and once a way into a fact is chosen, the next ways into the same fact, as many as the most promising ways: a reading that fills a fact for one of two people promises less on its own than a wrong one that fills a fact for both, and the second person needs the second way into the same fact");

const PAIRED_CHOICES: usize = 8;
because!(PAIRED_CHOICES, LearnedWording, "the ways to read a form, the best of them by the whole wording alone, that are judged in pairs too: a sentence that tells the same thing of two people gains little from one of its two readings, as the second person's fact is missing, and a wrong reading that fills something for both can win alone");

const MOST_FORM_LINES: usize = 5;
because!(MOST_FORM_LINES, LearnedWording, "the most rules that read one sentence form: a sentence tells a few things, and a longer list of readings for one form learns its few lessons one by one");

const JOINT_PASSES: usize = 3;

const MOST_REPASSED: usize = 64;
because!(MOST_REPASSED, LearnedWording, "after the first pass, the most sentence forms, the most told first, whose readings are chosen again although they have none: a form that gained nothing in the first pass seldom gains in a later one, and a kind of many hundreds of forms would spend each later pass on all of them, while a form with readings is always chosen again");
because!(JOINT_PASSES, LearnedWording, "the most passes over the sentence forms in which the reading of each form is chosen again while the others stay: a change of one form can make another reading of an earlier form better, as the rules of all forms feed the same questions");

const JUDGE_LEARN_PERCENT: usize = 50;
because!(JUDGE_LEARN_PERCENT, LearnedWording, "the share of the lessons in the first of the two halves of a judge: the wording is learned from each half and judged on the other, so that the halves are as large as each other");

fn own_places(pattern: &Pattern) -> Vec<usize> {
    places_of(pattern).into_iter().filter(|place| *place < MARKED_FIRST).collect()
}
because!(own_places, LearnedWording, "the open places of a sentence form or a window that its words fill, without the places before and after a window that may hold the rest of the sentence");

fn sentence_windows(lessons: &[&Case], (frame, values): (&HashSet<String>, &HashSet<Vec<String>>), covered: &HashSet<Pattern>) -> Vec<(Pattern, Vec<(usize, Vec<Vec<String>>)>)> {
    let mut found: HashMap<Pattern, Vec<(usize, Vec<Vec<String>>)>> = HashMap::new();
    for (at, case) in lessons.iter().enumerate() {
        for sentence in &case.sentences {
            let Some(whole) = sentence_form(&fact_words(sentence), frame, values, usize::MAX) else { continue };
            if covered.contains(&whole.0) {
                continue;
            }
            let split = split_runs(&whole);
            for (pieces, runs) in [whole, split] {
            for start in 0..pieces.len() {
                for size in WINDOW_SIZES {
                    let end = start + size;
                    if end > pieces.len() || (start == 0 && end == pieces.len()) {
                        continue;
                    }
                    let inner = &pieces[start..end];
                    let opened: Vec<usize> = inner.iter().filter_map(|piece| if let LogicPiece::Open(place) = piece { Some(*place) } else { None }).collect();
                    if opened.is_empty() || opened.len() > MOST_TOLD_PLACES || !inner.iter().any(|piece| matches!(piece, LogicPiece::Word(word) if word_not_mark(word_text(*word)))) {
                        continue;
                    }
                    let mut window: Pattern = Vec::new();
                    if start > 0 {
                        window.push(LogicPiece::Open(MARKED_FIRST));
                    }
                    window.extend(inner.iter().map(|piece| match piece {
                        LogicPiece::Open(place) => LogicPiece::Open(opened.iter().position(|known| known == place).unwrap_or_default()),
                        other => other.clone(),
                    }));
                    if end < pieces.len() {
                        window.push(LogicPiece::Open(MARKED_FIRST + 1));
                    }
                    let window_runs: Vec<Vec<String>> = opened.iter().map(|place| runs[*place].clone()).collect();
                    let uses = found.entry(window).or_default();
                    if uses.last().is_none_or(|(lesson, _)| *lesson != at) {
                        uses.push((at, window_runs));
                    }
                }
            }
            }
        }
    }
    let mut out: Vec<(Pattern, Vec<(usize, Vec<Vec<String>>)>)> = found.into_iter().filter(|(_, uses)| uses.len() >= LEAST_SUPPORT).collect();
    out.sort_by(|one, other| other.1.len().cmp(&one.1.len()).then_with(|| library_writing(&one.0).cmp(&library_writing(&other.0))));
    out.truncate(MOST_WINDOWS);
    out
}
because!(sentence_windows, LearnedWording, "the windows of the told sentences: a few pieces of a sentence in a row (words of the frame and runs of other words), with a place that may hold marks for the rest of the sentence before and after them, kept when enough lessons tell them, the most told first, so that a list of any length or a sentence that never comes twice is read by the parts that come again (a red pen, in a list of things); only sentences whose whole form is not told often enough give windows, as a sentence form that comes again is read whole");

fn told_names(lessons: &[&Case]) -> HashSet<String> {
    let mut named: HashMap<String, (usize, usize)> = HashMap::new();
    for case in lessons {
        capitals_inside(&case.question, &mut named);
        for sentence in &case.sentences {
            capitals_inside(sentence, &mut named);
        }
    }
    named.into_iter().filter(|(_, (inside, capital))| *inside > usize::default() && capital * WHOLE_SHARE >= inside * NAMED_PERCENT).map(|(word, _)| word).collect()
}
because!(told_names, LearnedWording, "the names of the told sentences of a kind: the words that they or the questions write with a capital inside a sentence in most of their uses there");

fn capitals_inside(text: &str, named: &mut HashMap<String, (usize, usize)>) {
    let words = fact_words(text);
    let written = text_words(text);
    for (at, word) in words.iter().enumerate() {
        if at > usize::default() && written.get(at - 1).is_some_and(|before| !SENTENCE_STARTS.contains(&before.as_str())) {
            let entry = named.entry(word.clone()).or_default();
            entry.0 += 1;
            entry.1 += usize::from(written.get(at).is_some_and(|text| text.chars().next().is_some_and(char::is_uppercase) && text.chars().nth(1).is_some()));
        }
    }
}
because!(capitals_inside, LearnedWording, "for each word of a sentence that stands inside it (not first, and not after a mark that starts a sentence), one use more, and one capital more when it is written with a capital, a name being the word that keeps its capital there");

#[derive(Clone, PartialEq)]
struct ItemEdge {
    before: Vec<String>,
    text_before: bool,
    after: Vec<String>,
    text_after: bool,
}
because!(ItemEdge, LearnedWording, "where an item of a told list stands: the marks and joining words before it and after it (a comma, a colon, and), and whether more of the sentence stands beyond them");

const LIST_MARKS: &[&str] = &[",", ":", ";", "and", "or"];
because!(LIST_MARKS, LearnedWording, "the marks and words between the items of a told list, as English writes a list (a, b, and c)");

const LEAST_ITEMS: usize = 2;
because!(LEAST_ITEMS, LearnedWording, "the fewest parts of one sentence with the same form that make the sentence a list of items: a pair joined by and is already a list, while one part alone is a clause");

fn list_items(lessons: &[&Case], (frame, values): (&HashSet<String>, &HashSet<Vec<String>>)) -> Vec<(Pattern, Vec<(usize, Vec<Vec<String>>)>, Vec<ItemEdge>, Vec<(Pattern, usize)>)> {
    let mut found: Vec<(Pattern, Vec<(usize, Vec<Vec<String>>)>, Vec<ItemEdge>)> = Vec::new();
    let mut stretches: Vec<(Pattern, Pattern, usize)> = Vec::new();
    for (at, case) in lessons.iter().enumerate() {
        for sentence in &case.sentences {
            let words = fact_words(sentence);
            let mut parts: Vec<(usize, usize)> = Vec::new();
            let mut start = usize::default();
            for (place, word) in words.iter().enumerate() {
                if LIST_MARKS.contains(&word.as_str()) {
                    if place > start {
                        parts.push((start, place));
                    }
                    start = place + 1;
                }
            }
            if start < words.len() {
                parts.push((start, words.len()));
            }
            let shapes: Vec<Option<(Pattern, Vec<Vec<String>>)>> = parts.iter().map(|(from, to)| sentence_form(&words[*from..*to], frame, values, MOST_TOLD_PLACES)).collect();
            let told_items = |shape: &Pattern| shapes.iter().filter(|other| other.as_ref().is_some_and(|(known, _)| known == shape)).count();
            for (index, ((from, to), shape)) in parts.iter().zip(&shapes).enumerate() {
                let Some((item, runs)) = shape else { continue };
                if told_items(item) < LEAST_ITEMS {
                    for (other, _) in shapes.iter().flatten() {
                        if let Some(place) = stretched_at(item, other)
                            && told_items(other) >= LEAST_ITEMS
                            && !stretches.iter().any(|(base, variant, known)| base == other && variant == item && *known == place)
                        {
                            stretches.push((other.clone(), item.clone(), place));
                        }
                    }
                }
                let worded = item.iter().any(|piece| matches!(piece, LogicPiece::Word(word) if word_not_mark(word_text(*word))));
                if places_of(item).is_empty() || !worded || told_items(item) < LEAST_ITEMS {
                    continue;
                }
                let previous_end = index.checked_sub(1).map_or(usize::default(), |earlier| parts[earlier].1);
                let next_start = parts.get(index + 1).map_or(words.len(), |later| later.0);
                let after_end = if index + 1 < parts.len() { next_start.min(*to + 1) } else { next_start };
                let edge = ItemEdge { before: words[previous_end..*from].to_vec(), text_before: index > usize::default(), after: words[*to..after_end].to_vec(), text_after: index + 1 < parts.len() };
                match found.iter_mut().find(|(known, _, _)| known == item) {
                    Some((_, uses, sides)) => {
                        if uses.last().is_none_or(|(lesson, known)| *lesson != at || known != runs) {
                            uses.push((at, runs.clone()));
                        }
                        if !sides.contains(&edge) {
                            sides.push(edge);
                        }
                    }
                    None => found.push((item.clone(), vec![(at, runs.clone())], vec![edge])),
                }
            }
        }
    }
    found.retain(|(_, uses, _)| uses.len() >= LEAST_SUPPORT);
    found.sort_by(|one, other| other.1.len().cmp(&one.1.len()).then_with(|| library_writing(&one.0).cmp(&library_writing(&other.0))));
    found.truncate(MOST_WINDOWS);
    found
        .into_iter()
        .map(|(item, uses, sides)| {
            let variants = stretches.iter().filter(|(base, _, _)| *base == item).map(|(_, variant, place)| (variant.clone(), *place)).collect();
            (item, uses, sides, variants)
        })
        .collect()
}
because!(list_items, LearnedWording, "the items of the told lists: a sentence cut at its commas, colons and joining words into parts, and a part whose form comes at least twice in the sentence, with a word of the frame, is an item, with the marks and joining words before it at each place where it stands (after the colon, after a comma, after the last and) and the first mark after it, as what follows that mark belongs to the next item (a list of three tells its middle item before a comma and an and, a longer list before a comma alone); and for each item, the parts of its lists whose form is the item with a value that a mark joins (a title with a hyphen before its number where the item tells a title of one word), each with the place of that value; the item forms that enough lessons tell, the most told first, so that one reading of an item form, written for each place where its items stand, reads a list of any length");

fn item_window(item: &Pattern, edge: &ItemEdge) -> Pattern {
    let mut out: Pattern = Vec::new();
    if edge.text_before {
        out.push(LogicPiece::Open(MARKED_FIRST));
    }
    out.extend(edge.before.iter().map(|word| LogicPiece::Word(word_number(word))));
    out.extend(item.iter().cloned());
    out.extend(edge.after.iter().map(|word| LogicPiece::Word(word_number(word))));
    if edge.text_after {
        out.push(LogicPiece::Open(MARKED_FIRST + 1));
    }
    out
}
because!(item_window, LearnedWording, "the pattern of an item of a list at one of its places: the rest of the sentence before it in a place that may hold marks, the marks and joining words before it, the item, those after it, and the rest after it");

const STRETCH_PIECES: usize = 3;
because!(STRETCH_PIECES, LearnedWording, "the pieces of a value that a mark joins in a told list: a run, the mark and the run after it (a title with a hyphen before its number)");

fn stretched_at(part: &Pattern, item: &Pattern) -> Option<usize> {
    part.windows(STRETCH_PIECES).enumerate().find_map(|(at, pieces)| {
        let [LogicPiece::Open(first), LogicPiece::Word(mark), LogicPiece::Open(second)] = pieces else { return None };
        if word_not_mark(word_text(*mark)) || LIST_MARKS.contains(&word_text(*mark)) || *second != *first + 1 {
            return None;
        }
        let joined: Pattern = part[..=at]
            .iter()
            .cloned()
            .chain(part[at + STRETCH_PIECES..].iter().map(|piece| match piece {
                LogicPiece::Open(place) if place > first => LogicPiece::Open(place - 1),
                other => other.clone(),
            }))
            .collect();
        (joined == *item).then_some(*first)
    })
}
because!(stretched_at, LearnedWording, "the place of an item whose value a mark joins in a part of a list: the part is the item with a run, a mark that is not a mark of the list, and a run in place of that value, so that a book whose title holds a hyphen before its number, among books of one word, is read as the same item");

fn stretched_line(window: &Pattern, (variant, place): (&Pattern, usize), target: &Pattern, to: &[usize]) -> String {
    let stretch: Vec<LogicPiece> = variant.windows(STRETCH_PIECES).find(|pieces| matches!(pieces.first(), Some(LogicPiece::Open(first)) if *first == place)).map(<[LogicPiece]>::to_vec).unwrap_or_default();
    let outcome: Pattern = renamed(target, &places_of(target), to)
        .into_iter()
        .flat_map(|piece| match piece {
            LogicPiece::Open(at) if at == place => stretch.clone(),
            LogicPiece::Open(at) if at > place => vec![LogicPiece::Open(at + 1)],
            other => vec![other],
        })
        .collect();
    format!("{TOLD_IF}{}{TOLD_THEN}{}{STATEMENT_END}", library_writing(window), library_writing(&outcome))
}
because!(stretched_line, LearnedWording, "a reading of an item written again for a part of the list whose value a mark joins: the same fact, with the joined value (the run, the mark and the run after it) where the reading gives the value of the item, so that a reading chosen for the item reads that part too");

type FormUses = Vec<(usize, Vec<Vec<String>>)>;
because!(FormUses, LearnedWording, "the uses of a told sentence form: the lesson of each use and the words that fill each place of the form there");

fn opened_at(form: &Pattern, at: usize) -> Option<(Pattern, usize)> {
    let LogicPiece::Word(word) = form.get(at)? else { return None };
    if !word_not_mark(word_text(*word)) {
        return None;
    }
    let mut out: Pattern = Vec::new();
    let mut inserted = usize::default();
    let mut next = usize::default();
    for (place, logic_piece) in form.iter().enumerate() {
        if place == at {
            inserted = next;
            out.push(LogicPiece::Open(next));
            next += 1;
            continue;
        }
        match logic_piece {
            LogicPiece::Open(_) => {
                out.push(LogicPiece::Open(next));
                next += 1;
            }
            other => out.push(other.clone()),
        }
    }
    Some((out, inserted))
}
because!(opened_at, LearnedWording, "a sentence form with the fixed word at one place opened, the places numbered again in their order, and the number of the new place; nothing where the form holds a place or a mark");

fn merged_told_forms(forms: Vec<(Pattern, FormUses)>, (events_at, reach): (&dyn Fn(&Pattern, usize) -> usize, &dyn Fn(&Pattern, usize) -> Vec<WordId>), mut joins: Option<&mut Vec<FormJoin>>) -> Vec<(Pattern, FormUses)> {
    let mut forms = forms;
    let mut events: HashMap<(Pattern, usize), usize> = HashMap::new();
    loop {
        let mut groups: HashMap<(Pattern, usize), Vec<usize>> = HashMap::new();
        for (index, (form, _)) in forms.iter().enumerate() {
            for at in 0..form.len() {
                if let Some((opened, _)) = opened_at(form, at) {
                    groups.entry((opened, at)).or_default().push(index);
                }
            }
        }
        let mut best: Option<(&(Pattern, usize), &Vec<usize>, usize)> = None;
        for (key, members) in &groups {
            if members.len() < LEAST_ITEMS {
                continue;
            }
            if places_of(&key.0).into_iter().any(|place| *events.entry((key.0.clone(), place)).or_insert_with(|| events_at(&key.0, place)) > 1) {
                continue;
            }
            let uses: usize = members.iter().map(|index| forms[*index].1.len()).sum();
            let better = best.is_none_or(|(known, known_members, known_uses)| uses > known_uses || (uses == known_uses && (members.len() > known_members.len() || (members.len() == known_members.len() && library_writing(&key.0).cmp(&library_writing(&known.0)).then_with(|| key.1.cmp(&known.1)).is_lt()))));
            if better {
                best = Some((key, members, uses));
            }
        }
        let Some(((opened, at), members, _)) = best else { break };
        let (opened, at) = (opened.clone(), *at);
        let members = members.clone();
        if let Some(joins) = joins.as_deref_mut()
            && let Some((_, inserted)) = opened_at(&forms[members[0]].0, at)
        {
            let words: Vec<WordId> = members.iter().filter_map(|index| match forms[*index].0.get(at) {
                Some(LogicPiece::Word(word)) => Some(*word),
                _ => None,
            }).collect();
            let beyond: Vec<String> = reach(&opened, inserted).into_iter().filter(|word| !words.contains(word)).map(|word| word_text(word).to_string()).collect();
            joins.push(FormJoin { members: members.iter().map(|index| library_writing(&forms[*index].0)).collect(), joined: library_writing(&opened), beyond });
        }
        let mut merged: FormUses = Vec::new();
        for index in &members {
            let (form, uses) = &forms[*index];
            let Some((_, inserted)) = opened_at(form, at) else { continue };
            let LogicPiece::Word(word) = &form[at] else { continue };
            for (lesson, runs) in uses {
                let mut longer = runs.clone();
                longer.insert(inserted.min(longer.len()), vec![word_text(*word).to_string()]);
                merged.push((*lesson, longer));
            }
        }
        merged.sort_by_key(|(lesson, _)| *lesson);
        let mut kept: Vec<(Pattern, FormUses)> = forms.into_iter().enumerate().filter(|(index, _)| !members.contains(index)).map(|(_, form)| form).collect();
        kept.push((opened, merged));
        forms = kept;
    }
    forms
}
because!(merged_told_forms, LearnedWording, "the told sentence forms with the forms that differ only in one fixed word joined into one form with a place there (first, then and finally before the same swap), the join that holds the most uses first, again until no two forms differ in one word, so that one reading reads them all where each would need its own, and readings that pay off only together are fewer; not where the joined form would read, in the told sentences of the lessons, two or more words at one of its places that the rules of the text read as events of their own (text_events_at), a place opened by an earlier join too, as a join over the names or things of forms that each opened their verb for one name widens that verb's place to every story");

fn text_events_at(topic: &Topic, lessons: &[&Case], (opened, place): (&Pattern, usize)) -> usize {
    let readers: Vec<&Pattern> = topic.rules.iter().flat_map(|rule| rule.conditions.iter()).filter_map(|condition| match condition {
        Condition::Fact { pattern, .. } => Some(pattern),
        _ => None,
    }).collect();
    let mut seen: HashSet<Vec<WordId>> = HashSet::new();
    let mut events: Vec<WordId> = Vec::new();
    for sentence in lessons.iter().flat_map(|case| case.sentences.iter().map(|sentence| fact_numbers(sentence))) {
        if !seen.insert(sentence.clone()) {
            continue;
        }
        let mut fits = Vec::new();
        fit_into(opened, &sentence, &empty_filling(), &mut fits);
        for filling in fits {
            let Some(&[word]) = filling.get(place) else { continue };
            if events.contains(&word) {
                continue;
            }
            let read = readers.iter().any(|pattern| pattern.contains(&LogicPiece::Word(word)) && {
                let mut read_fits = Vec::new();
                fit_into(pattern, &sentence, &empty_filling(), &mut read_fits);
                !read_fits.is_empty()
            });
            if read {
                events.push(word);
            }
        }
    }
    events.len()
}
fn words_at(lessons: &[&Case], (opened, place): (&Pattern, usize)) -> Vec<WordId> {
    let mut seen: HashSet<Vec<WordId>> = HashSet::new();
    let mut words: Vec<WordId> = Vec::new();
    for sentence in lessons.iter().flat_map(|case| case.sentences.iter().map(|sentence| fact_numbers(sentence))) {
        if !seen.insert(sentence.clone()) {
            continue;
        }
        let mut fits = Vec::new();
        fit_into(opened, &sentence, &empty_filling(), &mut fits);
        for filling in fits {
            if let Some(&[word]) = filling.get(place)
                && !words.contains(&word)
            {
                words.push(word);
            }
        }
    }
    words
}
because!(words_at, LearnedWording, "the words of one word that a told form reads at a place in the told sentences of the lessons, each once, in the order in which they come, for the report of a join of told forms");

because!(text_events_at, LearnedWording, "how many words a joined told form would read at its new place in the told sentences of the lessons that a rule of the text reads with that word as its own (A got the B there, A dropped the B there): the text tells such words apart as events, a take and a drop, so a join over two of them reads both as one event, as its place takes every word that a story puts there, not only the words of the forms that it joined (took, grabbed, got, and then also dropped), and each reading of it puts that word in the place of a thing or a person");

fn split_runs(form: &(Pattern, Vec<Vec<String>>)) -> (Pattern, Vec<Vec<String>>) {
    let (pieces, runs) = form;
    let mut out: Pattern = Vec::new();
    let mut split: Vec<Vec<String>> = Vec::new();
    for piece in pieces {
        match piece {
            LogicPiece::Open(place) => {
                for word in &runs[*place] {
                    out.push(LogicPiece::Open(split.len()));
                    split.push(vec![word.clone()]);
                }
            }
            other => out.push(other.clone()),
        }
    }
    (out, split)
}
because!(split_runs, LearnedWording, "a sentence form with each run of other words cut into its words, each an open place, so that a window may read the words of a run apart, as a colour and the thing that has it");

const WINDOW_SIZES: [usize; 4] = [3, 4, 5, 6];
because!(WINDOW_SIZES, LearnedWording, "how many pieces of a sentence a window holds: one value with a word on each side at the least, and two values with the words around and between them at the most, as one item of a list or two items side by side");

const MOST_WINDOWS: usize = 16;
because!(MOST_WINDOWS, LearnedWording, "the most windows, the most told first, whose readings are judged: each window is judged as a sentence form is, and the judge of a whole reading costs the same for each, so that windows add a bounded share to the time of a kind of whole forms");
