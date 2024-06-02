![Little Bobby Diff Tool](https://github.com/oniony/little-bobby-diff-tool/blob/main/graphics/lbdt.png?raw=true)

[![Build Status](https://github.com/oniony/little-bobby-diff-tool/actions/workflows/release.yml/badge.svg)](https://github.com/oniony/little-bobby-diff-tool/actions/workflows/release.yml)

# Overview

Little Bobby Diff Tool is a CLI tool to compare database schemas.

It currently compares the following across one or more schemas.

- [X] Tables
  - [X] Optionally ignoring column ordering differences
  - [X] Constraints
- [X] Views
- [X] Routines
  - [X] Optionally ignoring whitespace differences

The following are currently not compared:

- [ ] Sequences
- [ ] Permissions
  - [ ] Roles
  - [ ] Column grants
  - [ ] Table grants
  - [ ] Routine grants
  - [ ] Table privileges
- [ ] Triggers
- [ ] User Defined Types

# Compilation

* Install Rust from <https://www.rust-lang.org/>
* Build Little Bobby Diff Tool:

      $ git clone git@github.com:oniony/lbdt.git
      $ cd lbdt
      $ cargo build
    
# Usage

```sh
lbdt --left URL --right URL --schema SCHEMA [--schema SCHEMA ...]
```

## Examples

```sh
lbdt --left postgres://user:pass@localhost:5432/postgres \
     --right postgres://user:pass@localhost:85432/postgres \
     --schema public \
     --schema other \
     --ignore-column-ordinal
```

# Versions

## 0.8.0

* Generate report rather than print in-situ

## 0.6.0

* Added `--ignore-column-ordinal` flag to ignore column ordering differences.
* Added `--ignore-whitespace` flag to ignore routine whitespace differences.

# About

Little Bobby Diff Tool is written and maintained by Paul Ruane
(<paul.ruane@oniony.com>) and is available at
<http://github.com/oniony/little-bobby-diff-tool/>.

Written in Rust: <http://rust-lang.org/>

- - -

Copyright 2024 Paul Ruane

Copying and distribution of this file, with or without modification, are
permitted in any medium without royalty provided the copyright notice and this
notice are preserved.  This file is offered as-is, without any warranty.
