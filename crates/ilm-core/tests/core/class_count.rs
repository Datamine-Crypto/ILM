use ilm_core::class_count::{LINE_GAP, NOT_COUNTED, WordClassing, all_rows, count_tokens, each_token, frequent_words, neighbour_classes, row_width, word_codes};
use std::collections::HashMap;

fn tokens(line: &str) -> Vec<String> {
    let mut token = String::new();
    let mut found = Vec::new();
    each_token(line, &mut token, &mut |word| found.push(word.to_string()));
    found
}

#[test]
fn a_token_is_a_run_of_letters_with_one_inner_apostrophe_or_a_number() {
    assert_eq!(tokens("Don't stop: rock'n'roll in 1969, ABC123x and caf\u{e9}s!"), ["don't", "stop", "rock'n", "roll", "in", "0", "abc", "0", "x", "and", "caf", "s"]);
    assert_eq!(tokens("it's 'quoted' end'"), ["it's", "quoted", "end"]);
}

#[test]
fn the_most_frequent_words_come_first_and_equal_counts_go_by_their_signs() {
    let mut counts: HashMap<String, u64> = HashMap::new();
    count_tokens("b a c a b a d", &mut counts);
    let words = frequent_words(counts, 3);
    assert_eq!(words, [("a".to_string(), 3), ("b".to_string(), 2), ("c".to_string(), 1)]);
}

#[test]
fn a_word_counts_its_neighbours_in_its_own_line_only() {
    let numbers: HashMap<String, u32> = [("the", 0), ("cat", 1), ("dog", 2)].into_iter().map(|(word, number)| (word.to_string(), number)).collect();
    let mut codes = Vec::new();
    word_codes("the cat sat\nthe dog", &numbers, &mut codes);
    assert_eq!(codes, [0, 1, NOT_COUNTED, LINE_GAP, 0, 2, LINE_GAP]);
    let context = 1;
    let width = row_width(context);
    let rows = all_rows(&[codes], 3, context, 2);
    assert_eq!(&rows[width..2 * width], &[1, 0, 0, 1]);
    assert_eq!(&rows[0..width], &[0, 0, 0, 2]);
    assert_eq!(&rows[2 * width..3 * width], &[1, 0, 0, 0]);
}

fn grouped_rows() -> (Vec<u32>, usize) {
    let width = 4;
    let mut rows = Vec::new();
    for word in 0..12u32 {
        let row: [u32; 4] = if word % 2 == 0 { [40 + word, 30, 1, 0] } else { [0, 1, 30, 40 + word] };
        rows.extend(row);
    }
    rows.extend([1, 1, 1, 1]);
    (rows, width)
}

#[test]
fn words_with_alike_neighbours_fall_into_one_class() {
    let (rows, width) = grouped_rows();
    let classing = neighbour_classes(&rows, width, 2, 25, 1);
    let even = classing.classes[0];
    let odd = classing.classes[1];
    assert!(even.is_some() && odd.is_some() && even != odd);
    for word in 0..12 {
        assert_eq!(classing.classes[word], if word % 2 == 0 { even } else { odd });
    }
    assert_eq!(classing.classes[12], None);
}

#[test]
fn the_classes_are_the_same_for_every_number_of_threads() {
    let (rows, width) = grouped_rows();
    let one = neighbour_classes(&rows, width, 3, 25, 1);
    for lanes in [2, 3, 8] {
        assert_eq!(neighbour_classes(&rows, width, 3, 25, lanes), one);
    }
}

fn classes_row_by_row(rows: &[u32], width: usize, classes: usize, most_runs: usize) -> WordClassing {
    let words = rows.len() / width;
    let mut kept: Vec<usize> = Vec::new();
    let mut made: Vec<Vec<(usize, f64)>> = Vec::new();
    for (word, row) in rows.chunks(width).enumerate() {
        let seen: u64 = row.iter().map(|count| u64::from(*count)).sum();
        if seen < 40 {
            continue;
        }
        let length = (seen as f64).sqrt().max(1e-9);
        kept.push(word);
        made.push(row.iter().enumerate().filter(|(_, count)| **count > 0).map(|(place, count)| (place, f64::from(*count).sqrt() / length)).collect());
    }
    let mut classing = WordClassing { classes: vec![None; words], runs: 0 };
    if made.is_empty() {
        return classing;
    }
    let mut centres: Vec<Vec<f64>> = (0..classes)
        .map(|centre| {
            let row = if classes == 1 { 0 } else { centre * (made.len() - 1) / (classes - 1) };
            let mut dense = vec![0.0; width];
            made[row].iter().for_each(|(place, value)| dense[*place] = *value);
            dense
        })
        .collect();
    let mut nearest: Vec<usize> = Vec::new();
    for _ in 0..most_runs {
        let mut columns = vec![0.0; width * classes];
        for (centre, values) in centres.iter().enumerate() {
            for (place, value) in values.iter().enumerate() {
                columns[place * classes + centre] = *value;
            }
        }
        nearest = made
            .iter()
            .map(|row| {
                let mut sums = vec![0.0; classes];
                for (place, value) in row {
                    let column = &columns[place * classes..(place + 1) * classes];
                    sums.iter_mut().zip(column).for_each(|(sum, centre)| *sum += value * centre);
                }
                let mut best = 0usize;
                for (centre, sum) in sums.iter().enumerate() {
                    if *sum > sums[best] {
                        best = centre;
                    }
                }
                best
            })
            .collect();
        classing.runs += 1;
        let mut members: Vec<Vec<usize>> = vec![Vec::new(); classes];
        nearest.iter().enumerate().for_each(|(row, centre)| members[*centre].push(row));
        let mut any = false;
        for (centre, rows_of) in members.iter().enumerate() {
            if rows_of.is_empty() {
                continue;
            }
            let mut mean = vec![0.0; width];
            for member in rows_of {
                for (place, value) in &made[*member] {
                    mean[*place] += value;
                }
            }
            let count = rows_of.len() as f64;
            mean.iter_mut().for_each(|value| *value /= count);
            let length = mean.iter().map(|value| value * value).sum::<f64>().sqrt().max(1e-9);
            mean.iter_mut().for_each(|value| *value /= length);
            let change: f64 = mean.iter().zip(&centres[centre]).map(|(new, was)| (new - was).abs()).sum();
            any |= change > 1e-4;
            centres[centre] = mean;
        }
        if !any {
            break;
        }
    }
    for (row, word) in kept.iter().enumerate() {
        classing.classes[*word] = Some(nearest[row]);
    }
    classing
}

#[test]
fn classes_of_rows_in_batches_are_the_classes_row_by_row() {
    let mut seed: u64 = 17;
    let mut draw = |most: u64| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) % most
    };
    let width = 2 * (30 + 1);
    let words = 300;
    let mut rows: Vec<u32> = Vec::with_capacity(words * width);
    for word in 0..words {
        let kind = word % 7;
        for place in 0..width {
            let near = (place + kind * 5) % width < 12;
            let count = if near { draw(60) } else if draw(4) == 0 { draw(6) } else { 0 };
            rows.push(count as u32);
        }
    }
    let alone = classes_row_by_row(&rows, width, 20, 25);
    assert!(alone.classes.iter().filter(|class| class.is_some()).count() > 200);
    for lanes in [1, 3, 8] {
        assert_eq!(neighbour_classes(&rows, width, 20, 25, lanes), alone);
    }
}
