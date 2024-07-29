# Bereal Data Transformer

Exports data from BeReal dump (obtained by contacting BeReal support) to other
formats. WIP.

The utility also allows grouping and filtering of Memories (posts). Grouping
creates subfolders for years, months or days. Time-based filtering is possible
as well as posts' caption text-based filtering (regex).

## Build

    cargo b

    or for release build:

    cargo b --release

## Run

Only "working" subcommands are listed.
As input folder, specify your unzipped BeReal dump.

**Disclaimer**: some commands run parallel conversions of images, they **will** eat up all your CPU cores' performance for a while!

### Memories export

    cargo r --release -- memories --help

Example:

    cargo r --release -- --input ./my/input/data  --output ./out-mem memories --image-format jpeg --group day-flat

### Realmojis export

    cargo r --release -- realmojis --help

Example:

    cargo r --release -- --input ./my/input/data  --output ./out-moji realmojis --group emoji

Check out subcommands' respective `--help` messages for more information.

