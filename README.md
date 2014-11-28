# Rdit

[![Build Status](https://travis-ci.org/gchp/rdit.svg?branch=master)](https://travis-ci.org/gchp/rdit)

Rdit is a terminal-based text-editor written in Rust.

This is really an experimental project that I'm using as I learn Rust.
It's very basic right now, just opening existing files and moving the cursor
around. One day it will be a "real" editor, though! (maybe). Pretty much everything
at this stage is subject to change.
I'm also interested in a better name for this, definitely open to suggestions. 

I've never built an editor before, so this is new territory for me. Please
don't look at the early history of the project, it's full of my silly
mistakes and awful workarounds as I try figure out how this all works.

## Usage

Clone the project and run `cargo build`.

Then to start the editor run `./target/rdit /path/to/file.txt`.

You can move the cursor around with the arrow keys.

To save, press `Ctrl-s`.
To exit, press `Ctrl-q`, followed by `Ctrl-c`.

## TODO list

- [ ] Creating new files
