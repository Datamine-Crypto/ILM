use crate::base::lanes::on_lanes;
use crate::base::sums::compensated;
use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words as cut_words};
use crate::cases::Case;
use crate::tools::pattern::classes::{ABSENT, KIND_MARK, sorted_texts, word_classes};
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::PatternWords;
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::WordTexts;
use crate::tools::pattern::told::{CHOICE_TURNS, FIT_GAIN, Known, LEAST_STORIES, MOST_ALIKE, MOST_ANSWERS, MOST_HOLDERS, MOST_MISSES, MOST_REFERS, QUICK_SHARE, Shape, Sure, Switch, TINY, TOLD_LEAST, TOLD_LESSONS, TRIED_STORIES, TableRow, ToldRules, View, as_written, class_text, embed, in_order, shape_name};
use crate::tools::pattern::told_read::ReadStory;
use patterns::{because, source};
use std::collections::HashSet;

pub struct TableFit;
source!(TableFit, "the fit of a reading of the told sentences to the lessons: the table of answers by row is counted on the lessons, and each row adds the lessons that a guess by the shares of its answers gets right: its counts squared and added, over its lessons; the rows are added with the compensated sum, so equal fits compare as equal");

pub struct LessonStories {
    pub stories: Vec<Vec<Vec<u32>>>,
    pub texts: Vec<Vec<String>>,
    pub asks: Vec<Vec<(Vec<u32>, Vec<u32>)>>,
    pub lanes: usize,
}
because!(LessonStories, TableFit, "the lessons of the reader by story: each different story once, in the order of its first lesson, with its read sentences, its texts, and its questions with their answers; and the threads on which the stories are read");

pub type Rows = FirstSightTable<TableRow, FirstSightTable<Vec<u32>, u32>>;
because!(Rows, TableFit, "the table of answers of the lessons: for each row, the answers of the lessons of the row with their counts, in the order of first sight");

fn total<K: Clone + Eq + std::hash::Hash>(counts: &FirstSightTable<K, u32>) -> u32 {
    counts.rows().iter().map(|(_, count)| *count).sum()
}
because!(total, TableFit, "the lessons of one row of the table");

fn guessed(rows: &Rows) -> f64 {
    compensated(rows.rows().iter().map(|(_, counts)| {
        let squares: u64 = counts.rows().iter().map(|(_, count)| u64::from(*count) * u64::from(*count)).sum();
        squares as f64 / f64::from(total(counts))
    }))
}
because!(guessed, TableFit, "the fit of a table: for each row its counts squared and added, over its lessons, added over the rows in their order");

fn sure_share_of<K: Clone + Eq + std::hash::Hash>(counts: &FirstSightTable<K, u32>, share: f64, least: u32) -> bool {
    let all = total(counts);
    all >= least && counts.first_most().is_some_and(|(_, hit)| f64::from(*hit) >= share * f64::from(all))
}
because!(sure_share_of, TableFit, "a row has one answer in the given share of its lessons, and enough lessons");

impl ToldRules {
    fn story_rows<N: WordTexts>(&self, words: &PatternWords, names: &N, lessons: &LessonStories, use_parts: bool, story: usize) -> Vec<(TableRow, usize)> {
        let view = View::default();
        let ReadStory { facts, rules, things, .. } = self.read(words, &view, &lessons.stories[story], !use_parts);
        let given = self.closure(names, &facts, &rules, &things);
        let mut have: HashSet<Known> = facts.into_iter().collect();
        have.extend(given);
        lessons.asks[story].iter().enumerate().filter_map(|(place, (q, _))| self.row(words, &view, q, &have).map(|row| (row, place))).collect()
    }

    pub fn fit<N: WordTexts + Sync>(&self, words: &PatternWords, names: &N, lessons: &LessonStories, use_parts: bool, only: Option<&HashSet<usize>>) -> (f64, Rows) {
        let stories: Vec<usize> = (0..lessons.asks.len()).filter(|story| only.is_none_or(|chosen| chosen.contains(story))).collect();
        let found: Vec<Vec<(TableRow, usize)>> = if lessons.lanes > 1 && stories.len() >= PARALLEL_STORIES {
            on_lanes(stories.len(), lessons.lanes, &|place| self.story_rows(words, names, lessons, use_parts, stories[place]))
        } else {
            stories.iter().map(|story| self.story_rows(words, names, lessons, use_parts, *story)).collect()
        };
        let mut rows: Rows = FirstSightTable::new();
        for (story, story_rows) in stories.iter().zip(found) {
            for (row, place) in story_rows {
                rows.entry(row, FirstSightTable::new()).add(lessons.asks[*story][place].1.clone(), 1);
            }
        }
        (guessed(&rows), rows)
    }

    pub fn learn(words: &PatternWords, symbols: &mut Symbols, cases: &[Case], sure_n: u32, sure_share: f64, lanes: usize) -> ToldRules {
        let mut told = ToldRules { last: true, capital: ABSENT, ..ToldRules::default() };
        let cases = &cases[..cases.len().min(TOLD_LESSONS)];
        let asked: Vec<(Vec<u32>, Vec<u32>)> = cases.iter().map(|case| (words.read(symbols, &cut_words(&case.question)), words.read(symbols, &answer_words(&case.answer)))).collect();
        let questions: HashSet<Vec<u32>> = asked.iter().map(|(q, _)| q.clone()).collect();
        let roots = word_classes(symbols, &sorted_texts(symbols, &questions), &[]);
        let mut often: CheapMap<u32, u32> = CheapMap::default();
        for word in asked.iter().flat_map(|(q, _)| q.iter()) {
            *often.entry(*word).or_insert(0) += 1;
        }
        let mut first: CheapMap<u32, u32> = CheapMap::default();
        let members = roots.members();
        for word in &members {
            let Some(root) = roots.class(*word) else { continue };
            let count = often.get(word).copied().unwrap_or(0);
            let better = first.get(&root).is_none_or(|best| {
                let top = often.get(best).copied().unwrap_or(0);
                count > top || (count == top && symbols.name(*word) < symbols.name(*best))
            });
            if better {
                first.insert(root, *word);
            }
        }
        for word in &members {
            if let Some(root) = roots.class(*word) {
                told.cls.insert(*word, first[&root]);
            }
        }
        let cls_shape = |q: &[u32]| -> Shape { q.iter().map(|word| told.cls.get(word).map_or(*word, |class| KIND_MARK | class)).collect() };
        let mut said: FirstSightTable<Shape, FirstSightTable<Vec<u32>, u32>> = FirstSightTable::new();
        for (q, answer) in &asked {
            said.entry(cls_shape(q), FirstSightTable::new()).add(answer.clone(), 1);
        }
        let few = |counts: &FirstSightTable<Vec<u32>, u32>| {
            let all = total(counts);
            let mut top: Vec<u32> = counts.rows().iter().map(|(_, count)| *count).collect();
            top.sort_unstable_by(|a, b| b.cmp(a));
            let most: u32 = top.iter().take(MOST_ANSWERS).sum();
            all >= TOLD_LEAST && f64::from(most) >= sure_share * f64::from(all)
        };
        let forms: HashSet<Shape> = said.rows().iter().filter(|(shape, counts)| few(counts) && shape.iter().any(|entry| entry & KIND_MARK != 0)).map(|(shape, _)| shape.clone()).collect();
        let asked_forms: u32 = said.rows().iter().filter(|(shape, _)| forms.contains(shape)).map(|(_, counts)| total(counts)).sum();
        if asked_forms < sure_n {
            return told;
        }
        let mut mine: CheapMap<u32, Vec<u32>> = CheapMap::default();
        for (word, class) in told.cls.iter() {
            let kind = words.kind_of(*word);
            if kind != ABSENT {
                let held = mine.entry(kind).or_default();
                if !held.contains(class) {
                    held.push(*class);
                }
            }
        }
        for (kind, classes) in mine.iter() {
            if classes.len() == 1 {
                told.kmap.insert(*kind, classes[0]);
            }
        }
        let mut fixed_words: Vec<u32> = forms.iter().flatten().copied().filter(|entry| entry & KIND_MARK == 0).collect();
        fixed_words.sort_unstable();
        fixed_words.dedup();
        fixed_words.sort_by(|a, b| symbols.name(*a).cmp(symbols.name(*b)));
        for word in fixed_words {
            let kind = words.kind_of(word);
            if kind == ABSENT || mine.contains_key(&kind) {
                continue;
            }
            let others: Vec<u32> = words.members[kind as usize].iter().copied().filter(|other| *other != word).collect();
            if others.len() <= MOST_ALIKE {
                for other in others {
                    told.alias.insert(other, word);
                }
            }
        }
        let view = View::default();
        let mut index: FirstSightTable<Vec<String>, usize> = FirstSightTable::new();
        let mut lessons = LessonStories { stories: Vec::new(), texts: Vec::new(), asks: Vec::new(), lanes };
        let mut questions_of: Vec<Vec<u32>> = Vec::with_capacity(cases.len());
        for (case, (q, answer)) in cases.iter().zip(&asked) {
            let known = index.holds(&case.sentences);
            let story = *index.entry(case.sentences.clone(), lessons.stories.len());
            if !known {
                let sents: Vec<Vec<u32>> = case.sentences.iter().map(|text| {
                    let read = words.read(symbols, &cut_words(text));
                    told.tok(&read)
                }).collect();
                lessons.stories.push(sents);
                lessons.texts.push(case.sentences.clone());
                lessons.asks.push(Vec::new());
            }
            let q = told.tok(q);
            questions_of.push(q.clone());
            lessons.asks[story].push((q, answer.clone()));
        }
        let told_shapes: HashSet<Shape> = lessons.stories.iter().flatten().map(|tokens| told.shape_of(words, &view, tokens)).collect();
        let ordered = in_order(symbols, forms.iter().cloned());
        for shape in &ordered {
            for base in &ordered {
                if base.len() < shape.len() {
                    if let Some(keep) = embed(base, shape) {
                        told.pair.push((shape.clone(), base.clone(), keep));
                        break;
                    }
                }
            }
        }
        for (shape, base, keep) in &told.pair {
            told.pairs_of.entry(base.clone()).or_default().push(shape.clone());
            for (at, word) in shape.iter().enumerate() {
                if !keep.contains(&at) {
                    told.neg_words.insert(*word);
                }
            }
            for (place, at) in keep.iter().enumerate() {
                let from = if place > 0 { keep[place - 1] + 1 } else { 0 };
                if from < *at {
                    told.before.insert((base.clone(), place, shape[from..*at].to_vec()), shape.clone());
                }
            }
        }
        for q in &questions_of {
            let shape = told.shape_of(words, &view, q);
            if forms.contains(&shape) {
                for (entry, word) in shape.iter().zip(q) {
                    if entry & KIND_MARK != 0 {
                        told.named.entry(entry & !KIND_MARK, FirstSightTable::new()).add(*word, 1);
                    }
                }
            }
        }
        let mut inside: CheapMap<u32, usize> = CheapMap::default();
        for story in &lessons.stories {
            let held: HashSet<u32> = story.iter().flatten().copied().collect();
            for word in held {
                *inside.entry(word).or_insert(0) += 1;
            }
        }
        for (word, count) in inside.iter() {
            let class = told.class_of(words, &view, *word);
            if *count >= LEAST_STORIES && class != ABSENT && told.named.get(&class).is_some_and(|named| !named.holds(word)) {
                told.open.insert(*word);
            }
        }
        for (class, counts) in told.named.rows() {
            if sure_share_of(counts, sure_share, sure_n) {
                if let Some((word, _)) = counts.first_most() {
                    told.fixed.insert(*class, *word);
                }
            }
        }
        let mut statements: Vec<(u32, String, Shape)> = forms.iter().filter(|shape| told.pair_of(shape).is_none()).map(|shape| (said.get(shape).map_or(0, total), shape_name(symbols, shape), shape.clone())).collect();
        statements.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        for (_, _, shape) in statements {
            let mut counted: FirstSightTable<u32, u32> = FirstSightTable::new();
            for entry in shape.iter().filter(|entry| *entry & KIND_MARK != 0) {
                counted.add(entry & !KIND_MARK, 1);
            }
            for (class, count) in counted.rows() {
                if *count == 1 && !told.single.contains_key(class) {
                    told.single.insert(*class, shape.clone());
                }
            }
        }
        let norm_written = |raw: &str, symbols: &Symbols| symbols.find(&raw.to_lowercase()).map(|word| words.norm(word));
        let mut written: FirstSightTable<u32, (u32, u32)> = FirstSightTable::new();
        for texts in &lessons.texts {
            for text in texts {
                for raw in as_written(text).iter().skip(1) {
                    if let Some(class) = norm_written(raw, symbols).and_then(|word| told.cls.get(&word)) {
                        let upper = raw.chars().next().is_some_and(char::is_uppercase);
                        let held = written.entry(*class, (0, 0));
                        held.0 += u32::from(upper);
                        held.1 += 1;
                    }
                }
            }
        }
        let capital: Vec<u32> = written.rows().iter().filter(|(_, (upper, all))| *all >= sure_n && f64::from(*upper) >= sure_share * f64::from(*all)).map(|(class, _)| *class).collect();
        if capital.len() == 1 {
            told.capital = capital[0];
            let mut after: CheapMap<u32, usize> = CheapMap::default();
            for texts in &lessons.texts {
                let mut seen: HashSet<u32> = HashSet::new();
                for text in texts {
                    let raws = as_written(text);
                    if raws.len() > 1 && norm_written(raws[0], symbols).and_then(|word| told.cls.get(&word)) == Some(&told.capital) {
                        if let Some(next) = norm_written(raws[1], symbols) {
                            seen.insert(next);
                        }
                    }
                }
                for word in seen {
                    *after.entry(word).or_insert(0) += 1;
                }
            }
            told.after_capital = after.iter().filter(|(_, count)| **count >= LEAST_STORIES).map(|(word, _)| *word).collect();
        }
        told.pos = forms.iter().filter(|shape| told.pair_of(shape).is_none() || told_shapes.contains(*shape)).cloned().collect();
        for (shape, base, keep) in told.pair.clone() {
            if !told_shapes.contains(&shape) {
                told.neg.insert(shape, (base, keep));
            }
        }
        told.setup(symbols);
        let mut free: Vec<u32> = told.named.rows().iter().map(|(class, _)| *class).filter(|class| !told.fixed.contains_key(class)).collect();
        free.sort_by_key(|class| class_text(symbols, *class));
        let is_named = |told: &ToldRules, word: u32| {
            let class = told.class_of(words, &view, word);
            class != ABSENT && told.named.get(&class).is_some_and(|named| named.holds(&word))
        };
        let mut lacking: FirstSightTable<(u32, u32), u32> = FirstSightTable::new();
        let mut beside: CheapMap<(u32, u32), u32> = CheapMap::default();
        let mut holding: CheapMap<(u32, u32), HashSet<usize>> = CheapMap::default();
        for (story, sents) in lessons.stories.iter().enumerate() {
            for tokens in sents {
                let have: HashSet<u32> = tokens.iter().filter(|word| is_named(&told, **word)).map(|word| told.class_of(words, &view, *word)).collect();
                if have.is_empty() {
                    continue;
                }
                let others: HashSet<u32> = tokens
                    .iter()
                    .copied()
                    .filter(|word| !is_named(&told, *word) && !told.neg_words.contains(word) && !told.fixed.contains_key(&told.class_of(words, &view, *word)))
                    .collect();
                for class in &free {
                    for word in &others {
                        let own = told.class_of(words, &view, *word);
                        if own == ABSENT || own == *class {
                            if have.contains(class) {
                                *beside.entry((*word, *class)).or_insert(0) += 1;
                            } else {
                                lacking.add((*word, *class), 1);
                                holding.entry((*word, *class)).or_default().insert(story);
                            }
                        }
                    }
                }
            }
        }
        let mut both: Vec<((u32, u32), u32)> = lacking
            .rows()
            .iter()
            .map(|(pair, count)| (*pair, (*count).min(beside.get(pair).copied().unwrap_or(0))))
            .filter(|(_, count)| *count as usize >= LEAST_STORIES)
            .collect();
        both.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| symbols.name(a.0.0).cmp(symbols.name(b.0.0))).then_with(|| class_text(symbols, a.0.1).cmp(&class_text(symbols, b.0.1))));
        let mut switches: Vec<Switch> = vec![Switch::Last, Switch::Facts, Switch::Carry];
        switches.extend(told.pair.iter().map(|(shape, _, _)| Switch::Absent(shape.clone())));
        switches.extend(both.iter().take(MOST_REFERS).map(|((word, class), _)| Switch::Refer(*word, *class)));
        for class in &free {
            switches.push(Switch::Start(*class));
            switches.push(Switch::Must(*class));
        }
        let some: HashSet<usize> = (0..lessons.stories.len().min(TRIED_STORIES)).collect();
        let mut found = false;
        for facts_too in [true, false] {
            told.facts_too = facts_too;
            let (_, rows) = told.fit(words, symbols, &lessons, true, Some(&some));
            if rows.get(&(false, true, false)).is_some_and(|counts| sure_share_of(counts, QUICK_SHARE, TOLD_LEAST)) {
                found = true;
            }
        }
        if !found {
            return told;
        }
        let mut best = told.fit(words, symbols, &lessons, true, Some(&some)).0;
        for turn in 0..CHOICE_TURNS {
            let mut changed = false;
            for switch in &switches {
                if let Switch::Refer(word, class) = switch {
                    if turn > 0 {
                        continue;
                    }
                    let mut every: Vec<usize> = holding.get(&(*word, *class)).map(|held| held.iter().copied().collect()).unwrap_or_default();
                    every.sort_by(|a, b| lessons.texts[*a].cmp(&lessons.texts[*b]));
                    let step = (every.len() / MOST_HOLDERS).max(1);
                    let spread: Vec<usize> = every.iter().copied().step_by(step).collect();
                    let mut taken = true;
                    for (place, (chosen, gain)) in [(spread, TINY), (every.clone(), WORD_GAIN)].into_iter().enumerate() {
                        let halves: [HashSet<usize>; STORY_HALVES] = [chosen.iter().copied().step_by(STORY_HALVES).collect(), chosen.iter().copied().skip(1).step_by(STORY_HALVES).collect()];
                        if halves[1].is_empty() {
                            taken = false;
                            break;
                        }
                        let before: Vec<f64> = halves.iter().map(|half| told.fit(words, symbols, &lessons, true, Some(half)).0).collect();
                        told.flip(symbols, switch);
                        let after: Vec<f64> = halves.iter().map(|half| told.fit(words, symbols, &lessons, true, Some(half)).0).collect();
                        told.flip(symbols, switch);
                        let rises = compensated(after.iter().copied()) > compensated(before.iter().copied()) + gain;
                        if !(rises && before.iter().zip(&after).all(|(a, b)| *b > *a - TINY)) {
                            taken = false;
                            break;
                        }
                        if place == 0 && chosen.len() == every.len() {
                            break;
                        }
                    }
                    if taken {
                        told.flip(symbols, switch);
                        best = told.fit(words, symbols, &lessons, true, Some(&some)).0;
                        changed = true;
                    }
                    continue;
                }
                told.flip(symbols, switch);
                let now = told.fit(words, symbols, &lessons, true, Some(&some)).0;
                if now > best + FIT_GAIN {
                    best = now;
                    changed = true;
                } else {
                    told.flip(symbols, switch);
                }
            }
            if !changed {
                break;
            }
        }
        told.flip(symbols, &Switch::Last);
        let other = told.fit(words, symbols, &lessons, true, None).0;
        told.flip(symbols, &Switch::Last);
        for _ in 0..CHOICE_TURNS {
            let mut seen: CheapMap<Shape, FirstSightTable<Vec<u32>, u32>> = CheapMap::default();
            for (story, asks) in lessons.asks.iter().enumerate() {
                let ReadStory { facts, rules, things, .. } = told.read(words, &view, &lessons.stories[story], false);
                let given = told.closure(symbols, &facts, &rules, &things);
                let mut have: HashSet<Known> = facts.into_iter().collect();
                have.extend(given);
                for (q, answer) in asks {
                    let shape = told.shape_of(words, &view, q);
                    if told.neg.contains_key(&shape) && told.row(words, &view, q, &have) == Some((true, false, true)) {
                        seen.entry(shape).or_default().add(answer.clone(), 1);
                    }
                }
            }
            let wrong: Vec<Shape> = told.neg.keys().filter(|shape| !seen.get(*shape).is_some_and(|counts| sure_share_of(counts, sure_share, TOLD_LEAST))).cloned().collect();
            if wrong.is_empty() {
                break;
            }
            for shape in wrong {
                told.neg.remove(&shape);
            }
            told.setup(symbols);
        }
        let plain = told.fit(words, symbols, &lessons, false, None).0;
        let mut holders: CheapMap<Shape, HashSet<usize>> = CheapMap::default();
        let mut seen_forms: FirstSightTable<Shape, u32> = FirstSightTable::new();
        for (story, sents) in lessons.stories.iter().enumerate() {
            for (form, atoms) in told.sort(words, &view, sents).sentences {
                if !atoms.is_empty() {
                    holders.entry(form.clone()).or_default().insert(story);
                    seen_forms.add(form, 1);
                }
            }
        }
        loop {
            let mut gains: Vec<(f64, Shape)> = Vec::new();
            for (form, _) in seen_forms.rows() {
                let Some(stories) = holders.get(form) else { continue };
                if told.bad.contains(form) || stories.len() < LEAST_STORIES {
                    continue;
                }
                let before = told.fit(words, symbols, &lessons, true, Some(stories)).0;
                told.bad.insert(form.clone());
                let gain = told.fit(words, symbols, &lessons, true, Some(stories)).0 - before;
                told.bad.remove(form);
                if gain > TINY {
                    gains.push((gain, form.clone()));
                }
            }
            let mut top: Option<&(f64, Shape)> = None;
            for found in &gains {
                if top.is_none_or(|held| found.0 > held.0) {
                    top = Some(found);
                }
            }
            match top {
                Some((_, form)) => {
                    told.bad.insert(form.clone());
                }
                None => break,
            }
        }
        let (mut best, mut rows) = told.fit(words, symbols, &lessons, true, None);
        told.rules_on = best > plain + TINY || (best > plain - TINY && best > other + TINY);
        if !told.rules_on {
            (best, rows) = told.fit(words, symbols, &lessons, false, None);
        }
        let _ = best;
        for (row, counts) in rows.rows() {
            if sure_share_of(counts, sure_share, TOLD_LEAST) {
                if let Some((answer, hit)) = counts.first_most() {
                    told.table.entry(*row, Sure { answer: answer.clone(), lessons: total(counts), hit: *hit, sure: true });
                }
            }
        }
        let missing = [((false, false, false), (false, true, false)), ((true, false, false), (true, false, true))];
        for (open_row, told_row) in missing {
            let (Some(counts), Some(found)) = (rows.get(&open_row), told.table.get(&told_row)) else { continue };
            if told.table.holds(&open_row) {
                continue;
            }
            let miss = found.answer.clone();
            let hits = found.hit;
            let mut others: FirstSightTable<Vec<u32>, u32> = FirstSightTable::new();
            for (answer, count) in counts.rows() {
                if *answer != miss {
                    others.add(answer.clone(), *count);
                }
            }
            let missed = counts.get(&miss).copied().unwrap_or(0);
            if sure_share_of(&others, sure_share, TOLD_LEAST) && f64::from(missed) <= MOST_MISSES * f64::from(missed + hits) {
                if let Some((answer, hit)) = others.first_most() {
                    told.table.entry(open_row, Sure { answer: answer.clone(), lessons: total(counts), hit: *hit, sure: false });
                }
            }
        }
        told.on = told.table.holds(&(false, true, false)) && told.table.holds(&(false, false, false));

        told
    }
}

const PARALLEL_STORIES: usize = 16;
because!(PARALLEL_STORIES, TableFit, "below this many stories a fit reads them on one thread: starting the threads costs more than it saves");

const STORY_HALVES: usize = 2;
because!(STORY_HALVES, TableFit, "the stories with a word are cut into halves by taking every other story, and a choice about the word must not lower the fit on any half: a rise that comes from a few stories alone is not taken");

const WORD_GAIN: f64 = 2.0;
because!(WORD_GAIN, TableFit, "a choice about one word is taken on all its stories only when the fit rises by more than this many lessons: the stories of one word are few, so the rise must stand above the noise of a guess on few lessons");
