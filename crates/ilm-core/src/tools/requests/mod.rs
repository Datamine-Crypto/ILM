pub mod frames;
pub mod model;
pub mod read;
pub mod shares;
pub mod tags;
pub mod whole;

use crate::tools::call::Tool;
use crate::tools::requests::model::learn_requests;
use patterns::{because, source};

pub struct RequestsTool;
source!(RequestsTool, "a reader of requests that people said or wrote: the words of a request that fill a slot, and the label of the whole request. Nothing is written by hand: no list of cities, days or names and no word of a set; which questions ask for a slot, which words fill which slot and which frames mark a slot are all counted from the lessons");

pub const REQUESTS_TOOL: Tool = Tool { name: "requests", needs_labels: false, probe: false, asks: false, over: None, learn: learn_requests };
because!(REQUESTS_TOOL, RequestsTool, "the reader of requests in the list of the tools: it needs no label of a lesson sentence, it has no rule by the form of the question alone, and it asks for no answer; it reads the table of word classes of the run from its lessons");
