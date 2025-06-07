use std::env;
use std::sync::Mutex;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

pub struct TempEnv {
    vars: Vec<(String, Option<String>)>,
}

impl TempEnv {
    pub fn new() -> Self {
        Self { vars: Vec::new() }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        // Store the original value (if any) for restoration
        let original = env::var(key).ok();
        self.vars.push((key.to_string(), original));
        unsafe {
            env::set_var(key, value);
        }
    }
}

impl Drop for TempEnv {
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

pub fn acquire_env_lock() -> std::sync::MutexGuard<'static, ()> {
    match ENV_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            // If the mutex is poisoned, recover from it
            poisoned.into_inner()
        }
    }
}
