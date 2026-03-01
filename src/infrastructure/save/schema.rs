use std::fmt;

use serde::Deserialize;

/// Current save file schema version.
pub const SAVE_VERSION: u32 = 5;

/// Errors that can occur during save/load operations.
#[derive(Debug)]
pub enum SaveError {
    /// Save file version doesn't match current version.
    VersionMismatch { expected: u32, found: u32 },
    /// Failed to parse save data.
    ParseError(String),
    /// I/O error during save/load.
    IoError(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::VersionMismatch { expected, found } => {
                write!(f, "Save version mismatch: expected {expected}, found {found}")
            }
            SaveError::ParseError(msg) => write!(f, "Save parse error: {msg}"),
            SaveError::IoError(msg) => write!(f, "Save I/O error: {msg}"),
        }
    }
}

/// Helper struct to extract just the schema_version from RON data.
#[derive(Deserialize)]
struct VersionHeader {
    schema_version: u32,
}

/// Checks the schema version in a RON string.
/// Accepts version 1 (needs migration) or current version.
/// Returns the version found.
///
/// **Design note:** Each supported old version is listed explicitly so that
/// bumping `SAVE_VERSION` forces the developer to add a new migration path
/// and update this function. Do NOT use `<= SAVE_VERSION` — that would
/// silently accept versions without a corresponding migration.
pub fn check_version(ron_str: &str) -> Result<u32, SaveError> {
    let header: VersionHeader = ron::from_str(ron_str)
        .map_err(|e| SaveError::ParseError(format!("{e}")))?;

    if header.schema_version != SAVE_VERSION
        && header.schema_version != 1
        && header.schema_version != 2
        && header.schema_version != 3
        && header.schema_version != 4
    {
        return Err(SaveError::VersionMismatch {
            expected: SAVE_VERSION,
            found: header.schema_version,
        });
    }

    Ok(header.schema_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_version_is_five() {
        assert_eq!(SAVE_VERSION, 5);
    }

    #[test]
    fn check_version_valid_current() {
        let ron_str = r#"(schema_version: 5, position: (1.0, 2.0))"#;
        let version = check_version(ron_str).expect("Should parse valid version");
        assert_eq!(version, 5);
    }

    #[test]
    fn check_version_accepts_v4() {
        let ron_str = r#"(schema_version: 4, position: (1.0, 2.0))"#;
        let version = check_version(ron_str).expect("Should accept v4 for migration");
        assert_eq!(version, 4);
    }

    #[test]
    fn check_version_accepts_v3() {
        let ron_str = r#"(schema_version: 3, position: (1.0, 2.0))"#;
        let version = check_version(ron_str).expect("Should accept v3 for migration");
        assert_eq!(version, 3);
    }

    #[test]
    fn check_version_accepts_v1() {
        let ron_str = r#"(schema_version: 1, position: (1.0, 2.0))"#;
        let version = check_version(ron_str).expect("Should accept v1 for migration");
        assert_eq!(version, 1);
    }

    #[test]
    fn check_version_accepts_v2() {
        let ron_str = r#"(schema_version: 2, position: (1.0, 2.0))"#;
        let version = check_version(ron_str).expect("Should accept v2 for migration");
        assert_eq!(version, 2);
    }

    #[test]
    fn check_version_mismatch() {
        let ron_str = r#"(schema_version: 99)"#;
        let result = check_version(ron_str);
        assert!(result.is_err());
        if let Err(SaveError::VersionMismatch { expected, found }) = result {
            assert_eq!(expected, SAVE_VERSION);
            assert_eq!(found, 99);
        } else {
            panic!("Expected VersionMismatch error");
        }
    }

    #[test]
    fn check_version_parse_error() {
        let result = check_version("not valid ron");
        assert!(result.is_err());
        assert!(matches!(result, Err(SaveError::ParseError(_))));
    }

    #[test]
    fn check_version_missing_field() {
        let ron_str = r#"(position: (1.0, 2.0))"#;
        let result = check_version(ron_str);
        assert!(result.is_err());
        assert!(matches!(result, Err(SaveError::ParseError(_))));
    }

    #[test]
    fn save_error_display() {
        let err = SaveError::VersionMismatch { expected: 1, found: 2 };
        let msg = format!("{err}");
        assert!(msg.contains("expected 1"));
        assert!(msg.contains("found 2"));

        let err = SaveError::ParseError("bad data".to_string());
        assert!(format!("{err}").contains("bad data"));

        let err = SaveError::IoError("file not found".to_string());
        assert!(format!("{err}").contains("file not found"));
    }
}
