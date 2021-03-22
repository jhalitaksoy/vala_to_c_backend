use rocket::http::Method;
use rocket_cors::AllowedHeaders;
use rocket_cors::AllowedOrigins;
use rocket_cors::CorsOptions;

pub fn get_cors_options() -> CorsOptions {
    let allowed_origins =
        AllowedOrigins::some_exact(&["https://hoppscotch.io", "http://localhost:5000"]);

    // You can also deserialize this
    let cors_options = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    };

    return cors_options;
}
