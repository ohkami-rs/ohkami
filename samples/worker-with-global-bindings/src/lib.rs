use ohkami::prelude::*;
use ohkami::fang::Context;

#[ohkami::bindings]
struct Bindings {
    DB: ohkami::bindings::D1,
    MY_KV: ohkami::bindings::KV,
}

#[ohkami::worker]
async fn ohkami(Bindings { DB, MY_KV }: Bindings) -> Ohkami {
    // just check to be able to retrieve even in openapi generation
    let _ = MY_KV;

    Ohkami::new((
        Context::new(D1UserRepository::new(DB)),
        "/users".By(routes::users_ohkami::<D1UserRepository>())
    ))
}

enum Error {
    Worker(worker::Error),
    Repository(String),
    UserIdNotFound { id: u32 },
}
impl From<worker::Error> for Error {
    fn from(e: worker::Error) -> Self {
        Self::Worker(e)
    }
}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Worker(e) => {
                worker::console_error!("Worker's internal error: {e}");
                Response::InternalServerError()
            }
            Self::Repository(e) => {
                worker::console_error!("Error from repository: {e}");
                Response::InternalServerError()
            }
            Self::UserIdNotFound { id } => {
                worker::console_error!("Error: user not found by id: `{id}`");
                Response::NotFound()
            }
        }
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> ohkami::openapi::Responses {
        // there seems nothing needed to document
        ohkami::openapi::Responses::new([])
    }
}

#[derive(Clone)]
struct D1UserRepository {
    d1: std::rc::Rc<ohkami::bindings::D1>,
}
impl D1UserRepository {
    fn new(d1: ohkami::bindings::D1) -> Self {
        Self { d1: std::rc::Rc::new(d1) }
    }
}
impl repository::UserRepository for D1UserRepository {
    async fn get_all(&self) -> Result<Vec<repository::User>, Error> {
        self.d1
            .prepare("SELECT id, name, age FROM users")
            .all().await?
            .results()
            .map_err(Into::into)
    }

    async fn get_by_id(&self, id: u32) -> Result<Option<repository::User>, Error> {
        self.d1
            .prepare("SELECT id, name, age FROM users WHERE id = ?")
            .bind(&[id.into()])?
            .first(None).await
            .map_err(Into::into)
    }

    async fn create_returning_id(&self, params: repository::CreateUserParams<'_>) -> Result<u32, Error> {
        let id = self.d1
            .prepare("INSERT INTO users (name, age) VALUES (?, ?) RETURNING id")
            .bind(&[params.name.into(), params.age.into()])?
            .first(Some("id")).await?
            .ok_or_else(|| Error::Repository(format!("`id` not found in `RETURNING id` qruery result")))?;
        Ok(id)
    }
}

mod repository {
    #[derive(ohkami::serde::Deserialize)]
    pub struct User {
        pub id: u32,
        pub name: String,
        pub age: Option<u8>,
    }

    pub struct CreateUserParams<'r> {
        pub name: &'r str,
        pub age: Option<u8>,
    }

    pub trait UserRepository: 'static {
        async fn get_all(&self) -> Result<Vec<User>, crate::Error>;

        async fn get_by_id(&self, id: u32) -> Result<Option<User>, crate::Error>;

        async fn create_returning_id(&self, params: CreateUserParams<'_>) -> Result<u32, crate::Error>;
    }
}

mod routes {
    use crate::repository::{self, UserRepository};
    use ohkami::{Ohkami, Route};
    use ohkami::format::JSON;
    use ohkami::fang::Context;
    use ohkami::typed::status;
    use ohkami::serde::{Serialize, Deserialize};

    pub fn users_ohkami<U: UserRepository>() -> Ohkami {
        Ohkami::new((
            "/"
                .GET(list_users::<U>)
                .POST(create_user::<U>),
            "/:id"
                .GET(show_user::<U>),
        ))
    }

    #[derive(Serialize)]
    #[cfg_attr(feature="openapi", derive(ohkami::openapi::Schema))]
    pub struct User {
        id: u32,
        name: String,
        age: Option<u8>,
    }

    #[derive(Deserialize)]
    #[cfg_attr(feature="openapi", derive(ohkami::openapi::Schema))]
    pub struct CreateUserRequest<'req> {
        name: &'req str,
        age: Option<u8>,
    }

    pub async fn list_users<U: UserRepository>(
        Context(r): Context<'_, U>,
    ) -> Result<JSON<Vec<User>>, crate::Error> {
        let user_rows = r.get_all().await?;

        Ok(JSON(user_rows.into_iter().map(|r| User {
            id: r.id,
            name: r.name,
            age: r.age,
        }).collect()))
    }

    pub async fn show_user<U: UserRepository>(
        id: u32,
        Context(r): Context<'_, U>,
    ) -> Result<JSON<User>, crate::Error> {
        let user_row = r.get_by_id(id).await?
            .ok_or(crate::Error::UserIdNotFound { id })?;

        Ok(JSON(User {
            id: user_row.id,
            name: user_row.name,
            age: user_row.age,
        }))
    }

    pub async fn create_user<U: UserRepository>(
        JSON(req): JSON<CreateUserRequest<'_>>,
        Context(r): Context<'_, U>,
    ) -> Result<status::Created<JSON<User>>, crate::Error> {
        let created_id = r.create_returning_id(repository::CreateUserParams {
            name: &req.name,
            age: req.age,
        }).await?;

        Ok(status::Created(JSON(User {
            id: created_id,
            name: req.name.to_string(),
            age: req.age,
        })))
    }
}
