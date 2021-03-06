Fillout
=======
[![Crates.io]][Crates.io badge]
[![Changelog]][Changelog badge]

`Fillout` generates files by filling out placeholders in templates.

I made `fillout` because I needed to generate one-time DNS quizes consisting of
a [zone file] and a questionaire file.
However nothing in its design is specific to or specially geared towards zone
files or questionaires.


## Features

* **Dead-simple template syntax**.
  Placeholders are marked up with double curly braces.
  The template syntax is meant to be on par with your average group mail
  software.
* **Validation**.
  If one of your templates contains a placeholder that isn't present in the
  data you provide, this is reported as an error and no files are written.
* **Integration**.
  The data file is read from STDIN to streamline generating files
  from dynamically generated data.


## Installation

[Install stable Rust and Cargo]:

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Compile and install `fillout` using Cargo:

```
$ cargo install fillout
```


## Usage

For the purpose of this demonstration, let's create some example data:

```
$ mkdir templates
$ echo 'Lorem {{alpha}} {{beta}} sit.' > templates/first.txt.tmpl
$ echo 'Duis {{gamma}} irure {{beta}}.' > templates/second.txt.tmpl
$ echo 'alpha, ipsum' > input.data
$ echo 'beta, dolor' >> input.data
$ echo 'gamma, aute' >> input.data
```

Generate a set of files into the output directory:

```
$ mkdir output
$ fillout -o output templates/* < input.data
$ ls -l output/
total 8K
-rw------- 1 mattias mattias 23 apr 18 12:35 first.txt
-rw------- 1 mattias mattias 23 apr 18 12:35 second.txt
$ cat output/first.txt
Lorem ipsum dolor sit.
$ cat output/second.txt
Duis aute irure dolor.
```

Now let's try the validation.
Add another template with a misspelled placeholder and attempt to generate a new
set of files:

```
$ echo 'Dolor magna eget est lorem {{ahpla}}.' > templates/third.txt.tmpl
$ fillout -o output templates/* < input.data
Error: Failed to validate data file against template-files ["/tmp/templates/first.txt.tmpl", "/tmp/templates/second.txt.tmpl", "/tmp/templates/third.txt.tmpl"]

Caused by:
    Placeholders used in templates are not defined in data-file: ["ahpla"]
$ ls -l output/
total 8K
-rw------- 1 mattias mattias 23 apr 18 12:35 first.txt
-rw------- 1 mattias mattias 23 apr 18 12:35 second.txt
```

Note that no new files were created and that the files that already existed are
unchanged (as indicated by their timestamps and file sizes).

Finally, let's take a look at `fillout`'s own usage documentation:

```
$ fillout --help
fillout 0.1.1
Fill out placeholders in templates

Reads a data file and a set of template-files and writes a
set of output-files corresponding to the templates files.

Placeholders in the template-files are replaced with values
from the data file in the output-files. Each placeholder is
marked up in the template-files by its name in double
braces. E.g. {{ recipient }}. Whitespace surrounding the
name is optional. A single placeholder name may optionally
recur multiple times in multiple in multiple template-files.
If so, they are replaced with the same value each time.

The data file is a headerless CSV file, read from STDIN.
Each record must have two fields - a placeholder-name and a
value. The delimiter is an ASCII comma. Leading and trailing
whitespace are trimmed from both placeholder-names and
values. Records are terminated with CR, LF or CRLF. Fields
may be quoted with ASCII double quote characters. If you
need to use an ASCII double quote you can escape it by
doubling it. If a record starts with a hash character, this
line is ignored.

USAGE:
    fillout [OPTIONS] [TEMPLATE-FILE]...

FLAGS:
    -h, --help       
            Prints help information

    -V, --version    
            Prints version information


OPTIONS:
    -o, --output-dir <DIR>
            The directory where output-files are written -
            if none is given each output-file is written to
            the directory of its corresponding template-file

ARGS:
    <TEMPLATE-FILE>...
            A template-file - must have the .tmpl file
            extension
```


## Contact

To ask questions, report bugs or suggest features, please use the [issue
tracker].




[Changelog badge]: https://github.com/mattias-p/fillout/blob/main/CHANGELOG.md
[Changelog]: https://raw.githubusercontent.com/mattias-p/fillout/main/img/changelog.svg?sanitize=true
[Crates.io badge]: https://crates.io/crates/fillout
[Crates.io]: https://img.shields.io/crates/v/fillout.svg
[Install stable Rust and Cargo]: https://www.rust-lang.org/tools/install
[Issue tracker]: https://github.com/mattias-p/fillout/issues
[License]: LICENSE
[Zone file]: https://en.wikipedia.org/wiki/Zone_file
