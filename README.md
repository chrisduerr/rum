# RUM - Rust Userstyle Manager

### Purpose of RUM

I created RUM because WebExtension Userstyle managers do not have direct access to the filesystem anymore and are not able to change the Browser's UI. This might be disired because having files makes it easy to replace and manage your styles with standard *NIX tools. So I had to create a way to convinently manage my userChrome and userContent with one tool.

RUM makes it possible to manage styles from userstyles.org, your filesystem or any URL without having to leave the CLI.

### Usage Example

#### Adding a Style from userstyles.org

To add a Style from userstyles.org, you need to get the ID first. If the URL of the style is `https://userstyles.org/styles/37035/github-dark`, the ID is `37035`.

```
$ rum add 37035
Adding '37035':

[color] Base color scheme:
    (0) #4183C4
    (1) Custom
# Select the ID of your option here, or just enter nothing for the Default
[Default 0] > 1
# If you select "Custom" you can now enter any String
[custom] > #ff00ff

# Repeat this selection process for every option of the style
...

Added style '37035'

Added all styles!
```

#### Adding a Style from a local file or URL

Adding a Style from a local file or URL is a bit different because there are no settings but you need to provide some other information.
```
# Using a path requires the full path, not a relative one
# Using a link requires a valid URL
$ rum add ~/MyStyles/CoolStyle.css
Adding '/home/rumuser/MyStyles/CoolStyle.css':
Please select a name for this style:
# You are free to choose any name, this just is for convinience when updating or removing a style
 > Cool Style
Do you want to add a domain?
# Every style in Firefox that is not global needs to have a domain associated to it
# If your style does not have a "@-moz-document" annotation you probably want to add a domain
[y/N] > Y
Please select a target domain:
Example: 'domain("kernel.org")'
# The example already shows a simple domain annotation option that works in most cases
# For more information read ![this](https://developer.mozilla.org/en-US/docs/Web/CSS/@document)
 > domain("coolstyles.com")
Added style '/home/rumuser/MyStyles/CoolStyle.css'

Added all styles!
```

#### UserChrome

By default `rum add` adds styles to the userContent.css, which does not work for modifying the Browser's UI. If you wish to add a style that applies to the Browser UI, you need to add the `--chrome` flag. Example: `rum -c ~/UIStyle.css`.

#### Other management tools

If you want to find out what RUM can do beyond adding styles, you can read up on it using `rum --help` or `rum <subbcommand> --help` (Example: `rum add --help`).
Here is the documentation of the basic RUM commands:

```
RUM - Rust Userstyle Manager 0.1.0
Christian Dürr <contact@christianduerr.com>
A userstyle manager for Firefox that uses the userContent.css

USAGE:
    rum [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add       Add a new Style
    help      Prints this message or the help of the given subcommand(s)
    list      List all installed styles
    remove    Remove a style
    update    Update styles
```

#### Issues and Support

If RUM is not working the way you would expect it to work, or you have any other problem with it, please feel free to create an issue on github.
