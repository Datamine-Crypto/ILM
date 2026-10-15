use crate::base::keys::padded;
use crate::base::sums::compensated;
use crate::base::symbols::Symbols;
use crate::write::check::CheckRun;
use crate::write::measure::FactList;
use crate::write::numbers::{e_to, natural_log};
use crate::write::signs::low_tokens;
use crate::base::first_sight::{FirstSightTable, NumberTable};
use patterns::{because, source};

pub struct RunOverlap;
source!(RunOverlap, "the measure of Papineni, Roukos, Ward and Zhu for a written text against texts by persons: how many runs of tokens of the written texts stand in a reference of the same input, over a whole part, as the mean of the logarithms of the shares for each length of run, with a penalty for texts that are shorter than their references");

pub const WRITE_OVERLAP_RUN: usize = 4;
because!(WRITE_OVERLAP_RUN, RunOverlap, "the longest run of tokens that the measure counts, as its authors set it");

pub const WRITE_OVERLAP_SCALE: f64 = 100.0;
because!(WRITE_OVERLAP_SCALE, RunOverlap, "the measure is given in hundredths, as the field reports it");

fn run_counts(toks: &[u32], size: usize) -> FirstSightTable<CheckRun, i64> {
    let mut seen: FirstSightTable<CheckRun, i64> = FirstSightTable::new();
    for run in toks.windows(size) {
        *seen.at(padded(run)) += 1;
    }
    seen
}
because!(run_counts, RunOverlap, "how often each run of one length stands in a text");

pub fn run_overlap(written: &[Vec<u32>], references: &[Vec<Vec<u32>>]) -> f64 {
    let mut matched = [0i64; WRITE_OVERLAP_RUN];
    let mut total = [0i64; WRITE_OVERLAP_RUN];
    let mut length = 0i64;
    let mut reference_length = 0i64;
    for (text, refs) in written.iter().zip(references) {
        length += text.len() as i64;
        let nearest = refs.iter().map(|reference| ((reference.len() as i64 - text.len() as i64).abs(), reference.len() as i64)).min();
        reference_length += nearest.map_or(0, |(_, size)| size);
        for size in 1..=WRITE_OVERLAP_RUN {
            let mut most: NumberTable<CheckRun, i64> = NumberTable::default();
            for reference in refs {
                for (run, many) in run_counts(reference, size).rows() {
                    let seen = most.entry(*run).or_insert(0);
                    if *many > *seen {
                        *seen = *many;
                    }
                }
            }
            matched[size - 1] += run_counts(text, size).rows().iter().map(|(run, many)| (*many).min(most.get(run).copied().unwrap_or(0))).sum::<i64>();
            total[size - 1] += (text.len() + 1).saturating_sub(size) as i64;
        }
    }
    if matched.contains(&0) || length == 0 {
        return f64::default();
    }
    let one = f64::from(true);
    let penalty = if length > reference_length { one } else { e_to(one - (reference_length as f64) / (length as f64)) };
    let logs = compensated(matched.iter().zip(&total).map(|(got, all)| natural_log((*got as f64) / (*all as f64))));
    WRITE_OVERLAP_SCALE * penalty * e_to(logs / (WRITE_OVERLAP_RUN as f64))
}
because!(run_overlap, RunOverlap, "the measure for the written texts of a part, each with its references: for each length of run the runs of the written text that a reference holds, counted at most as often as the reference with the most has them; the reference length of a text is the length nearest to its own, the shorter of equals");

pub fn person_overlap(lists: &[FactList]) -> f64 {
    let mut toks = Symbols::new();
    let mut numbered = |text: &str| -> Vec<u32> { low_tokens(text).iter().map(|token| toks.number(token)).collect() };
    let several: Vec<&FactList> = lists.iter().filter(|list| list.references.len() > 1).collect();
    let first: Vec<Vec<u32>> = several.iter().map(|list| numbered(&list.references[0])).collect();
    let others: Vec<Vec<Vec<u32>>> = several.iter().map(|list| list.references[1..].iter().map(|text| numbered(text)).collect()).collect();
    run_overlap(&first, &others)
}
because!(person_overlap, RunOverlap, "the overlap of a person: for the lists with more than one reference, the first reference against the others; a writer is measured against the same other references beside it");
