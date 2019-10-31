use trek::App;
use trek_core::context::Context;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.router()
        .get("/", |_| async { "Home" })
        .get("/rust", |_| async { "Rust" })
        .get("/2018", |_| async { "2018" })
        .get("/users/:id", |cx: Context<()>| {
            async move { cx.params::<String>().unwrap() }
        });

    let _ = app.run("127.0.0.1:8000").await;
}
