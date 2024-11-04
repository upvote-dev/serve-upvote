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

#[derive(serde::Serialize)]
struct Version {
    version: String,
    name: String,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            version: String::from(env!("CARGO_PKG_VERSION")),
            name: String::from(env!("CARGO_PKG_NAME")),
        }
    }
}

#[actix_web::get("")]
async fn version() -> actix_web::web::Json<Version> {
    actix_web::web::Json(Version {
        version: String::from(env!("CARGO_PKG_VERSION")),
        name: String::from(env!("CARGO_PKG_NAME")),
    })
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

    let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder().build(manager).unwrap();

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(actix_web::web::JsonConfig::default().error_handler(|err, _req| {
                actix_web::error::InternalError::from_response(
                    "",
                    actix_web::HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(r#"{{"error":"{}"}}"#, err)),
                )
                    .into()
            }))
            .app_data(actix_web::web::Data::new(pool.clone()))
            .service(
                actix_web::web::scope("/api/v0_noauth")
                    .service(upvote_backend::routes::review::read_many),
            )
            .service(
                actix_web::web::scope("/api/v0")
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
                actix_web::web::scope("/api")
                    .service(rust_actix_diesel_auth_scaffold::routes::token::token)
                    .service(rust_actix_diesel_auth_scaffold::routes::authorisation::authorise)
                    .service(version),
            )
            .service(
                actix_web::web::scope("/secured")
                    .wrap(actix_web::middleware::Compat::new(
                        actix_web_httpauth::middleware::HttpAuthentication::bearer(
                            rust_actix_diesel_auth_scaffold::middleware::bearer::validator,
                        ),
                    ))
                    .service(rust_actix_diesel_auth_scaffold::routes::secret::secret)
                    .service(rust_actix_diesel_auth_scaffold::routes::logout::logout),
            )
    })
        .bind((args.hostname.as_str(), args.port))?
        .run()
        .await
}
