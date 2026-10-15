use ilm_core::base::digest::Sha256;

fn hashed(parts: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.add(part);
    }
    hash.hex()
}

#[test]
fn short_hash_gives_the_vectors_of_the_standard() {
    assert_eq!(hashed(&[b"abc"]), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(hashed(&[b""]), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    let two_blocks: &[u8] = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    assert_eq!(hashed(&[two_blocks]), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
}

#[test]
fn short_hash_is_the_same_when_fed_in_parts() {
    let message: Vec<u8> = (0..1000u32).map(|place| (place % 251) as u8).collect();
    let whole = hashed(&[&message]);
    let parts = hashed(&[&message[..1], &message[1..63], &message[63..64], &message[64..200], &message[200..]]);
    assert_eq!(whole, parts);
}
