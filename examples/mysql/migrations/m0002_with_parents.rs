use sqlx::MySqlConnection;
use sqlx_migrator::error::Error;
use sqlx_migrator::migration::MigrationTrait;
use sqlx_migrator::operation::OperationTrait;
use sqlx_migrator::sqlx::MySql;

pub(crate) struct M0002Operation;

#[async_trait::async_trait]
impl OperationTrait<MySql> for M0002Operation {
    async fn up(&self, connection: &mut MySqlConnection) -> Result<(), Error> {
        sqlx::query("INSERT INTO sample (id, name) VALUES (99, 'Some text') ")
            .execute(connection)
            .await?;
        Ok(())
    }

    async fn down(&self, connection: &mut MySqlConnection) -> Result<(), Error> {
        sqlx::query("DELETE FROM sample WHERE id = 99")
            .execute(connection)
            .await?;
        Ok(())
    }
}

pub(crate) struct M0002Migration;

#[async_trait::async_trait]
impl MigrationTrait<MySql> for M0002Migration {
    fn app(&self) -> &str {
        "main"
    }

    fn name(&self) -> &str {
        "m0002_with_parents"
    }

    fn parents(&self) -> Vec<Box<dyn MigrationTrait<MySql>>> {
        vec![Box::new(crate::migrations::m0001_simple::M0001Migration)]
    }

    fn operations(&self) -> Vec<Box<dyn OperationTrait<MySql>>> {
        vec![Box::new(M0002Operation)]
    }
}
