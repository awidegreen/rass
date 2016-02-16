# rass 

A [password-store](https://www.passwordstore.org/) clone written in Rust. 

The name `rass` is a combination of **r**ust and p**ass** whereas `pass` being 
the command line tool for password-store.

## Requirements

Due to `rass` dependency to [rust-gpgme](https://crates.io/crates/gpgme/) you 
need to have a recent version of GPGme installed, 
[see](https://github.com/johnschug/rust-gpgme). 

## Installation

```shell
$ cargo install rass
```

## Limitations

In comparison to `pass`, `rass` does not support [yet]: 
* show the content of the password in tree-style: I'm currently lacking a 
proper way of showing a tree like structure from rust
* relay git command (like `rass git ...`) directly to `git`
* no clipboard support
* editing of pass entries
* initialize
* not all environment variables are support
  * supported: `PASSWORD_STORE_DIR`
  * not supported: `PASSWORD_STORE_DIR`, `PASSWORD_STORE_GIT`,
  `PASSWORD_STORE_X_SELECTION`, `PASSWORD_STORE_CLIP_TIME`, 
  `PASSWORD_STORE_UMASK`, `EDITOR`
 

## Usage

**Note**: `rass` is not yet able to create and initialize a new password store,
therefore use `pass init <gpg-ide>`. 

As `pass`, `rass` assume that your password store is located in 
`$HOME/.password-store`. If your store is in a different location, set the 
`PASSWORD_STORE_DIR` variable. 

Show the help
```shell
$ rass -h

# some detailed subcommand help
$ rass insert -h
```

List all store entries (subcommand `ls`)
```shell
$ rass 
```

Show an entry
```shell
$ rass PASS_ENTRY


Insert a new entry (subcommand `insert` or `add`)
```shell
# single-line
$ rass insert foobar

# multi-line 
$ rass insert -m foobar
```


## ToDo

* subcommands
  * `init`
  * `generate`
  * `mv`
  * `cp`
  * `git` - how?
  * `edit`
* some more tests
* a simple CLI UI?


## License

Copyright (C) 2016 by Armin Widegreen

This is free software, licensed under The [ISC License](LICENSE).
