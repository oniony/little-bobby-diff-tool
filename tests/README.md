# Integration Tests

## Usage

Run specific tests:

```sh
./run added-table view-changed
```

Run all tests:

```sh
./run
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
