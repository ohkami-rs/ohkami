use ohkami::prelude::*;
use ohkami::utils::{Payload, File};

#[Payload(FormData)]
struct FormData {
    account_name: String,
    pics:         Vec<File>,
}

#[Payload(FormData)]
struct FormData2 { /* Should be NOT ACCEPTABLE */
    account_name: Option<String>,
    pics:         Vec<File>,
}

async fn get_form(c: Context) -> Response {
    c.OK().html(include_str!("../form.html"))
}

async fn post_submit(c: Context, form_data: FormData) -> Response {

    c.NoContent()
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/form"  .GET(get_form),
        "/submit".POST(post_submit),
    )).howl(5000).await
}
