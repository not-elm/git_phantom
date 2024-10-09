use crate::error::{ServerError, ServerResult};
use crate::middleware::user_id::UserId;
use sqlx::{PgPool, Row};

pub trait RoomsTable {
    async fn update_room_status(&self, user_id: UserId, is_open: bool) -> ServerResult;

    async fn is_open_room(&self, user_id: UserId) -> ServerResult<bool>;
}

impl RoomsTable for PgPool {
    async fn update_room_status(&self, user_id: UserId, is_open: bool) -> ServerResult {
        sqlx::query(r#"
        INSERT INTO rooms(user_id, is_open) VALUES($1, $2)
        ON CONFLICT(user_id) DO UPDATE SET is_open=$2
        "#)
            .bind(user_id.0)
            .bind(is_open)
            .execute(self)
            .await?;
        Ok(())
    }

    async fn is_open_room(&self, user_id: UserId) -> ServerResult<bool> {
        let result = sqlx::query(r#"
        SELECT is_open FROM rooms WHERE user_id=$1
        "#)
            .bind(user_id.0)
            .fetch_one(self)
            .await;
        match result {
            Ok(row) => Ok(row.get(0)),
            Err(sqlx::Error::RowNotFound) => Err(ServerError::UserRoomIsNotOpen),
            Err(e) => Err(ServerError::Sqlx(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::rooms::RoomsTable;
    use crate::error::ServerError;
    use crate::middleware::user_id::UserId;
    use crate::test::TestResult;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn err_if_user_not_exists(pool: PgPool) {
        let result = pool.is_open_room(UserId::USER1).await;
        assert!(matches!(result, Err(ServerError::UserRoomIsNotOpen)));
    }

    #[sqlx::test]
    async fn ok_open(pool: PgPool) -> TestResult {
        pool.update_room_status(UserId::USER1, true).await?;
        let is_open = pool.is_open_room(UserId::USER1).await?;
        assert!(is_open);
        Ok(())
    }

    #[sqlx::test]
    async fn ok_close_room(pool: PgPool) -> TestResult {
        pool.update_room_status(UserId::USER1, false).await?;
        let is_open = pool.is_open_room(UserId::USER1).await?;
        assert!(!is_open);
        Ok(())
    }
}