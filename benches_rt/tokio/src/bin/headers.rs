use ohkami::prelude::*;

#[derive(Clone)]
struct SetHeaders;
impl FangAction for SetHeaders {
    async fn back(&self, res: &mut Response) {
        res.headers.set()
            .server("Ohkami")
            .cross_origin_embedder_policy("require-corp")
            .cross_origin_resource_policy("same-origin")
            .referrer_policy("no-referrer")
            .strict_transport_security("max-age=15552000; includeSubDomains")
            .x_content_type_options("nosniff")
            .x_frame_options("SAMEORIGIN")
        ;
    }
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .event_interval(11)
        .global_queue_interval(31)
        .build()
        .expect("Failed building the Runtime")
        .block_on(Ohkami::new((
            SetHeaders,
            "/user/:id"
                .GET(|Path(id): Path<String>| async {id}),
        )).run("0.0.0.0:3000"))
}
