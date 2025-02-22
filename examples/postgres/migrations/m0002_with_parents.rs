use sqlx::PgConnection;
use sqlx_migrator::error::Error;
use sqlx_migrator::migration::MigrationTrait;
use sqlx_migrator::operation::OperationTrait;
use sqlx_migrator::sqlx::Postgres;

pub(crate) struct M0002Operation;

#[async_trait::async_trait]
impl OperationTrait<Postgres> for M0002Operation {
    async fn up(&self, connection: &mut PgConnection) -> Result<(), Error> {
        sqlx::query("INSERT INTO sample (id, name) VALUES (99, 'Some text') ")
            .execute(connection)
            .await?;
        Ok(())
    }

    async fn down(&self, connection: &mut PgConnection) -> Result<(), Error> {
        sqlx::query("DELETE FROM sample WHERE id = 99")
            .execute(connection)
            .await?;
        Ok(())
    }
}

pub(crate) struct M0002Migration;

#[async_trait::async_trait]
impl MigrationTrait<Postgres> for M0002Migration {
    fn app(&self) -> &str {
        "main"
    }

    fn name(&self) -> &str {
        "m0002_with_parents"
    }

    fn parents(&self) -> Vec<Box<dyn MigrationTrait<Postgres>>> {
        vec![Box::new(crate::migrations::m0001_simple::M0001Migration)]
    }

    fn operations(&self) -> Vec<Box<dyn OperationTrait<Postgres>>> {
        vec![Box::new(M0002Operation)]
    }
}
