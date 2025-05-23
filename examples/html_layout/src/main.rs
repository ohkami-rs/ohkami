use ohkami::prelude::*;
use ohkami::serde::Deserialize;
use ohkami::format::{Query, HTML};
use uibeam::{UI, Beam};

struct Layout {
    title: String,
    children: UI,
}
impl Beam for Layout {
    fn render(self) -> UI {
        UI! {
            <html>
                <head>
                    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" />
                    <title>{&*self.title}</title>
                </head>
                <body>
                    {self.children}
                </body>
            </html>
        }
    }
}
impl Layout {
    fn fang_with_title(title: &str) -> impl FangAction {
        #[derive(Clone)]
        struct Fang {
            title: String,
        }

        impl FangAction for Fang {
            async fn back(&self, res: &mut Response) {
                if res.headers.ContentType().is_some_and(|x| x.starts_with("text/html")) {
                    let content = res.drop_content().into_bytes().unwrap();
                    let content = std::str::from_utf8(&*content).unwrap();
                    res.set_html(uibeam::shoot(UI! {
                        <Layout title={self.title.clone()}>
                            unsafe {content}
                        </Layout>
                    }));
                }
            }
        }

        Fang {
            title: title.to_string(),
        }
    }
}

struct Counter {
    initial_count: i32,
}
impl Beam for Counter {
    fn render(self) -> UI {
        UI! {
            <div>
                <h1 class="text-2xl font-bold">
                    "count: "<span id="count">{self.initial_count}</span>
                </h1>
                <button
                    id="increment"
                    class="bg-blue-500 text-white px-4 py-2 rounded"
                >"+"</button>
                <button
                    id="decrement"
                    class="bg-red-500 text-white px-4 py-2 rounded"
                >"-"</button>

                <script>r#"
                    const count = document.getElementById('count');
                    document.getElementById('increment').addEventListener('click', () => {
                        count.innerText = parseInt(count.innerText) + 1;
                    });
                    document.getElementById('decrement').addEventListener('click', () => {
                        count.innerText = parseInt(count.innerText) - 1;
                    });
                "#</script>
            </div>
        }
    }
}

#[derive(Deserialize)]
struct CounterMeta {
    init: Option<i32>,
}

async fn index(Query(q): Query<CounterMeta>) -> HTML<std::borrow::Cow<'static, str>> {
    let initial_count = q.init.unwrap_or(0);
    
    HTML(uibeam::shoot(UI! {
        <Counter initial_count={initial_count} />
    }))
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        Layout::fang_with_title("Counter Example"),
        "/".GET(index),
    )).howl("localhost:5555").await
}
