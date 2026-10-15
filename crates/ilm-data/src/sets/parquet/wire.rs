use patterns::{because, source};

pub struct VariableNumbers;
source!(VariableNumbers, "the variable-length numbers of Protocol Buffers, which thrift, Snappy and the runs of parquet all use: seven bits of value in each byte, the least first, and the high bit set in every byte but the last");

pub const WIRE_MORE: u8 = 128;
because!(WIRE_MORE, VariableNumbers, "the high bit of a byte of a variable-length number: more bytes follow");

pub const WIRE_VALUE: u8 = 127;
because!(WIRE_VALUE, VariableNumbers, "the low bits of a byte of a variable-length number, which carry the value");

pub const WIRE_VALUE_BITS: u32 = 7;
because!(WIRE_VALUE_BITS, VariableNumbers, "the bits of value in each byte of a variable-length number: all bits of a byte but the one that says more bytes follow");

pub const WIRE_BYTE_BITS: usize = 8;
because!(WIRE_BYTE_BITS, VariableNumbers, "the bits of a byte, for numbers of some bytes and for truth values packed in bytes");

pub fn wire_number(bytes: &[u8], at: &mut usize) -> Result<u64, String> {
    let mut value = 0u64;
    let mut shift = 0u32;
    loop {
        let byte = *bytes.get(*at).ok_or("the bytes end in a variable-length number")?;
        *at += 1;
        value |= u64::from(byte & WIRE_VALUE) << shift;
        if byte & WIRE_MORE == 0 {
            return Ok(value);
        }
        shift += WIRE_VALUE_BITS;
        if shift >= u64::BITS {
            return Err("a variable-length number is too long".to_string());
        }
    }
}
because!(wire_number, VariableNumbers, "the variable-length number at a place, and the place after it; a number longer than the widest whole number is refused");
