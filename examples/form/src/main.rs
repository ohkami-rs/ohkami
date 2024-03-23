use ohkami::prelude::*;
use ohkami::typed::{Payload, status::NoContent};
use ohkami::builtin::payload::{Multipart, utils::File};


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
    account_name: &'req str,
    
    pics:         Vec<File<'req>>,
}

async fn post_submit(form_data: FormData<'_>) -> NoContent {
    println!("\n\
        ===== submit =====\n\
        [account name] `{}`\n\
        [  pictures  ] {} files (mime: [{}])\n\
        ==================\n",
        form_data.account_name,
        form_data.pics.len(),
        form_data.pics.iter().map(|f| f.mimetype).collect::<Vec<_>>().join(", "),
    );

    NoContent
}


struct Logger;
impl BackFang for Logger {
    type Error = std::convert::Infallible;
    fn bite(&self, res: &mut Response, req: &Request) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        println!();
        println!("[ req ]\n{:?}", req);
        println!("[ res ]\n{:?}", res);

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
