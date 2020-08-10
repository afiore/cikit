## CI Kit

Playground project to learn Rust while building a minimum viable CI reporting experience.

### Status

🚧 This is a work in progress and is not feature complete yet!

### Goals

- Provide a minimal but functional UI to display JUnit test reports both through the CLI and as an HTML document.
- Notify test outcome through various channels: i.e. slack web-hooks, github comments, etc.
- Have fun!

### Usage

```
cikit 0.1.0
The continuous integration reporting toolkit

USAGE:
    cikit --config-path <config-path> [project-dir] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config-path <config-path>    Input file

ARGS:
    <project-dir>

SUBCOMMANDS:
    help           Prints this message or the help of the given subcommand(s)
    notify         Notifies the build outcome via Slack
    test-report    Reads the Junit test report
```

### Building and running

Assuming you have `rustup` installed with the default toolchain, simply:

`cargo build`

and 

`cargo test` to run the unit tests.

Finally, you can iteratively recompile and run the program by prefixing the normal executable call with `cargo run --`:

```bash
RUST_LOG='cikit=debug' RUST_BACKTRACE=1 cargo run -- \ 
   -c sample.config.toml ~/code/project-with-junit \ 
   test-report html -o test-report -f
```

You might want to amend the `report_dir_pattern` config value with a glob expression that matches your project junit XML report dir/s.