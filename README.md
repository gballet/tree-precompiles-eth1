[![CircleCI](https://circleci.com/gh/gballet/tree-precompiles-eth1.svg?style=svg)](https://circleci.com/gh/gballet/tree-precompiles-eth1

# Ethereum 1.x tree-management precompile

# Running tests

Tests aren't thread-safe because they use the same input source. As a result, multi-threaded should be disabled to run the tests:

```
$ cargo test -- --test-threads=1
```
