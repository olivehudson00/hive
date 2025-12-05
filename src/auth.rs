/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use base64::{
    engine::general_purpose::URL_SAFE as b64,
    Engine as _,
};

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(auth) = request.headers().get("Authorization") {
        if let Some((kind, auth)) = String::from_utf8_lossy(auth.as_bytes()).split_once(' ') {
            if kind == "Basic" {
                if let Ok(auth) = b64.decode(auth) {
                    if let Some((user, _)) = String::from_utf8_lossy(&auth).split_once(':') {
                        if let Ok(id) = user.parse::<i32>() {
                            request.extensions_mut().insert(id);
                            return next.run(request).await;
                        }
                    }
                }
            }
        }
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Basic realm=\"site\"")
        .body(Body::from(""))
        .unwrap()
}
