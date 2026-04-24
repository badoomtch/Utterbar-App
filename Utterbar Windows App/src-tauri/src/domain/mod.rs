//! Shared domain types. All types here mirror to TypeScript via `ts-rs`.
//!
//! The generated bindings land in `../../src/generated/` (see `scripts/sync-types.ts`).

pub mod events;
pub mod history_entry;
pub mod licence;
pub mod mode;
pub mod model;
pub mod settings;
pub mod shortcut;
pub mod vocabulary;

pub use events::*;
pub use history_entry::*;
// `licence::*` is intentionally not re-exported at the crate-domain level —
// licence types are only ever referenced through their full path
// (`crate::domain::licence::...`) so the service layer stays grep-able and so
// the blast radius of a rename stays inside the licence module.
pub use mode::*;
pub use model::*;
pub use settings::*;
pub use shortcut::*;
pub use vocabulary::*;
