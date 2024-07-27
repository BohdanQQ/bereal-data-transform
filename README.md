# Bereal Data Transformer

Exports data from BeReal dump (obtained by contacting BeReal support) to other
formats. WIP.

The utility also allows grouping and filtering of Memories (posts). Grouping
creates subfolders for years, months or days. Time-based filtering is possible
as well as posts' caption text-based filtering (regex).

## Build

    cargo b

## Run

Only "working" subcommands are listed.
As input folder, specify your unzipped BeReal dump.

### Memories export

**Disclaimer**: this command runs parallel conversions of images, it **will** eat up all your CPU cores' performance 
for a while!

    cargo r -- memories --help

Example:

    cargo r -- --input ./my/input/data  --output ./out memories --image-format jpeg --group day-flat