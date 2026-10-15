use crate::tools::parts::found::{Cells, Chart, ChartCell, FormsFound, READINGS_MOST, Top, holds_run};
use crate::tools::parts::language::{WordRow, Word};
use crate::tools::parts::ordered::Few;
use crate::tools::parts::places::{Bit, PartKind, Lexicon, OPEN_KIND, is_text_word, laid_out};
use patterns::{because, source};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use crate::base::first_sight::NumberTable;
use crate::base::lanes::on_lanes;

pub struct CheapestReading;
source!(CheapestReading, "the reading of a text with the found forms as a chart parser in the way of Cocke, Younger and Kasami, from the inside out: every run of the text gets the readings that its forms give from the readings of its parts; of several readings of the whole text the cheapest is taken, where the cost of a form in a place of another form is minus the logarithm of its share there in the readings of the lessons, as in a probabilistic grammar");

const DEEP_MOST: usize = 8;
because!(DEEP_MOST, CheapestReading, "the readings that are kept for one run of a text and one kind with the form that made each: enough that the true reading of a lesson is among them, few enough that a long text is read quickly");

const WAYS_MOST: usize = 6;
because!(WAYS_MOST, CheapestReading, "the ways to one reading that are kept: a reading that many forms give is counted for the first of them, the cheapest first");

const UNSEEN: f64 = 100.0;
because!(UNSEEN, CheapestReading, "the cost of a form in a place of another form where no lesson showed it: far above the cost of every place that a lesson showed, so such a reading is taken only when no other is there");

const ONE_FORM: f64 = 1.0;
because!(ONE_FORM, CheapestReading, "the cost that each form of a reading adds beside the cost of its place, so that of two readings with places of equal cost the one with fewer forms is cheaper");

const NO_COST: u32 = 1073741824;
because!(NO_COST, CheapestReading, "the cost of a reading that is not there yet, above every cost that the forms of a text can add up to");

pub type ReadPart = (usize, usize, PartKind, (WordRow, Top));
because!(ReadPart, CheapestReading, "one part of a reading: its first and end place in the text, its kind, and its answer with what made it");

#[derive(Clone, Debug)]
pub struct DeepEntry {
    pub cost: f64,
    pub ways: Vec<(f64, Top, Vec<ReadPart>)>,
}
because!(DeepEntry, CheapestReading, "one reading of a run of a text: its least cost, and the ways to it with their costs and their parts, the cheapest first");

pub type DeepCell = Few<PartKind, Few<(WordRow, Top), DeepEntry>>;
because!(DeepCell, CheapestReading, "the readings of one run of a text by kind, each by its answer and what made it");

pub type LessonCount = (HashSet<(Top, usize, Top)>, HashSet<Top>, HashSet<Vec<Word>>, HashSet<Top>);
because!(LessonCount, CheapestReading, "what the readings of one lesson show, before the lessons are summed: the forms in the places of forms, the forms of the whole text, the single parts, and the forms used");

pub type DeepChart = Cells<DeepCell>;
because!(DeepChart, CheapestReading, "the readings of every run of a text with what made each");

pub type WholeReading = (f64, isize, WordRow, Top, Vec<ReadPart>);
because!(WholeReading, CheapestReading, "a reading of a whole text: its cost, minus the runs that gave its form, its answer, its form and its parts");

pub type ReadingCounts = (NumberTable<(Top, usize, Top), usize>, NumberTable<(Top, usize), usize>, NumberTable<Top, usize>, NumberTable<Vec<Word>, usize>, HashSet<Top>, usize);
because!(ReadingCounts, CheapestReading, "what the readings of the lessons show: the lessons with a form in a place of a form, the lessons with a place of a form, the lessons with a form of the whole text, the lessons with a single part, the forms used, and the lessons with a reading that gives their answer");

fn cheaper(one: (f64, isize), other: (f64, isize)) -> bool {
    match one.0.partial_cmp(&other.0) {
        Some(Ordering::Less) => true,
        Some(Ordering::Equal) => one.1 < other.1,
        _ => false,
    }
}
because!(cheaper, CheapestReading, "whether one pair of a cost and a count is below another, the cost first; costs that are equal as numbers are equal also when one is a negative zero");

impl FormsFound {
    pub fn chart(&self, text: &[Word], answer: Option<&WordRow>, given: Option<&[(usize, PartKind, WordRow)]>) -> Chart {
        let n = text.len();
        self.count_cells(n);
        let words: HashSet<Word> = text.iter().copied().collect();
        let usable = self.usable(&words);
        let free = usable.get(&None);
        let held = |x: usize, y: usize| if y == x + 1 { given.and_then(|given| given.iter().find(|(place, _, _)| *place == x)) } else { None };
        let row_at = if self.views { Some(self.rows(text, given)) } else { None };
        let mut table = Chart::new(n);
        for size in 1..=n {
            for x in 0..=n - size {
                let y = x + size;
                let mut found: ChartCell = Few::default();
                if let Some((_, kind, row)) = held(x, y) {
                    found.put(*kind, Few::of(row.clone(), 0));
                } else if let Some(row_at) = &row_at {
                    if let Some((kind, row)) = row_at.get(&(x, y)) {
                        found.put(*kind, Few::of(row.clone(), 1));
                    }
                } else if size <= self.single_most {
                    if let Some((kind, row)) = self.one(text, x, y) {
                        if answer.is_none_or(|answer| holds_run(answer, &row)) {
                            found.put(kind, Few::of(row, 1));
                        }
                    }
                }
                if size > 1 {
                    for forms in [usable.get(&Some(text[x])), free] {
                        for form in forms.into_iter().flatten() {
                            if form.form.pattern.len() > size {
                                continue;
                            }
                            for (filled, cost) in self.laid(text, x, y, &form.form.pattern, 0, &table) {
                                let Some(row) = laid_out(&form.form.out, &filled) else { continue };
                                if answer.is_some_and(|answer| !holds_run(answer, &row)) {
                                    continue;
                                }
                                let rows = found.or_put(form.form.kind, Few::default);
                                if cost + 1 < rows.get(&row).copied().unwrap_or(NO_COST) {
                                    rows.put(row, cost + 1);
                                }
                            }
                        }
                    }
                }
                if !found.is_empty() {
                    for (_, rows) in found.items.iter_mut() {
                        if rows.len() > READINGS_MOST {
                            rows.items.sort_by_key(|(_, cost)| *cost);
                            rows.items.truncate(READINGS_MOST);
                        }
                    }
                    table.set(x, y, found);
                }
            }
        }
        table
    }

    pub fn rows(&self, text: &[Word], given: Option<&[(usize, PartKind, WordRow)]>) -> HashMap<(usize, usize), (PartKind, WordRow)> {
        let mut found: HashMap<(usize, usize), (PartKind, WordRow)> = HashMap::new();
        let mut start: Option<(usize, PartKind, WordRow)> = None;
        for x in 0..=text.len() {
            let held = given.is_some_and(|given| given.iter().any(|(place, _, _)| *place == x));
            let single = if x < text.len() && !held { self.one(text, x, x + 1) } else { None };
            if let Some((first, kind, row)) = &start {
                if single.as_ref().is_none_or(|(other, _)| other != kind) || !self.rows_on {
                    found.insert((*first, x), (*kind, row.clone()));
                    start = None;
                }
            }
            if let Some((kind, row)) = single {
                start = match start {
                    None => Some((x, kind, row)),
                    Some((first, held, mut so_far)) => {
                        so_far.extend(row);
                        Some((first, held, so_far))
                    }
                };
            }
        }
        found
    }

    pub fn one(&self, text: &[Word], x: usize, y: usize) -> Option<(PartKind, WordRow)> {
        if let Some((kind, row)) = self.single.find(&text[x..y]) {
            return Some((*kind, row.clone()));
        }
        let word = text[x];
        match (self.open_kind, &self.known_words) {
            (Some(open), Some(known)) if y - x == 1 && is_text_word(word) && !known.contains(&word) => Some((open, vec![word])),
            _ => None,
        }
    }

    pub fn opens(&self, pattern: &[Bit], word: Word) -> bool {
        is_text_word(word) && (self.open_words.get(pattern).is_some_and(|words| words.contains(&word)) || self.known_words.as_ref().is_some_and(|known| !known.contains(&word)))
    }

    pub fn laid(&self, text: &[Word], x: usize, y: usize, pattern: &[Bit], k: usize, table: &Chart) -> Vec<(Vec<WordRow>, u32)> {
        if k == pattern.len() {
            return if x == y { vec![(Vec::new(), 0)] } else { Vec::new() };
        }
        let left = pattern.len() - k - 1;
        let mut out: Vec<(Vec<WordRow>, u32)> = Vec::new();
        match pattern[k] {
            Bit::Word(word) => {
                if x < y && text[x] == word {
                    return self.laid(text, x + 1, y, pattern, k + 1, table);
                }
            }
            Bit::Part(OPEN_KIND) => {
                if x + left < y && self.opens(pattern, text[x]) {
                    for (rest, more) in self.laid(text, x + 1, y, pattern, k + 1, table) {
                        let mut filled = vec![vec![text[x]]];
                        filled.extend(rest);
                        out.push((filled, more));
                    }
                }
            }
            Bit::Part(kind) => {
                if y >= left {
                    for z in x + 1..=y - left {
                        let Some(rows) = table.get(x, z).and_then(|cell| cell.get(&kind)) else { continue };
                        for (row, cost) in rows.iter() {
                            for (rest, more) in self.laid(text, z, y, pattern, k + 1, table) {
                                let mut filled = vec![row.clone()];
                                filled.extend(rest);
                                out.push((filled, cost + more));
                            }
                        }
                    }
                }
            }
        }
        out
    }

    fn price(&self, top: &Top, slot: usize, inner: &Top) -> f64 {
        if self.seen.is_empty() {
            return f64::default();
        }
        match self.seen.get(&(top.clone(), slot, inner.clone())) {
            None => UNSEEN,
            Some(count) => -((*count as f64) / (self.places_seen.get(&(top.clone(), slot)).copied().unwrap_or_default() as f64)).ln(),
        }
    }

    fn lexical(&self, run: &[Word], kind: PartKind) -> f64 {
        if self.lex_seen.is_empty() {
            return f64::default();
        }
        match self.lex_seen.get(run) {
            None => UNSEEN,
            Some(count) => -((*count as f64) / (self.lex_all.get(&kind).copied().unwrap_or_default() as f64)).ln(),
        }
    }

    pub fn deep(&self, text: &[Word], answer: Option<&WordRow>) -> DeepChart {
        let n = text.len();
        let words: HashSet<Word> = text.iter().copied().collect();
        let usable = self.usable(&words);
        let free = usable.get(&None);
        let row_at = if self.views { Some(self.rows(text, None)) } else { None };
        let mut table = DeepChart::new(n);
        for size in 1..=n {
            for x in 0..=n - size {
                let y = x + size;
                let mut found: DeepCell = Few::default();
                if let Some(row_at) = &row_at {
                    if let Some((kind, row)) = row_at.get(&(x, y)) {
                        found.put(*kind, Few::of((row.clone(), Top::InRow(*kind)), DeepEntry { cost: f64::default(), ways: Vec::new() }));
                    }
                } else if size <= self.single_most {
                    if let Some((kind, row)) = self.one(text, x, y) {
                        if answer.is_none_or(|answer| holds_run(answer, &row)) {
                            let cost = self.lexical(&text[x..y], kind);
                            found.put(kind, Few::of((row, Top::Single(kind)), DeepEntry { cost, ways: Vec::new() }));
                        }
                    }
                }
                if size > 1 {
                    for forms in [usable.get(&Some(text[x])), free] {
                        for form in forms.into_iter().flatten() {
                            if form.form.pattern.len() > size {
                                continue;
                            }
                            let top = Top::Form(form.form.clone());
                            for (filled, cost) in self.laid_deep(text, x, y, &form.form.pattern, 0, &table, &top, 0) {
                                let answers: Vec<WordRow> = filled.iter().map(|part| part.3.0.clone()).collect();
                                let Some(row) = laid_out(&form.form.out, &answers) else { continue };
                                if answer.is_some_and(|answer| !holds_run(answer, &row)) {
                                    continue;
                                }
                                let rows = found.or_put(form.form.kind, Few::default);
                                let cost = cost + ONE_FORM;
                                let key = (row, top.clone());
                                match rows.get_mut(&key) {
                                    None => rows.put(key, DeepEntry { cost, ways: vec![(cost, top.clone(), filled)] }),
                                    Some(old) => {
                                        if cost < old.cost {
                                            old.cost = cost;
                                            old.ways.insert(0, (cost, top.clone(), filled));
                                            old.ways.truncate(WAYS_MOST);
                                        } else if old.ways.len() < WAYS_MOST {
                                            old.ways.push((cost, top.clone(), filled));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !found.is_empty() {
                    for (_, rows) in found.items.iter_mut() {
                        if rows.len() > DEEP_MOST {
                            rows.items.sort_by(|one, other| one.1.cost.partial_cmp(&other.1.cost).unwrap_or(Ordering::Equal));
                            rows.items.truncate(DEEP_MOST);
                        }
                    }
                    table.set(x, y, found);
                }
            }
        }
        table
    }

    #[allow(clippy::too_many_arguments)]
    fn laid_deep(&self, text: &[Word], x: usize, y: usize, pattern: &[Bit], k: usize, table: &DeepChart, top: &Top, slot: usize) -> Vec<(Vec<ReadPart>, f64)> {
        if k == pattern.len() {
            return if x == y { vec![(Vec::new(), f64::default())] } else { Vec::new() };
        }
        let left = pattern.len() - k - 1;
        let mut out: Vec<(Vec<ReadPart>, f64)> = Vec::new();
        match pattern[k] {
            Bit::Word(word) => {
                if x < y && text[x] == word {
                    return self.laid_deep(text, x + 1, y, pattern, k + 1, table, top, slot);
                }
            }
            Bit::Part(OPEN_KIND) => {
                if x + left < y && self.opens(pattern, text[x]) {
                    let here = self.price(top, slot, &Top::Open);
                    for (rest, more) in self.laid_deep(text, x + 1, y, pattern, k + 1, table, top, slot + 1) {
                        let mut filled = vec![(x, x + 1, OPEN_KIND, (vec![text[x]], Top::Open))];
                        filled.extend(rest);
                        out.push((filled, here + more));
                    }
                }
            }
            Bit::Part(kind) => {
                if y >= left {
                    for z in x + 1..=y - left {
                        let Some(rows) = table.get(x, z).and_then(|cell| cell.get(&kind)) else { continue };
                        for (key, entry) in rows.iter() {
                            let here = entry.cost + self.price(top, slot, &key.1);
                            for (rest, more) in self.laid_deep(text, z, y, pattern, k + 1, table, top, slot + 1) {
                                let mut filled = vec![(x, z, kind, key.clone())];
                                filled.extend(rest);
                                out.push((filled, here + more));
                            }
                        }
                    }
                }
            }
        }
        out
    }

    pub fn whole(&self, text: &[Word], table: &DeepChart) -> Vec<WholeReading> {
        let mut found: Vec<WholeReading> = Vec::new();
        let words: HashSet<Word> = text.iter().copied().collect();
        let total: usize = self.whole_seen.values().sum();
        for whole in &self.wholes {
            if whole.form.pattern.len() > text.len() || !whole.fixed.iter().all(|word| words.contains(word)) {
                continue;
            }
            let top = Top::Form(whole.form.clone());
            let price = if total == 0 {
                f64::default()
            } else {
                match self.whole_seen.get(&top) {
                    None => UNSEEN,
                    Some(count) => -((*count as f64) / (total as f64)).ln(),
                }
            };
            for (filled, cost) in self.laid_deep(text, 0, text.len(), &whole.form.pattern, 0, table, &top, 0) {
                let answers: Vec<WordRow> = filled.iter().map(|part| part.3.0.clone()).collect();
                if let Some(row) = laid_out(&whole.form.out, &answers) {
                    found.push((cost + price, -(whole.count as isize), row, top.clone(), filled));
                }
            }
        }
        found
    }

    fn count_one(&self, text: &[Word], answer: &WordRow, every: bool) -> Option<LessonCount> {
        let mut used: HashSet<Top> = HashSet::new();
        let table = self.deep(text, if self.views { None } else { Some(answer) });
        let mut right: Vec<WholeReading> = self.whole(text, &table).into_iter().filter(|reading| reading.2 == *answer).collect();
        if right.is_empty() {
            return None;
        }
        if !every {
            let mut best = 0;
            for place in 1..right.len() {
                if cheaper((right[place].0, right[place].1), (right[best].0, right[best].1)) {
                    best = place;
                }
            }
            right = vec![right.swap_remove(best)];
        }
        let mut here: HashSet<(Top, usize, Top)> = HashSet::new();
        let mut singles: HashSet<Vec<Word>> = HashSet::new();
        let mut done: HashSet<ReadPart> = HashSet::new();
        let tops: HashSet<Top> = right.iter().map(|reading| reading.3.clone()).collect();
        let mut todo: Vec<(Top, Vec<ReadPart>)> = right.into_iter().map(|reading| (reading.3, reading.4)).collect();
        while let Some((top, filled)) = todo.pop() {
            used.insert(top.clone());
            for (slot, part) in filled.iter().enumerate() {
                let (x, y, kind, key) = part;
                here.insert((top.clone(), slot, key.1.clone()));
                match &key.1 {
                    Top::Single(_) => {
                        singles.insert(text[*x..*y].to_vec());
                    }
                    Top::InRow(_) => {
                        for z in *x..*y {
                            singles.insert(text[z..=z].to_vec());
                        }
                    }
                    Top::Open => {}
                    Top::Form(_) => {
                        if done.insert(part.clone()) {
                            if let Some(entry) = table.get(*x, *y).and_then(|cell| cell.get(kind)).and_then(|rows| rows.get(key)) {
                                let take = if every { entry.ways.len() } else { entry.ways.len().min(1) };
                                todo.extend(entry.ways[..take].iter().map(|way| (way.1.clone(), way.2.clone())));
                            }
                        }
                    }
                }
            }
        }
        Some((here, tops, singles, used))
    }

    fn count(&self, lessons: &[(Vec<Word>, WordRow)], every: bool) -> ReadingCounts {
        let mut seen: NumberTable<(Top, usize, Top), usize> = NumberTable::default();
        let mut places: NumberTable<(Top, usize), usize> = NumberTable::default();
        let mut whole: NumberTable<Top, usize> = NumberTable::default();
        let mut lex: NumberTable<Vec<Word>, usize> = NumberTable::default();
        let mut used: HashSet<Top> = HashSet::new();
        let mut covered = 0;
        let each: Vec<Option<LessonCount>> = on_lanes(lessons.len(), self.lanes, &|piece| self.count_one(&lessons[piece].0, &lessons[piece].1, every));
        for (here, tops, singles, mine) in each.into_iter().flatten() {
            covered += 1;
            used.extend(mine);
            let pairs: HashSet<(Top, usize)> = here.iter().map(|triple| (triple.0.clone(), triple.1)).collect();
            for triple in here {
                *seen.entry(triple).or_default() += 1;
            }
            for pair in pairs {
                *places.entry(pair).or_default() += 1;
            }
            for top in tops {
                *whole.entry(top).or_default() += 1;
            }
            for run in singles {
                *lex.entry(run).or_default() += 1;
            }
        }
        (seen, places, whole, lex, used, covered)
    }

    fn set_counts(&mut self, seen: NumberTable<(Top, usize, Top), usize>, places: NumberTable<(Top, usize), usize>, whole: NumberTable<Top, usize>, lex: NumberTable<Vec<Word>, usize>) {
        self.seen = seen;
        self.places_seen = places;
        self.whole_seen = whole;
        self.lex_seen = lex;
        self.lex_all = HashMap::new();
        for (run, count) in &self.lex_seen {
            if let Some((kind, _)) = self.single.find(run.as_slice()) {
                *self.lex_all.entry(*kind).or_default() += count;
            }
        }
    }

    pub fn nestings(&mut self, lessons: &[(Vec<Word>, WordRow)]) {
        let (seen, places, whole, lex, _, _) = self.count(lessons, true);
        self.set_counts(seen, places, whole, lex);
        let (seen, places, whole, lex, used, covered) = self.count(lessons, false);
        if !self.views {
            self.single.retain(|run, _| lex.contains_key(run));
        }
        self.forms.retain(|form, _| used.contains(&Top::Form(form.clone())));
        self.single_most = self.single.keys().map(Vec::len).max().unwrap_or_default();
        self.index();
        self.set_counts(seen, places, whole, lex);
        self.right = covered;
    }

    pub fn read(&self, text: &[Word], lexicon: &Lexicon) -> Option<WordRow> {
        let table = self.deep(text, None);
        let mut best: crate::tools::parts::ordered::Kept<WordRow, (f64, isize)> = crate::tools::parts::ordered::Kept::default();
        for (cost, count, row, _, _) in self.whole(text, &table) {
            let floor = (f64::from(NO_COST), 0);
            if cheaper((cost, count), best.get(&row).copied().unwrap_or(floor)) {
                best.put(row, (cost, count));
            }
        }
        let mut readings: Vec<(f64, isize, WordRow)> = best.into_pairs().map(|(row, (cost, count))| (cost, count, row)).collect();
        readings.sort_by(|one, other| one.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal).then(one.1.cmp(&other.1)).then_with(|| lexicon.cmp_rows(&one.2, &other.2)));
        readings.into_iter().next().map(|(_, _, row)| row)
    }
}
