//! Test helpers: golden-file assertions and tempfile re-export.

pub use tempfile;

use crate::Result;

/// Assert that `actual` matches the bytes stored at `path`.
///
/// If the environment variable `UPDATE_GOLDEN` is set to `"1"` the file
/// is (re-)written with the new bytes and the assertion is skipped.
pub fn assert_golden(actual: &[u8], path: &std::path::Path) -> Result<()> {
    if std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, actual)?;
        return Ok(());
    }

    let expected = std::fs::read(path)?;
    if actual != expected.as_slice() {
        return Err(crate::Error::msg(format!(
            "golden mismatch at {}: actual {} bytes, expected {} bytes",
            path.display(),
            actual.len(),
            expected.len(),
        )));
    }
    Ok(())
}
