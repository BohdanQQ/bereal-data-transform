# Bereal Data Transformer

Exports data from BeReal dump (obtained by contacting BeReal support) to other
formats. WIP.

The utility also allows grouping and filtering of Memories (posts). Grouping
creates subfolders for years, months or days. Time-based filtering is possible
as well as posts' caption text-based filtering (regex).

## Running

As input folder, specify your *unzipped* BeReal dump.

### Memories export

    cargo r --release -- memories --help

Example:

    cargo r --release -- --input ./my/input/data  --output ./out-mem memories --image-format jpeg --group day-flat

### Realmojis export

    cargo r --release -- realmojis --help

Example:

    cargo r --release -- --input ./my/input/data  --output ./out-moji realmojis --group emoji

Check out subcommands' respective `--help` messages for more information.

