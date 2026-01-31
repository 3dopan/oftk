pub mod data;
pub mod core;
pub mod ui;
pub mod app;
pub mod platform;
pub mod utils;

// Re-export commonly used items for benchmarking
pub use data::models::FileAlias;
pub use core::search::SearchEngine;
pub use core::alias::AliasManager;
pub use core::file_manager::FileManager;
