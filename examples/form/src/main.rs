use ohkami::{prelude::*, IntoFang, Fang};
use ohkami::utils::{Payload, File};

#[Payload(FormData)]
struct FormData {
    account_name: String,
    pics:         Vec<File>,
}

async fn get_form(c: Context) -> Response {
    c.OK().html(include_str!("../form.html"))
}

async fn post_submit(c: Context, form_data: FormData) -> Response {
    println!("\n\
        ===== submit =====
        [account] {}\n\
        [pictures] {} files (mime: [{}])\n\
        ==================",
        form_data.account_name,
        form_data.pics.len(),
        form_data.pics.iter().map(|f| f.mime_type()).collect::<Vec<_>>().join(", "),
    );

    c.NoContent()
}

struct Logger;
impl IntoFang for Logger {
    fn bite(self) -> ohkami::Fang {
        Fang(|_: &mut Context, req: Request| {
            println!("[request] {} {}", req.method(), req.path());
            req
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
