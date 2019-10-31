## Routing

### Example:

```rust
async fn get_users(_: Context<State>) -> String {
    String::from("get users")
}

fn get_user(_: Context<State>) -> impl Future<Output = String> {
    ready(String::from("get users :id"))
}

fn update_user(_: Context<State>) -> impl Future<Output = String> {
    async { String::from("update users :id") }
}

struct Geocoder {}

impl Geocoder {
    fn new(_: Context<State>) -> impl Future<Output = String> {
        ready(String::from("new: geocoder"))
    }

    fn show(_: Context<State>) -> impl Future<Output = String> {
        async { String::from("show: geocoder") }
    }
}

async fn book_new(_: Context<State>) -> String {
    String::from("new: book")
}

fn book_show(_: Context<State>) -> impl Future<Output = String> {
    async { String::from("show: book") }
}

let mut router = Router::<Context<State>>::new();

router
    // Add middlewares
    .middleware(MiddlewareA::new())
    .middleware(MiddlewareB::new())

    // Add root handler: `/`
    .get("/", |_| async { "home" })

    // Scope or Group: v1, `/v1'
    .scope("/v1", |v1| {
        // Clear all middlewares: `MiddlewareA`, `MiddlewareB`
        v1.clear_middleware()
            .middleware(MiddlewareC::new());

        // POST `/v1/login`
        // POST `/v1/submit`
        // POST `/v1/read`
        v1.post("/login", |_| async { "v1 login" })
            .post("/submit", |_| async { "v1 submit" })
            .post("/read", |_| async { "v1 read" });

        // Scope or Group: users, `/v1/users'
        v1.scope("/users", |users| {
            // GET `/v1/users`
            // GET `/v1/users/:id`
            // POST `/v1/users/:id`
            users.get("", get_users);
            users.get("/:id", get_user);
            users.post("/:id", update_user);
        });
    })

    // Resource: `/geocoder`
    //  GET `/geocoder`
    //  GET `/geocoder/new`
    .resource(
        "/geocoder",
        &[
            (Resource::Show, into_box_dyn_handler(Geocoder::show)),
            (Resource::New, into_box_dyn_handler(Geocoder::new)),
        ],
    )

    // Resources: `/books`
    //  GET `/books/:book_id`
    //  GET `/books/new`
    .resources(
        "/books",
        &[
            (Resources::Show, into_box_dyn_handler(book_show)),
            (Resources::New, into_box_dyn_handler(book_new)),
        ],
    )

    // Scope or Group: admin, `/admin`
    .scope("admin", |a| {
        a.any("", |_| async { "6" });
    });
```
