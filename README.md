# JV

JV is a simple JSON viewer that supports jq-like queries. Incidentally it works
also as a basic viewer for plain text files.

[![asciicast](https://asciinema.org/a/233199.svg)](https://asciinema.org/a/233199)

## Features

- To quit hit <kbd>Q</kbd> and to quit any mode focusing the view hit
  <kbd>ESC</kbd>.
- Basic navigation with <kbd>&leftarrow;</kbd>, <kbd>&rightarrow;</kbd>,
  <kbd>&uparrow;</kbd>, <kbd>&downarrow;</kbd>, <kbd>PgUp</kbd>,
  <kbd>PgDown</kbd>
- Go to a given line and or column by entering command mode with <kbd>:</kbd>
  and then enter the line number and optionally <kbd>:</kbd> followed by the
  column. If you want to go to a given column of the current row just omit the
  line number before <kbd>:</kbd>. Valid examples: "1:20", ":20" and "1".
- Use a jq-like query to quickly jump to an element of the JSON schema. First,
  enter query mode with <kbd>#</kbd> and then enter "/" separated object keys or
  array indices. Example queries: "#/", "#/array/23/name", "#/23".
- Automatically go to reference under cursor by clicking enter.
- Syntax highlighting.

## Install

```bash
$ cargo install --git https://github.com/d-dorazio/jv
$ jv --help
$ jv hello.json
$ jv data.txt
```
