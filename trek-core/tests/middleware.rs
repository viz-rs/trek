#[test]
fn middleware_example() {
    use futures::{
        executor::block_on,
        future::{BoxFuture, Future},
    };

    #[derive(Clone, Copy)]
    struct Context<'a> {
        handler: &'a (dyn (for<'r> Fn(Context<'r>) -> BoxFuture<'static, String>)
                 + 'static
                 + Send
                 + Sync),
        stack: &'a [Box<
            (dyn for<'r> Fn(Context<'r>) -> BoxFuture<'r, String> + 'static + Send + Sync),
        >],
    };

    impl<'a> Context<'a> {
        fn next(mut self) -> BoxFuture<'a, String> {
            if let Some((current, next)) = self.stack.split_first() {
                self.stack = next;
                (current)(self)
            } else {
                (self.handler)(self)
            }
        }
    }

    let mut stack: Vec<
        Box<(dyn for<'r> Fn(Context<'r>) -> BoxFuture<'r, String> + 'static + Send + Sync)>,
    > = Vec::new();

    fn a<'a>(cx: Context<'a>) -> BoxFuture<'a, String> {
        Box::pin(async move {
            println!("middle: {} {}", "a", 0);
            let res = cx.next().await;
            println!("middle: {} {}", "a", 5);
            res
        })
    }

    fn b<'a>(cx: Context<'a>) -> BoxFuture<'a, String> {
        Box::pin(async move {
            println!("middle: {} {}", "b", 1);
            let res = cx.next().await;
            println!("middle: {} {}", "b", 4);
            res
        })
    }

    fn c<'a>(cx: Context<'a>) -> BoxFuture<'a, String> {
        Box::pin(async move {
            println!("middle: {} {}", "c", 2);
            let res = cx.next().await;
            println!("middle: {} {}", "c", 3);
            res
        })
    }

    fn h(cx: Context<'_>) -> impl Future<Output = String> {
        println!("handler: {} {}", "h", "handler");
        async { String::from("trek") }
    }

    async fn ha<'r>(cx: Context<'r>) -> String {
        println!("handler: {} {}", "h", "handler");
        String::from("trek")
    }

    fn make<Output, Fut>(
        h: impl Fn(Context<'_>) -> Fut + 'static + Send + Sync,
    ) -> Box<dyn (for<'r> Fn(Context<'r>) -> BoxFuture<'static, Output>) + 'static + Send + Sync>
    where
        Fut: Future<Output = Output> + 'static + Send + Sync,
        // Output: Send + 'static,
    {
        Box::new(move |cx| {
            let fut = (h)(cx);
            Box::pin(async move { fut.await })
        })
    }

    stack.push(Box::new(a));
    stack.push(Box::new(b));
    stack.push(Box::new(c));

    let cx = Context {
        handler: &make(h),
        // handler: &make(ha),
        stack: stack.as_slice(),
    };

    block_on(async move {
        let res = cx.next().await;
        dbg!(res);
    });
}
