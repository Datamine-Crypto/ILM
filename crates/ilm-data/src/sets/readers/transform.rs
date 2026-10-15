use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{even_lessons, table_columns};
use patterns::{because, source};

pub struct TransformSet;
source!(TransformSet, "the syntactic transformations of Mueller and others: a sentence into a question and into the passive, in English and in German, each part of the set with a train file, a test file from the distribution of the train file, and a generalization file of a build that no train sentence has; the question of a case is the prefix of the row and its sentence, the answer the changed sentence; a row with an empty sentence or an empty answer is no case");

const SOURCE_COLUMN: &str = "src";
because!(SOURCE_COLUMN, TransformSet, "the column of the sentence as it is before the change, which with its prefix is the question of a case");

const TARGET_COLUMN: &str = "tgt";
because!(TARGET_COLUMN, TransformSet, "the column of the sentence after the change, which is the answer of a case");

const PREFIX_COLUMN: &str = "prefix";
because!(PREFIX_COLUMN, TransformSet, "the column of the words before the sentence that say which change is asked");

const TRAIN_FILE: &str = "train";
because!(TRAIN_FILE, TransformSet, "the role of the train file of a part of the set");

const TEST_FILE: &str = "test";
because!(TEST_FILE, TransformSet, "the role of the test file of a part, and the end of the name of a kind that reads it");

const GENERALIZATION_FILE: &str = "generalization";
because!(GENERALIZATION_FILE, TransformSet, "the role of the generalization file of a part, which a kind with another end of its name reads");

const KIND_END_MARK: char = '_';
because!(KIND_END_MARK, TransformSet, "the mark before the end of the name of a kind, which says which file of its part it reads");

fn transform_part(described: &Described, name: &str, which: &str) -> Result<Vec<Case>, String> {
    let place = described.file(name, which)?;
    let [sources, targets, prefixes] = table_columns(&place, [SOURCE_COLUMN, TARGET_COLUMN, PREFIX_COLUMN])?;
    let filled = |value: &Value| value.text().filter(|text| !text.is_empty()).map(str::to_string);
    let mut out: Vec<Case> = Vec::new();
    for ((source, target), prefix) in sources.iter().zip(&targets).zip(&prefixes) {
        if let (Some(sentence), Some(changed)) = (filled(source), filled(target)) {
            let prefix = prefix.text().unwrap_or_default();
            out.push(Case { sentences: Vec::new(), question: format!("{prefix}{sentence}"), answer: changed });
        }
    }
    Ok(out)
}
because!(transform_part, TransformSet, "the cases of one file of a part of the set, in the order of the file");

pub fn transform_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let (name, end) = kind.rsplit_once(KIND_END_MARK).ok_or_else(|| format!("the kind {kind} names no file"))?;
    let which = if end == TEST_FILE { TEST_FILE } else { GENERALIZATION_FILE };
    let train = transform_part(described, name, TRAIN_FILE)?;
    let read_from = Some(train.len());
    Ok(KindCases { cases: even_lessons(train, transform_part(described, name, which)?), read_from, ..KindCases::default() })
}
because!(transform_cases, TransformSet, "the cases of a kind: the lessons spread evenly over the train file of its part, then the file that the end of its name says");
