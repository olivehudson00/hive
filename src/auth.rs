/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

#[derive(Serialize, Deserialize)]
struct Context {
    sub: i32,
    exp: usize,
}

#[derive(Deserialize)]
struct AuthForm {
    id_token: String,
}

pub async fn auth_middleware(
    State(pool): State<Pool>,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let mut jar = CookieJar::from_headers(request.headers());
    let jwt = jar.get("jwt");

    let mut user = 0;
    if let Some(jwt) = jwt {
        if let Ok(claims) = validate_jwt::<Context>(JWT_SIGNING_KEY, jwt.value()) {
            if claims.exp < SystemTime::now().duration_since(UNIX_EPOCH) {
                request.extensions_mut().insert(claims.sub);
                let response = next.run(request).await;
                return (jar, response).into_response();
            }
        }
    }

    /* redirect to login page */
    todo!();
}

pub async fn auth_redirect(
    State(pool): State<pool>,
    Form(form): Form<AuthForm>,
) -> impl IntoResponse {
    
}
