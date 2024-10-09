pub mod users;
pub mod channel;

#[cfg(test)]
pub(crate) mod test {
    use crate::middleware::session_token::SessionToken;
    use crate::middleware::user_id::UserId;
    use sqlx::types::Uuid;
    use sqlx::{Executor, PgPool};

    pub const SESSION1: SessionToken = SessionToken(Uuid::from_u128(0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8u128));

    pub trait DBInit {
        async fn init(&self);
    }
    impl DBInit for PgPool {
        async fn init(&self) {
            let sql = include_str!("../migrations/000_init.sql");
            self.execute(sql).await.unwrap();
            sqlx::query(r#"
            INSERT INTO users(user_id, session_token) VALUES($1, $2)
            "#)
                .bind(UserId::USER1.0)
                .bind(SESSION1.0)
                .execute(self)
                .await
                .unwrap();
        }
    }
}