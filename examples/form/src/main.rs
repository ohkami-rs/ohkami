use ohkami::prelude::*;
use ohkami::typed::{Payload, status::NoContent};
use ohkami::builtin::{payload::Multipart, utils::File};


struct FormTemplate;
impl ohkami::IntoResponse for FormTemplate {
    fn into_response(self) -> Response {
        Response::OK().html(include_str!("../form.html"))
    }
}

async fn get_form() -> FormTemplate {
    FormTemplate
}


#[Payload(Multipart/D)]
struct FormData<'req> {
    #[serde(rename = "account-name")]
    account_name: Option<&'req str>,
    
    pics: Vec<File<'req>>,
}

async fn post_submit(form_data: FormData<'_>) -> NoContent {
    println!("\n\
        ===== submit =====\n\
        [account name] {:?}\n\
        [  pictures  ] {} files (mime: [{}])\n\
        ==================",
        form_data.account_name,
        form_data.pics.len(),
        form_data.pics.iter().map(|f| f.mimetype).collect::<Vec<_>>().join(", "),
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
    Ohkami::with((Logger,), (
        "/form"  .GET(get_form),
        "/submit".POST(post_submit),
    )).howl("localhost:5000").await
}
