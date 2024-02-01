use crate::errors::RealWorldError;
use std::process::{Command, Stdio};


#[allow(non_snake_case)]
struct TestDB {
    POSTGRES_PASSWORD: &'static str,
    POSTGRES_USER:     &'static str,
    POSTGRES_PORT:     u32,
    POSTGRES_DB:       &'static str,
}
impl TestDB {
    const CONTAINER_IMAGE: &'static str = "postgres:15-alpine";
    const CONTAINER_NAME:  &'static str = "test-postgres";

    fn db_url(&self) -> String {
        let Self { POSTGRES_PASSWORD, POSTGRES_USER, POSTGRES_PORT, POSTGRES_DB } = self;

        format!("postgresql://{POSTGRES_USER}:{POSTGRES_PASSWORD}@localhost:{POSTGRES_PORT}/{POSTGRES_DB}?sslmode=disable")
    }

    async fn setup(&self) -> Result<sqlx::PgPool, RealWorldError> {
        let Self { POSTGRES_PASSWORD, POSTGRES_USER, POSTGRES_PORT, POSTGRES_DB } = self;

        Command::new("docker").stdout(Stdio::piped()).stderr(Stdio::piped()).args([
            format!("container"), format!("run"),
            format!("--name"), format!("{}", Self::CONTAINER_NAME),
            format!("-e"), format!("POSTGRES_PASSWORD={POSTGRES_PASSWORD}"),
            format!("-e"), format!("POSTGRES_USER={POSTGRES_USER}"),
            format!("-e"), format!("POSTGRES_PORT={POSTGRES_PORT}"),
            format!("-e"), format!("POSTGRES_DB={POSTGRES_DB}"),
            format!("-p"), format!("{POSTGRES_PORT}:{POSTGRES_PORT}"),
            format!("--rm"),
            format!("{}", Self::CONTAINER_IMAGE),
        ]).spawn().map_err(|e| RealWorldError::Config(e.to_string()))?;

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(42)
            .min_connections(42)
            .connect(&self.db_url()).await
            .map_err(RealWorldError::DB)?;

        Command::new("sqlx").stdout(Stdio::piped()).stderr(Stdio::piped()).args([
            "migrate", "run", "--database-url", &self.db_url()
        ]).spawn().map_err(|e| RealWorldError::DB(sqlx::Error::Migrate(Box::new(sqlx::migrate::MigrateError::Execute(sqlx::Error::Io(e))))))?;

        Ok(pool)
    }
}
impl Drop for TestDB {
    fn drop(&mut self) {
        Command::new("docker").stdout(Stdio::piped()).stderr(Stdio::piped()).args([
            "container", "stop", Self::CONTAINER_NAME
        ]).spawn().unwrap();
    }
}


#[tokio::test] async fn senario() {
    dotenvy::dotenv().unwrap();
    
    let db = TestDB {
        POSTGRES_PASSWORD: "password",
        POSTGRES_USER:     "ohkami",
        POSTGRES_PORT:     2345,
        POSTGRES_DB:       "test",
    };
    
    let t = crate::handlers::realworld_ohkami(
        db.setup().await.unwrap()
    );

    use ohkami::testing::*;
    use ohkami::http::Status;
    use crate::models::{*, request::*, response::*};


    /*===== Play the test senario =====*/


    // Jacob registers to the service

    let req = TestRequest::POST("/api/users")
        .json(RegisterRequest {
            username: "Jacob",
            email:    "jake@jake.jake",
            password: "jakejake",
        });
    let res = t.oneshot(req).await;

    assert_eq!(res.status(), Status::Created);

    let UserResponse {
        user: User { email, jwt, name, bio, image }
    } = res.json().unwrap().unwrap();

    assert_eq!(email, "jake@jake.jake");
    assert_eq!(name, "Jacob");
    assert_eq!(bio, None);
    assert_eq!(image, None);


    // He checks current profile of himself

    let req = TestRequest::GET("/api/user")
        .header("Authorization", format!("Bearer {jwt}"));
    let res = t.oneshot(req).await;

    assert_eq!(res.status(), Status::OK);

    let UserResponse {
        user: User { email, jwt, name, bio, image }
    } = res.json().unwrap().unwrap();

    assert_eq!(email, "jake@jake.jake");
    assert_eq!(name, "Jacob");
    assert_eq!(bio, None);
    assert_eq!(image, None);


    // He writes his bio and image

    let req = TestRequest::PUT("/api/user")
        .header("Authorization", format!("Bearer {jwt}"))
        .json_lit(r#"
            {
                "user": {
                    "bio": "I like to skateboard",
                    "image": "https://i.stack.imgur.com/xHWG8.jpg"
                }
            }
        "#);
    let res = t.oneshot(req).await;

    assert_eq!(res.status(), Status::OK);

    let UserResponse {
        user: User { email, jwt, name, bio, image }
    } = res.json().unwrap().unwrap();

    assert_eq!(email, "jake@jake.jake");
    assert_eq!(name, "Jacob");
    assert_eq!(bio.unwrap(), "I like to skateboard");
    assert_eq!(image.unwrap(), "https://i.stack.imgur.com/xHWG8.jpg");


    // He writes the first article in this service
    
}
