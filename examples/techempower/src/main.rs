mod components; use components::{
    WorldRow, Fortune, html_from,
    random_i32,
    PREPARE_GET_WORLD, PREPARE_GET_FORTUNE, PREPARE_UPDATE_WORLDS, 
};

use ohkami::{prelude::*, json};
use sqlx::postgres::PgPoolOptions;


fn main() -> Result<()> {
    let config = Config {
        db_profile: DBprofile {
            pool_options: PgPoolOptions::new().max_connections(10000),
            url:          "postgres://benchmarkdbuser:benchmarkdbpass@tfb-database/ohkami",
        },
        ..Default::default()
    };

    Server::setup_with(config)
        .GET("/json",      move |_| async {Response::OK(json!("message": "Hello, World!"))})
        .GET("/plaintext", move |_| async {Response::OK(Body::text("Hello, World!"))})
        .GET("/db",        get_db)
        .GET("/fortunes",  get_fortunes)
        .GET("/queries",   get_queries)
        .GET("/updates",   get_updates)
        .serve_on(":3000")
}

async fn get_db(ctx: Context) -> Result<Response> {
    let id = random_i32();
    let world = sqlx::query_as::<_, WorldRow>(PREPARE_GET_WORLD)
        .bind(id)
        .fetch_one(ctx.pool())
        .await?;
    Response::OK(JSON::from_struct(&world)?)
}

async fn get_fortunes(ctx: Context) -> Result<Response> {
    let mut fortunes = sqlx::query_as::<_, Fortune>(PREPARE_GET_FORTUNE)
        .fetch_all(ctx.pool())
        .await?;
    fortunes.push(Fortune {
        id:      0,
        message: "Additional fortune added at request time.".into(),
    });

    Response::OK(html_from(fortunes))
}

async fn get_queries(ctx: Context) -> Result<Response> {
    let q = ctx.query("q").unwrap().parse::<u32>().unwrap();
    // let worlds = 
    todo!()
}

async fn get_updates(ctx: Context) -> Result<Response> {
    let q = ctx.query("q").unwrap().parse::<u32>().unwrap();
    todo!()

}
