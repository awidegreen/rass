# rass
[![Build Status](https://travis-ci.org/awidegreen/rass.svg?branch=master)](https://travis-ci.org/awidegreen/rass)

A [password-store](https://www.passwordstore.org/) clone written in Rust.

The name `rass` is a combination of **r**ust and p**ass** whereas `pass` being
the command line tool for password-store.

[Documentation](https://awidegreen.github.io/rass/)

## Requirements

Due to `rass` dependency to [rust-gpgme](https://crates.io/crates/gpgme/) you
need to have a recent version of GPGme installed,
[see](https://github.com/johnschug/rust-gpgme).

## Installation

From source:
```shell
$ git clone https://github.com/awidegreen/rass.git
$ cd rass
$ cargo build --release
```

NOTE: Not released on crates.io yet
```shell
$ cargo install rust-rass
```

## Limitations

In comparison to `pass`, `rass` does not support [yet]:
* no clipboard support
* not all environment variables are support
  * supported: `PASSWORD_STORE_DIR`
  * not supported: `PASSWORD_STORE_GIT`, `PASSWORD_STORE_X_SELECTION`,
  `PASSWORD_STORE_CLIP_TIME`, `PASSWORD_STORE_UMASK`


## Usage

As `pass`, `rass` assume that your password store is located in
`$HOME/.password-store`. If your store is in a different location, set the
`PASSWORD_STORE_DIR` variable.

Show the help
```shell
$ rass -h

# some detailed subcommand help
$ rass insert -h
```

Initialize a new password-store.
```
$ rass init /path/to/new/store
```

List all store entries (subcommand `ls`)
```shell
$ rass
```

Show an entry
```shell
$ rass PASS_ENTRY
```

Insert a new entry (subcommand `insert` or `add`)
```shell
# single-line
$ rass insert foobar

# multi-line
$ rass insert -m foobar
```

Dispatch `git` command to rass, executed within the password-store
```shell
# push new entries to origin
$ rass git push origin master

# pull latest changes
$ rass git pull

# show the git log of the password-store
$ rass git log
```

Grep for a string in the password store.
```shell
# search for "foobar" in the entire store
$ rass grep foobar
```
For more information see the help: `rass help`

Edit an entry will make use of the `EDITOR` environment variable. If the variable
is not, `vim` will be assumed.

```shell
# edit entry foobar
$ rass edit foobar
```

## ToDo

* subcommands
  * `generate`
  * `mv`
  * `cp`
* some more tests
* a simple CLI UI?


## License

Copyright (C) 2017 by Armin Widegreen

This is free software, licensed under The [ISC License](LICENSE).
