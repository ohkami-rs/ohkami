#![allow(unused)]
use ohkami::Query;

#[Query]
struct GetTasksQuery1 {
    limit:      usize,
    name_start: String,
    name_end:   String,
}

#[Query]
struct GetTasksQuery2 {
    limit:      Option<usize>,
    name_start: String,
    name_end:   String,
}

#[Query]
struct GetTasksQuery3<'ns, 'ne> {
    limit:      usize,
    name_start: &'ns str,
    name_end:   &'ne str,
}

fn main() {}
