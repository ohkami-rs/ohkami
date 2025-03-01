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

#[derive(Clone)]
struct Logger;
impl FangAction for Logger {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        println!("[Logger] req: {req:?}");
        Ok(())
    }

    async fn back<'a>(&'a self, res: &'a mut Response) {
        println!("[Logger] res: {res:?}");
    }
}

#[tokio::main]
async fn main() {
    let db = Arc::new(mock::DB::new());

    let o = Ohkami::new((
        Logger,
        Context::new(db),
        "/pets"
            .GET(list_pets)
            .POST(create_pet),
        "/pets/:petId"
            .GET(show_pet_by_id)
            .PUT(edit_pet_profile),
        "/pets/admin"
            .GET(show_pets_detail),
    ));

    o.generate(openapi::OpenAPI {
        title: "Petstore API",
        version: "1.0.0",
        servers: &[
            openapi::Server::at("http://localhost:5050")
        ]
    });

    o.howl("localhost:5050").await
}

#[openapi::operation({200: "All pets stored in this pet store"})]
async fn list_pets(
    Query(q): Query<ListPetsMeta>,
    Context(db): Context<'_, Arc<mock::DB>>,
) -> Result<JSON<Vec<Pet>>, Error> {
    let mut pets = db.read().await.values().cloned().collect::<Vec<_>>();
    if let Some(limit) = q.limit {
        pets.truncate(limit);
    }
    
    Ok(JSON(pets))
}
#[derive(Deserialize, openapi::Schema)]
/// metadata for `list_pets` operation
struct ListPetsMeta {
    /// limit of number of pets responded by `list_pets` operation
    limit: Option<usize>,
}

#[openapi::operation(createPet {
    // 500 is not defined in `Error::openapi_responses`,
    // but this *overrides* it to this description
    500: "an internal error",
})]
async fn create_pet(
    JSON(req): JSON<CreatePetRequest<'_>>,
    Context(db): Context<'_, Arc<mock::DB>>,
) -> Result<status::Created<JSON<Pet>>, Error> {
    if db.read().await.values().any(|p| p.name == req.name) {
        return Err(Error {
            status_code: 400,
            message: format!("A pet of name `{}` already exists", req.name)
        })
    }

    let created_pet = {
        let db = &mut *db.write().await;
        let id = db.len() as u64 + 1;
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
#[derive(Deserialize, openapi::Schema)]
#[openapi(component)]
struct CreatePetRequest<'req> {
    #[serde(rename = "petName")]
    name: &'req str,
    tag:  Option<&'req str>,
}

#[openapi::operation(showPetById {
    summary: "find a pet of the `id`",
    200: "Successfully found a pet",
    default: "Something went wrong in finding a pet",
})]
/// Find a pet of the `id`.
/// The parameter `id` must be unsigned 64-bit integer.
async fn show_pet_by_id(
    id: u64,
    Context(db): Context<'_, Arc<mock::DB>>,
) -> Result<JSON<Pet>, Error> {
    let pet = db.read().await.get(&id)
        .cloned()
        .ok_or_else(|| Error {
            status_code: 404,
            message: format!("A pet of id `{id}` not found"),
        })?;
    Ok(JSON(pet))
}

async fn edit_pet_profile(_id: u64) {}

async fn show_pets_detail() {}


#[derive(Serialize, Clone, openapi::Schema)]
#[openapi(component)]
struct Pet {
    id:   u64,
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
#[serde(rename_all = "camelCase")]
struct Error {
    status_code: u16,
    message:     String,
}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Response::new(Status::from(self.status_code))
            .with_json(self)
    }
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([])
            .or_default(openapi::Response::when("Unexpected error")
                .content("application/json", <Self as openapi::Schema>::schema())
            )
    }
}
