use anyhow::Result;
use axum::Router;
use axum::http::Method;
use axum::http::header::CONTENT_TYPE;
use axum::routing::{get, post};
use services::{auth, dispatch, errors, sr_tools};
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;

mod config;
mod services;

const PORT: u16 = 21000;

pub async fn start_sdkserver() -> Result<()> {
    let span = tracing::span!(Level::DEBUG, "main");
    let _ = span.enter();

    let router = Router::new()
        .route(
            dispatch::QUERY_DISPATCH_ENDPOINT,
            get(dispatch::query_dispatch),
        )
        .route(
            dispatch::QUERY_GATEWAY_ENDPOINT,
            get(dispatch::query_gateway),
        )
        .route(auth::RISKY_API_CHECK_ENDPOINT, post(auth::risky_api_check))
        .route(
            auth::LOGIN_WITH_PASSWORD_ENDPOINT,
            post(auth::login_with_password),
        )
        .route(
            auth::LOGIN_WITH_SESSION_TOKEN_ENDPOINT,
            post(auth::login_with_session_token),
        )
        .route(
            auth::GRANTER_LOGIN_VERIFICATION_ENDPOINT,
            post(auth::granter_login_verification),
        )
        .route(
            sr_tools::SRTOOLS_UPLOAD_ENDPOINT,
            post(sr_tools::sr_tool_save),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_headers([CONTENT_TYPE]),
        )
        .fallback(errors::not_found);

    let addr = format!("0.0.0.0:{PORT}");
    let server = axum_server::bind(addr.parse()?);

    tracing::info!("sdkserver is listening at {addr}");
    server.serve(router.into_make_service()).await?;

    Ok(())
}
