use xxhash_rust::xxh3::xxh3_64;

pub fn derive(parent: u64, index: u64) -> u64 {
    let mut buf = [0u8; 16];
    buf[..8].copy_from_slice(&parent.to_le_bytes());
    buf[8..].copy_from_slice(&index.to_le_bytes());
    xxh3_64(&buf)
}

pub fn derive_label(parent: u64, label: &str) -> u64 {
    let label_bytes = label.as_bytes();
    let mut buf = Vec::with_capacity(8 + label_bytes.len());
    buf.extend_from_slice(&parent.to_le_bytes());
    buf.extend_from_slice(label_bytes);
    xxh3_64(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        assert_eq!(derive(100, 0), derive(100, 0));
        assert_eq!(
            derive_label(100, "structure"),
            derive_label(100, "structure")
        );
    }

    #[test]
    fn different_inputs_differ() {
        assert_ne!(derive(100, 0), derive(100, 1));
        assert_ne!(derive(100, 0), derive(200, 0));
    }

    #[test]
    fn label_derivation() {
        let a = derive_label(42, "structure");
        let b = derive_label(42, "bricks");
        assert_ne!(a, b);
    }

    #[test]
    fn avalanche_small_change() {
        let a = derive(0, 0);
        let b = derive(0, 1);
        assert_ne!(a, b);
        let diff = (a ^ b).count_ones();
        assert!(diff > 20, "poor avalanche: {diff} bits differ");
    }
}
