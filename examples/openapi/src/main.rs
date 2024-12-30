use ohkami::prelude::*;
use ohkami::format::{JSON, Query};
use ohkami::typed::status;
use ohkami::openapi;
use std::sync::Arc;

type ID = u64;

mod mock {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    pub(super) struct DB(RwLock<HashMap<ID, Pet>>);
    impl DB {
        pub fn new() -> Self {
            Self(RwLock::new(HashMap::new()))
        }
    }
    const _: () = {
        impl std::ops::Deref for DB {
            type Target = RwLock<HashMap<ID, Pet>>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

#[tokio::main]
async fn main() {
    let db = Arc::new(mock::DB::new());

    let o = Ohkami::with(Memory::new(db), (
        "/pets"
            .GET(list_pets)
            .POST(create_pet),
        "/pets/:petId"
            .GET(show_pet_by_id)
    ));

    o.generate(openapi::OpenAPI::json("Petstore API", "1.0.0", [
        openapi::Server::at("http://localhost:5050")
    ]));

    o.howl("localhost:5050").await
}


async fn list_pets(
    Query(q): Query<ListPetsMeta>,
    Memory(db): Memory<'_, Arc<mock::DB>>,
) -> Result<JSON<Vec<Pet>>, Error> {
    let mut pets = db.read().await.values().cloned().collect::<Vec<_>>();
    if let Some(limit) = q.limit {
        pets.truncate(limit);
    }
    
    Ok(JSON(pets))
}
#[derive(Deserialize)]
struct ListPetsMeta {
    limit: Option<usize>,
}
impl openapi::Schema for ListPetsMeta {
    fn schema() -> impl Into<openapi::schema::SchemaRef> {
        openapi::object()
            .optional("limit", openapi::integer().format("uint64"))
    }
}

async fn create_pet(
    JSON(req): JSON<CreatePetRequest<'_>>,
    Memory(db): Memory<'_, Arc<mock::DB>>,
) -> Result<status::Created<JSON<Pet>>, Error> {
    if db.read().await.values().any(|p| p.name == req.name) {
        return Err(Error {
            code: 400,
            message: format!("A pet of name `{}` already exists", req.name)
        })
    }

    let created_pet = {
        let db = &mut *db.write().await;
        let id = db.len() as u64;
        let created = Pet {
            id,
            name: req.name.to_string(),
            tag:  req.tag.map(str::to_string),
        };
        db.insert(id, created.clone());
        created
    };

    Ok(status::Created(JSON(created_pet)))
}
#[derive(Deserialize)]
struct CreatePetRequest<'req> {
    name: &'req str,
    tag:  Option<&'req str>,
}
impl openapi::Schema for CreatePetRequest<'_> {
    fn schema() -> impl Into<openapi::schema::SchemaRef> {
        openapi::object()
            .property("name", openapi::string())
            .optional("tag", openapi::string())
    }
}

async fn show_pet_by_id(
    id: u64,
    Memory(db): Memory<'_, Arc<mock::DB>>,
) -> Result<JSON<Pet>, Error> {
    let pet = db.read().await.get(&id)
        .cloned()
        .ok_or_else(|| Error {
            code: 404,
            message: format!("A pet of id `{id}` not found"),
        })?;
    Ok(JSON(pet))
}


#[derive(Serialize, Clone)]
struct Pet {
    id:   u64,
    name: String,
    tag:  Option<String>,
}
impl openapi::Schema for Pet {
    fn schema() -> impl Into<openapi::schema::SchemaRef> {
        openapi::component("Pet", openapi::object()
            .property("id", openapi::integer().format("uint64"))
            .property("name", openapi::string())
            .optional("tag", openapi::string())
        )
    }
}

#[derive(Serialize)]
struct Error {
    code:    u16,
    message: String,
}
impl openapi::Schema for Error {
    fn schema() -> impl Into<openapi::schema::SchemaRef> {
        openapi::component("Error", openapi::object()
            .property("code", openapi::integer().format("uint32"))
            .property("message", openapi::string())
        )
    }
}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Response::of(Status::from(self.code))
            .with_text(self.message)
    }
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::enumerated([])
            .or_default(openapi::Response::when("Unexpected error")
                .content("application/json", <Self as openapi::Schema>::schema())
            )
    }
}
