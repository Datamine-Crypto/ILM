pub mod bigbench;
pub mod common;
pub mod formal;
pub mod graphs;
pub mod inflect;
pub mod jsonsets;
pub mod kinship;
pub mod lonli;
pub mod lookup;
pub mod pcfg;
pub mod proofs;
pub mod regex;
pub mod requests;
pub mod scan;
pub mod stories;
pub mod tomi;
pub mod transform;
pub mod wordproblems;
pub mod writing;

use ilm_core::cases::Case;
use ilm_core::base::lanes::hardware_lanes;
use crate::sets::export::{Described, EXPORT_FIELD, FILE_KEY, KindCases, LabelLines, LessonLabels, write_set};
use patterns::{because, source};
use std::path::Path;
use std::process::ExitCode;

pub struct SetReaders;
source!(SetReaders, "the readers of the public sets: each holds the logic of one set (which raw files, which columns, which rows, in which order, how a case is made), and takes the names of its raw files from the description of the set");

pub type SetReader = fn(&Described, &str) -> Result<KindCases, String>;
because!(SetReader, SetReaders, "the reader of one set: the cases of one kind from the raw files that the description names");

const READERS: &[(&str, SetReader)] = &[("src_scan_step", scan::scan_step_cases), ("src_pcfg", pcfg::pcfg_cases), ("src_regex", regex::regex_cases), ("src_transform", transform::transform_cases),
    ("src_stepgame", stories::by_value_cases),
    ("src_fld", stories::by_value_cases),
    ("src_boardgame", stories::boardgame_cases),
    ("src_mindgames", stories::by_value_cases),
    ("src_prosqa", stories::prosqa_cases),
    ("src_ruletaker", stories::by_value_cases),
    ("src_lonli", lonli::lonli_cases),
    ("src_nlgraph", graphs::nlgraph_cases),
    ("src_tomqa", graphs::tomqa_cases),
    ("src_babi", jsonsets::babi_cases),
    ("src_spartqa", jsonsets::spartqa_cases),
    ("src_prontoqa", jsonsets::prontoqa_cases),
    ("src_atis", requests::atis_cases),
    ("src_snips", requests::snips_cases),
    ("src_tomi", tomi::tomi_cases),
    ("src_clutrr", kinship::clutrr_cases),
    ("src_clutrr_graph", kinship::clutrr_graph_cases),
    ("src_proofwriter", proofs::proof_cases),
    ("src_pararules", proofs::proof_cases),
    ("src_bigbench", bigbench::bench_cases),
    ("src_wordproblems", wordproblems::problem_kind_cases),
    ("src_lookup", lookup::lookup_cases),
    ("src_inflect", inflect::inflect_cases),
];
pub type SetLabels = fn(&Described, &str, &[&Case]) -> Result<Vec<LabelLines>, String>;
because!(SetLabels, SetReaders, "the labels of the lessons of one set: the label lines of each lesson case of a kind from the raw files that the description names");

const LABELLED: &[(&str, SetLabels)] = &[("src_proofwriter", formal::formal_labels), ("src_pararules", formal::formal_labels), ("src_clutrr", kinship::clutrr_labels)];
because!(LABELLED, SetReaders, "the sets that give formal forms beside their lesson sentences, with what reads them");

because!(READERS, SetReaders, "the sets that the program reads itself, each by the name that its description and its folder of case files have");

fn missing_raw_files(about: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(about) else { return Vec::new() };
    text.lines()
        .filter(|line| line.split(EXPORT_FIELD).next() == Some(FILE_KEY))
        .filter_map(|line| line.rsplit(EXPORT_FIELD).next())
        .filter(|place| !Path::new(place).exists())
        .map(str::to_string)
        .collect()
}
because!(missing_raw_files, SetReaders, "the raw files that a description names and that are not there, in the order of the description; a description that cannot be read gives none here, and its own read says why");

pub fn export_command(rest: &[String]) -> ExitCode {
    let [source, about, folder] = rest else {
        eprintln!("usage: ilm export <set> <description file> <folder of the case files>");
        return ExitCode::FAILURE;
    };
    let missing = missing_raw_files(Path::new(about));
    if let Some(first) = missing.first() {
        eprintln!("{first}: a raw file of the set is missing ({} in all); fetch them first: ilm fetch data/sources.txt sets={source}", missing.len());
        return ExitCode::FAILURE;
    }
    if let Ok(described) = Described::read(Path::new(about))
        && let Some(written) = writing::write_rows(&described, Path::new(folder)).or_else(|| crate::sets::knowledge::write_knowledge(&described, Path::new(folder)))
    {
        return match written {
            Ok(said) => {
                said.iter().for_each(|line| println!("{line}"));
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("{why}");
                ExitCode::FAILURE
            }
        };
    }
    let Some((_, reader)) = READERS.iter().find(|(name, _)| name == source) else {
        eprintln!("no reader for the set {source}; the sets: {}", READERS.iter().map(|(name, _)| *name).collect::<Vec<&str>>().join(", "));
        return ExitCode::FAILURE;
    };
    let described = match Described::read(Path::new(about)) {
        Ok(described) => described,
        Err(why) => {
            eprintln!("{why}");
            return ExitCode::FAILURE;
        }
    };
    let labels = LABELLED.iter().find(|(name, _)| name == source).map(|(_, give)| *give);
    let labelled = |kind: &str, lessons: &[&Case]| labels.map_or_else(|| Ok(Vec::new()), |give| give(&described, kind, lessons));
    let read = |kind: &str| reader(&described, kind);
    match write_set(source, &described, Path::new(folder), &read, labels.map(|_| &labelled as LessonLabels), hardware_lanes()) {
        Ok(said) => {
            said.iter().for_each(|line| println!("{line}"));
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}
because!(export_command, SetReaders, "the case files of one set written from its raw files by its reader: the set, its description, and the folder that takes the files");
