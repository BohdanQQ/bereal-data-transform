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

# A few notes on the export layout and WebP images

***disclaimer**: what follows are some quick, unpolished notes on various quirks of the bereal export, they can be incorrect, require redundant/too complex procedures otherwise possible with less effort, ... The approaches presented are quick and dirty one-off solutions and should be approached as such. Due to time constraints, I will not be polishing or handling these anomalies anytime soon in the tool itself, automatically.*

## `realmoji.json` file

The file seems to be a fuse of two files of the same name. Some tools may only extract one of those. If `bereal-data-transform` reports anything containing `isInstant` as an error, your unzipping tool extracted the wrong (part of the) file.

On linux you can do

```bash
unzip -p ./your-archive.zip realmojis.json > ./somewhere/realmojis.json
```

which will yield the entire `realmojis.json` file. The file however, will be a concatenation of 2 JSON arrays (which in itself is not parsable by the JSON parser we use). **For now**, you are free to remove the first array (enclosed in `[` `]`, on multiple lines). You'll know you've done this correctly when you see the contents of the file starting thus (presence of `"isInstant"` field is crucial):

```json
[
    {
        "postId": "xxxx",
        "media": {
            "bucket": "storage.bere.al",
            "height": 500,
            "path": "/Photos/yyyy/realmoji/yyyy-realmoji-happy-nnnn.jpg",
            "width": 500
        },
        "emoji": "😃",
        "isInstant": false,
        "postedAt": "datetime"
    },
```

## Image layout and formats

Bereal dumps differ based on the age of your account. In (Dec 15) 2022, the `Photos/bereal` directory is no longer used to store memories/posts.

Furthermore, the WebP image manipulation libraries we use are not mature enough to handle all
variations of the WebP image format. (long story short: they compose but create broken images, particularly when trying to convert `VP8` webp images into `VP8X` ones). It is possible the libraries will improve but for now, the tool **requires** all `.webp` files to be in the extended file format (`VP8X` chunk).

If you see metadata failures or broken images in the output of the tool when exporting **WebP**, you have to do the following - or similar (requires [`cwebp`](https://developers.google.com/speed/webp/docs/cwebp) and [`webmux`](https://developers.google.com/speed/webp/docs/webpmux)):

### Old dumps with the `bereal` directory

The directory may contain a mix of `jpg` and `webp` images (if not, skip to step 2).

1. convert `.jpg` images to `.webp` images, rough outline:
    * `ls ./*.jpg | xargs -I {} cwebp -q 100 -metadata exif {} -o tmpdir/{})`
    * append the `webp` extension 
    ```bash
    cd tmpdir && ls ./*.jpg | xargs -I {} mv {} `basename {} .jpg`.webp
    # remove the jpg files in bereal directory with the files in the tmpdir directory
    ```
2. convert all `.webp` files to `.webp` files with metadata (we currently need the [`VP8X` chunk](https://developers.google.com/speed/webp/docs/riff_container#extended_file_format))
    * *obtain* metadata via `webpmux -get exif input -o output.exif`
        * ideally the image shall contain only the `Original Date Time` metadata or an empty `Description`  (as this will 100% get re-written)
    * in `bereal` directory, set the sample exif metadata: `ls ./*.webp | xargs -I {} echo {} && webpmux -set exif output.exif  {} -o {}`
    * **do not `webpmux -strip exif` afterwards!**, this results in the incompatible `VP8` chunks (non-extended format)
    * other ways of forcing the presence of the `VP8X` chunk can be found [here](https://developers.google.com/speed/webp/docs/riff_container#extended_file_format)
3. manipulate `memories.json` to point to the newly created files
    * only `memories.json` is used by this tool, make sure you're **not** editing the similar `posts.json`
    * replace occurences of `.jpg"` with `.jpg.webp"`
4. run the tool, e.g. `cargo r --release -- -i ./input-dir -v -o  ./output-dir/  memories --desc-prefix "BeReal Memory: "`

### (iOS?) dumps

Furthermore, some dumps also contain non-extended (`VP8`) `.webp` images in the new `Photos/post` directory. As I have not inspected many exports, the only difference I saw in the exports I had was that it could be connected to an old (with `Photos/bereal` directory) account used on an iPhone.

In the case such files (`VP8`) are present in the `post` directory, the above conversion (step `2`) must also be performed. (should be indicated in the error outputs of `bereal-data-transform`)
