use axum::http::StatusCode;

pub async fn readyz_handler() -> StatusCode {
    StatusCode::OK
}

pub async fn healthz_handler() -> StatusCode {
    StatusCode::OK
}
