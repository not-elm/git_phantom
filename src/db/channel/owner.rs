use crate::db::channel::RequestNotify;
use crate::error::ServerResult;
use crate::middleware::user_id::UserId;
use async_stream::__private::AsyncStream;
use gph_core::types::RequestId;
use sqlx::postgres::PgListener;
use sqlx::{Executor, PgPool, Postgres};
use std::future::Future;


pub async fn listen(pool: &PgPool, user_id: UserId) -> ServerResult<AsyncStream<RequestNotify, impl Future<Output=()> + Send>> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen("owner").await?;
    Ok(async_stream::stream! {
        while let Ok(notify) = listener.recv().await {
            let Ok(meta) = serde_json::from_str::<RequestNotify>(notify.payload()) else {
                continue;
            };
            if meta.to == user_id {
                yield meta;
            }
        }
    })
}

pub async fn response<'c, E>(pool: E, request_id: &RequestId, response: &[u8]) -> ServerResult
where
    E: Executor<'c, Database=Postgres>,
{
    sqlx::query(r#"
    UPDATE requests SET response = $1 WHERE request_id=$2
    "#)
        .bind(response)
        .bind(request_id.0)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db::channel::guest::{new_request, pop_response};
    use crate::db::channel::owner::response;
    use crate::error::ServerError;
    use crate::test::TestResult;
    use sqlx::postgres::PgListener;
    use sqlx::{PgPool, Row};

    #[sqlx::test]
    async fn ok_response(pool: PgPool) -> TestResult {
        let request_id = new_request(&pool).await?;
        let output = vec![1, 2, 3];
        response(&pool, &request_id, &output).await?;
        let actual = pop_response(&pool, &request_id).await?;
        assert_eq!(output, actual);
        Ok(())
    }

    #[sqlx::test]
    async fn err_response_if_not_exists_response(pool: PgPool) -> TestResult {
        let request_id = new_request(&pool).await?;
        let result = pop_response(&pool, &request_id).await;
        assert!(matches!(result, Err(ServerError::Sqlx(_))));
        Ok(())
    }

    #[sqlx::test]
    async fn request_deleted_after_pop(pool: PgPool) -> TestResult {
        let request_id = new_request(&pool).await?;
        let output = vec![1, 2, 3];
        response(&pool, &request_id, &output).await?;
        assert_eq!(requests_count(&pool).await?, 1);

        pop_response(&pool, &request_id).await?;
        assert_eq!(requests_count(&pool).await?, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn recv_notify(pool: PgPool) -> TestResult {
        let mut guest_listener = PgListener::connect_with(&pool).await?;
        guest_listener.listen("guest").await?;

        let request_id = new_request(&pool).await?;
        let output = vec![1, 2, 3];
        response(&pool, &request_id, &output).await?;

        let notify = guest_listener.recv().await?;
        assert_eq!(notify.payload(), request_id.0.to_string());
        Ok(())
    }

    async fn requests_count(pool: &PgPool) -> TestResult<i64> {
        let count: i64 = sqlx::query("SELECT count(*) FROM requests")
            .fetch_one(pool)
            .await?
            .get(0);
        Ok(count)
    }
}
