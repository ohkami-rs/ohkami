use std::fmt::Write;
use ohkami::prelude::Body;
use serde::Serialize;
use sqlx::FromRow;


#[derive(FromRow, Serialize)]
pub(crate) struct WorldRow {
    id:            i32,
    random_number: i32,
}

#[derive(FromRow, Serialize)]
pub(crate) struct Fortune {
    pub id:     i32,
    pub message: String
}
pub(crate) fn html_from(mut fortunes: Vec<Fortune>) -> Body {
    fortunes.sort_unstable_by(|it, next|
        it.message.cmp(&next.message)
    );

    Body::text_html(fortunes
        .into_iter()
        .fold(
            String::from("<!DOCTYPE html><html><head><title>Fortunes</title></head><body><table><tr><th>id</th><th>message</th></tr>"),
            |it, next| it + &format!("<tr><td>{}</td><td>{}</td></tr>", next.id, next.message)
        ) + "</table></body></html>"
    )
}

pub(crate) fn random_i32() -> i32 {
    (rand::random::<u32>() % 10000 + 1) as i32
}

pub(crate) const PREPARE_GET_WORLD:   &'static str = "SELECT id, randomnumber FROM world WHERE id = $1";
pub(crate) const PREPARE_GET_FORTUNE: &'static str = "SELECT id, message FROM fortune";
#[allow(non_snake_case)]
pub(crate) fn PREPARE_UPDATE_WORLDS(random_number: i32) -> String {
    // let mut updates = Vec::with_capacity(500);
    // for num in 1..=500_u16 {
        let (mut query, mut pl) = (String::with_capacity(73 + 27*random_number as usize), 1_u16);
        query.push_str("UPDATE world SET randomnumber = CASE id ");
        for _ in 1..=random_number {
            let _ = write!(&mut query, "when ${} then ${} ", pl, pl + 1);
            pl += 2
        }
        query.push_str("ELSE randomnumber END WHERE id IN (");
        for _ in 1..=random_number {
            let _ = write!(&mut query, "${},", pl);
            pl += 1
        }
        query.pop();
        query.push(')');

        query.shrink_to_fit();
        query
}