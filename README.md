# pbtools
PocketBook Ereader Tools

This is a small set of tools for users of PocketBook ereaders. In particular they are
meant for working with PocketBook theme files, which have a ".pbt" extension.

The tools are written in rust for portability and safety.

* `rpbres` - this is (yet another) reimplementation of the `pbres` tool from PocketBook,
  which currently allows you to list the contents of a theme file, and unpack (extract)
  a resource;
* `res2image` - this is a small tool to convert a bitmap resource from a theme into a
  more useful image format, such as PNG;
* `image2res` - convert a PNG (or BMP or TIFF) back into an image resource.

All tools have `-h/--help` and `-V/--version` options. 

Currently `rpbres` has `-l/list` and `-u/unpack` subcommands.

The output of `rpbres -l` (list) is slightly different from the other tools, in that it
tries to guess the format of each resource. For example:

```
$ rpbpres -l "../InkPad Color 3/Line.pbt"
resource                                                 size  compressed size  verbose
---------------------------------------------------------------------------------------------------------
                                                       193900            31671  Configuration
AppStore:4                                              23416             2898  Bitmap 301 x 77 8bpp
CardLogo:4                                             248460            75326  Bitmap 238 x 347 24bpp
GooglePlay:4                                            25288             3020  Bitmap 318 x 79 8bpp
about:4                                                 15136              669  Bitmap 122 x 122 8bpp
activate_account_on_eink:4                              61264              705  Bitmap 247 x 247 8bpp
activate_account_on_smartphone:4                        61264             1301  Bitmap 247 x 247 8bpp
add_to_cloud:4                                          15136             1158  Bitmap 122 x 122 8bpp
adjustments:4                                           15136              168  Bitmap 122 x 122 8bpp
adjustments_inv:4                                       15136              801  Bitmap 122 x 122 8bpp
adobe_activation_layout:4                               11342             1170  JSON?
app_viewer_layout:4                                       915              383  JSON?
archive:4                                               15136              495  Bitmap 122 x 122 8bpp
archive_inv:4                                           15136             1212  Bitmap 122 x 122 8bpp
arrow_down:4                                              560              108  Bitmap 23 x 23 8bpp
arrow_down_inv:4                                          560              110  Bitmap 24 x 23 8bpp
arrow_left:4                                              560               85  Bitmap 23 x 23 8bpp
arrow_left_wizard:4                                      1040              216  Bitmap 24 x 43 8bpp
arrow_left_wizard_disabled:4                             1040              207  Bitmap 24 x 43 8bpp
arrow_page_next:4                                         863              339  Bitmap 39 x 19 8bpp *
arrow_page_prev:4                                         863              343  Bitmap 39 x 19 8bpp *
[...]
```

Note the trailing `*` on some images indicates the high bit is set on the bit depth field, which
may indicate image transparency.

`res2image` is very simple. Once you've unpacked a resource from a theme file (eg
`about:4`) then you'll want to convert it to something useful.

```bash
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

```bash
$ rpbres -u "../InkPad Color 3/Line.pbt" ""
$ file theme.cfg
theme.cfg: ASCII text
```

The `image2res` tool converts any normal image file into a 24bpp resource file. The output filename
is the same as the original image but without any extension.

```bash
$ image2res /tmp/example.png
$ ls -al /tmp/example*
-rw-r--r--@ 1 cjr  wheel    87604 16 Jul 07:27 /tmp/example.png
-rw-r--r--  1 cjr  wheel  7884872 16 Jul 07:29 /tmp/example
```
