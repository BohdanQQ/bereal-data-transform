# Bereal Data Transformer

Exports data from a BeReal dump (obtained by [contacting BeReal support](#requesting-your-data)) to other
formats. The tool also inserts metadata to the exported post images (a.k.a. memories), e.g. description and original creation time.

The utility also allows grouping and filtering of Memories (posts). Grouping
creates sub-folders for years, months or days. Time-based filtering is possible
as well as posts' caption text-based filtering (regex).

## (no-guarantee, non-exhaustive) WIP features
* realmoji metadata (emoji, creation time)
* BTS video metadata (creation time)

## Running

As input folder, specify your *unzipped* BeReal dump.

I use `cargo`. This will both build and run the program. In case you have a binary of this program at hand, replace `cargo r --release --` with the path to your binary.

### Memories export

    cargo r --release -- memories --help

Example:

    cargo r --release -- --input ./my/input/data  --output ./out-mem memories --image-format jpeg --group day-flat

### Realmojis export

    cargo r --release -- realmojis --help

Example:

    cargo r --release -- --input ./my/input/data  --output ./out-moji realmojis --group emoji

Check out subcommands' respective `--help` messages for more information.

## Troubleshooting

Due to the usage of a new and rapidly developing module for writing metadata into the exported files, you may experience crashes. If you're okay with having no metadata attached to the image,
use the `--no-meta` flag. The export should be successful.

# Requesting your data

For completeness, here is the support request that gets you a link to your data (remember to fill `<your username>`, credit a lost nickname of a Reddit person IIRC):

        To Whom It May Concern: In accordance with Art. 15(3) GDPR, provide me with a copy of all personal data concerning me that you are processing, including any potential pseudonymised data on me as per Article 4(5) GDPR. Please make the personal data concerning me, which I have provided to you, available to me in a structured, commonly used and machine-readable format as laid down in Article 20(1) GDPR. I include the following information necessary to identify me: username: <your username>. Thanks, <your username>