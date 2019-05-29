use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub use localstoragefs::fs;
    } else {
        pub use std::fs;
    }
}
