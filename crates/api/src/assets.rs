use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;

pub async fn favicon() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("image/x-icon"),
        )],
        include_bytes!("../assets/favicon.ico").as_slice(),
    )
}

pub async fn logo_svg() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("image/svg+xml"),
        )],
        include_bytes!("../assets/logo.svg").as_slice(),
    )
}

pub async fn logo_png() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, HeaderValue::from_static("image/png"))],
        include_bytes!("../assets/logo.png").as_slice(),
    )
}
