use ohkami::{prelude::*, IntoFang, Fang};
use ohkami::utils::{Payload, File, HTML};
use ohkami::http::Status;

#[Payload(Form)]
#[derive(Debug)]
struct FormData {
    account_name: String,
    pics:         Vec<File>,
}

async fn get_form() -> HTML {
    HTML(include_str!("../form.html"))
}

async fn post_submit(form_data: FormData) -> Status {
    println!("\n\
        {form_data:#?}\n\n\
        ===== submit =====\n\
        [account]  {}\n\
        [pictures] {} files (mime: [{}])\n\
        ==================\n",
        form_data.account_name,
        form_data.pics.len(),
        form_data.pics.iter().map(|f| f.mime_type()).collect::<Vec<_>>().join(", "),
    );

    Status::NoContent
}

struct Logger;
impl IntoFang for Logger {
    fn into_fang(self) -> ohkami::Fang {
        Fang(|req: &Request| {
            println!("[request] {} {}", req.method, req.path());

            if let Some(body) = req.payload() {
                let content_type = req.headers.ContentType().unwrap_or("");
                println!("[payload] {content_type:?}\n{}", body.escape_ascii());
            }
        })
    }
}

#[tokio::main]
async fn main() {
    Ohkami::with(Logger, (
        "/form"  .GET(get_form),
        "/submit".POST(post_submit),
    )).howl(5000).await
}
