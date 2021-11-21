# Chopstick

Chopstick provides two commandline programs (`chop` and `stick`) to allow for quick splitting of files into parts.
`chop` breaks files into parts, and `stick` puts them back together again

## Chop

`chop` takes in the path to a file and either the number of parts you wish to split the file into, or the size of the parts you desire.
To ensure the safety of data, `chop` requires at least enough free space on the disk for the size of each part.
After creating each part, the original file is truncated (shortened) before the next part is created.
This way, `chop` requires minimal additional disk space without the risk of losing any data.
It also means `chop`'s memory usage is relatively low, as only one part (as opposed to the whole file,) needs to be held in memory at a given time.
This makes `chop` suitable for splitting up very large multi-gigabyte files.

### Usage

```
USAGE:
    chop [OPTIONS] <file>

ARGS:
    <file>
            The file to split

OPTIONS:
    -h, --help
            Print help information

    -n, --parts <num_parts>
            The number of parts to chop the file into

    -s, --size <part_size>
            The maximum size each part should be.Accepts units - e.g. 1GB, 20K, 128MiB

    -V, --version
            Print version information
```

## Stick

*To be written...*

## Roadmap

* Add support for processing multiple files with a single command
* Add verbose commandline option
* Add dry run commandline option
* And 'unsafe' mode which requires no additional disk space (by truncating before writing)
