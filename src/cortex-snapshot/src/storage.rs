//! Snapshot storage and persistence.

use crate::{Result, RevertPoint, Snapshot, SnapshotError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info};

/// Persistent storage for snapshots.
pub struct SnapshotStorage {
    storage_path: PathBuf,
}

impl SnapshotStorage {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let storage_path = data_dir.into().join("snapshot_meta");
        Self { storage_path }
    }

    /// Initialize storage directory.
    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.storage_path).await?;
        Ok(())
    }

    /// Save a snapshot.
    pub async fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        self.init().await?;
        let path = self.snapshot_path(&snapshot.id);
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| SnapshotError::CreateFailed(e.to_string()))?;
        fs::write(&path, json).await?;
        debug!("Saved snapshot: {}", snapshot.id);
        Ok(())
    }

    /// Load a snapshot by ID.
    pub async fn load_snapshot(&self, id: &str) -> Result<Snapshot> {
        let json = self
            .read_first_existing(self.snapshot_paths(id))
            .await
            .map_err(|_| SnapshotError::NotFound(id.to_string()))?;
        let snapshot: Snapshot =
            serde_json::from_str(&json).map_err(|e| SnapshotError::NotFound(e.to_string()))?;
        Ok(snapshot)
    }

    /// Delete a snapshot.
    pub async fn delete_snapshot(&self, id: &str) -> Result<()> {
        for path in self.snapshot_paths(id) {
            if path.exists() {
                fs::remove_file(&path).await?;
                debug!("Deleted snapshot: {}", id);
            }
        }
        Ok(())
    }

    /// List all snapshots.
    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        self.init().await?;
        let mut snapshots = Vec::new();

        let mut entries = fs::read_dir(&self.storage_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = fs::read_to_string(&path).await {
                    if let Ok(snapshot) = serde_json::from_str::<Snapshot>(&json) {
                        snapshots.push(snapshot);
                    }
                }
            }
        }

        // Sort by creation time
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    /// List snapshots for a session.
    pub async fn list_session_snapshots(&self, session_id: &str) -> Result<Vec<Snapshot>> {
        let all = self.list_snapshots().await?;
        Ok(all
            .into_iter()
            .filter(|s| s.session_id.as_deref() == Some(session_id))
            .collect())
    }

    /// Save session revert history.
    pub async fn save_revert_history(
        &self,
        session_id: &str,
        history: &[RevertPoint],
    ) -> Result<()> {
        self.init().await?;
        let path = self.history_path(session_id);
        let json = serde_json::to_string_pretty(history)
            .map_err(|e| SnapshotError::CreateFailed(e.to_string()))?;
        fs::write(&path, json).await?;
        debug!("Saved revert history for session: {}", session_id);
        Ok(())
    }

    /// Load session revert history.
    pub async fn load_revert_history(&self, session_id: &str) -> Result<Vec<RevertPoint>> {
        let json = match self
            .read_first_existing(self.history_paths(session_id))
            .await
        {
            Ok(json) => json,
            Err(_) => {
                return Ok(Vec::new());
            }
        };
        let history: Vec<RevertPoint> =
            serde_json::from_str(&json).map_err(|e| SnapshotError::NotFound(e.to_string()))?;
        Ok(history)
    }

    /// Clean up old snapshots (keep only the most recent N).
    pub async fn cleanup(&self, keep_count: usize) -> Result<usize> {
        let mut snapshots = self.list_snapshots().await?;
        let mut removed = 0;

        if snapshots.len() > keep_count {
            // Keep the most recent ones
            let to_remove: Vec<_> = snapshots.drain(keep_count..).collect();
            for snapshot in to_remove {
                self.delete_snapshot(&snapshot.id).await?;
                removed += 1;
            }
            info!("Cleaned up {} old snapshots", removed);
        }

        Ok(removed)
    }

    async fn read_first_existing(&self, paths: Vec<PathBuf>) -> std::io::Result<String> {
        for path in paths {
            if path.exists() {
                return fs::read_to_string(&path).await;
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ))
    }

    fn snapshot_paths(&self, id: &str) -> Vec<PathBuf> {
        let mut paths = vec![self.snapshot_path(id)];
        if let Some(path) = self.legacy_snapshot_path(id) {
            paths.push(path);
        }
        paths
    }

    fn history_paths(&self, session_id: &str) -> Vec<PathBuf> {
        let mut paths = vec![self.history_path(session_id)];
        if let Some(path) = self.legacy_history_path(session_id) {
            paths.push(path);
        }
        paths
    }

    fn snapshot_path(&self, id: &str) -> PathBuf {
        self.storage_path
            .join(format!("snapshot_{}.json", Self::hashed_id(id)))
    }

    fn history_path(&self, session_id: &str) -> PathBuf {
        self.storage_path
            .join(format!("history_{}.json", Self::hashed_id(session_id)))
    }

    fn legacy_snapshot_path(&self, id: &str) -> Option<PathBuf> {
        if Self::is_safe_legacy_id(id) {
            Some(self.storage_path.join(format!("{}.json", id)))
        } else {
            None
        }
    }

    fn legacy_history_path(&self, session_id: &str) -> Option<PathBuf> {
        if Self::is_safe_legacy_id(session_id) {
            Some(
                self.storage_path
                    .join(format!("history_{}.json", session_id)),
            )
        } else {
            None
        }
    }

    fn hashed_id(id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn is_safe_legacy_id(id: &str) -> bool {
        if id.is_empty() {
            return false;
        }

        id.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }
}

/// Index for fast snapshot lookup.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SnapshotIndex {
    /// Map from session ID to snapshot IDs.
    pub by_session: HashMap<String, Vec<String>>,
    /// Map from message ID to snapshot ID.
    pub by_message: HashMap<String, String>,
    /// All snapshot IDs in chronological order.
    pub chronological: Vec<String>,
}

impl SnapshotIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, snapshot: &Snapshot) {
        if let Some(ref session_id) = snapshot.session_id {
            self.by_session
                .entry(session_id.clone())
                .or_default()
                .push(snapshot.id.clone());
        }

        if let Some(ref message_id) = snapshot.message_id {
            self.by_message
                .insert(message_id.clone(), snapshot.id.clone());
        }

        self.chronological.push(snapshot.id.clone());
    }

    pub fn remove(&mut self, snapshot_id: &str) {
        self.chronological.retain(|id| id != snapshot_id);

        for ids in self.by_session.values_mut() {
            ids.retain(|id| id != snapshot_id);
        }

        self.by_message.retain(|_, id| id != snapshot_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_snapshot_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SnapshotStorage::new(temp_dir.path());

        let snapshot = Snapshot::new("test_hash".to_string()).with_description("Test snapshot");

        storage.save_snapshot(&snapshot).await.unwrap();

        let loaded = storage.load_snapshot(&snapshot.id).await.unwrap();
        assert_eq!(loaded.tree_hash, snapshot.tree_hash);
        assert_eq!(loaded.description, snapshot.description);
    }

    #[test]
    fn snapshot_paths_hash_untrusted_ids_into_storage_dir() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SnapshotStorage::new(temp_dir.path());
        let path = storage.snapshot_path("../escape/owned");

        assert_eq!(path.parent(), Some(storage.storage_path.as_path()));

        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(filename.starts_with("snapshot_"));
        assert!(filename.ends_with(".json"));
        assert!(!filename.contains(".."));
        assert!(!filename.contains('/'));
        assert!(!filename.contains('\\'));
    }

    #[test]
    fn history_paths_hash_untrusted_ids_into_storage_dir() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SnapshotStorage::new(temp_dir.path());
        let path = storage.history_path("session/../../escape/owned");

        assert_eq!(path.parent(), Some(storage.storage_path.as_path()));

        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(filename.starts_with("history_"));
        assert!(filename.ends_with(".json"));
        assert!(!filename.contains(".."));
        assert!(!filename.contains('/'));
        assert!(!filename.contains('\\'));
    }

    #[tokio::test]
    async fn save_snapshot_with_path_components_does_not_escape_storage_dir() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SnapshotStorage::new(temp_dir.path());
        let escape_dir = temp_dir.path().join("escape");
        fs::create_dir_all(&escape_dir).await.unwrap();

        let mut snapshot = Snapshot::new("test_hash".to_string());
        snapshot.id = "../escape/owned".to_string();

        storage.save_snapshot(&snapshot).await.unwrap();

        assert!(!escape_dir.join("owned.json").exists());
        assert!(storage.snapshot_path(&snapshot.id).exists());

        let loaded = storage.load_snapshot(&snapshot.id).await.unwrap();
        assert_eq!(loaded.id, snapshot.id);
    }

    #[tokio::test]
    async fn load_snapshot_with_path_components_ignores_legacy_traversal_path() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SnapshotStorage::new(temp_dir.path());
        let escape_dir = temp_dir.path().join("escape");
        fs::create_dir_all(&escape_dir).await.unwrap();

        let mut snapshot = Snapshot::new("test_hash".to_string());
        snapshot.id = "../escape/owned".to_string();
        let json = serde_json::to_string(&snapshot).unwrap();
        fs::write(escape_dir.join("owned.json"), json)
            .await
            .unwrap();

        assert!(storage.load_snapshot(&snapshot.id).await.is_err());
    }

    #[tokio::test]
    async fn save_history_with_path_components_does_not_escape_storage_dir() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SnapshotStorage::new(temp_dir.path());
        let escape_dir = temp_dir.path().join("escape");
        fs::create_dir_all(&escape_dir).await.unwrap();

        let session_id = "session/../../escape/owned";

        storage.save_revert_history(session_id, &[]).await.unwrap();

        assert!(!escape_dir.join("owned.json").exists());
        assert!(storage.history_path(session_id).exists());

        let loaded = storage.load_revert_history(session_id).await.unwrap();
        assert!(loaded.is_empty());
    }

    #[tokio::test]
    async fn load_history_with_path_components_ignores_legacy_traversal_path() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SnapshotStorage::new(temp_dir.path());
        let escape_dir = temp_dir.path().join("escape");
        fs::create_dir_all(&escape_dir).await.unwrap();

        let session_id = "session/../../escape/owned";
        fs::write(escape_dir.join("owned.json"), "[]")
            .await
            .unwrap();

        let loaded = storage.load_revert_history(session_id).await.unwrap();
        assert!(loaded.is_empty());
    }
}
