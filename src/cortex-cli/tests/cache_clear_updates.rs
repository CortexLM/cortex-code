use cortex_cli::cache_cmd::{CacheClearArgs, CacheCli, CacheSubcommand};
use serial_test::serial;
use std::ffi::OsString;
use std::fs;
use tempfile::tempdir;

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set_path(key: &'static str, value: &std::path::Path) -> Self {
        let original = std::env::var_os(key);

        // SAFETY: Tests using this helper are serialized and restore the original
        // environment on drop before any parallel code depends on it.
        unsafe {
            std::env::set_var(key, value);
        }

        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // SAFETY: These tests are serialized. Restoring the original environment
        // value is scoped to test teardown.
        unsafe {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

#[tokio::test]
#[serial]
async fn clear_updates_removes_update_metadata_file() {
    let temp = tempdir().expect("Failed to create temp directory");
    let home_dir = temp.path().join("home");
    let cache_dir = home_dir.join(".cache").join("cortex");
    let updates_dir = cache_dir.join("updates");
    let metadata_file = home_dir.join(".cortex").join("update_cache.json");

    fs::create_dir_all(&updates_dir).expect("Failed to create updates dir");
    fs::create_dir_all(metadata_file.parent().expect("Metadata parent should exist"))
        .expect("Failed to create metadata parent dir");
    fs::write(updates_dir.join("u1"), "x").expect("Failed to write updates cache entry");
    fs::write(&metadata_file, "{\"last_checked\":\"x\"}")
        .expect("Failed to write update metadata file");

    let _home_guard = EnvVarGuard::set_path("HOME", &home_dir);
    let _xdg_cache_guard = EnvVarGuard::set_path("XDG_CACHE_HOME", &home_dir.join(".cache"));

    CacheCli {
        subcommand: Some(CacheSubcommand::Clear(CacheClearArgs {
            models: false,
            responses: false,
            updates: true,
            yes: true,
            dry_run: false,
        })),
    }
    .run()
    .await
    .expect("cache clear should succeed");

    assert!(!updates_dir.exists(), "updates directory should be removed");
    assert!(
        !metadata_file.exists(),
        "update metadata file should be removed"
    );
}

#[tokio::test]
#[serial]
async fn clear_updates_handles_missing_cache_dir_with_metadata_file() {
    let temp = tempdir().expect("Failed to create temp directory");
    let home_dir = temp.path().join("home");
    let metadata_file = home_dir.join(".cortex").join("update_cache.json");

    fs::create_dir_all(metadata_file.parent().expect("Metadata parent should exist"))
        .expect("Failed to create metadata parent dir");
    fs::write(&metadata_file, "{\"last_checked\":\"x\"}")
        .expect("Failed to write update metadata file");

    let _home_guard = EnvVarGuard::set_path("HOME", &home_dir);
    let _xdg_cache_guard = EnvVarGuard::set_path("XDG_CACHE_HOME", &home_dir.join(".cache"));

    CacheCli {
        subcommand: Some(CacheSubcommand::Clear(CacheClearArgs {
            models: false,
            responses: false,
            updates: true,
            yes: true,
            dry_run: false,
        })),
    }
    .run()
    .await
    .expect("cache clear should succeed");

    assert!(
        !metadata_file.exists(),
        "update metadata file should be removed even without cache dir"
    );
}

#[tokio::test]
#[serial]
async fn clear_all_also_removes_update_metadata_file() {
    let temp = tempdir().expect("Failed to create temp directory");
    let home_dir = temp.path().join("home");
    let cache_dir = home_dir.join(".cache").join("cortex");
    let models_dir = cache_dir.join("models");
    let updates_dir = cache_dir.join("updates");
    let metadata_file = home_dir.join(".cortex").join("update_cache.json");

    fs::create_dir_all(&models_dir).expect("Failed to create models dir");
    fs::create_dir_all(&updates_dir).expect("Failed to create updates dir");
    fs::create_dir_all(metadata_file.parent().expect("Metadata parent should exist"))
        .expect("Failed to create metadata parent dir");
    fs::write(models_dir.join("m1"), "model").expect("Failed to write model cache entry");
    fs::write(updates_dir.join("u1"), "x").expect("Failed to write updates cache entry");
    fs::write(&metadata_file, "{\"last_checked\":\"x\"}")
        .expect("Failed to write update metadata file");

    let _home_guard = EnvVarGuard::set_path("HOME", &home_dir);
    let _xdg_cache_guard = EnvVarGuard::set_path("XDG_CACHE_HOME", &home_dir.join(".cache"));

    CacheCli {
        subcommand: Some(CacheSubcommand::Clear(CacheClearArgs {
            models: false,
            responses: false,
            updates: false,
            yes: true,
            dry_run: false,
        })),
    }
    .run()
    .await
    .expect("cache clear should succeed");

    assert!(!models_dir.exists(), "models directory should be removed");
    assert!(!updates_dir.exists(), "updates directory should be removed");
    assert!(
        !metadata_file.exists(),
        "update metadata file should be removed during full clear"
    );
}

#[tokio::test]
#[serial]
async fn clear_updates_dry_run_preserves_update_metadata_file() {
    let temp = tempdir().expect("Failed to create temp directory");
    let home_dir = temp.path().join("home");
    let cache_dir = home_dir.join(".cache").join("cortex");
    let updates_dir = cache_dir.join("updates");
    let metadata_file = home_dir.join(".cortex").join("update_cache.json");
    let update_entry = updates_dir.join("u1");

    fs::create_dir_all(&updates_dir).expect("Failed to create updates dir");
    fs::create_dir_all(metadata_file.parent().expect("Metadata parent should exist"))
        .expect("Failed to create metadata parent dir");
    fs::write(&update_entry, "x").expect("Failed to write updates cache entry");
    fs::write(&metadata_file, "{\"last_checked\":\"x\"}")
        .expect("Failed to write update metadata file");

    let _home_guard = EnvVarGuard::set_path("HOME", &home_dir);
    let _xdg_cache_guard = EnvVarGuard::set_path("XDG_CACHE_HOME", &home_dir.join(".cache"));

    CacheCli {
        subcommand: Some(CacheSubcommand::Clear(CacheClearArgs {
            models: false,
            responses: false,
            updates: true,
            yes: true,
            dry_run: true,
        })),
    }
    .run()
    .await
    .expect("cache clear dry-run should succeed");

    assert!(update_entry.exists(), "updates cache entry should remain after dry-run");
    assert!(
        metadata_file.exists(),
        "update metadata file should remain after dry-run"
    );
}

#[tokio::test]
#[serial]
async fn clear_updates_removes_zero_byte_metadata_file() {
    let temp = tempdir().expect("Failed to create temp directory");
    let home_dir = temp.path().join("home");
    let metadata_file = home_dir.join(".cortex").join("update_cache.json");

    fs::create_dir_all(metadata_file.parent().expect("Metadata parent should exist"))
        .expect("Failed to create metadata parent dir");
    fs::File::create(&metadata_file).expect("Failed to create zero-byte metadata file");

    let _home_guard = EnvVarGuard::set_path("HOME", &home_dir);
    let _xdg_cache_guard = EnvVarGuard::set_path("XDG_CACHE_HOME", &home_dir.join(".cache"));

    CacheCli {
        subcommand: Some(CacheSubcommand::Clear(CacheClearArgs {
            models: false,
            responses: false,
            updates: true,
            yes: true,
            dry_run: false,
        })),
    }
    .run()
    .await
    .expect("cache clear should succeed");

    assert!(
        !metadata_file.exists(),
        "zero-byte update metadata file should still be removed"
    );
}

#[tokio::test]
#[serial]
async fn clear_updates_removes_empty_updates_directory() {
    let temp = tempdir().expect("Failed to create temp directory");
    let home_dir = temp.path().join("home");
    let updates_dir = home_dir.join(".cache").join("cortex").join("updates");

    fs::create_dir_all(&updates_dir).expect("Failed to create empty updates dir");

    let _home_guard = EnvVarGuard::set_path("HOME", &home_dir);
    let _xdg_cache_guard = EnvVarGuard::set_path("XDG_CACHE_HOME", &home_dir.join(".cache"));

    CacheCli {
        subcommand: Some(CacheSubcommand::Clear(CacheClearArgs {
            models: false,
            responses: false,
            updates: true,
            yes: true,
            dry_run: false,
        })),
    }
    .run()
    .await
    .expect("cache clear should succeed");

    assert!(
        !updates_dir.exists(),
        "empty updates directory should still be removed"
    );
}
