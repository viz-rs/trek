use trek::App;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.router()
        .get("/", |_| async { "Hello" })
        .get("/rust", |_| async { "Rust" })
        .get("/2018", |_| async { "2018" });

    let _ = app.run("127.0.0.1:8000").await;
}
