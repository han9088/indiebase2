//! ULID path segment detection for Data API dual-path routing (§6.2.3).

/// Returns true if `segment` is a 26-character lowercase Crockford Base32 ULID.
pub fn is_project_ulid(segment: &str) -> bool {
    const CROCKFORD: &[u8] = b"0123456789abcdefghjkmnpqrstvwxyz";
    if segment.len() != 26 {
        return false;
    }
    segment.bytes().all(|b| CROCKFORD.contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_sample_ulid() {
        assert!(is_project_ulid("01jcqz4sxf7k2m8n3p5r6t9vwx"));
    }

    #[test]
    fn rejects_table_name() {
        assert!(!is_project_ulid("users"));
        assert!(!is_project_ulid("01SHORT"));
        assert!(!is_project_ulid("01JCQZ4SXF7K2M8N3P5R6T9VWX")); // uppercase
    }
}
