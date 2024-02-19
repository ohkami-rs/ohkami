use ohkami::prelude::*;
use ohkami::utils::HTML;
use ohkami::typed::{Payload, File};

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
impl BackFang for Logger {
    type Error = std::convert::Infallible;
    fn bite(&self, res: &mut Response, req: &Request) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        println!("[request ] {:?}", req);
        println!("[response] {:?}", res);

        async {Ok(())}
    }
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/form"  .GET(get_form),
        "/submit".POST(post_submit),
    )).howl_with(Logger, "localhost:5000").await
}
