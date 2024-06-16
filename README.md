# pbtools
PocketBook Ereader Tools

This is a small set of tools for users of PocketBook ereaders. In particular they are
meant for working with PocketBook theme files, which have a ".pbt" extension.

The tools are written in rust for portability and safety.

* `rpbres` - this is (yet another) reimplementation of the `pbres` tool from PocketBook,
  which currently allows you to list the contents of a theme file, and unpack (extract)
  a resource;
* `res2image` - this is a small tool to convert a bitmap resource from a theme into a
  more useful image format, such as PNG.

Both tools have `-h/--help` and `-V/--version` options. 

Currently `rpbres` has `-l/list` and `-u/unpack` subcommands.

The output of `rpbres -l` (list) is slightly different from the other tools, in that it
tries to guess the format of each resource. For example:

```
$ rpbpres -l "../InkPad Color 3/Line.pbt"
resource                                                    size  compressed  verbose
-----------------------------------------------------------------------------------------------------------
                                                          193900       31671  Configuration
AppStore:4                                                 23416        2898  Bitmap 301 x 77 8bpp
CardLogo:4                                                248460       75326  Bitmap 238 x 347 24bpp
GooglePlay:4                                               25288        3020  Bitmap 318 x 79 8bpp
about:4                                                    15136         669  Bitmap 122 x 122 8bpp
activate_account_on_eink:4                                 61264         705  Bitmap 247 x 247 8bpp
activate_account_on_smartphone:4                           61264        1301  Bitmap 247 x 247 8bpp
add_to_cloud:4                                             15136        1158  Bitmap 122 x 122 8bpp
adjustments:4                                              15136         168  Bitmap 122 x 122 8bpp
adjustments_inv:4                                          15136         801  Bitmap 122 x 122 8bpp
adobe_activation_layout:4                                  11342        1170  JSON?
[...]
```

`res2image` is very simple. Once you've unpacked a resource from a theme file (eg
`about:4`) then you'll want to convert it to something useful.

```
$ rpbres -u "../InkPad Color 3/Line.pbt" about:4
$ file about:4
about:4: data
$ res2image --png about:4
$ file about:4.png
about:4.png: PNG image data, 122 x 122, 8-bit/color RGB, non-interlaced
```

You will end up with a file called `about:4.png`. Mac users will notice that the Finder
and other GUI applications will display the filename with a slash instead of a colon, ie
`about/4.png`. This is nothing to worry about.

The theme configuration file (usually the first file in the `-l` output) has an empty name.
Unpacking will save it into a file called `theme.cfg`.

```
$ rpbres -u "../InkPad Color 3/Line.pbt" ""
$ file theme.cfg
theme.cfg: ASCII text
```