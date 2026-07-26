const BASE62_CHARS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn encode(mut value: u64) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let mut buf = [0u8; 11];
    let mut i = 11;
    while value > 0 {
        i -= 1;
        buf[i] = BASE62_CHARS[(value % 62) as usize];
        value /= 62;
    }
    String::from_utf8(buf[i..].to_vec()).unwrap()
}

pub fn decode(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    let mut value: u64 = 0;
    for &b in s.as_bytes() {
        let digit = BASE62_CHARS.iter().position(|&c| c == b)? as u64;
        value = value.checked_mul(62)?.checked_add(digit)?;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_zero() {
        assert_eq!(encode(0), "0");
        assert_eq!(decode("0"), Some(0));
    }

    #[test]
    fn roundtrip_max() {
        let code = encode(u64::MAX);
        assert_eq!(decode(&code), Some(u64::MAX));
    }

    #[test]
    fn roundtrip_various() {
        let values = [1u64, 42, 1000, 65535, 1_000_000, u64::MAX / 2];
        for v in values {
            let code = encode(v);
            assert_eq!(decode(&code), Some(v), "roundtrip failed for {v}");
        }
    }

    #[test]
    fn invalid_decode() {
        assert_eq!(decode(""), None);
    }

    #[test]
    fn compact_encoding() {
        assert_eq!(encode(1), "1");
        assert_eq!(encode(61), "z");
        assert_eq!(encode(62), "10");
    }
}
