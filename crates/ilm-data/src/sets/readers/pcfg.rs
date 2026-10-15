use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::readers::common::{even_lessons, lines_of, stripped, text_of};
use patterns::{because, source};

pub struct PcfgSet;
source!(PcfgSet, "PCFG SET of Hupkes, Dankers, Mul and Bruni: texts of function words and items in prefix notation and the rows of items that they give, in three splits, as plain files of the repository of the authors: one line of a text file is a text, the same line of a target file its row; a pair where either side is empty after the strip of its white space is no case");

const PCFG_TRAIN: &str = "train";
because!(PCFG_TRAIN, PcfgSet, "the part of a split whose cases are the lessons");

const PCFG_TEST: &str = "test";
because!(PCFG_TEST, PcfgSet, "the part of a split whose cases are read");

const TEXTS: &str = "src";
because!(TEXTS, PcfgSet, "the role in the description of the file with the texts of a part");

const ROWS: &str = "tgt";
because!(ROWS, PcfgSet, "the role in the description of the file with the rows of a part");

fn pcfg_part(described: &Described, kind: &str, part: &str) -> Result<Vec<Case>, String> {
    let read = |end: &str| -> Result<Vec<String>, String> {
        let place = described.file(kind, &format!("{part}.{end}"))?;
        Ok(lines_of(&text_of(&place)?).iter().map(|line| stripped(line).to_string()).collect())
    };
    let (texts, rows) = (read(TEXTS)?, read(ROWS)?);
    if texts.len() != rows.len() {
        return Err(format!("{kind} {part}: {} texts and {} rows", texts.len(), rows.len()));
    }
    Ok(texts.into_iter().zip(rows).filter(|(text, row)| !text.is_empty() && !row.is_empty()).map(|(question, answer)| Case { sentences: Vec::new(), question, answer }).collect())
}
because!(pcfg_part, PcfgSet, "the cases of one part of a split, in the order of its lines");

pub fn pcfg_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let train = pcfg_part(described, kind, PCFG_TRAIN)?;
    let test = pcfg_part(described, kind, PCFG_TEST)?;
    let read_from = Some(train.len());
    Ok(KindCases { cases: even_lessons(train, test), read_from, ..KindCases::default() })
}
because!(pcfg_cases, PcfgSet, "the cases of a kind of PCFG SET with its lessons spread evenly over the train part, and the place of the first test case");
