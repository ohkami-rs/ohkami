use ohkami::{prelude::*, IntoFang, Fang};
use ohkami::utils::{Payload, File};

#[Payload(FormData)]
#[derive(Debug)]
struct FormData {
    account_name: String,
    pics:         Vec<File>,
}

async fn get_form(c: Context) -> Response {
    c.OK().html(include_str!("../form.html"))
}

async fn post_submit(c: Context, form_data: FormData) -> Response {
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

    c.NoContent()
}

struct Logger;
impl IntoFang for Logger {
    fn bite(self) -> ohkami::Fang {
        Fang(|_: &mut Context, req: &mut Request| {
            println!("[request] {} {}", req.method(), req.path());

            if let Some ((content_type, body)) = req.payload() {
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
