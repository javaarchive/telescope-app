# telescope_core
## Building
The most strict dependency is [aws-lc-rs](https://github.com/aws/aws-lc-rs/). On Windows you need to have CMake installed. This builds on my machine but I haven't been able to get this to build on my CI windows server yet so there is not yet an exhaustive list of dependencies. After that you should just be able to run `cargo build`.