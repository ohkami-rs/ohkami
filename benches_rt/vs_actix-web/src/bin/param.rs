use actix_web::{get, web, App, HttpServer};

#[get("/user/{id}")]
async fn echo(id: web::Path<String>) -> String {
    id.into_inner()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(echo)
    })
    .bind("0.0.0.0:3000")?
    .run().await
}
