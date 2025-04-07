# Mini-HTTP Server

I started this implementation through the CodeCrafters challenge: ["Build Your Own HTTP server"](https://app.codecrafters.io/courses/http-server/overview). It provided a nice structure and good pointers to get started. The automated testing they do on every git push is really awesome.

## Next Steps

- Document the implementation. Target audience: beginners in Rust and/or networking

Current implementation is very limited both protocol-wise and performance-wise. It only supports a mini fraction of the HTTP1 protocol and although the implementation is concurrent (multi-threaded), it is not asynchronous yet. So next steps to go more in depth may be:
  - Protocol-wise
    - implement the complete protocol
    - add support for HTTP2 features
  - Performance-wise
    - enable async (with/without Tokio?)
    - load-testing to monitor perf gains

## Running the Server

### Manual

1. Compile the Rust program into an executable binary: `cargo build`
2. Run the executable: `./target/release/flyweight-http-server`

### Shell Script

The `compile_and_run.sh` script automates the 2 steps described above.

You can simply run `./compile_and_run.sh` and get the http-server going!
