# cbzMaker

Create `.cbz` (Comic Book ZIP) archives from images.
Mainly created for the [Tachiyomi manga reader](https://github.com/tachiyomiorg/tachiyomi), to not have thousands of single images flying around the system (and maybe save some space?).


## Fair waring
Running this **WILL OVERWRITE** existing files (if names match)


## Input directory structure
`cbzMaker/src/<Comic Name>/<Chapter Name>/<Pages>`
- pages should be ordered by name


## Output directory structure
`cbzMaker/out/<Comic Name>/<Chapter Name>.cbz`
- an additional `.nomedia` file is created next to the chapter archives, so that gallery apps ignore potentially created thumbnail/cover images.
