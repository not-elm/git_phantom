use crate::middleware::user_id::UserId;

pub async fn user_id(
    user_id: UserId
) -> String {
    user_id.0.to_string()
}