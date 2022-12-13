use ohkami::{prelude::*, json};
use sqlx::postgres::PgPoolOptions;
mod components; use components::{
    consts::{
        DB_URL, GET_WORLD_STATEMENT, GET_FORTUNE_STATEMENT, UPDATE_WORLD_STATEMENT, MAX_CONNECTIONS,
    },
    models::{
        World, Fortune,
    },
    functions::{
        random_i32, random_i32s, html_from,
    },
};

fn main() -> Result<()> {
    let config = Config {
        db_profile: DBprofile {
            pool_options: PgPoolOptions::new().max_connections(MAX_CONNECTIONS),
            url:          DB_URL,
        },
        ..Default::default()
    };

    Server::setup_with(config)
        .GET("/json",      |_| async {Response::OK(json!("message": "Hello, World!"))})
        .GET("/plaintext", |_| async {Response::OK(Body::text("Hello, World!"))})
        .GET("/db",        get_db)
        .GET("/fortunes",  get_fortunes)
        .GET("/queries",   get_queries)
        .GET("/updates",   get_updates)
        .serve_on(":3000")
}

async fn get_db(ctx: Context) -> Result<Response> {
    let id = random_i32();
    let world = sqlx::query_as::<_, World>(GET_WORLD_STATEMENT)
        .bind(id)
        .fetch_one(ctx.pool())
        .await?;
    Response::OK(JSON::from_struct(&world)?)
}

async fn get_fortunes(ctx: Context) -> Result<Response> {
    let mut fortunes = sqlx::query_as::<_, Fortune>(GET_FORTUNE_STATEMENT)
        .fetch_all(ctx.pool())
        .await?;
    fortunes.push(Fortune {
        id:      0,
        message: "Additional fortune added at request time.".into(),
    });
    fortunes.sort_unstable_by(|it, next| it.message.cmp(&next.message));
    Response::OK(html_from(fortunes))
}

async fn get_queries(ctx: Context) -> Result<Response> {
    let count = {
        let queries = ctx.query("queries").unwrap_or("1").parse::<u32>().unwrap_or(1);
        if queries < 1 {1} else if 500 < queries {500} else {queries}
    } as usize;
    let mut worlds = Vec::with_capacity(count);
    for id in random_i32s(count) {
        worlds.push(
            sqlx::query_as::<_, World>(GET_WORLD_STATEMENT)
                .bind(id)
                .fetch_one(ctx.pool())
                .await?
        )
    }
    Response::OK(JSON::from_struct(&worlds)?)
}

async fn get_updates(ctx: Context) -> Result<Response> {
    let count = {
        let queries = ctx.query("queries").unwrap_or("1").parse::<u32>().unwrap_or(1);
        if queries < 1 {1} else if 500 < queries {500} else {queries}
    } as usize;
    let mut worlds = Vec::with_capacity(count);
    let mut new_randomnumbers = random_i32s(count);
    for id in random_i32s(count) {
        let randomnumber = new_randomnumbers.next().unwrap();
        worlds.push(
            sqlx::query_as::<_, World>(UPDATE_WORLD_STATEMENT)
                .bind(randomnumber)
                .bind(id)
                .fetch_one(ctx.pool())
                .await?
        )
    }
    Response::OK(JSON::from_struct(&worlds)?)
}
