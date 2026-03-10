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

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.path));
    }
}

pub(crate) fn write_page(root: &Path, relative_path: &str, content: &str) {
    let path = root.join(relative_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}
