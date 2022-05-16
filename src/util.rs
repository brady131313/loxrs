/// Splits a u16 into two u8s, where the first byte in tuple is the original
/// u16 shifted right and casted
pub fn split_u16(x: u16) -> (u8, u8) {
    ((x >> 8) as u8, x as u8)
}

/// Join two u8s assuming first byte is the right shifted one
pub fn join_u8s(b1: u8, b2: u8) -> u16 {
    ((b1 as u16) << 8) | b2 as u16
}
