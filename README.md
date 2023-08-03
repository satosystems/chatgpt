# Simple ChatGPT library and CLI for Rust

This project is the Rust chatgpt library, or a CLI tool that utilizes this library.

## Getting Started

```shell-session
$ cargo run -- Say hello!
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/chatgpt`
Hello! How can I assist you today?
>> Make one pun.
Sure, here's a pun for you:

I'm reading a book on anti-gravity. It's impossible to put down! 📚🚀
>>
```

Pless `Ctrl+C` or `Ctrl+D` stop prompt if you need.

If you want to use it as a library, see [lib.rs](src/lib.rs) and [chatgpt.rs](src/bin/chatgpt.rs).

### Prerequisites

You must have previously created an OpenAI API key and set it to the `OPENAI_API_KEY` environment variable.

Define the `OPENAI_DEBUG=1` environment variable if you want to see what happens when an unexpected error occurs.

### Installing

```shell-session
$ cargo install --path .
...
$
```

Installed to `~/.cargo/bin/chatgpt`.

## Running the tests

```shell-session
$ cargo test
...
$
```

### And coding style tests

```shell-session
$ cargo fmt
...
$
```

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/satosystems/simple-chatgpt/tags).

## Authors

* **Satoshi Ogata** - *Initial work* - [satosystems](https://github.com/satosystems)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
