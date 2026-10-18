use patterns::{because, source};

pub struct SecureHash;
source!(SecureHash, "the Secure Hash Standard of the National Institute of Standards and Technology, in its variant with words of sixty-four bits and a result of eight such words: a message is padded to whole blocks of sixteen words, each block is spread into a schedule of words, and the words are mixed into the running words by rounds of rotations, choices and majorities");

const SHA_START: [u64; SHA_STATE] = [0x6a09_e667_f3bc_c908, 0xbb67_ae85_84ca_a73b, 0x3c6e_f372_fe94_f82b, 0xa54f_f53a_5f1d_36f1, 0x510e_527f_ade6_82d1, 0x9b05_688c_2b3e_6c1f, 0x1f83_d9ab_fb41_bd6b, 0x5be0_cd19_137e_2179];
because!(SHA_START, SecureHash, "the first running words that the standard gives: the first bits of the parts below one of the square roots of the first primes");

const SHA_ROUND: [u64; SHA_ROUNDS] = [
    0x428a_2f98_d728_ae22, 0x7137_4491_23ef_65cd, 0xb5c0_fbcf_ec4d_3b2f, 0xe9b5_dba5_8189_dbbc, 0x3956_c25b_f348_b538, 0x59f1_11f1_b605_d019, 0x923f_82a4_af19_4f9b, 0xab1c_5ed5_da6d_8118,
    0xd807_aa98_a303_0242, 0x1283_5b01_4570_6fbe, 0x2431_85be_4ee4_b28c, 0x550c_7dc3_d5ff_b4e2, 0x72be_5d74_f27b_896f, 0x80de_b1fe_3b16_96b1, 0x9bdc_06a7_25c7_1235, 0xc19b_f174_cf69_2694,
    0xe49b_69c1_9ef1_4ad2, 0xefbe_4786_384f_25e3, 0x0fc1_9dc6_8b8c_d5b5, 0x240c_a1cc_77ac_9c65, 0x2de9_2c6f_592b_0275, 0x4a74_84aa_6ea6_e483, 0x5cb0_a9dc_bd41_fbd4, 0x76f9_88da_8311_53b5,
    0x983e_5152_ee66_dfab, 0xa831_c66d_2db4_3210, 0xb003_27c8_98fb_213f, 0xbf59_7fc7_beef_0ee4, 0xc6e0_0bf3_3da8_8fc2, 0xd5a7_9147_930a_a725, 0x06ca_6351_e003_826f, 0x1429_2967_0a0e_6e70,
    0x27b7_0a85_46d2_2ffc, 0x2e1b_2138_5c26_c926, 0x4d2c_6dfc_5ac4_2aed, 0x5338_0d13_9d95_b3df, 0x650a_7354_8baf_63de, 0x766a_0abb_3c77_b2a8, 0x81c2_c92e_47ed_aee6, 0x9272_2c85_1482_353b,
    0xa2bf_e8a1_4cf1_0364, 0xa81a_664b_bc42_3001, 0xc24b_8b70_d0f8_9791, 0xc76c_51a3_0654_be30, 0xd192_e819_d6ef_5218, 0xd699_0624_5565_a910, 0xf40e_3585_5771_202a, 0x106a_a070_32bb_d1b8,
    0x19a4_c116_b8d2_d0c8, 0x1e37_6c08_5141_ab53, 0x2748_774c_df8e_eb99, 0x34b0_bcb5_e19b_48a8, 0x391c_0cb3_c5c9_5a63, 0x4ed8_aa4a_e341_8acb, 0x5b9c_ca4f_7763_e373, 0x682e_6ff3_d6b2_b8a3,
    0x748f_82ee_5def_b2fc, 0x78a5_636f_4317_2f60, 0x84c8_7814_a1f0_ab72, 0x8cc7_0208_1a64_39ec, 0x90be_fffa_2363_1e28, 0xa450_6ceb_de82_bde9, 0xbef9_a3f7_b2c6_7915, 0xc671_78f2_e372_532b,
    0xca27_3ece_ea26_619c, 0xd186_b8c7_21c0_c207, 0xeada_7dd6_cde0_eb1e, 0xf57d_4f7f_ee6e_d178, 0x06f0_67aa_7217_6fba, 0x0a63_7dc5_a2c8_98a6, 0x113f_9804_bef9_0dae, 0x1b71_0b35_131c_471b,
    0x28db_77f5_2304_7d84, 0x32ca_ab7b_40c7_2493, 0x3c9e_be0a_15c9_bebc, 0x431d_67c4_9c10_0d4c, 0x4cc5_d4be_cb3e_42b6, 0x597f_299c_fc65_7e2a, 0x5fcb_6fab_3ad6_faec, 0x6c44_198c_4a47_5817,
];
because!(SHA_ROUND, SecureHash, "the word of each round that the standard gives: the first bits of the parts below one of the cube roots of the first primes");

const SHA_STATE: usize = 8;
because!(SHA_STATE, SecureHash, "the running words of the hash, which are also its result");

const SHA_ROUNDS: usize = 80;
because!(SHA_ROUNDS, SecureHash, "the rounds of mixing of one block, and the words of its schedule");

const SHA_MARK: u8 = 0x80;
because!(SHA_MARK, SecureHash, "the byte with one bit that follows the message before the padding");

const SHA_BYTE_BITS: u128 = 8;
because!(SHA_BYTE_BITS, SecureHash, "the bits of one byte, for the length of the message in bits");

const SHA_BACK_TWO: usize = 2;
because!(SHA_BACK_TWO, SecureHash, "a word of the schedule takes the word this many places before it through the second small sigma");

const SHA_BACK_SEVEN: usize = 7;
because!(SHA_BACK_SEVEN, SecureHash, "a word of the schedule takes the word this many places before it as it is");

const SHA_BACK_FIFTEEN: usize = 15;
because!(SHA_BACK_FIFTEEN, SecureHash, "a word of the schedule takes the word this many places before it through the first small sigma");

pub struct ShortHash;
source!(ShortHash, "the variant of the Secure Hash Standard with words of thirty-two bits and a result of eight such words, which the hub and the other hosts give for their files: its first running words and the words of its rounds are the high halves of those of the long variant, since both are the first bits of the parts below one of the square and cube roots of the first primes; a file is hashed block by block as it is read, so a large file needs no room for all of it");

const SHORT_ROUNDS: usize = 64;
because!(SHORT_ROUNDS, ShortHash, "the rounds of mixing of one block, and the words of its schedule");

const SHORT_BLOCK: usize = 64;
because!(SHORT_BLOCK, ShortHash, "the bytes of one block of the message");

const SHORT_WORD: usize = 4;
because!(SHORT_WORD, ShortHash, "the bytes of one word of the hash");

const SHORT_LENGTH: usize = 8;
because!(SHORT_LENGTH, ShortHash, "the bytes at the end of the padded message that hold its length in bits");

const SHORT_FIRST_WORDS: usize = 16;
because!(SHORT_FIRST_WORDS, ShortHash, "the words of a block before the schedule grows");

const SHORT_HALF: u32 = 32;
because!(SHORT_HALF, ShortHash, "the shift that takes the high half of a word of the long variant");

const SHORT_SMALL_ONE: (u32, u32, u32) = (7, 18, 3);
because!(SHORT_SMALL_ONE, ShortHash, "the two rotations and the shift of the first small sigma of the schedule");

const SHORT_SMALL_TWO: (u32, u32, u32) = (17, 19, 10);
because!(SHORT_SMALL_TWO, ShortHash, "the two rotations and the shift of the second small sigma of the schedule");

const SHORT_LARGE_ONE: (u32, u32, u32) = (2, 13, 22);
because!(SHORT_LARGE_ONE, ShortHash, "the three rotations of the first large sigma of a round");

const SHORT_LARGE_TWO: (u32, u32, u32) = (6, 11, 25);
because!(SHORT_LARGE_TWO, ShortHash, "the three rotations of the second large sigma of a round");

fn short_small(word: u32, (first, second, shift): (u32, u32, u32)) -> u32 {
    word.rotate_right(first) ^ word.rotate_right(second) ^ (word >> shift)
}
because!(short_small, ShortHash, "a small sigma of the schedule of the short variant: two rotations and a shift");

fn short_large(word: u32, (first, second, third): (u32, u32, u32)) -> u32 {
    word.rotate_right(first) ^ word.rotate_right(second) ^ word.rotate_right(third)
}
because!(short_large, ShortHash, "a large sigma of a round of the short variant: three rotations");

fn high_half(word: u64) -> u32 {
    (word >> SHORT_HALF) as u32
}
because!(high_half, ShortHash, "the high half of a word of the long variant");

#[derive(Clone, Debug)]
pub struct Sha256 {
    state: [u32; SHA_STATE],
    waiting: Vec<u8>,
    length: u64,
}
because!(Sha256, ShortHash, "a hash of the short variant that is fed in parts: the running words, the bytes that wait for a whole block, and the length so far. new starts it; add feeds bytes; hex ends it and gives the result as small hexadecimal signs, the way the hosts write it");

impl Default for Sha256 {
    fn default() -> Sha256 {
        Sha256::new()
    }
}

impl Sha256 {
    pub fn new() -> Sha256 {
        Sha256 { state: SHA_START.map(high_half), waiting: Vec::with_capacity(SHORT_BLOCK), length: 0 }
    }

    fn block(&mut self, block: &[u8]) {
        let mut schedule = [0u32; SHORT_ROUNDS];
        for (place, bytes) in block.chunks(SHORT_WORD).enumerate() {
            let mut word = [0u8; SHORT_WORD];
            word.copy_from_slice(bytes);
            schedule[place] = u32::from_be_bytes(word);
        }
        for place in SHORT_FIRST_WORDS..SHORT_ROUNDS {
            schedule[place] = short_small(schedule[place - SHA_BACK_TWO], SHORT_SMALL_TWO).wrapping_add(schedule[place - SHA_BACK_SEVEN]).wrapping_add(short_small(schedule[place - SHA_BACK_FIFTEEN], SHORT_SMALL_ONE)).wrapping_add(schedule[place - SHORT_FIRST_WORDS]);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for round in 0..SHORT_ROUNDS {
            let choice = (e & f) ^ (!e & g);
            let first = h.wrapping_add(short_large(e, SHORT_LARGE_TWO)).wrapping_add(choice).wrapping_add(high_half(SHA_ROUND[round])).wrapping_add(schedule[round]);
            let most = (a & b) ^ (a & c) ^ (b & c);
            let second = short_large(a, SHORT_LARGE_ONE).wrapping_add(most);
            (h, g, f, e, d, c, b, a) = (g, f, e, d.wrapping_add(first), c, b, a, first.wrapping_add(second));
        }
        for (held, added) in self.state.iter_mut().zip([a, b, c, d].into_iter().chain([e, f, g, h])) {
            *held = held.wrapping_add(added);
        }
    }

    pub fn add(&mut self, bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u64);
        let mut rest = bytes;
        if !self.waiting.is_empty() {
            let take = (SHORT_BLOCK - self.waiting.len()).min(rest.len());
            self.waiting.extend_from_slice(&rest[..take]);
            rest = &rest[take..];
            if self.waiting.len() < SHORT_BLOCK {
                return;
            }
            let block = std::mem::take(&mut self.waiting);
            self.block(&block);
        }
        let whole = rest.len() - rest.len() % SHORT_BLOCK;
        for block in rest[..whole].chunks(SHORT_BLOCK) {
            self.block(block);
        }
        self.waiting.extend_from_slice(&rest[whole..]);
    }

    pub fn hex(mut self) -> String {
        let bits = self.length.wrapping_mul(SHA_BYTE_BITS as u64);
        let mut tail = std::mem::take(&mut self.waiting);
        tail.push(SHA_MARK);
        while tail.len() % SHORT_BLOCK != SHORT_BLOCK - SHORT_LENGTH {
            tail.push(0);
        }
        tail.extend_from_slice(&bits.to_be_bytes());
        for block in tail.chunks(SHORT_BLOCK) {
            self.block(block);
        }
        self.state.iter().flat_map(|word| word.to_be_bytes()).map(|byte| format!("{byte:02x}")).collect()
    }
}
