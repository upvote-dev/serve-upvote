use crate::Version;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
struct VersionForTest {
    version: String,
    upvote_backend: String,
    radas: String,
    name: String,
}

impl VersionForTest {
    fn eq(&self, other: &Version) -> bool {
        self.radas == other.radas
            && self.version == other.version
            && self.upvote_backend == other.upvote_backend
            && self.name == other.name
    }
}

#[actix_web::test]
async fn test_version_get() {
    let app = actix_web::test::init_service(
        actix_web::App::new().service(actix_web::web::scope("/api").service(crate::version)),
    )
    .await;
    let req = actix_web::test::TestRequest::get().uri("/api").to_request();
    let version: VersionForTest = actix_web::test::call_and_read_body_json(&app, req).await;
    assert!(version.eq(&crate::VERSION));
}
