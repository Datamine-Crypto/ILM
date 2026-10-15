pub mod deduce;
pub mod forms;
pub mod lessons;
pub mod marks;
pub mod model;
pub mod pieces;
pub mod tables;
pub mod tokens;

use crate::tools::call::Tool;
use crate::tools::rule::model::learn_rule;
use patterns::{because, source};

pub struct RuleTool;
source!(RuleTool, "deduction with told rules: a reader that is counted from lessons with formal forms turns each told sentence into a fact or a rule, and a small exact part draws what follows. The tool holds no word of any set: which words are things, which marks begin a condition or a head, which mark means no and which word names which relation are all counted");

pub const RULE_TOOL: Tool = Tool { name: "rule", needs_labels: true, probe: false, asks: false, over: None, learn: learn_rule };
because!(RULE_TOOL, RuleTool, "the rule tool in the list of the tools: it learns from the labels of the lesson sentences, so it is not run where the lessons have none; it has no rule by the form of the question alone, and it asks for no answer");
