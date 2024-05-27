# Integration Tests

## Usage

To run a single test:

```sh
./run added-table
```

To run all tests:

```sh
./run-all
```

## Structure

Each test-case is a directory containing the following files:

* `left.sql`
* `right.sql`
* `expected.txt`

The `run` script starts two databases, nominally left and right, and runs the
corresponding initialisation script into each.

`run` writes the output of comparing the two databases into a file called
`actual.txt` in the test directory. It then compares `actual.txt` against the
`expected.txt` file.
