use crate::base::symbols::Symbols;
use crate::write::check::{FactCheck, FactFault, FactMarks};
use crate::write::measure::FactList;
use crate::write::overlap::{RunOverlap, run_overlap};
use crate::write::signs::low_tokens;
use patterns::because;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeasuredTexts {
    pub run_overlap: f64,
    pub beside: f64,
    pub right: usize,
    pub lost: usize,
    pub wrong: usize,
    pub added: usize,
    pub tokens: f64,
    pub different: usize,
}
because!(MeasuredTexts, FactMarks, RunOverlap, "the numbers of the measure for the texts of one writer on one part: the overlap with all references; the overlap with the references after the first, for the lists with more than one, where the first reference is measured beside the writer as a person; the texts that the check finds right; the faults by sort; the mean number of tokens of a text; and how many different texts were written");

pub fn measure_texts(check: &FactCheck, lists: &[FactList], texts: &[String]) -> MeasuredTexts {
    let mut toks = Symbols::new();
    let mut numbered = |text: &str| -> Vec<u32> { low_tokens(text).iter().map(|token| toks.number(token)).collect() };
    let references: Vec<Vec<Vec<u32>>> = lists.iter().map(|list| list.references.iter().map(|text| numbered(text)).collect()).collect();
    let written: Vec<Vec<u32>> = texts.iter().map(|text| numbered(text)).collect();
    let several: Vec<usize> = (0..lists.len()).filter(|at| references[*at].len() > 1).collect();
    let mut out = MeasuredTexts { run_overlap: run_overlap(&written, &references), ..MeasuredTexts::default() };
    let beside_written: Vec<Vec<u32>> = several.iter().map(|at| written[*at].clone()).collect();
    let beside_references: Vec<Vec<Vec<u32>>> = several.iter().map(|at| references[*at][1..].to_vec()).collect();
    out.beside = run_overlap(&beside_written, &beside_references);
    for (list, text) in lists.iter().zip(texts) {
        let faults = check.faults(&list.facts, text);
        out.right += usize::from(faults.is_empty());
        out.lost += faults.iter().filter(|fault| **fault == FactFault::Lost).count();
        out.wrong += faults.iter().filter(|fault| **fault == FactFault::Wrong).count();
        out.added += faults.iter().filter(|fault| **fault == FactFault::Added).count();
    }
    out.tokens = (written.iter().map(Vec::len).sum::<usize>() as f64) / (written.len().max(1) as f64);
    let mut different: Vec<&String> = texts.iter().collect();
    different.sort();
    different.dedup();
    out.different = different.len();
    out
}
because!(measure_texts, FactMarks, RunOverlap, "the measure of the texts of a writer for the lists of facts of a part, each list with its references");
