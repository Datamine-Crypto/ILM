pub mod answer;
pub mod choices;
pub mod classes;
pub mod codes;
pub mod counting;
pub mod finder;
pub mod grouping;
pub mod groups;
pub mod hash;
pub mod holes;
pub mod indexed;
pub mod kinds;
pub mod learn;
pub mod lessons;
pub mod long_story;
pub mod list;
pub mod model;
pub mod names;
pub mod new_words;
pub mod show;
pub mod splits;
pub mod story;
pub mod striped;
pub mod table;
pub mod texts;
pub mod told;
pub mod told_learn;
pub mod told_read;
pub mod write_back;

use crate::tools::call::Tool;
use crate::tools::pattern::answer::learn_patterns;
use patterns::{because, source};

pub struct PatternTool;
source!(PatternTool, "one tool among the tools of the choice: the tool is named for its way of reading and not for a task, and the choice gives it the kinds where its counted patterns read more lessons right than the other tools do");

pub const PATTERNS_TOOL: Tool = Tool { name: "patterns", needs_labels: false, probe: true, asks: false, over: None, learn: learn_patterns };
because!(PATTERNS_TOOL, PatternTool, "the pattern learner as a tool of the choice: it needs no labels of the lesson sentences and asks for no answer of a text that it builds; its model gives the control by the form of the question, because a pattern with no told sentence is an answer from the question alone");
