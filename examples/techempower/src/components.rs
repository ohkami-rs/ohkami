use std::fmt::Write;
use serde::Serialize;
use sqlx::FromRow;


#[derive(FromRow, Serialize)]
pub(crate) struct WorldRow {
    id:            i32,
    random_number: i32,
}

#[derive(FromRow, Serialize)]
pub(crate) struct Fortune<'msg> {
    id:     i32,
    message: &'msg str
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