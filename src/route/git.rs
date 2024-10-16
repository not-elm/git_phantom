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
use reqwest::StatusCode;
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

    let request_id = db::channel::guest::new_request(&pool, user_id, request_body.to_bytes().as_ref()).await?;
    request_notify.id = request_id;
    let stream = db::channel::guest::listen(pool.clone(), request_id).await?;
    pin_mut!(stream);

    db::channel::guest::request_to_owner(&pool, &request_notify).await?;

    let response = stream
        .next()
        .await
        .ok_or(ServerError::FailedRecvGitResponse)?;

    convert_to_response(&response)
}

fn convert_to_response(output: &[u8]) -> ServerResult<Response> {
    let mut response = Response::<Body>::default();
    let (status_code, body_index, headers) = parse_headers(output)?;
    *response.status_mut() = status_code;
    *response.headers_mut() = headers;
    if body_index < output.len() {
        *response.body_mut() = Body::from(output[body_index..].to_vec());
    }
    Ok(response)
}

fn parse_headers(output: &[u8]) -> ServerResult<(StatusCode, usize, HeaderMap<HeaderValue>)> {
    let header_end_index = header_end_index(output).ok_or(ServerError::FailedParseGitResponse)?;
    let mut status = StatusCode::OK;
    let mut headers = HeaderMap::new();
    for line in std::str::from_utf8(&output[..=header_end_index]).unwrap().split("\r\n") {
        let mut header = line.split(": ");
        let Some(header_name) = header.next() else {
            continue;
        };
        let Some(header_value) = header.next() else {
            continue;
        };
        if header_name == "Status" {
            let row_status = header_value.split(" ").next().ok_or(ServerError::FailedParseGitResponse)?;
            status = StatusCode::from_bytes(row_status.as_ref()).map_err(|_| ServerError::FailedParseGitResponse)?;
        } else {
            let name = HeaderName::from_str(header_name).map_err(|_| ServerError::FailedParseGitResponse)?;
            let value = HeaderValue::from_str(header_value).map_err(|_| ServerError::FailedParseGitResponse)?;
            headers.insert(name, value);
        }
    }

    Ok((status, header_end_index + 5, headers))
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
    use crate::route::git::convert_to_response;
    use crate::test::{test_app, TestResult};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[test]
    fn status_code_is_404() {
        let output = [83, 116, 97, 116, 117, 115, 58, 32, 52, 48, 52, 32, 78, 111, 116, 32, 70, 111, 117, 110, 100, 13, 10, 69, 120, 112, 105, 114, 101, 115, 58, 32, 70, 114, 105, 44, 32, 48, 49, 32, 74, 97, 110, 32, 49, 57, 56, 48, 32, 48, 48, 58, 48, 48, 58, 48, 48, 32, 71, 77, 84, 13, 10, 80, 114, 97, 103, 109, 97, 58, 32, 110, 111, 45, 99, 97, 99, 104, 101, 13, 10, 67, 97, 99, 104, 101, 45, 67, 111, 110, 116, 114, 111, 108, 58, 32, 110, 111, 45, 99, 97, 99, 104, 101, 44, 32, 109, 97, 120, 45, 97, 103, 101, 61, 48, 44, 32, 109, 117, 115, 116, 45, 114, 101, 118, 97, 108, 105, 100, 97, 116, 101, 13, 10, 13, 10];
        let response = convert_to_response(&output).unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn status_code_is_500() {
        let output = [83, 116, 97, 116, 117, 115, 58, 32, 53, 48, 48, 32, 73, 110, 116, 101, 114, 110, 97, 108, 32, 83, 101, 114, 118, 101, 114, 32, 69, 114, 114, 111, 114, 13, 10, 69, 120, 112, 105, 114, 101, 115, 58, 32, 70, 114, 105, 44, 32, 48, 49, 32, 74, 97, 110, 32, 49, 57, 56, 48, 32, 48, 48, 58, 48, 48, 58, 48, 48, 32, 71, 77, 84, 13, 10, 80, 114, 97, 103, 109, 97, 58, 32, 110, 111, 45, 99, 97, 99, 104, 101, 13, 10, 67, 97, 99, 104, 101, 45, 67, 111, 110, 116, 114, 111, 108, 58, 32, 110, 111, 45, 99, 97, 99, 104, 101, 44, 32, 109, 97, 120, 45, 97, 103, 101, 61, 48, 44, 32, 109, 117, 115, 116, 45, 114, 101, 118, 97, 108, 105, 100, 97, 116, 101, 13, 10, 13, 10];
        let response = convert_to_response(&output).unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

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