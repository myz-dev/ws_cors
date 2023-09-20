I wanted to test how to configure a Rust axum web server to accept web socket connections from different origins.
It turns out, you do not need to do that, as web socket connections are not subject to CORS rules.
So I have just put together this working example of a web socket connection from a different origin.
This example is basically a combination of different examples that can be found in the axum
[examples directory](https://github.com/tokio-rs/axum/tree/main/examples).
This is pulling in the main branch version of axum so with future changes this example might not work anymore.

# Run
Just run `cargo run`
# See
Open `localhost:3000` in a web browser and open the console to see the messages coming in.
