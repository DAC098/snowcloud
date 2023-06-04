pub fn read_i64(bytes: &[u8]) -> Option<i64> {
    let mut array = [0u8; 8];

    if bytes.len() != 8 {
        return None;
    }

    bytes.read_exact(&mut array)?;

    Some(i64::from_be_bytes(array))
}
