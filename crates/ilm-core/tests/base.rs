use ilm_core::base::lanes::{on_lanes, on_lanes_then, shards};
use ilm_core::base::sums::compensated;
use ilm_core::base::symbols::Symbols;
use ilm_core::base::words::{answer_words, words};

#[test]
fn words_of_a_text() {
    assert_eq!(words("Mary moved to the bathroom. 3 + (4*5) = twenty-one!"), ["mary", "moved", "to", "the", "bathroom", "3", "+", "(", "4", "*", "5", ")", "=", "twenty", "-", "one"]);
    assert!(words(" ?! ").is_empty());
}

#[test]
fn words_of_an_answer() {
    assert_eq!(answer_words("s,e"), ["south", "east"]);
    assert_eq!(answer_words(" N , w "), ["north", "west"]);
    assert_eq!(answer_words("n"), ["n"]);
    assert_eq!(answer_words("The Kitchen."), ["the", "kitchen"]);
}

#[test]
fn numbers_of_words_by_first_sight() {
    let mut table = Symbols::new();
    assert_eq!(table.number("mary"), 0);
    assert_eq!(table.number("kitchen"), 1);
    assert_eq!(table.number("mary"), 0);
    assert_eq!(table.find("kitchen"), Some(1));
    assert_eq!(table.find("garden"), None);
    assert_eq!(table.name(1), "kitchen");
    for word in 0..5000 {
        table.number(&format!("w{word}"));
    }
    assert_eq!(table.len(), 5002);
    assert_eq!(table.find("w4999"), Some(5001));
    assert_eq!(table.find("mary"), Some(0));
}

#[test]
fn results_in_the_order_of_the_pieces() {
    let alone = on_lanes(100, 1, &|piece| piece * piece);
    for lanes in [2, 7, 24, 200] {
        assert_eq!(on_lanes(100, lanes, &|piece| piece * piece), alone);
    }
    assert!(on_lanes(0, 8, &|piece| piece).is_empty());
}

#[test]
fn asked_pieces_in_the_order_of_the_pieces() {
    let work = |piece: usize| piece * 3;
    let wants = |piece: usize, done: &usize| piece % 4 == 1 && *done > 0;
    let then = |piece: usize| format!("more {piece}");
    let alone = on_lanes_then(50, 1, &work, &wants, &then);
    assert_eq!(alone.0, (0..50).map(|piece| piece * 3).collect::<Vec<usize>>());
    assert_eq!(alone.1.iter().filter(|asked| asked.is_some()).count(), 13);
    assert_eq!(alone.1[5], Some("more 5".to_string()));
    assert_eq!(alone.1[4], None);
    for lanes in [2, 7, 24, 200] {
        assert_eq!(on_lanes_then(50, lanes, &work, &wants, &then), alone);
    }
    let empty = on_lanes_then(0, 8, &work, &wants, &then);
    assert!(empty.0.is_empty() && empty.1.is_empty());
}

#[test]
fn shards_hold_each_item_once() {
    for (items, lanes) in [(0, 4), (1, 4), (10, 3), (10, 24), (1000, 7)] {
        let runs = shards(items, lanes);
        let all: Vec<usize> = runs.iter().flat_map(|run| run.clone()).collect();
        assert_eq!(all, (0..items).collect::<Vec<usize>>());
    }
}

#[test]
fn sum_that_keeps_the_lost_parts() {
    let values = [1e16, 1.0, -1e16];
    assert_eq!(values.iter().sum::<f64>(), 0.0);
    assert_eq!(compensated(values), 1.0);
    assert_eq!(compensated([0.1; 10]), 1.0);
    assert_eq!(compensated(std::iter::empty()), 0.0);
}
