use sqlx_migrator::migrator::{Migrator, MigratorTrait};
use sqlx_migrator::sqlx::Postgres;

mod migrations;
#[tokio::main]
async fn main() {
    let uri = std::env::var("POSTGRES_DATABASE_URL").unwrap();
    let pool = sqlx::Pool::<Postgres>::connect(&uri).await.unwrap();
    let mut migrator = Migrator::new(&pool);
    migrator.add_migrations(migrations::migrations());
    // There are two way to run migration. Either you can create cli as shown below
    sqlx_migrator::cli::run(Box::new(migrator)).await.unwrap();
    // Or you can directly use migrator apply_all function instead of creating
    // cli
    // migrator.apply_all().await.unwrap();
}
