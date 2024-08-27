# detect cpu memory reorder

run with release build

* detect memory reorder

```
cargo r --release
```

* disable memory reorder

```
cargo r --release --features mfence
```

# reference

1. https://preshing.com/20120515/memory-reordering-caught-in-the-act/
