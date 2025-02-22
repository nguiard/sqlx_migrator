use sqlx::PgConnection;
use sqlx_migrator::error::Error;
use sqlx_migrator::migration::MigrationTrait;
use sqlx_migrator::operation::OperationTrait;
use sqlx_migrator::sqlx::Postgres;

pub(crate) struct M0001Operation;

#[async_trait::async_trait]
impl OperationTrait<Postgres> for M0001Operation {
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        sqlx::query("CREATE TABLE sample (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(connection)
            .await?;
        Ok(())
    }

    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        sqlx::query("DROP TABLE sample").execute(connection).await?;
        Ok(())
    }
}

pub(crate) struct M0001Migration;

#[async_trait::async_trait]
impl MigrationTrait<Postgres> for M0001Migration {
    fn app(&self) -> &str {
        "main"
    }

    fn name(&self) -> &str {
        "m0001_simple"
    }

    fn parents(&self) -> Vec<Box<dyn MigrationTrait<Postgres>>> {
        vec![]
    }

    fn operations(&self) -> Vec<Box<dyn OperationTrait<Postgres>>> {
        vec![Box::new(M0001Operation)]
    }
}
