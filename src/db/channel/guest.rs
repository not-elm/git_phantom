use crate::db::channel::RequestNotify;
use crate::error::ServerResult;
use async_stream::__private::AsyncStream;
use gph_core::types::RequestId;
use sqlx::postgres::PgListener;
use sqlx::{PgPool, Row};
use std::future::Future;

pub async fn listen(pool: PgPool, request_id: RequestId) -> ServerResult<AsyncStream<Vec<u8>, impl Future<Output=()> + Send + 'static>> {
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("guest").await?;

    let request_id_string = request_id.0.to_string();

    Ok(async_stream::stream! {
        while let Ok(notify) = listener.recv().await {
            if request_id_string != notify.payload() {
                continue;
            }

            if let Ok(response) = pop_response(&pool, &request_id).await{
                yield response;
            }
       }
    })
}

pub async fn request_to_owner(pool: &PgPool, request: &RequestNotify) -> ServerResult {
    sqlx::query(r#"
    SELECT PG_NOTIFY('owner', $1)
    "#)
        .bind(serde_json::to_string(request).unwrap())
        .execute(pool)
        .await?;
    Ok(())
}

pub(crate) async fn new_request(pool: &PgPool, request_body: &[u8]) -> ServerResult<RequestId> {
    let request_id = sqlx::query(r#"
    INSERT INTO requests(request_body) VALUES($1) RETURNING request_id
    "#)
        .bind(request_body)
        .fetch_one(pool)
        .await?
        .get(0);
    Ok(RequestId(request_id))
}

pub(crate) async fn pop_response(pool: &PgPool, request_id: &RequestId) -> ServerResult<Vec<u8>> {
    let response = sqlx::query(r#"
    DELETE FROM requests WHERE request_id=$1 RETURNING response
    "#)
        .bind(request_id.0)
        .fetch_one(pool)
        .await?
        .try_get(0)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::db::channel;
    use crate::db::channel::guest::{new_request, request_to_owner};
    use crate::db::channel::{convert_to_git_request, RequestNotify};
    use crate::db::test::DBInit;
    use crate::middleware::user_id::UserId;
    use crate::test::TestResult;
    use futures_util::pin_mut;
    use futures_util::stream::StreamExt;
    use sqlx::PgPool;
    use std::time::Duration;

    #[sqlx::test]
    async fn ok_new_request(pool: PgPool) -> TestResult {
        let id = new_request(&pool, &[]).await?;
        assert!(!id.0.as_bytes().is_empty());
        Ok(())
    }

    #[sqlx::test]
    async fn ok_recv_response(pool: PgPool) -> TestResult {
        let id = new_request(&pool, &[]).await?;
        let stream = channel::guest::listen(pool.clone(), id).await?;
        pin_mut!(stream);

        let data = vec![1, 2, 3];
        channel::owner::response(&pool, &id, &data).await?;

        tokio::select! {
            actual =  stream.next() => {
                assert_eq!(actual.unwrap(), data);
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                panic!("time out");
            }
        }
        Ok(())
    }

    #[sqlx::test]
    async fn ok_recv_request(pool: PgPool) -> TestResult {
        pool.init().await;
        let stream = channel::owner::listen(pool.clone(), UserId::USER1).await?;
        pin_mut!(stream);
        let request_id = new_request(&pool, &[]).await?;
        let request = RequestNotify {
            id: request_id,
            to: UserId::USER1,
            ..Default::default()
        };

        tokio::time::sleep(Duration::from_millis(100)).await;
        request_to_owner(&pool, &request).await?;
        tokio::select! {
            actual =  stream.next() => {
                assert_eq!(actual.unwrap(), convert_to_git_request(request, vec![]));
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                panic!("time out");
            }
        }

        Ok(())
    }

    #[sqlx::test]
    async fn no_recv_request(pool: PgPool) -> TestResult {
        let stream = channel::owner::listen(pool.clone(), UserId::USER1).await?;
        pin_mut!(stream);
        let request_id = new_request(&pool, &[]).await?;
        let request = RequestNotify {
            id: request_id,
            ..Default::default()
        };
        request_to_owner(&pool, &request).await?;
        tokio::select! {
            _ =  stream.next() => {
                panic!("Expect not to recv request but was not")
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // OK
            }
        }
        Ok(())
    }
}