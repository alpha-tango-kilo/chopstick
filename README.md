# Chopstick

[![Codeberg CI](https://ci.codeberg.org/api/badges/alpha-tango-kilo/chopstick/status.svg)](https://ci.codeberg.org/alpha-tango-kilo/chopstick)
[![Crates.io](https://img.shields.io/crates/v/chopstick.svg)](https://crates.io/crates/chopstick)

Chopstick provides two commandline programs (`chop` and `stick`) to allow for quick splitting of files into parts.
`chop` breaks files into parts, and `stick` puts them back together again

The project aims to do this in as simple of a manner as possible with minimally sized executables.
I do not expect to add a large number of features, though I do have some in mind which may or may not make the cut (see [Roadmap](#Roadmap)).

## Chop

`chop` takes in the path to a file and either the number of parts you wish to split the file into, or the size of the parts you desire.
To ensure the safety of data, `chop` requires at least enough free space on the disk for the size of each part.
Given an error, you can end up in a partially completed state, however all of the bytes of your files will still be intact.
After creating each part, the original file is truncated (shortened) before the next part is created.
This way, `chop` requires minimal additional disk space without the risk of losing any data.
It also means `chop`'s memory usage is relatively low, as only one part (as opposed to the whole file,) needs to be held in memory at a given time.
This makes `chop` suitable for splitting up very large multi-gigabyte files.

### Usage

```
USAGE:
    chop [OPTIONS] <--size <part_size>|--parts <num_parts>> <file>

ARGS:
    <file>
            The file to split

OPTIONS:
    -h, --help
            Print help information

    -n, --parts <num_parts>
            The number of parts to chop the file into. Parts will all be the same size (except the       
            last one potentially)

    -r, --retain
            Don't delete the original file (requires more disk space)

            [aliases: no-delete, preserve]

    -s, --size <part_size>
            The maximum size each part should be. Accepts units - e.g. 1GB, 20K, 128MiB. The last
            part may be smaller than the others

    -V, --version
            Print version information
```

## Stick

`stick` takes the name of a chopped file (extension not needed), attempts to discover the other parts within the same directory, and then puts them back together.
This is done by reading a part into memory, writing it to the original file, deleting the part; and rinse & repeat for all parts.
By doing things this way, `stick` only needs as much additional disk space as one part occupies.
As with `chop`, there is no risk of losing any data as nothing is deleted before it has been successfully written.
Given an error you can end up in a partially completed state, however all of the bytes of your files will still be intact.

### Usage

```
USAGE:
    stick [OPTIONS] <file_name>

ARGS:
    <file_name>
            The file to reconstruct. You only need to specify one part, providing the extension is
            optional

OPTIONS:
    -h, --help
            Print help information

    -r, --retain
            Don't delete the part files (requires more disk space)

            [aliases: no-delete, preserve]

    -V, --version
            Print version information
```

## Roadmap

### To stable! (v1.0.0)

* Better testing
* ~~Move to [`pico_args`](https://github.com/RazrFalcon/pico-args) to reduce binary size~~

### After v1.0.0

* Add support for processing multiple files with a single command
* Add verbose commandline option
* Add dry run commandline option
* And 'unsafe' mode which requires no additional disk space (by truncating before writing)
* Recovering from mid-way aborted states
