use ulid::Ulid;

/// Generate a lowercase ULID string (26 chars).
pub fn new_ulid() -> String {
    Ulid::new().to_string().to_ascii_lowercase()
}
