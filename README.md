## CI Kit

Playground project to learn Rust while building a better CI reporting experience.

### Goals

- Provide a minimal but functional UI to display JUnit test reports (both CLI and web-based).
- Notify test outcome through various channels: i.e. slack, github comments, etc.
- Have fun!

### Build, test, and run

Assuming you have `rustup` installed with the default toolchain, simply:

`cargo build`

and 

`cargo test` to run the unit tests.

Finally, you can iteratively recompile and run the program with:

```
RUST_BACKTRACE=1 cargo run -- -c sample.config.toml test-report -s 'time asc' -p ~/path/to/project-with-junit-testreports
```

You might have to amend the `report_dir_pattern` config value with a glob expression that matches your project junit XML report dirs.
