use crate::cases::Case;
use crate::classes::WordClasses;
use patterns::{because, source};

pub struct OneCall;
source!(OneCall, "one call for every tool: each tool is called in one way, it learns from lessons and then answers a told text with a question in words and with a note that says how the answer came; so the choice among the tools is made by lessons that a tool did not learn from, and no tool is told the task");

pub const FAULT_DETAIL: &str = "; what was tried: ";
because!(FAULT_DETAIL, OneCall, "the words between the fault that a tool says when it does not read a kind and what it tried, so that a short line names the fault and a long one names what came nearest");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Said {
    pub words: Vec<String>,
    pub note: String,
    pub mark: AnswerMark,
}
because!(Said, OneCall, "what a tool says for one case: the words of its answer as the measure compares them, the note that names the rule or the program that gave it, and how the answer stands, so that a reader tells a proved answer from a guess");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnswerMark {
    Proof,
    Default,
    Guess,
    #[default]
    NoAnswer,
}
because!(AnswerMark, OneCall, "how an answer stands: by proof, when steps prove it from a story read only in ways that the lessons showed; by default, when it holds only otherwise; a guess, when no step proves it or when the story holds a sentence that no lesson showed in its form, which the steps may have read as something else; and no answer. A model says where its answer rests, and a wrong answer that claims a proof is a fault of the model, not a guess");

pub trait Learner: Send + Sync {
    fn answer(&self, sentences: &[String], question: &str) -> Said;

    fn reading(&self) -> Option<Reading> {
        None
    }

    fn used_labels(&self) -> bool {
        false
    }

    fn used_kinds_before(&self) -> bool {
        false
    }

    fn kinds_used_before(&self) -> Vec<usize> {
        Vec::new()
    }

    fn facts(&self, _text: &str) -> Option<Vec<(String, String)>> {
        None
    }

    fn explain(&self, _text: &str) -> Vec<String> {
        Vec::new()
    }

    fn story<'a>(&'a self, _sentences: &[String]) -> Option<Box<dyn StoryAsk + 'a>> {
        None
    }

    fn bound_stops(&self) -> usize {
        usize::default()
    }

    fn proof(&self, _sentences: &[String], _question: &str) -> Vec<String> {
        Vec::new()
    }

    fn missing(&self, _sentences: &[String], _question: &str, _expected: &[String]) -> Vec<String> {
        Vec::new()
    }

    fn rules_used(&self, _sentences: &[String], _question: &str) -> Vec<(String, bool)> {
        Vec::new()
    }

    fn facts_used(&self, _sentences: &[String], _question: &str) -> Vec<(String, bool)> {
        Vec::new()
    }

    fn lines_from_lessons(&self) -> Vec<String> {
        Vec::new()
    }

    fn named_words(&self) -> Option<(usize, usize)> {
        None
    }

    fn answer_counts(&self, _question: &str, _lessons: &[Case]) -> Vec<String> {
        Vec::new()
    }
}
because!(Learner, OneCall, "a model that a tool made from lessons: it answers a case and changes nothing while it reads, so many threads read cases with one model. reading says how the model reads its kind: by lines learned from the lessons of the kind alone, by lines of the library alone, or by both; nothing for a model that says nothing of it (the controls). facts and explain are what a reader of requests to a service gives for one request (each slot with its words, and how each word was read), and nothing for a model that reads no requests. bound_stops is how many readings of the model have ended at a bound of its work so far, so that a measure can say where an answer may miss what it needed; none for a model with no such bound. proof gives the lines of the proof of an answer, from the answer back to the told sentences, for a model that can explain one, and nothing for the others; missing gives, for an answer that the model did not give, the answer of its texts that holds furthest and the first condition that the facts of the story miss, and nothing for a model that cannot say; rules_used gives the rules that an answer rests on, each with whether a text of the library holds it (given) or the model learned it from the lessons (discovered), and nothing for a model with no rules; lines_from_lessons gives the lines that the model learned from the lessons of its kind, as lines of the library, and nothing for a model that learned none or cannot say; used_kinds_before says whether the model learned over the rules that the kinds before its kind in its set taught, so that a board marks a kind that borrowed lessons, and never for a model that learned from its own lessons alone; kinds_used_before gives the places of those kinds in the order of the set before the kind, each once, the kinds whose lessons gave some line of the model, so that a board names them, and none for a model that learned from its own lessons alone; named_words gives, for a model of learned question forms, how many of them name a word that a learned form with fewer fixed words reads too, of all its forms, a sign of forms learned value by value, and nothing for other models; answer_counts gives, for the learned form that reads a question, each of its answers with the lessons where its condition holds, by their answers, on the first lessons of that form, so that a person sees a condition that holds on lessons of another answer, and nothing for other models");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    Learned,
    Written,
    Mixed,
}
because!(Reading, OneCall, "how a model reads a kind: learned, when every line that reads the told text or answers the question was learned from the lessons of the kind (the rules of concepts and knowledge that it leans on are general knowledge); written, when the lines that read and answer are lines of the library written by a person; mixed, when the model needs both");

pub trait StoryAsk {
    fn ask(&mut self, question: &str) -> Said;
}
because!(StoryAsk, OneCall, "a story that a model has read, asked one question after another: a model that keeps an index of a story answers each question from it, and every answer is the answer of the model for the story with that question");

pub struct EachTime<'a> {
    model: &'a dyn Learner,
    sentences: Vec<String>,
}
because!(EachTime, OneCall, "a story asked through a model that keeps no index: each question is answered with the whole story, as one case");

impl StoryAsk for EachTime<'_> {
    fn ask(&mut self, question: &str) -> Said {
        self.model.answer(&self.sentences, question)
    }
}

pub fn read_story<'a>(model: &'a dyn Learner, sentences: &[String]) -> Box<dyn StoryAsk + 'a> {
    model.story(sentences).unwrap_or_else(|| Box::new(EachTime { model, sentences: sentences.to_vec() }))
}
because!(read_story, OneCall, "a story read once by a model that keeps an index of a story, and through the plain way of one case for each question by every other model");

#[derive(Clone, Copy)]
pub struct Lessons<'a> {
    pub cases: &'a [Case],
    pub labels: &'a [Vec<String>],
    pub ask: Option<&'a (dyn Fn(&str) -> Option<String> + Sync)>,
    pub classes: Option<&'a WordClasses>,
    pub before: Option<&'a (dyn Fn() -> Vec<Vec<Case>> + Sync)>,
}
because!(Lessons, OneCall, "the lessons that a tool learns from: the cases; for a set that gives them the label lines of each case, which reach a tool for its lessons and for no other case (no labels, or as many lists as cases); and for a kind that can answer them a way to ask for the answer of a text that the tool builds, which gives nothing for a text that is kept to be read; the table of word classes when the run has one; and a way to read the lessons of the kinds that come before it in its set, in the order of the set, as a book teaches a step after the steps that it needs (only lessons, never a read case of another kind), read only by a tool that needs them, as most kinds learn alone and the files of the kinds before a kind cost time. of gives lessons with no labels, no way to ask, no classes and no kinds before; labelled says whether a lesson has a label; part gives other cases with the same way to ask, the same classes and the same kinds before");

impl<'a> Lessons<'a> {
    pub fn of(cases: &'a [Case]) -> Lessons<'a> {
        Lessons { cases, labels: &[], ask: None, classes: None, before: None }
    }

    pub fn labelled(&self) -> bool {
        self.labels.iter().any(|lines| !lines.is_empty())
    }

    pub fn part(&self, cases: &'a [Case], labels: &'a [Vec<String>]) -> Lessons<'a> {
        Lessons { cases, labels, ask: self.ask, classes: self.classes, before: self.before }
    }
}

pub type Learned = Result<Box<dyn Learner>, String>;
because!(Learned, OneCall, "the model of a tool, or the words with which the tool says that it does not read this kind");

#[derive(Clone, Copy)]
pub struct Tool {
    pub name: &'static str,
    pub needs_labels: bool,
    pub learn: fn(Lessons, usize) -> Learned,
}
because!(Tool, OneCall, "a tool of the entry point: its name; whether it learns from labels of the lesson sentences, so that it is not run where the lessons have none; and the call that makes its model from lessons on a given number of threads");
