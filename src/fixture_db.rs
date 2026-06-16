//! COMBINATORIAL MAXIMALISM: Feature 3.5 — Test Fixture Database
//!
//! Persistent, indexed storage for receipt fixtures.
//! Uses a JSON backend with B-Tree indexes and atomic writes.

use crate::types::Receipt;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A stored fixture: metadata + the full receipt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fixture {
    /// Stable surrogate key; never changes after insert.
    pub id: String,
    /// Human-readable fixture name; must be unique within the DB.
    pub name: String,
    /// Searchable tags; may be empty array.
    pub tags: Vec<String>,
    /// `receipt.events.len()` — denormalized for index efficiency.
    pub event_count: usize,
    /// Sorted unique event types from all events — denormalized.
    pub event_types: Vec<String>,
    /// `receipt.chain_hash.as_hex()` — indexed for dedup.
    pub chain_hash: String,
    /// Timestamp of insertion (ISO 8601 UTC).
    pub inserted_at: String,
    /// Full serialized receipt.
    pub receipt: Receipt,
}

/// Search filter for fixture queries.
#[derive(Debug, Default)]
pub struct FixtureQuery {
    /// Filter by exact fixture name (case-insensitive substring match).
    pub name_contains: Option<String>,
    /// Filter by exact tag membership.
    pub tag: Option<String>,
    /// Filter by event count (inclusive range).
    pub min_events: Option<usize>,
    pub max_events: Option<usize>,
    /// Filter by event type membership (any event in receipt has this type).
    pub event_type: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct JsonDb {
    schema_version: String,
    fixtures: Vec<Fixture>,
}

/// An append-indexed, searchable store of receipt fixtures.
pub struct FixtureDatabase {
    path: PathBuf,
    db: JsonDb,
    // B-Tree indexes for efficient searching
    index_by_name: BTreeMap<String, usize>,
    index_by_event_count: BTreeMap<usize, Vec<usize>>,
    index_by_chain_hash: BTreeMap<String, usize>,
}

impl FixtureDatabase {
    /// Open (or create) a fixture database at `path`.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let db = if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content).context("failed to parse fixture database")?
        } else {
            JsonDb {
                schema_version: "1".to_string(),
                fixtures: Vec::new(),
            }
        };

        let mut s = FixtureDatabase {
            path,
            db,
            index_by_name: BTreeMap::new(),
            index_by_event_count: BTreeMap::new(),
            index_by_chain_hash: BTreeMap::new(),
        };
        s.reindex()?;
        Ok(s)
    }

    /// Insert a receipt as a named fixture.
    pub fn insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) -> Result<Fixture> {
        let chain_hash = receipt.chain_hash.as_hex().to_string();
        
        // Dedup guard by chain_hash
        if self.index_by_chain_hash.contains_key(&chain_hash) {
            anyhow::bail!("fixture with chain_hash {} already exists", chain_hash);
        }

        // Uniqueness check for name
        if self.index_by_name.contains_key(name) {
            anyhow::bail!("fixture with name '{}' already exists", name);
        }

        let event_count = receipt.events.len();
        let mut event_types: Vec<String> = receipt.events.iter()
            .map(|e| e.event_type.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        event_types.sort();

        let inserted_at = iso8601_now();
        
        // Generate a stable ID using blake3
        let id_input = format!("{}:{}:{}", name, chain_hash, inserted_at);
        let id = blake3::hash(id_input.as_bytes()).to_hex().to_string();

        let fixture = Fixture {
            id,
            name: name.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            event_count,
            event_types,
            chain_hash: chain_hash.clone(),
            inserted_at,
            receipt,
        };

        let index = self.db.fixtures.len();
        self.db.fixtures.push(fixture.clone());

        // Update indexes
        self.index_by_name.insert(name.to_string(), index);
        self.index_by_chain_hash.insert(chain_hash, index);
        self.index_by_event_count.entry(event_count).or_default().push(index);

        Ok(fixture)
    }

    /// Find fixtures matching `query`. Returns results in insertion order.
    pub fn search(&self, query: &FixtureQuery) -> Vec<Fixture> {
        let mut indices: HashSet<usize> = (0..self.db.fixtures.len()).collect();

        // Filter by event count using B-Tree range
        if query.min_events.is_some() || query.max_events.is_some() {
            let min = query.min_events.unwrap_or(0);
            let max = query.max_events.unwrap_or(usize::MAX);
            
            let mut count_matches = HashSet::new();
            for (_count, idxs) in self.index_by_event_count.range(min..=max) {
                for &idx in idxs {
                    count_matches.insert(idx);
                }
            }
            indices.retain(|i| count_matches.contains(i));
        }

        // Other filters (can be optimized further if needed)
        let mut results: Vec<usize> = indices.into_iter().collect();
        results.sort(); // Maintain insertion order

        let mut filtered = Vec::new();
        for idx in results {
            let f = &self.db.fixtures[idx];
            
            if let Some(ref name_sub) = query.name_contains {
                if !f.name.to_lowercase().contains(&name_sub.to_lowercase()) {
                    continue;
                }
            }
            
            if let Some(ref tag) = query.tag {
                if !f.tags.contains(tag) {
                    continue;
                }
            }

            if let Some(ref etype) = query.event_type {
                if !f.event_types.contains(etype) {
                    continue;
                }
            }

            filtered.push(f.clone());
            if let Some(limit) = query.limit {
                if filtered.len() >= limit {
                    break;
                }
            }
        }

        filtered
    }

    /// Retrieve a fixture by exact name.
    pub fn get_by_name(&self, name: &str) -> Option<Fixture> {
        self.index_by_name.get(name).map(|&idx| self.db.fixtures[idx].clone())
    }

    /// Retrieve a fixture by chain_hash hex string.
    pub fn get_by_chain_hash(&self, chain_hash: &str) -> Option<Fixture> {
        self.index_by_chain_hash.get(chain_hash).map(|&idx| self.db.fixtures[idx].clone())
    }

    /// Return all fixtures, in insertion order.
    pub fn all(&self) -> Vec<Fixture> {
        self.db.fixtures.clone()
    }

    /// Number of stored fixtures.
    pub fn len(&self) -> usize {
        self.db.fixtures.len()
    }

    /// Whether the database is empty.
    pub fn is_empty(&self) -> bool {
        self.db.fixtures.is_empty()
    }

    /// Flush in-memory state to disk with atomic write.
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.db)?;
        atomic_write(&self.path, content.as_bytes())
    }

    /// Delete a fixture by name.
    pub fn delete_by_name(&mut self, name: &str) -> Result<bool> {
        if let Some(idx) = self.index_by_name.remove(name) {
            self.db.fixtures.remove(idx);
            self.reindex()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Rebuild all indexes from the stored fixtures.
    pub fn reindex(&mut self) -> Result<()> {
        self.index_by_name.clear();
        self.index_by_event_count.clear();
        self.index_by_chain_hash.clear();

        for (idx, f) in self.db.fixtures.iter().enumerate() {
            self.index_by_name.insert(f.name.clone(), idx);
            self.index_by_chain_hash.insert(f.chain_hash.clone(), idx);
            self.index_by_event_count.entry(f.event_count).or_default().push(idx);
        }
        Ok(())
    }
}

/// Simple ISO 8601 UTC timestamp.
fn iso8601_now() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Crude ISO 8601-ish for standalone demo without chrono
    format!("{}-01-01T00:00:00Z", 1970 + (secs / 31536000)) 
}

/// Perform an atomic write by writing to a temporary file and renaming it.
fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp_path = parent.join(path.file_name().unwrap());
    tmp_path.set_extension("tmp");

    fs::write(&tmp_path, data).context("failed to write temporary file")?;
    fs::rename(&tmp_path, path).context("failed to rename temporary file to destination")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainAssembler;
    use crate::ocel::{build_event, object_ref, SeqCounter};

    fn create_test_receipt(event_count: usize) -> Receipt {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        for i in 0..event_count {
            let event = build_event(
                format!("type-{}", i % 3),
                vec![object_ref(format!("obj-{}", i), "artifact")],
                format!("payload-{}", i).as_bytes(),
                &mut counter,
            ).unwrap();
            asm.append(event).unwrap();
        }
        asm.finalize()
    }

    #[test]
    fn test_fixture_db_basic_ops() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("fixtures.json");
        let mut db = FixtureDatabase::open(&db_path).unwrap();

        let r1 = create_test_receipt(3);
        let r2 = create_test_receipt(5);

        db.insert("fix-1", &["tag1"], r1.clone()).unwrap();
        db.insert("fix-2", &["tag2", "shared"], r2.clone()).unwrap();

        assert_eq!(db.len(), 2);
        assert_eq!(db.get_by_name("fix-1").unwrap().receipt, r1);
        
        let search_res = db.search(&FixtureQuery {
            min_events: Some(4),
            ..Default::default()
        });
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].name, "fix-2");

        db.save().unwrap();
        
        // Re-open
        let db2 = FixtureDatabase::open(&db_path).unwrap();
        assert_eq!(db2.len(), 2);
        assert_eq!(db2.get_by_name("fix-1").unwrap().receipt, r1);
    }
}

fn main() -> Result<()> {
    println!("FixtureDatabase Maximalist Implementation Demo");
    let db_path = PathBuf::from("fixtures_demo.json");
    let mut db = FixtureDatabase::open(&db_path)?;

    let mut asm = crate::chain::ChainAssembler::new();
    let mut counter = crate::ocel::SeqCounter::new();
    let e = crate::ocel::build_event("demo", vec![], b"data", &mut counter)?;
    asm.append(e)?;
    let receipt = asm.finalize();

    match db.insert("demo-fixture", &["demo", "maximalist"], receipt) {
        Ok(f) => println!("Inserted fixture: {} (ID: {})", f.name, f.id),
        Err(e) => println!("Note: {}", e),
    }

    db.save()?;
    println!("Database saved to {:?}", db_path);

    let query = FixtureQuery {
        tag: Some("maximalist".to_string()),
        ..Default::default()
    };
    let results = db.search(&query);
    println!("Search found {} matches", results.len());

    Ok(())
}
