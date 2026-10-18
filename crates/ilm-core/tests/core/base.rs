use ilm_core::base::lanes::{Worked, add_scope_work, in_work_scope, learned_within, note_store_taken, on_lanes, on_lanes_then, on_spare_lanes, scope_spent, shards};
use ilm_core::base::sums::compensated;
use ilm_core::base::symbols::Symbols;
use ilm_core::base::words::{answer_words, words};
use ilm_core::cases::read_kind;

#[test]
fn case_file_lines_end_with_or_without_a_return() {
    let path = std::env::temp_dir().join(format!("ilm_case_lines_{}.txt", std::process::id()));
    std::fs::write(&path, "#\tname=x\tread_from=1\r\nS\tMary went home.\r\nL\tlabel line\nQ\tWhere is Mary?\r\nA\thome\r\n\r\nS\tJohn left.\nQ\tWhere is John?\nA\tout").expect("write");
    let kind = read_kind(&path).expect("read");
    std::fs::remove_file(&path).ok();
    assert_eq!(kind.head, vec![("name".to_string(), "x".to_string()), ("read_from".to_string(), "1".to_string())]);
    assert_eq!(kind.cases.len(), 2);
    assert_eq!(kind.cases[0].sentences, vec!["Mary went home.".to_string()]);
    assert_eq!(kind.cases[0].question, "Where is Mary?");
    assert_eq!(kind.cases[0].answer, "home");
    assert_eq!(kind.labels[0], vec!["L\tlabel line".to_string()]);
    assert_eq!(kind.cases[1].sentences, vec!["John left.".to_string()]);
    assert_eq!(kind.cases[1].answer, "out");
    assert!(kind.labels[1].is_empty());
}

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
fn a_run_with_spare_lanes_keeps_the_order_of_the_pieces() {
    let squares: Vec<usize> = (0..5000).map(|piece| piece * piece).collect();
    assert_eq!(on_spare_lanes(5000, 1, &|piece| piece * piece), squares);
    assert_eq!(on_spare_lanes(5000, 6, &|piece| piece * piece), squares);
    assert!(on_spare_lanes(0, 1, &|piece| piece).is_empty());
    let work = |job: usize| if job == 0 { on_spare_lanes(5000, 1, &|piece| piece * piece) } else { vec![job] };
    let (firsts, _) = on_lanes_then(8, 4, &work, &|_, _| false, &|piece| piece);
    assert_eq!(firsts[0], squares);
    assert_eq!(firsts[7], vec![7]);
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

#[test]
fn work_done_on_other_threads_counts_for_the_scope_that_started_them() {
    let pieces = [1usize, 2, 3, 4, 5, 6, 7, 8];
    let total: usize = pieces.iter().sum();
    let outer = in_work_scope(usize::MAX, || {
        let inner = in_work_scope(usize::MAX, || on_lanes(pieces.len(), pieces.len(), &|at| add_scope_work(pieces[at], false)));
        assert_eq!(inner.work, total);
        on_spare_lanes(pieces.len(), 1, &|at| add_scope_work(pieces[at], false));
        let (firsts, _) = on_lanes_then(pieces.len(), pieces.len(), &|at| add_scope_work(pieces[at], false), &|_, _| false, &|_| ());
        assert_eq!(firsts.len(), pieces.len());
    });
    assert_eq!(outer.work, total + total + total);
    assert!(!outer.stopped);
}

#[test]
fn a_kept_learning_is_taken_only_within_the_work_left_and_kept_only_when_it_did_not_stop() {
    let kept = Worked { value: "kept", work: 1, stopped: false, taken: Vec::new() };
    let taken = in_work_scope(1, || learned_within(Some(&kept), || "new"));
    assert_eq!((taken.value.0, taken.value.1.is_none(), taken.work), ("kept", true, 1));
    let passed = in_work_scope(usize::default(), || learned_within(Some(&kept), || "new"));
    assert_eq!(passed.value.0, "new");
    assert_eq!(passed.value.1, Some(Worked { value: "new", work: usize::default(), stopped: false, taken: Vec::new() }));
    let stopped = Worked { stopped: true, ..kept.clone() };
    let again = in_work_scope(usize::MAX, || learned_within(Some(&stopped), || "new"));
    assert_eq!(again.value.0, "new");
    let ran_away = in_work_scope(usize::default(), || learned_within(None, || {
        add_scope_work(1, false);
        scope_spent()
    }));
    assert!(ran_away.value.0 && ran_away.value.1.is_none() && ran_away.stopped);
}

#[test]
fn a_scope_past_its_bound_is_spent_and_its_stop_reaches_the_scopes_around_it() {
    let outer = in_work_scope(usize::MAX, || {
        let inner = in_work_scope(usize::default(), || {
            assert!(!scope_spent());
            add_scope_work(1, false);
            scope_spent()
        });
        assert!(inner.value && inner.stopped);
        assert!(!scope_spent());
    });
    assert!(outer.stopped && outer.work == 1);
    let again = Worked { value: "kept", work: 1, stopped: false, taken: Vec::new() };
    let taken = in_work_scope(usize::MAX, || again.again());
    assert_eq!((taken.value, taken.work, taken.stopped), ("kept", 1, false));
}

#[test]
fn the_kinds_of_stored_lines_that_a_learning_took_reach_the_scopes_around_it_and_come_again_with_a_kept_learning() {
    let outer = in_work_scope(usize::MAX, || {
        let inner = in_work_scope(usize::MAX, || {
            note_store_taken("kin_two");
            note_store_taken("kin_one");
            note_store_taken("kin_two");
        });
        assert_eq!(inner.taken, vec!["kin_one".to_string(), "kin_two".to_string()], "each kind once, in the order of the names");
        inner
    });
    assert_eq!(outer.taken, outer.value.taken);
    let kept = outer.value;
    let replayed = in_work_scope(usize::MAX, || learned_within(Some(&kept), || ()));
    assert_eq!(replayed.taken, kept.taken, "a kept learning given again names the same kinds");
    assert!(in_work_scope(usize::MAX, || ()).taken.is_empty());
}
