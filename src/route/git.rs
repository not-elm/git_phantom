use crate::db;
use crate::db::channel::RequestNotify;
use crate::db::rooms::RoomsTable;
use crate::error::{ServerError, ServerResult};
use crate::middleware::user_id::UserId;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue};
use axum::response::{IntoResponse, Response};
use futures_util::{pin_mut, StreamExt};
use http_body_util::BodyExt;
use sqlx::PgPool;
use std::str::FromStr;

pub async fn git(
    Path((user_id, path)): Path<(i64, String)>,
    State(pool): State<PgPool>,
    request: Request,
) -> Response {
    let user_id = UserId(user_id);
    if !pool.is_open_room(user_id).await.is_ok_and(|is_open| is_open) {
        return ServerError::UserRoomIsNotOpen.into_response();
    }

    listen_request(pool, path, user_id, request).await.unwrap_or_else(|e| e.into_response())
}

async fn listen_request(
    pool: PgPool,
    path_info: String,
    user_id: UserId,
    request: Request,
) -> ServerResult<Response> {
    let mut request_notify = RequestNotify {
        to: user_id,
        id: Default::default(),
        path_info,
        request_method: request.method().to_string(),
        query_string: request.uri().query().map(String::from),
        content_length: request.headers().get(header::CONTENT_LENGTH).and_then(|v| v.to_str().ok()).map(String::from),
        content_type: request.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).map(String::from),
    };

    let Ok(request_body) = request.into_body().collect().await else {
        return Err(ServerError::FailedParseRequestBody);
    };

    let request_id = db::channel::guest::new_request(&pool, request_body.to_bytes().as_ref()).await?;
    request_notify.id = request_id;
    let stream = db::channel::guest::listen(pool.clone(), request_id).await?;
    pin_mut!(stream);

    db::channel::guest::request_to_owner(&pool, &request_notify).await?;

    let response = stream
        .next()
        .await
        .ok_or(ServerError::FailedRecvGitResponse)?;

    Ok(convert_to_response(&response))
}

fn convert_to_response(output: &[u8]) -> Response {
    let mut response = Response::<Body>::default();
    let (body_index, headers) = parse_headers(output);
    *response.headers_mut() = headers;
    if body_index < output.len() {
        *response.body_mut() = Body::from(output[body_index..].to_vec());
    }
    response
}

fn parse_headers(output: &[u8]) -> (usize, HeaderMap<HeaderValue>) {
    let header_end_index = header_end_index(output).unwrap();
    let mut headers = HeaderMap::new();
    for line in std::str::from_utf8(&output[..=header_end_index]).unwrap().split("\r\n") {
        let mut header = line.split(": ");
        let Some(header_name) = header.next() else {
            continue;
        };
        let Some(header_value) = header.next() else {
            continue;
        };
        headers.insert(HeaderName::from_str(header_name).unwrap(), HeaderValue::from_str(header_value).unwrap());
    }

    (header_end_index + 5, headers)
}

fn header_end_index(output: &[u8]) -> Option<usize> {
    for i in 1..(output.len()) {
        if !(output[i] == b'\r' && output[i + 1] == b'\n') {
            continue;
        }
        if output[i + 2] == b'\r' && output[i + 3] == b'\n' {
            return Some(i - 1);
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use crate::middleware::user_id::UserId;
    use crate::test::{test_app, TestResult};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn err_if_invalid_user(pool: PgPool) -> TestResult {
        let app = test_app(pool).await;
        let response = app
            .oneshot(git_request(UserId(0), "sample.git", "/info/refs"))
            .await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }

    fn git_request(user_id: UserId, repository: &str, path: &str) -> Request {
        Request::get(format!("/git/{}/{repository}{path}", user_id.0)).body(Body::empty()).unwrap()
    }
}