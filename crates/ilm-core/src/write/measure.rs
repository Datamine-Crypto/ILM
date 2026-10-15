use crate::write::rows::FactRow;
use crate::base::first_sight::NumberTable;
use patterns::{because, source};

pub struct WriterMeasure;
source!(WriterMeasure, "the common measure for a writer from facts to text: the lists of facts of the train part of a set are put in the order of their first row; the rows of the first lists are the lessons and the rows of the last lists are the part on which a writer is developed, so that no lesson holds a list of that part; lessons are taken at even steps and never by chance; the texts of one list of facts by several persons are its references");

pub const WRITE_LEARN_LISTS: f64 = 0.8;
because!(WRITE_LEARN_LISTS, WriterMeasure, "the share of the lists of facts of the train part whose rows are the lessons; the rows of the other lists are the part for development");

pub type FactKey = Vec<(String, String)>;
because!(FactKey, WriterMeasure, "a list of facts as one key: its facts in the order of their texts, so that two rows with the same facts in another order are one list");

pub fn fact_key(row: &FactRow) -> FactKey {
    let mut key = row.facts.clone();
    key.sort();
    key
}
because!(fact_key, WriterMeasure, "the key of the list of facts of a row");

pub fn learn_and_dev(train: &[FactRow]) -> (Vec<FactRow>, Vec<FactRow>) {
    let mut order: NumberTable<FactKey, usize> = NumberTable::default();
    let keys: Vec<usize> = train
        .iter()
        .map(|row| {
            let fresh = order.len();
            *order.entry(fact_key(row)).or_insert(fresh)
        })
        .collect();
    let learn = ((order.len() as f64) * WRITE_LEARN_LISTS) as usize;
    let lessons = train.iter().zip(&keys).filter(|(_, key)| **key < learn).map(|(row, _)| row.clone()).collect();
    let dev = train.iter().zip(&keys).filter(|(_, key)| **key >= learn).map(|(row, _)| row.clone()).collect();
    (lessons, dev)
}
because!(learn_and_dev, WriterMeasure, "the rows of the learn lists and the rows of the other lists of the train part, each in the order of the file");

pub fn at_even_steps(rows: &[FactRow], wanted: usize) -> Vec<FactRow> {
    if wanted == 0 || wanted >= rows.len() {
        return rows.to_vec();
    }
    (0..wanted).map(|step| rows[(((step * rows.len()) as f64) / (wanted as f64)) as usize].clone()).collect()
}
because!(at_even_steps, WriterMeasure, "so many rows at even steps through the rows, the same rows in every run; all rows when fewer are there or when nought is asked");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FactList {
    pub facts: Vec<(String, String)>,
    pub references: Vec<String>,
}
because!(FactList, WriterMeasure, "one list of facts of a part, with its facts in the order of its first row and the texts of every row that has these facts");

pub fn grouped_lists(rows: &[FactRow]) -> Vec<FactList> {
    let mut place: NumberTable<FactKey, usize> = NumberTable::default();
    let mut lists: Vec<FactList> = Vec::new();
    for row in rows {
        let fresh = lists.len();
        let at = *place.entry(fact_key(row)).or_insert(fresh);
        if at == fresh {
            lists.push(FactList { facts: row.facts.clone(), references: Vec::new() });
        }
        lists[at].references.push(row.text.clone());
    }
    lists
}
because!(grouped_lists, WriterMeasure, "the lists of facts of a part in the order of their first row, each with its references");
