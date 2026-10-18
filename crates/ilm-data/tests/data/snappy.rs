use ilm_data::sets::parquet::snappy::decompress;

fn varint(mut value: usize, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

#[test]
fn a_block_with_a_literal_a_copy_and_a_copy_that_overlaps() {
    let mut block = Vec::new();
    varint(18, &mut block);
    block.push((8 - 1) << 2);
    block.extend_from_slice(b"abcdefgh");
    block.extend_from_slice(&[1, 8]);
    block.extend_from_slice(&[((6 - 1) << 2) | 2, 1, 0]);
    assert_eq!(decompress(&block, 18).unwrap(), b"abcdefghabcddddddd".to_vec());
}

#[test]
fn a_block_that_is_not_well_formed_says_what_is_wrong() {
    let mut block = Vec::new();
    varint(10, &mut block);
    block.push((4 - 1) << 2);
    block.extend_from_slice(b"abcd");
    block.extend_from_slice(&[((6 - 1) << 2) | 2, 9, 0]);
    assert_eq!(decompress(&block, 10), Err("a Snappy copy points before the start".to_string()));
    let mut short = Vec::new();
    varint(10, &mut short);
    short.push((4 - 1) << 2);
    short.extend_from_slice(b"abcd");
    assert_eq!(decompress(&short, 10), Err("a Snappy block gave 4 bytes and says 10".to_string()));
    assert_eq!(decompress(&short, 4), Err("a Snappy block says 10 bytes and the page says 4".to_string()));
    let mut long = Vec::new();
    varint(3, &mut long);
    long.push((4 - 1) << 2);
    long.extend_from_slice(b"abcd");
    assert_eq!(decompress(&long, 3), Err("a Snappy block gave 4 bytes and says 3".to_string()));
}

#[test]
fn copies_give_the_bytes_of_a_copy_byte_by_byte() {
    let mut seed: u64 = 29;
    let mut draw = |most: usize| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % most
    };
    for _ in 0..200 {
        let mut expected: Vec<u8> = Vec::new();
        let mut body: Vec<u8> = Vec::new();
        for _ in 0..30 {
            if expected.is_empty() || draw(3) == 0 {
                let size = 1 + draw(50);
                let bytes: Vec<u8> = (0..size).map(|_| b'a' + draw(4) as u8).collect();
                body.push(((size - 1) << 2) as u8);
                body.extend_from_slice(&bytes);
                expected.extend_from_slice(&bytes);
            } else {
                let offset = 1 + draw(expected.len().min(60));
                let size = 1 + draw(60);
                body.extend_from_slice(&[(((size - 1) << 2) | 2) as u8, offset as u8, (offset >> 8) as u8]);
                let from = expected.len() - offset;
                for place in 0..size {
                    let byte = expected[from + place];
                    expected.push(byte);
                }
            }
        }
        let mut block = Vec::new();
        varint(expected.len(), &mut block);
        block.extend_from_slice(&body);
        assert_eq!(decompress(&block, expected.len()).unwrap(), expected);
    }
}
