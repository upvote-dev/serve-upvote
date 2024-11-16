use utoipa::{Modify, OpenApi};
use utoipa_actix_web::AppExt;
use utoipa_redoc::Servable;
use utoipa_scalar::Servable as ScalarServable;

#[cfg(test)]
mod tests;

pub const CARGO_PKG_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
pub const CARGO_PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
pub const CARGO_PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Hostname
    #[arg(long, default_value = "localhost", env = "SADAS_HOSTNAME")]
    hostname: String,

    /// Port
    #[arg(short, long, default_value_t = 3000u16, env = "SADAS_PORT")]
    port: u16,

    /// Avoid inheriting host environment variables
    #[arg(long, default_value_t = false)]
    no_host_env: bool,

    /// Env file, defaults to ".env"
    #[arg(long)]
    env_file: Option<String>,

    /// Env var (can be specified multiple times, like `-eFOO=5 -eBAR=can`)
    #[arg(short, long, action(clap::ArgAction::Append))]
    env: Option<Vec<String>>,
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Debug, PartialEq)]
struct Version {
    version: &'static str,
    upvote_backend: &'static str,
    radas: &'static str,
    name: &'static str,
}

impl Default for Version {
    fn default() -> Self {
        Self::const_default()
    }
}

impl Version {
    const fn const_default() -> Self {
        Self {
            version: CARGO_PKG_VERSION,
            radas: rust_actix_diesel_auth_scaffold::CARGO_PKG_VERSION,
            upvote_backend: upvote_backend::CARGO_PKG_VERSION,
            name: CARGO_PKG_NAME,
        }
    }
}

const VERSION: Version = Version::const_default();

#[actix_web::get("")]
async fn version() -> actix_web::web::Json<Version> {
    actix_web::web::Json(VERSION)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Cli = clap::Parser::parse();

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let mut env = indexmap::IndexMap::<String, String>::new();
    if !args.no_host_env {
        env.extend(std::env::vars());
    }
    let env_file = args.env_file.unwrap_or(String::from(".env"));
    if let Ok(file_iter) = dotenvy::from_filename_iter(env_file) {
        for res in file_iter {
            if let Ok((k, v)) = res {
                env.insert(k, v);
            }
        }
    }
    if let Some(env_vec) = args.env {
        env.extend(env_vec.iter().filter_map(|s| match s.split_once("=") {
            None => None,
            Some((k, v)) => Some((k.to_string(), v.to_string())),
        }));
    };
    env.iter().for_each(|(k, v)| std::env::set_var(k, v));
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    rust_actix_diesel_auth_scaffold::db_init();
    upvote_backend::db_init();

    let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder().build(manager).unwrap();

    #[derive(utoipa::OpenApi)]
    #[openapi(
        tags(
            (name = CARGO_PKG_NAME, description = CARGO_PKG_DESCRIPTION)
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
            components.add_security_scheme(
                "password",
                utoipa::openapi::security::SecurityScheme::OAuth2(
                    utoipa::openapi::security::OAuth2::new([
                        utoipa::openapi::security::Flow::Password(
                            utoipa::openapi::security::Password::new(
                                "/api/token",
                                utoipa::openapi::security::Scopes::new(),
                            ),
                        ),
                    ]),
                ),
            )
        }
    }

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .map(|app| app.wrap(actix_web::middleware::Logger::default()))
            .app_data(
                actix_web::web::JsonConfig::default().error_handler(|err, _req| {
                    actix_web::error::InternalError::from_response(
                        "",
                        actix_web::HttpResponse::BadRequest()
                            .content_type("application/json")
                            .body(format!(r#"{{"error":"{}"}}"#, err)),
                    )
                    .into()
                }),
            )
            .app_data(actix_web::web::Data::new(pool.clone()))
            .service(
                utoipa_actix_web::scope("/api/v0_noauth")
                    .service(upvote_backend::routes::review::read_many),
            )
            .service(
                utoipa_actix_web::scope("/api/v0")
                    .wrap(actix_web::middleware::Compat::new(
                        actix_web_httpauth::middleware::HttpAuthentication::bearer(
                            rust_actix_diesel_auth_scaffold::middleware::bearer::validator,
                        ),
                    ))
                    .service(upvote_backend::routes::profile::upsert)
                    .service(upvote_backend::routes::profile::read)
                    .service(upvote_backend::routes::review::upsert)
                    .service(upvote_backend::routes::review::read),
            )
            .service(
                utoipa_actix_web::scope("/api")
                    .service(rust_actix_diesel_auth_scaffold::routes::token::token),
                // .service(rust_actix_diesel_auth_scaffold::routes::authorisation::authorise)
                // .service(version),
            )
            .service(
                utoipa_actix_web::scope("/secured")
                    .wrap(actix_web::middleware::Compat::new(
                        actix_web_httpauth::middleware::HttpAuthentication::bearer(
                            rust_actix_diesel_auth_scaffold::middleware::bearer::validator,
                        ),
                    ))
                    .service(rust_actix_diesel_auth_scaffold::routes::secret::secret)
                    .service(rust_actix_diesel_auth_scaffold::routes::logout::logout),
            )
            .openapi_service(|api| utoipa_redoc::Redoc::with_url("/redoc", api))
            .openapi_service(|api| {
                utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", api)
            })
            .map(|app| {
                app.service(utoipa_rapidoc::RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            })
            .openapi_service(|api| utoipa_scalar::Scalar::with_url("/scalar", api))
            .into_app()
    })
    .bind((args.hostname.as_str(), args.port))?
    .run()
    .await
}
