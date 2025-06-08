use feed_crafter::common::FileSystem;
use std::env;
use std::sync::Mutex;

#[allow(dead_code)]
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub struct TempEnv {
    vars: Vec<(String, Option<String>)>,
}

#[cfg(test)]
impl TempEnv {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { vars: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn set(&mut self, key: &str, value: &str) {
        // Store the original value (if any) for restoration
        let original = env::var(key).ok();
        self.vars.push((key.to_string(), original));
        unsafe {
            env::set_var(key, value);
        }
    }
}

#[cfg(test)]
impl Drop for TempEnv {
    #[allow(dead_code)]
    fn drop(&mut self) {
        for (key, original_value) in &self.vars {
            unsafe {
                match original_value {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub fn acquire_env_lock() -> std::sync::MutexGuard<'static, ()> {
    match ENV_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            // If the mutex is poisoned, recover from it
            poisoned.into_inner()
        }
    }
}

#[cfg(test)]
pub struct MockFileSystem {
    pub file_exists: bool,
    pub file_content: Option<String>,
    pub written_files: std::cell::RefCell<std::collections::HashMap<String, String>>,
}

#[cfg(test)]
impl MockFileSystem {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            file_exists: false,
            file_content: None,
            written_files: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn with_existing_file(content: String) -> Self {
        Self {
            file_exists: true,
            file_content: Some(content),
            written_files: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    // Helper to check what was written during tests
    #[allow(dead_code)]
    pub fn get_written_content(&self, path: &str) -> Option<String> {
        self.written_files.borrow().get(path).cloned()
    }

    #[allow(dead_code)]
    pub fn was_file_written(&self, path: &str) -> bool {
        self.written_files.borrow().contains_key(path)
    }
}

#[cfg(test)]
impl FileSystem for MockFileSystem {
    type Reader = std::io::Cursor<Vec<u8>>;

    #[allow(dead_code)]
    fn exists(&self, _path: &str) -> bool {
        self.file_exists
    }

    #[allow(dead_code)]
    fn open(&self, _path: &str) -> Result<Self::Reader, std::io::Error> {
        match &self.file_content {
            Some(content) => Ok(std::io::Cursor::new(content.as_bytes().to_vec())),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            )),
        }
    }

    #[allow(dead_code)]
    fn write(&self, path: &str, contents: &str) -> Result<(), std::io::Error> {
        self.written_files
            .borrow_mut()
            .insert(path.to_string(), contents.to_string());
        Ok(())
    }
}
