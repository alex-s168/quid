# quid
fast, concurrent, lock-free UID (as u64) generation library

```rs
let uid: quid::UID = quid::UID::new();
// quid::UID implements Clone, Eq, Hash, Debug, and Display
```
