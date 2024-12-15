# Roaring Bitmaps Library in Rust

This project implements a Roaring Bitmaps library in Rust. Roaring Bitmaps are an efficient compressed data structure for storing sets of integers.

## Getting Started

To get started with this project, clone the repository and navigate to the project directory.

```sh
git clone https://github.com/rishitc/dicedb-reading-group.git
cd './dicedb-reading-group/Roaring Bitmaps/code'
```

## Running Tests

To run the tests for this project, use the following `cargo` command:

```sh
cargo test --color=always --package code --lib ""
```

## Expected Output

When you run the tests, you should see output similar to the following:

```rust
running 5 tests
test tests::union_same_msb_containers ... ok
test tests::intersection_non_empty ... ok
test tests::it_works ... ok
test tests::intersection_empty ... ok
test tests::union_different_msb_containers ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

This indicates that all tests have passed successfully.
