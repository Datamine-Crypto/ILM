pub mod chains;
pub mod joins;
pub mod model;
pub mod pick;
pub mod read;
pub mod trials;

use crate::tools::call::Tool;
use crate::tools::link::model::learn_link;
use patterns::{because, source};

pub struct LinkTool;
source!(LinkTool, "a learner of links between things: a way from one thing of the question to the other, and a join of the links along the way. No link and no join is written by hand: both are counted from lessons that give only the story, the question and the answer, and the words and their kinds come from the first step of the pattern learner");

pub const LINK_TOOL: Tool = Tool { name: "link", needs_labels: false, probe: false, asks: false, over: None, learn: learn_link };
because!(LINK_TOOL, LinkTool, "the link tool in the list of the tools: it needs no label of a lesson sentence, it has no rule by the form of the question alone, and it asks for no answer");
