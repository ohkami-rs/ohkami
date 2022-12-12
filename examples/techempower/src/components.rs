pub(crate) mod consts {
    use std::ops::RangeInclusive;

    pub const RAND_RANGE: RangeInclusive<usize>  = 1..=10000;

    pub const PREPARE_GET_WORLD:    &'static str = "SELECT id, randomnumber FROM world WHERE id = $1";
    pub const PREPARE_GET_FORTUNE:  &'static str = "SELECT id, message FROM fortune";
    pub const PREPARE_UPDATE_WORLD: &'static str = "UPDATE world SET randomnumber = $1 WHERE id = $2 RETURNUNG id, randomnumber";

    pub const DB_URL:              &'static str  = "postgres://benchmarkdbuser:benchmarkdbpass@tfb-database/hello_world?sslmode=disable";
    pub const MAX_CONNECTIONS:     u32           = 10000;
}

pub(crate) mod models {
    use serde::Serialize;
    use sqlx::FromRow;

    #[derive(FromRow, Serialize)]
    pub struct World {
        id:           i32,
        randomnumber: i32,
    }

    #[derive(FromRow, Serialize)]
    pub struct Fortune {
        pub id:     i32,
        pub message: String
    }
}

pub(crate) mod functions {
    use ohkami::response::Body;
    use rand::{Rng, rngs::ThreadRng};
    use super::{models::Fortune, consts::RAND_RANGE};

    pub fn html_from(fortunes: Vec<Fortune>) -> Body {
        Body::text_html(fortunes
            .into_iter()
            .fold(
                String::from("<!DOCTYPE html><html><head><title>Fortunes</title></head><body><table><tr><th>id</th><th>message</th></tr>"),
                |it, next| it + &format!("<tr><td>{}</td><td>{}</td></tr>", next.id, next.message)
            ) + "</table></body></html>"
        )
    }

    pub fn random_i32(generator: &mut ThreadRng) -> i32 {
        generator.gen_range(RAND_RANGE) as i32
    }
}





