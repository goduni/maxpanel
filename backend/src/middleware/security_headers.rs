use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

pub async fn security_headers(request: Request, next: Next) -> Response {
    let is_swagger = request.uri().path().starts_with("/swagger-ui")
        || request.uri().path().starts_with("/api-docs");

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=()"),
    );

    if is_swagger {
        headers.insert(
            "Content-Security-Policy",
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:;"
            ),
        );
    } else {
        headers.insert(
            "Content-Security-Policy",
            HeaderValue::from_static("default-src 'none'"),
        );
        // Prevent caching of API responses containing sensitive data
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("no-store"),
        );
    }

    response
}
