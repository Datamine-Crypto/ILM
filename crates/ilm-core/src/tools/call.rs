use crate::cases::Case;
use crate::classes::WordClasses;
use patterns::{because, source};

pub struct OneCall;
source!(OneCall, "one call for every tool: each tool is called in one way, it learns from lessons and then answers a told text with a question in words and with a note that says how the answer came; so the choice among the tools is made by lessons that a tool did not learn from, and no tool is told the task");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Said {
    pub words: Vec<String>,
    pub note: String,
}
because!(Said, OneCall, "what a tool says for one case: the words of its answer as the measure compares them, and the note that names the rule or the program that gave it");

pub trait Learner: Send + Sync {
    fn answer(&self, sentences: &[String], question: &str) -> Said;

    fn question_alone(&self, _sentences: &[String], _question: &str) -> Option<Vec<String>> {
        None
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
}
because!(Learner, OneCall, "a model that a tool made from lessons: it answers a case and changes nothing while it reads, so many threads read cases with one model. question_alone is what a model says from the form of the question with no told sentence in its rule, and nothing for a model that has no such rule: the measure reads it as a control, because a kind whose answers follow from the question needs no reading of the text. facts and explain are what a reader of requests to a service gives for one request (each slot with its words, and how each word was read), and nothing for a model that reads no requests");

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
}
because!(Lessons, OneCall, "the lessons that a tool learns from: the cases; for a set that gives them the label lines of each case, which reach a tool for its lessons and for no other case (no labels, or as many lists as cases); and for a kind that can answer them a way to ask for the answer of a text that the tool builds, which gives nothing for a text that is kept to be read; and the table of word classes when the run has one. of gives lessons with no labels, no way to ask and no classes; labelled says whether a lesson has a label; part gives other cases with the same way to ask and the same classes");

impl<'a> Lessons<'a> {
    pub fn of(cases: &'a [Case]) -> Lessons<'a> {
        Lessons { cases, labels: &[], ask: None, classes: None }
    }

    pub fn labelled(&self) -> bool {
        self.labels.iter().any(|lines| !lines.is_empty())
    }

    pub fn part(&self, cases: &'a [Case], labels: &'a [Vec<String>]) -> Lessons<'a> {
        Lessons { cases, labels, ask: self.ask, classes: self.classes }
    }
}

pub type Learned = Result<Box<dyn Learner>, String>;
because!(Learned, OneCall, "the model of a tool, or the words with which the tool says that it does not read this kind");

#[derive(Clone, Copy)]
pub struct Tool {
    pub name: &'static str,
    pub needs_labels: bool,
    pub probe: bool,
    pub asks: bool,
    pub over: Option<&'static str>,
    pub learn: fn(Lessons, usize) -> Learned,
}
because!(Tool, OneCall, "a tool of the entry point: its name; whether it learns from labels of the lesson sentences, so that it is not run where the lessons have none; whether its model gives the control by the form of the question; whether it asks for the answer of a text that it builds; for a tool that runs the learner of another tool on texts that it reads in its own way, the name of that tool; and the call that makes its model from lessons on a given number of threads");
