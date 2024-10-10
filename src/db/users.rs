use crate::error::{ServerError, ServerResult};
use crate::middleware::session_token::SessionToken;
use crate::middleware::user_id::UserId;
use async_trait::async_trait;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait UsersTable {
    async fn insert_into_users(&self, user_id: &UserId) -> ServerResult<SessionToken>;

    async fn select_from_users(&self, session_token: &SessionToken) -> ServerResult<UserId>;
}

#[async_trait]
impl UsersTable for PgPool {
    async fn insert_into_users(&self, user_id: &UserId) -> ServerResult<SessionToken> {
        let row = sqlx::query(r#"
        INSERT INTO users(user_id) VALUES($1)
        ON CONFLICT(user_id) DO UPDATE SET session_token=gen_random_uuid(), created_at=CURRENT_TIMESTAMP
        RETURNING session_token
        "#)
            .bind(user_id.0)
            .fetch_one(self)
            .await?;
        Ok(SessionToken(row.get(0)))
    }

    async fn select_from_users(&self, session_token: &SessionToken) -> ServerResult<UserId> {
        let result = sqlx::query(r#"
        SELECT user_id FROM users WHERE session_token=$1
        "#)
            .bind(session_token.0)
            .fetch_one(self)
            .await;
        match result {
            Ok(row) => {
                Ok(UserId(row.get(0)))
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(ServerError::InvalidSessionToken)
            }
            Err(e) => {
                Err(ServerError::Sqlx(e))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::db::users::UsersTable;
    use crate::error::ServerError;
    use crate::middleware::session_token::SessionToken;
    use crate::middleware::user_id::UserId;
    use crate::test::TestResult;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn ok_insert_user(pool: PgPool) {
        pool.insert_into_users(&UserId::USER1).await.unwrap();
    }

    #[sqlx::test]
    async fn ok_select_user(pool: PgPool) -> TestResult {
        let session_token = pool.insert_into_users(&UserId::USER1).await?;
        let user = pool.select_from_users(&session_token).await?;
        assert_eq!(user, UserId::USER1);
        Ok(())
    }

    #[sqlx::test]
    async fn err_select_user_if_not_exists(pool: PgPool) {
        let result = pool.select_from_users(&SessionToken::max()).await.unwrap_err();
        assert!(matches!(result, ServerError::InvalidSessionToken))
    }

    #[sqlx::test]
    async fn session_token_update_if_insert_again(pool: PgPool) -> TestResult {
        let session_token1 = pool.insert_into_users(&UserId::USER1).await?;
        let session_token2 = pool.insert_into_users(&UserId::USER1).await?;
        assert_ne!(session_token1, session_token2);
        Ok(())
    }
}