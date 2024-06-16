# pbtools
PocketBook Ereader Tools

This is a small set of tools for users of PocketBook ereaders. In particular they are
meant for working with PocketBook "theme" files, which have a ".pbt" extension.

The tools are written in rust for portability and safety.

* `rpbres` - this is (yet another) reimplementation of the `pbres` tool from PocketBook,
  which currently just allows you to list the contents of a theme file;
* `res2image` - this is a small tool to convert a bitmap resource from a theme into a
  more useful image format, such as PNG.

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

Currently `rpbres` only has a working `-l/list` subcommand, but all the code is there to
implement `-u/unpack` as well.

`res2image` is very simple. Once you've unpacked a resource from a theme file (eg
`about:4`) then you'll want to convert it to something useful. Currently you'll need to
unpack the resource using another tool, such as `pbres` from @Enyby.

```
$ res2image --png about:4
```

You will end up with a file called `about:4.png`.
