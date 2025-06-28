// blt_core/src/utils.rs
// Common utility functions.

// The parse_chunk_size function was moved here from lib.rs
// It's a utility for parsing human-readable size strings.
pub fn parse_chunk_size_str(s: &str) -> Result<usize, String> {
    let s_trimmed = s.trim();
    if s_trimmed.is_empty() {
        return Err("Input string is empty".to_string());
    }

    let s_upper = s_trimmed.to_uppercase();

    // Determine if there's a unit (KB or MB)
    let (num_part_str, unit_str) = if s_upper.ends_with("KB") {
        s_trimmed.split_at(s_trimmed.len() - 2)
    } else if s_upper.ends_with("MB") {
        s_trimmed.split_at(s_trimmed.len() - 2)
    } else if s_upper.chars().all(|c| c.is_digit(10)) {
        (s_trimmed, "") // No unit, all digits
    } else {
        // This case handles inputs like "1024X" or "abc" or "MB" alone after initial checks
        return Err(format!("Invalid unit or format: '{}'. Number must be followed by KB, MB, or be raw bytes.", s_trimmed));
    };

    if num_part_str.is_empty() && !unit_str.is_empty() {
        return Err(format!("Number part missing for unit '{}'", unit_str));
    }

    let num = num_part_str.parse::<usize>().map_err(|_| format!("Invalid number: '{}'", num_part_str))?;

    match unit_str.to_uppercase().as_str() {
        "KB" => Ok(num * 1024),
        "MB" => Ok(num * 1024 * 1024),
        "" => Ok(num), // Raw bytes
        _ => Err(format!("Unsupported unit: '{}'. Use KB or MB.", unit_str)), // Should be caught by earlier checks
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chunk_size_str_valid() {
        assert_eq!(parse_chunk_size_str("1024"), Ok(1024));
        assert_eq!(parse_chunk_size_str("1kb"), Ok(1024));
        assert_eq!(parse_chunk_size_str("1KB"), Ok(1024));
        assert_eq!(parse_chunk_size_str("2mb"), Ok(2 * 1024 * 1024));
        assert_eq!(parse_chunk_size_str("2MB"), Ok(2 * 1024 * 1024));
        assert_eq!(parse_chunk_size_str("10MB "), Ok(10 * 1024 * 1024)); // With space
    }

    #[test]
    fn test_parse_chunk_size_str_invalid() {
        assert!(parse_chunk_size_str("1gb").is_err());
        assert!(parse_chunk_size_str("mb1").is_err());
        assert!(parse_chunk_size_str("1024b").is_err());
        assert!(parse_chunk_size_str("").is_err());
        assert!(parse_chunk_size_str("abc").is_err());
        assert!(parse_chunk_size_str("10.5MB").is_err());
        assert!(parse_chunk_size_str("KB").is_err()); // Unit only
        assert!(parse_chunk_size_str(" MB").is_err()); // Unit only with space
    }
}
