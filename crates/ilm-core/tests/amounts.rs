use ilm_core::base::exact::Amount;
use std::cmp::Ordering;

fn amount(before: &str, after: &str) -> Amount {
    Amount::from_digits(before, after).expect("digits")
}

fn ratio(top: i128, bottom: i128) -> Amount {
    Amount::whole(top).over(Amount::whole(bottom)).expect("ratio")
}

#[test]
fn amounts_compare_by_their_values() {
    assert!(ratio(1, 3) < ratio(1, 2));
    assert!(ratio(-1, 2) < ratio(1, 3));
    assert!(ratio(-1, 2) < ratio(-1, 3));
    assert!(Amount::whole(2) > ratio(3, 2));
    assert!(ratio(7, 3) > ratio(9, 4));
    assert!(ratio(22, 7) > ratio(355, 113));
    assert_eq!(ratio(2, 4).cmp(&ratio(1, 2)), Ordering::Equal);
    assert_eq!(amount("0", "5"), ratio(1, 2));
    assert!(amount("0", "333") < ratio(1, 3));
    assert!(Amount::whole(0) < amount("0", "001"));
    assert!(Amount::whole(0) > amount("0", "001").negated().expect("negated"));
    assert!(Amount::whole(i128::MAX) > Amount::whole(i128::MAX - 1));
    assert!(ratio(i128::MAX - 2, i128::MAX - 1) < ratio(i128::MAX - 1, i128::MAX));
}

#[test]
fn a_sort_of_amounts_is_the_sort_of_their_values() {
    let mut found = vec![ratio(5, 2), Amount::whole(-3), ratio(1, 7), Amount::whole(0), ratio(-7, 3), amount("2", "25")];
    found.sort();
    let written: Vec<String> = found.iter().map(|value| value.written(12).unwrap_or_default()).collect();
    assert_eq!(written, ["-3", "", "0", "", "2.25", "2.5"]);
}
