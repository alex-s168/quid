use std::{cell::Cell, sync::atomic::{AtomicU64, Ordering}};

pub type UidTy = u64;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UID(UidTy);

impl std::fmt::Debug for UID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UID({})", self.0)
    }
}

impl std::fmt::Display for UID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Into<UidTy> for UID {
    fn into(self) -> UidTy {
        self.0
    }
}

static GLOBAL_NEXT_UID: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static UID_BASE: Cell<UidTy> = const { Cell::new(0) };
    static UID_REM: Cell<UidTy> = const { Cell::new(0) };
}

static TH_ALLOC_STEP: UidTy = 512;

impl UID {
    pub fn new() -> Self {
        if UID_REM.get() == 0 {
            let base = GLOBAL_NEXT_UID.fetch_add(TH_ALLOC_STEP, Ordering::Relaxed);
            UID_BASE.set(base);
            UID_REM.set(TH_ALLOC_STEP - 1);
            UID(base + TH_ALLOC_STEP - 1)
        } else {
            let val = UID_REM.get() - 1;
            UID_REM.set(val);
            UID(UID_BASE.get() + val)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UID;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_uid_uniqueness_and_type() {
        let uid1 = UID::new();
        let uid2 = UID::new();

        assert_ne!(uid1, uid2, "UIDs should be unique");
    }

    #[test]
    fn test_uid_concurrency_safety() {
        const THREADS: usize = 10;
        const IDS_PER_THREAD: usize = 10000;

        let all_ids = Arc::new(Mutex::new(HashSet::new()));
        let mut handles = vec![];

        for _ in 0..THREADS {
            let ids_clone = Arc::clone(&all_ids);
            let handle = thread::spawn(move || {
                let mut local_ids = vec![];
                for _ in 0..IDS_PER_THREAD {
                    let id = UID::new();
                    local_ids.push(id);
                }

                let mut shared = ids_clone.lock().unwrap();
                for id in local_ids {
                    assert!(
                        shared.insert(id.clone()),
                        "Duplicate UID found: {}",
                        id
                    );
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let total_ids = THREADS * IDS_PER_THREAD;
        let shared = all_ids.lock().unwrap();
        assert_eq!(
            shared.len(),
            total_ids,
            "Expected {} unique IDs, but found {}",
            total_ids,
            shared.len()
        );
    }
}
