//! Ship registry v0.1 — dependency resolver and content-addressed package cache.
//!
//! Public API surface for callers:
//!
//! - [`install::resolve_and_fetch`] — top-level orchestration.
//! - [`cache::PackageCache`] — content-addressed cache (`~/.ship/cache/`).
//! - [`constraint::parse_constraint`] / [`constraint::VersionConstraint`] — constraint parsing.
//! - [`resolver::resolve_version`] — git tag → exact commit resolution.
//! - [`hash::compute_tree_hash`] / [`hash::compute_file_hash`] — deterministic SHA-256 hashing.
//! - [`types`] — stub `ShipManifest`, `ShipLock`, `LockedPackage` until compiler crate exposes its types.

pub mod cache;
pub mod constraint;
pub mod fetch;
pub mod hash;
pub mod install;
pub mod resolver;
pub mod tracking;
pub mod types;

pub use cache::{CachedPackage, PackageCache};
pub use constraint::{VersionConstraint, parse_constraint};
pub use hash::{compute_file_hash, compute_tree_hash};
pub use install::{InstallOptions, InstallResult, resolve_and_fetch};
pub use resolver::{ResolvedVersion, resolve_version};
pub use types::{
    Dependency, LockedPackage, ShipLock, ShipManifest, SyncStatus,
    parse_ship_lock, serialize_ship_lock,
};
