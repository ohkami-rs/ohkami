use crate::errors::RealWorldError;
use std::process::Command;
use std::format as f;


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

        f!("postgresql://{POSTGRES_USER}:{POSTGRES_PASSWORD}@localhost:{POSTGRES_PORT}/{POSTGRES_DB}?sslmode=disable")
    }

    async fn setup(&self) -> Result<sqlx::PgPool, RealWorldError> {
        let Self { POSTGRES_PASSWORD, POSTGRES_USER, POSTGRES_PORT, POSTGRES_DB } = self;

        Command::new("docker").args([
            f!("container"), f!("run"),
            f!("--name"), f!("{}", Self::CONTAINER_NAME),
            f!("-e"), f!("POSTGRES_PASSWORD={POSTGRES_PASSWORD}"),
            f!("-e"), f!("POSTGRES_USER={POSTGRES_USER}"),
            f!("-e"), f!("POSTGRES_DB={POSTGRES_DB}"),
            f!("-p"), f!("{POSTGRES_PORT}:5432"),
            f!("-d"),
            f!("--rm"),
            f!("{}", Self::CONTAINER_IMAGE),
        ])
        .spawn().map_err(|e| RealWorldError::Config(e.to_string()))?
        .wait().map_err(|e| RealWorldError::Config(e.to_string()))?;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        Command::new("sqlx").args(["migrate", "run", "--database-url", &self.db_url()])
            .spawn().map_err(|e| RealWorldError::DB(sqlx::Error::Migrate(Box::new(sqlx::migrate::MigrateError::Execute(sqlx::Error::Io(e))))))?
            .wait().map_err(|e| RealWorldError::DB(sqlx::Error::Migrate(Box::new(sqlx::migrate::MigrateError::Execute(sqlx::Error::Io(e))))))?;

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(42)
            .min_connections(42)
            .connect(&self.db_url()).await
            .map_err(RealWorldError::DB)?;

        Ok(pool)
    }
}
impl Drop for TestDB {
    fn drop(&mut self) {
        Command::new("docker")
            .args(["container", "stop", Self::CONTAINER_NAME])
            .spawn().unwrap()
            .wait().unwrap();
    }
}


#[tokio::test]
pub async fn senario() {
    use ohkami::testing::*;
    use ohkami::Status;
    use crate::models::{*, request::*, response::*};

    dotenvy::dotenv().unwrap();
    
    let db = TestDB {
        POSTGRES_PASSWORD: "password",
        POSTGRES_USER:     "ohkami",
        POSTGRES_PORT:     2345,
        POSTGRES_DB:       "test",
    };
    
    let t = crate::handlers::realworld_ohkami(
        db.setup().await.unwrap()
    ).test();


    /*===== Play the test senario based on https://realworld-docs.netlify.app/docs/specs/backend-specs/endpoints =====*/

    // Jacob registers to the service

    let req = TestRequest::POST("/api/users")
        .json(RegisterRequest {
            user: RegisterRequestUser {
                username: "Jacob",
                email:    "jake@jake.jake",
                password: "jakejake",
            }
        });
    let res = t.oneshot(req).await;

    dbg!(res.text());
    assert_eq!(res.status(), Status::Created);

    let UserResponse {
        user: User { email, jwt, name, bio, image }
    } = res.json().unwrap();

    assert_eq!(email, "jake@jake.jake");
    assert_eq!(name,  "Jacob");
    assert_eq!(bio,   None);
    assert_eq!(image, None);


    // He checks current profile of himself

    let req = TestRequest::GET("/api/user")
        .header("Authorization", f!("Bearer {jwt}"));
    let res = t.oneshot(req).await;

    assert_eq!(res.status(), Status::OK);

    let UserResponse {
        user: User { email, jwt, name, bio, image }
    } = res.json().unwrap();

    assert_eq!(email, "jake@jake.jake");
    assert_eq!(name, "Jacob");
    assert_eq!(bio, None);
    assert_eq!(image, None);


    // He writes his bio and image

    let req = TestRequest::PUT("/api/user")
        .header("Authorization", f!("Bearer {jwt}"))
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
    } = res.json().unwrap();

    assert_eq!(email, "jake@jake.jake");
    assert_eq!(name,  "Jacob");
    assert_eq!(bio,   Some(f!("I like to skateboard")));
    assert_eq!(image, Some(f!("https://i.stack.imgur.com/xHWG8.jpg")));


    // He checks what tags exist at that time
    // (But found nothing because he's the first user)

    let req = TestRequest::GET("/api/tags");
    let res = t.oneshot(req).await;

    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.json::<ListOfTagsResponse>().unwrap(), ListOfTagsResponse {
        tags: Vec::new()
    });


    // He writes the first article in this service

    let req = TestRequest::POST("/api/articles")
        .header("Authorization", f!("Bearer {jwt}"))
        .json(CreateArticleRequest {
            article: CreateArticleRequestArticle {
                title:       "How to train your dragon",
                description: "Ever wonder how?",
                body:        "You have to believe",
                tag_list:    Some(vec![Tag::new("reactjs"), Tag::new("angularjs"), Tag::new("dragons")]),
            }
        });
    let res = t.oneshot(req).await;

    assert_eq!(res.status(), Status::Created);

    let SingleArticleResponse { article } = res.json().unwrap();

    assert_eq!(article.title,           "How to train your dragon");
    assert_eq!(article.slug,            "How-to-train-your-dragon");
    assert_eq!(article.description,     "Ever wonder how?");
    assert_eq!(article.body,            "You have to believe");
    assert_eq!(article.tag_list,        vec![f!("reactjs"), f!("angularjs"), f!("dragons")]);
    assert_eq!(article.favorited,       false);
    assert_eq!(article.favorites_count, 0);
    assert_eq!(article.author,          Profile {
        username:  f!("Jacob"),
        bio:       Some(f!("I like to skateboard")),
        image:     Some(f!("https://i.stack.imgur.com/xHWG8.jpg")),
        following: false,
    });


    // He checks tags again

    let req = TestRequest::GET("/api/tags");
    let res = t.oneshot(req).await;

    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.json::<ListOfTagsResponse>().unwrap(), ListOfTagsResponse {
        tags: vec![Tag::new("reactjs"), Tag::new("angularjs"), Tag::new("dragons")]
    });


    /*===== Clearn up =====*/

    drop(db);
}
