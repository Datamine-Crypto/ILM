use ilm_core::base::digest::sha512;
use ilm_core::base::twister::TwisterDraws;

#[test]
fn the_hash_of_a_seed_text() {
    let found: String = sha512(b"made family 3, draw 1").iter().take(16).map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(found, "76af8fd9c9bc06247e55af7c8a30da33");
    let empty: String = sha512(b"").iter().take(8).map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(empty, "cf83e1357eefb8bd");
}

#[test]
fn draws_of_a_named_seed() {
    let mut draws = TwisterDraws::seeded("made family 3, draw 1");
    let units: Vec<f64> = (0..3).map(|_| draws.unit()).collect();
    assert_eq!(units, [0.3658560716590782, 0.45947416648542827, 0.36446501840510703]);
    let between: Vec<usize> = (0..5).map(|_| draws.between(4, 12)).collect();
    assert_eq!(between, [10, 8, 4, 6, 9]);
    let letters: Vec<char> = (0..5).map(|_| "abcdefghijklmno".as_bytes()[draws.choice(15)] as char).collect();
    assert_eq!(letters, ['b', 'c', 'f', 'd', 'c']);
    let weighted: Vec<usize> = (0..8).map(|_| [1, 2, 3][draws.weighted(&[40, 45, 15])]).collect();
    assert_eq!(weighted, [1, 2, 1, 1, 2, 2, 3, 2]);
    let mut turned: Vec<usize> = (0..7).collect();
    draws.shuffle(&mut turned);
    assert_eq!(turned, [6, 1, 2, 4, 3, 0, 5]);
    assert_eq!(draws.pool_sample(5, 2), [0, 3]);
    for _ in 0..3 {
        assert_eq!(draws.choice(1), 0);
    }
    let more: Vec<f64> = (0..2).map(|_| draws.unit()).collect();
    assert_eq!(more, [0.8852552340952637, 0.7079968187416853]);
}
