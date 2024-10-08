pub mod users;
pub mod channel;

#[cfg(test)]
pub(crate) mod test {
    use sqlx::{Executor, PgPool};

    pub trait DBInit {
        async fn init(&self);
    }
    impl DBInit for PgPool {
        async fn init(&self) {
            let sql = include_str!("../migrations/000_init.sql");
            self.execute(sql).await.unwrap();
        }
    }
}