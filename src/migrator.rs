use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use petgraph::Graph;
use sqlx::{Pool, Transaction};

use crate::error::Error;
use crate::migration::Migration;

#[async_trait::async_trait]
pub trait MigratorTrait: Send {
    type Database: sqlx::Database;
    fn graph(&self) -> &Graph<Box<dyn Migration<Database = Self::Database>>, ()>;
    fn graph_mut(&mut self) -> &mut Graph<Box<dyn Migration<Database = Self::Database>>, ()>;
    fn migrations_map(&self) -> &HashMap<String, NodeIndex>;
    fn pool(&self) -> &Pool<Self::Database>;

    async fn ensure_migration_table(&self) -> Result<(), Error>;
    async fn add_migration_to_table<'t>(
        &self,
        migration_name: String,
        transaction: &mut Transaction<'t, Self::Database>,
    ) -> Result<(), Error>;
    async fn delete_migration_from_table<'t>(
        &self,
        migration_name: String,
        transaction: &mut Transaction<'t, Self::Database>,
    ) -> Result<(), Error>;
    async fn list_applied_migration(&self) -> Result<Vec<String>, Error>;

    /// Add vector of migrations to Migrator
    fn add_migrations(
        &mut self,
        migrations: Vec<Box<dyn Migration<Database = Self::Database>>>,
    ) -> Vec<NodeIndex> {
        let mut node_index_vec = Vec::new();
        for migration in migrations {
            node_index_vec.push(self.add_migration(migration));
        }
        node_index_vec
    }

    /// Add single migration to migrator
    fn add_migration(
        &mut self,
        migration: Box<dyn Migration<Database = Self::Database>>,
    ) -> NodeIndex {
        let parents = migration.parents();
        let mut migrations_map = self.migrations_map().clone();
        let &mut node_index = migrations_map
            .entry(migration.name())
            .or_insert_with(|| self.graph_mut().add_node(migration));
        for parent in parents {
            let parent_index = self.add_migration(parent);
            self.graph_mut().add_edge(parent_index, node_index, ());
        }
        node_index
    }

    async fn apply_all_plan(
        &self,
    ) -> Result<Vec<&Box<dyn Migration<Database = Self::Database>>>, Error> {
        let applied_migrations = self.list_applied_migration().await?;
        tracing::info!("Creating apply migration plan");
        let mut added_node = Vec::new();
        let mut plan_vec = Vec::<&Box<dyn Migration<Database = Self::Database>>>::new();
        while added_node.len() < self.graph().node_indices().len() {
            for node_index in self.graph().node_indices() {
                let mut dfs = petgraph::visit::Dfs::new(&self.graph(), node_index);
                while let Some(nx) = dfs.next(&self.graph()) {
                    if !added_node.contains(&nx) {
                        let migration = &self.graph()[nx];
                        let parent_added = self
                            .graph()
                            .neighbors_directed(nx, petgraph::Direction::Incoming)
                            .all(|x| added_node.contains(&x));
                        if parent_added {
                            added_node.push(nx);
                            if !applied_migrations.contains(&migration.name()) {
                                plan_vec.push(migration);
                            }
                        }
                    }
                }
            }
        }
        Ok(plan_vec)
    }

    /// Apply missing migration
    /// # Errors
    /// If any migration or operation fails
    async fn apply(&self) -> Result<(), Error> {
        self.ensure_migration_table().await?;
        for migration in self.apply_all_plan().await? {
            self.apply_migration(migration).await?;
        }
        Ok(())
    }

    #[allow(clippy::borrowed_box)]
    async fn apply_migration(
        &self,
        migration: &Box<dyn Migration<Database = Self::Database>>,
    ) -> Result<(), Error> {
        tracing::info!("Applying migration {}", migration.name());
        let mut transaction = self.pool().begin().await?;
        for operation in migration.operations() {
            operation.up(&mut transaction).await?;
        }

        self.add_migration_to_table(migration.name(), &mut transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn revert_all_plan(
        &self,
    ) -> Result<Vec<&Box<dyn Migration<Database = Self::Database>>>, Error> {
        let applied_migrations = self.list_applied_migration().await?;
        tracing::info!("Creating revert migration plan");
        let mut added_node = Vec::new();
        let mut plan_vec = Vec::<&Box<dyn Migration<Database = Self::Database>>>::new();
        while added_node.len() < self.graph().node_indices().len() {
            for node_index in self.graph().node_indices() {
                let mut dfs = petgraph::visit::Dfs::new(&self.graph(), node_index);
                while let Some(nx) = dfs.next(&self.graph()) {
                    if !added_node.contains(&nx) {
                        let migration = &self.graph()[nx];
                        let parent_added = self
                            .graph()
                            .neighbors_directed(nx, petgraph::Direction::Incoming)
                            .all(|x| added_node.contains(&x));
                        if parent_added {
                            added_node.push(nx);
                            if applied_migrations.contains(&migration.name()) {
                                plan_vec.push(migration);
                            }
                        }
                    }
                }
            }
        }
        plan_vec.reverse();
        Ok(plan_vec)
    }

    /// Revert all applied migration
    /// # Errors
    /// If any migration or operation fails
    async fn revert(&self) -> Result<(), Error> {
        self.ensure_migration_table().await?;
        for migration in self.revert_all_plan().await? {
            self.revert_migration(migration).await?;
        }
        Ok(())
    }

    #[allow(clippy::borrowed_box)]
    async fn revert_migration(
        &self,
        migration: &Box<dyn Migration<Database = Self::Database>>,
    ) -> Result<(), Error> {
        tracing::info!("Reverting migration {}", migration.name());
        let mut transaction = self.pool().begin().await?;
        let mut operations = migration.operations();
        operations.reverse();
        for operation in operations {
            operation.down(&mut transaction).await?;
        }
        self.delete_migration_from_table(migration.name(), &mut transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}
