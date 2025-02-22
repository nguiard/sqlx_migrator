//! Module defining migration trait
//!
//! To create own implement migration trait for type
//!
//! ### Example
//!
//! ```rust,no_run
//! use sqlx_migrator::error::Error;
//! use sqlx_migrator::migration::MigrationTrait;
//! use sqlx_migrator::operation::OperationTrait;
//! use sqlx_migrator::sqlx::Sqlite;
//!
//! struct ExampleMigration;
//! impl MigrationTrait<Sqlite> for ExampleMigration {
//!     fn app(&self) -> &str {
//!         "example"
//!     }
//!
//!     fn name(&self) -> &str {
//!         "first_migration"
//!     }
//!
//!     fn parents(&self) -> Vec<Box<dyn MigrationTrait<Sqlite>>> {
//!         vec![]
//!     }
//!
//!     fn operations(&self) -> Vec<Box<dyn OperationTrait<Sqlite>>> {
//!         vec![]
//!     }
//!
//!     fn replaces(&self) -> Vec<Box<dyn MigrationTrait<Sqlite>>> {
//!         vec![]
//!     }
//!
//!     fn run_before(&self) -> Vec<Box<dyn MigrationTrait<Sqlite>>> {
//!         vec![]
//!     }
//!
//!     fn is_atomic(&self) -> bool {
//!         true
//!     }
//! }
//! ```

use std::hash::Hash;

use chrono::{DateTime, Utc};

use crate::operation::OperationTrait;

/// Trait for migration
#[allow(clippy::module_name_repetitions)]
pub trait MigrationTrait<DB>: Send + Sync {
    /// Migration app name. Can be name of folder or library where migration is
    /// located
    fn app(&self) -> &str;

    /// Migration name. Can be file name without extension
    fn name(&self) -> &str;

    /// Parents of migration (migrations that should be applied before this
    /// migration)
    fn parents(&self) -> Vec<Box<dyn MigrationTrait<DB>>>;

    /// Operation performed for migration (create, drop, etc.)
    fn operations(&self) -> Vec<Box<dyn OperationTrait<DB>>>;

    /// Replace certain migrations. If any one of listed migration is applied
    /// than migration will not be applied else migration will apply instead of
    /// applying those migration.
    fn replaces(&self) -> Vec<Box<dyn MigrationTrait<DB>>> {
        vec![]
    }

    /// Run before certain migration. This can be helpful in condition where
    /// other library migration need to be applied after this migration
    fn run_before(&self) -> Vec<Box<dyn MigrationTrait<DB>>> {
        vec![]
    }

    /// Whether migration is atomic or not. By default it is true
    fn is_atomic(&self) -> bool {
        true
    }
}

impl<DB> PartialEq for dyn MigrationTrait<DB>
where
    DB: sqlx::Database,
{
    fn eq(&self, other: &Self) -> bool {
        self.app() == other.app() && self.name() == other.name()
    }
}

impl<DB> Eq for dyn MigrationTrait<DB> where DB: sqlx::Database {}

impl<DB> Hash for dyn MigrationTrait<DB>
where
    DB: sqlx::Database,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.app().hash(state);
        self.name().hash(state);
    }
}

/// Migration struct created from sql table. Struct contains 4 fields which maps
/// to `id`, `app`, `name`, `applied_time` sql fields. It is used to list
/// applied migrations
#[derive(sqlx::FromRow, Clone)]
pub struct AppliedMigrationSqlRow {
    id: i32,
    app: String,
    name: String,
    applied_time: DateTime<Utc>,
}

impl AppliedMigrationSqlRow {
    /// Return id value present on database
    #[must_use]
    pub fn id(&self) -> i32 {
        self.id
    }

    /// Return migration applied time
    #[must_use]
    pub fn applied_time(&self) -> DateTime<Utc> {
        self.applied_time
    }
}

impl<DB> PartialEq<Box<dyn MigrationTrait<DB>>> for AppliedMigrationSqlRow
where
    DB: sqlx::Database,
{
    fn eq(&self, other: &Box<dyn MigrationTrait<DB>>) -> bool {
        self.app == other.app() && self.name == other.name()
    }
}
