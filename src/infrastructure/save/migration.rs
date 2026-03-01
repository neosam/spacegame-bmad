use serde::Deserialize;

use super::schema::SaveError;
use super::world_save::WorldSave;
use super::schema::SAVE_VERSION;

/// V1 world save format (no chunk_deltas field).
#[derive(Deserialize)]
struct WorldSaveV1 {
    #[allow(dead_code)]
    schema_version: u32,
    seed: u64,
    explored_chunks: Vec<((i32, i32), String)>,
}

/// Migrates a v1 WorldSave RON string to v2 format by adding empty chunk_deltas.
pub fn migrate_world_v1_to_v2(ron_str: &str) -> Result<WorldSave, SaveError> {
    let v1: WorldSaveV1 = ron::from_str(ron_str)
        .map_err(|e| SaveError::ParseError(format!("v1 migration parse error: {e}")))?;

    Ok(WorldSave {
        schema_version: SAVE_VERSION,
        seed: v1.seed,
        explored_chunks: v1.explored_chunks,
        chunk_deltas: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_v1_to_v2_produces_valid_save() {
        let v1_ron = r#"(
            schema_version: 1,
            seed: 42,
            explored_chunks: [
                ((0, 0), "DeepSpace"),
                ((1, 0), "AsteroidField"),
            ],
        )"#;

        let migrated = migrate_world_v1_to_v2(v1_ron).expect("Migration should succeed");
        assert_eq!(migrated.schema_version, SAVE_VERSION);
        assert_eq!(migrated.seed, 42);
        assert_eq!(migrated.explored_chunks.len(), 2);
        assert!(migrated.chunk_deltas.is_empty());
    }

    #[test]
    fn migration_v1_empty_explored() {
        let v1_ron = r#"(
            schema_version: 1,
            seed: 123,
            explored_chunks: [],
        )"#;

        let migrated = migrate_world_v1_to_v2(v1_ron).expect("Migration should succeed");
        assert_eq!(migrated.seed, 123);
        assert!(migrated.explored_chunks.is_empty());
        assert!(migrated.chunk_deltas.is_empty());
    }

    #[test]
    fn migration_invalid_ron_returns_error() {
        let result = migrate_world_v1_to_v2("not valid ron");
        assert!(result.is_err());
    }
}
