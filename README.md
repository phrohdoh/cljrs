# About CLJRS

CLJRS is a (work-in-progress) Clojure reader implemented in the Rust language

# LICENSE

license: as proprietary as can be

this codebase isn't ready for public use, so we're not sharing anything yet

# file structure

- the top-level file *`Cargo.toml`* declares a [Cargo virtual workspace]()
- the top-level directory *`crates`* contains Rust crates
  - *`cljrs-toy`* explores sending (readtime) `Value`s between threads
  - *`cljrs-bevy`* explores sending (readtime) `Value`s between threads, in a [Bevy]() app/game

## running the binary crates

- ensure `asdf install` has been executed in this directory (the one containing the `.tool-versions` file)
- ensure `direnv allow` has been executed in this directory (the one containing the `.envrc` file)

1. create 2 terminals/shells/etc
1. let the first be referred to as ***runner***
1. let the second be referred to as ***inputter***

in ***runner***:

```sh
exec 9<> "${CLJRS_STDIN_FIFO_PATH}"

cat < "${CLJRS_STDIN_FIFO_PATH}" &
__cljrs_stdin_fifo_reader_pid=$!

cat > "${CLJRS_STDIN_FIFO_PATH}" &
__cljrs_stdin_fifo_writer_pid=$!

cargo run < "${CLJRS_STDIN_FIFO_PATH}"
```

then in ***inputter***

```sh
echo "hello" > "${CLJRS_STDIN_FIFO_PATH}"
```

once you're done (e.g. have CTRL-C'd the `cargo run`), don't forget to clean up ***runner***:

```sh
exec 9>&-

kill $__cljrs_stdin_fifo_reader_pid
unset __cljrs_stdin_fifo_reader_pid

kill $__cljrs_stdin_fifo_writer_pid
unset __cljrs_stdin_fifo_writer_pid
```

Why fd 9? I don't know why, but it works.
When I used 0, it didn't work. 🤷

### managing FIFOs

create the FIFO(s) (A.K.A. named pipes):

```sh
create-fifos.sh
```

remove the FIFO(s) (A.K.A. named pipes) for cleanup purposes:

```sh
remove-fifos.sh
```

These executables (scripts) are located in `{PROJECT_ROOT}/bin` which is automatically added to `$PATH` by [*`direnv`*]().

# formatted viewing of this file in a terminal

```sh
# https://github.com/charmbracelet/glow
brew install glow
```

```sh
glow ./README.md
```