![Little Bobby Diff Tool](https://raw.githubusercontent.com/oniony/little-bobby-diff-tool/main/graphics/lbdt.png)

# Overview

Little Bobby Diff Tool is a CLI tool to compare database schemas.

It currently compares the following across one or more schemas.

- [X] Tables
- [X] Table columns
- [X] Views
- [X] Routines
- [ ] Sequences
- [ ] Constraints
  - [ ] Check
  - [ ] Foreign Key
  - [ ] Primary Key
  - [ ] Unique
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
     --schema public
     --schema other
```

## Options

_tbd_

# Versions

_tbd_

# About

LBDT is written and maintained by Paul Ruane (<paul.ruane@oniony.com>) and is available at <http://github.com/oniony/lbdt/>.

LBDT is written in Rust: <http://rust-lang.org/>

- - -

Copyright 2024 Paul Ruane

Copying and distribution of this file, with or without modification,
are permitted in any medium without royalty provided the copyright
notice and this notice are preserved.  This file is offered as-is,
without any warranty.
