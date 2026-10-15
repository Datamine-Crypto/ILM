use crate::base::first_sight::{FirstSightCount, FirstSightTable, all_counts};
use crate::base::symbols::Symbols;
use crate::tools::link::joins::{Joins, LINK_LATER_STEPS, LINK_LEAST, LINK_PAIR, LinkLessons};
use patterns::{because, source};

pub struct ChainJoin;
source!(ChainJoin, "the join table of the link tool taught on chains of labels, for a tool that reads the label of each link of a way by itself: a lesson gives the labels of the links of its way in their order and its answer, and the label of a link is the label of the answer with the same words, so the table is counted at step zero and at the later steps as for the ways of the link tool. A pair of labels whose lessons have more than one answer gets the most frequent one at a step after all others, so it is used only where no other entry gives a join");

pub(crate) struct ChainJoins {
    lessons: LinkLessons,
    items: Symbols,
    joins: Joins,
}
because!(ChainJoins, ChainJoin, "the join table of chains of labels with the texts of its labels and the lessons that taught it. learn counts it from the chains of the lessons and how often each label and answer stands; joined gives the label of a whole chain, or nothing when no join or two equal ones give it, with the lent joins or with the counted joins alone");

impl ChainJoins {
    pub(crate) fn learn(chains: &[(Vec<String>, String)], often: &FirstSightCount<String>) -> ChainJoins {
        let mut items = Symbols::new();
        let mut lessons = LinkLessons::default();
        for (text, count) in often.rows() {
            let number = items.number(text);
            lessons.often.insert(number, *count as u32);
        }
        let mut results: Vec<u32> = often.rows().iter().map(|(text, _)| items.number(text)).collect();
        results.sort_by(|left, right| items.name(*left).cmp(items.name(*right)));
        lessons.results = results;
        lessons.lessons = chains.iter().map(|(labels, answer)| (labels.iter().map(|label| items.number(label)).collect(), items.number(answer))).collect();
        let (mut joins, _) = lessons.count(&|label| label, true, &items);
        let mut pairs: FirstSightTable<(u32, u32), FirstSightCount<u32>> = FirstSightTable::new();
        for (labels, answer) in &lessons.lessons {
            if labels.len() == LINK_PAIR {
                pairs.at((labels[0], labels[1])).add(*answer, 1);
            }
        }
        for (key, counts) in pairs.rows() {
            if joins.table.holds(key) || all_counts(counts) < i64::from(LINK_LEAST) {
                continue;
            }
            if let Some((answer, _)) = counts.first_most() {
                joins.table.put(*key, *answer);
                joins.step.insert(*key, LINK_LATER_STEPS + 1);
            }
        }
        ChainJoins { lessons, items, joins }
    }

    pub(crate) fn joined(&self, labels: &[String], lent: bool) -> Option<String> {
        let numbers: Vec<u32> = labels.iter().map(|label| self.items.find(label)).collect::<Option<Vec<u32>>>()?;
        let span = if lent { self.lessons.chart(&numbers, &self.joins) } else { self.lessons.chart_counted(&numbers, &self.joins) };
        let whole = span.get(numbers.len())?;
        self.lessons.best(whole, &self.items).map(|answer| self.items.name(answer).to_string())
    }
}
