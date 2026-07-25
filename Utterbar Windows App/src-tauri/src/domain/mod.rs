//! Shared domain types. All types here mirror to TypeScript via `ts-rs`.
//!
//! The generated bindings land in `../../src/generated/` (see `scripts/sync-types.ts`).

pub mod events;
pub mod history_entry;
pub mod mode;
pub mod model;
pub mod settings;
pub mod shortcut;
pub mod vocabulary;

pub use events::*;
pub use history_entry::*;
pub use mode::*;
pub use model::*;
pub use settings::*;
pub use shortcut::*;
pub use vocabulary::*;
