# chati

A project that leverages chatgpt.com to automatically translate C code block comments in a file.

## How is it working

This project is composed of several components:

* CommentExtractor
* Chati
* merger

### CommentExtractor

It use a Iterator model to extract C code block comments from a file.
Inside implementation, it use a state machine to extract comment.
See function `find_c_comments_internal`.

Currently, it can not handle large blocks of comments(larger than 4k).

### Chati

Chati uses webdriver and cdp(chrome devtools protocol) to interact with
https://chatgpt.com.

For webdriver, it uses [fantoccini](https://docs.rs/fantoccini/latest/fantoccini/) crates.
For cdp, it uses [tokio_tungstenite](https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/) crate to send and receive the protocol data.

It could try to login automatically to https://chatgpt.com. Sadly, most of time, it can't.
When it can not do that for you, you have to login in manually(with a little tricks).

Currently, if you encounter some errors when translating wokrs in process, you have to kill the program and restart.

To start to translate a file,

```shell
cargo r --bin bcct demo.c
```

You can play with chati with the executor chati,

```shell
cargo r --bin chati
```

### merger

It merges original code file with the translated comments generating from the bcct executable.

```shell
cargo r --bin merger demo.c demo.txt demo.out
```

## TODO

* handle large block of the C code comment

  test file: [nodeAgg.c](https://github.com/postgres/postgres/blob/db0c96cc18aec417101e37e59fcc53d4bf647915/src/backend/executor/nodeAgg.c)

* continue to translate the remaining blocks when the webpage gets refreshed
