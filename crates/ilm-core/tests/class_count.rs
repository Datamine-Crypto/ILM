use ilm_core::class_count::{LINE_GAP, NOT_COUNTED, all_rows, count_tokens, each_token, frequent_words, neighbour_classes, row_width, word_codes};
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
