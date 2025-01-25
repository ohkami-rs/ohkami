use ohkami::prelude::*;
use ohkami::typed::status::NoContent;
use ohkami::format::{Multipart, File};
use ohkami::serde::Deserialize;


struct FormTemplate;
impl ohkami::IntoResponse for FormTemplate {
    fn into_response(self) -> Response {
        Response::OK().with_html(include_str!("../form.html"))
    }
}

async fn get_form() -> FormTemplate {
    FormTemplate
}


#[derive(Deserialize)]
struct FormData<'req> {
    #[serde(rename = "account-name")]
    account_name: Option<&'req str>,
    pics: Vec<File<'req>>,
}

async fn post_submit(
    Multipart(form): Multipart<FormData<'_>>
) -> NoContent {
    println!("\n\
        ===== submit =====\n\
        [account name] {:?}\n\
        [  pictures  ] {} files (mime: [{}])\n\
        ==================",
        form.account_name,
        form.pics.len(),
        form.pics.iter().map(|f| f.mimetype).collect::<Vec<_>>().join(", "),
    );

    NoContent
}

#[derive(Clone)]
struct Logger;
impl FangAction for Logger {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        println!("\n[req]\n{req:#?}");
        Ok(())
    }
    async fn back<'a>(&'a self, res: &'a mut Response) {
        println!("\n[res]\n{res:#?}");
    }
}

#[tokio::main]
async fn main() {
    Ohkami::new((Logger,
        "/form"  .GET(get_form),
        "/submit".POST(post_submit),
    )).howl("localhost:5000").await
}
