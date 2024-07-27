# Bereal Data Transformer

Exports data from BeReal dump (obtained by contacting BeReal support) to other
formats. Heavily WIP (command exports data in a specified format and fails).

The utility also allows grouping and filtering of Memories (posts). Grouping
creates subfolders for years, months or days. Time-based filtering is possible
as well as posts' caption text-based filtering (regex).

## Build

    cargo b

## Run

Only "working" subcommands are listed. Again, the program now panics at the end of execution + is heavily WIP.

### Memories export

    cargo r -- memories --help

Example:

    cargo r -- --input ./my/input/data  --output ./out memories --image-format jpeg --group day-flat