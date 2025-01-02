# fileop

replace unintialized variables with the initialized

## generate var file

```shell
ast-grep --pattern "$A $B;" --lang c postgresql/src/ | grep -v '(' | grep -v '=' > var
```

NOTE: The above pattern will select more statements than pure c variable declarations.
Although we can use [rules](https://ast-grep.github.io/guide/rule-config.html),

```shell
ast-grep scan --rule c_declaration.yml postgresql/src/ > var
```

but its output format is not friendly for other tools.

## static linking

```shell
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release
```
