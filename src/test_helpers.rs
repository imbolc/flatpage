use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) struct TestDir {
    path: PathBuf,
}

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

impl TestDir {
    /// Creates a unique temporary directory for a test case.
    pub(crate) fn new() -> Self {
        for _ in 0..100 {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "flatpage-test-{}-{timestamp}-{counter}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self { path },
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("failed to create test directory {path:?}: {error}"),
            }
        }

        panic!("failed to create a unique test directory after 100 attempts");
    }

    /// Returns the filesystem path of the temporary directory.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    /// Removes the temporary directory when the helper is dropped.
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.path));
    }
}

/// Creates parent directories and writes a Markdown file for a test.
pub(crate) fn write_page(root: &Path, relative_path: &str, content: &str) {
    let path = root.join(relative_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}
