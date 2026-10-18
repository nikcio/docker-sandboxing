// Shared by every test module that mutates the CLI environment.
pub(crate) static CLI_TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// Edition 2024 marks `std::env::set_var`/`remove_var` unsafe because they are
// UB when another thread reads the environment concurrently. Every
// environment-mutating test in the crate holds `CLI_TEST_ENV_LOCK` while it
// runs, which serializes all of them, so no concurrent access is possible.
// The wrapper functions below contain that unsafety instead of spreading
// `unsafe {}` blocks across every call site; keep using only these in tests.

/// Safety contract: caller must hold `CLI_TEST_ENV_LOCK`.
pub(crate) fn set_var(key: &str, value: impl AsRef<std::ffi::OsStr>) {
    // SAFETY: guaranteed by the caller holding the env lock (see above).
    unsafe { std::env::set_var(key, value) }
}

/// Safety contract: caller must hold `CLI_TEST_ENV_LOCK`.
pub(crate) fn remove_var(key: &str) {
    // SAFETY: guaranteed by the caller holding the env lock (see above).
    unsafe { std::env::remove_var(key) }
}
