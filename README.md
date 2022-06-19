# unsubscan

A tool to help you find unsubscribe links in your emails

## About

I created `unsubscan` because I think that anyone should be able to quickly and easily look at their emails and:

- Unsubscribe from whatever they want
- Unsubscribe whenever they want
- Unsubscribe _for free_
- Unsubscribe without _yet another subscription service_
- Unsubscribe without having to give another company access to their emails
- Unsubscribe without having to forward emails to other companies

## Installation

Pre-compiled binaries of the latest release will be made available on the Releases page of this repository.

If you are using Windows, it is strongly recommended to install using the [Scoop package manager](https://scoop.sh) to
avoid issues with Windows
Defender [mistakenly](https://www.virustotal.com/gui/file/fb00e0c43336787eeb12151e35bbddf6f0190eded4431bfdcf7776fbda806fdf)
blocking the zip file that is made available for download on the Releases page.

```powershell
scoop bucket add extras
scoop install unsubscan
```

Alternatively, you may also compile this project from source if you have a working Rust development environment:

```
git clone https://github.com/LGUG2Z/unsubscan.git
cd unsubscan
cargo install --path .
```

## Preparing your emails

### Option 1: Get an archive of your emails from your provider as `.eml` files

Export your emails as an archive of `.eml` files from your provider. Instructions on how to do this with different
email providers are below:

- [FastMail](https://www.fastmail.help/hc/en-us/articles/360060590573-Download-all-your-data#transfermail)
- [Proton Mail](https://proton.me/support/export-emails-import-export-app)
- [ZohoMail](https://www.zoho.com/mail/help/import-export-emails.html#alink2)

Once downloaded, extract the archive of emails to a new folder.

### Option 2: Save your emails as `.eml` files using Thunderbird

If you host your email with a provider that does not provide exported archives of your emails as `.eml` files (e.g.
Gmail), you can use Thunderbird (or any desktop email client that allows you to save your emails as `.eml` files) to do
this yourself.

Select the desired subset of emails that you wish to scan (or all of them, if you'd like), and then
select `File -> Save As -> File` from the menu bar to save them to a new folder.

## How it works

If you are running on a system that allows you to drag a folder directly onto an application to use that folder as an
input (e.g. dragging and dropping a folder from Explorer onto an `exe` file on Windows), then all you have to drag is drag
your folder of emails on top of the `unsubscan` executable.

If you are more comfortable on the command line, you can also call the binary with the path to your extracted folder of
`.eml` files as the sole argument.

```
unsubscan 0.1.0
A tool to help you find unsubscribe links in your emails

USAGE:
    unsubscan [OPTIONS] <DIRECTORY>

ARGS:
    <DIRECTORY>    Directory of EML files to scan for unsubscribe links

OPTIONS:
        --debug              Enable debug logging
    -h, --help               Print help information
    -o, --output <OUTPUT>    The format in which to output scanned unsubscribe links [default: html]
                             [possible values: html, json]
    -V, --version            Print version information

```

The folder will be scanned for unsubscribe links and when the scanning is complete, an HTML page will open in your default
browser with a complete list of all the links found and further instructions and explanations.

If you are running `unsubscan` from the command line, you may also optionally receive the output in JSON format.

## Contributing

Please feel free to open a PR with links explaining how to export emails as `.eml` files with other email providers.

Please also feel free to make the binary releases available for installation via package managers on other operating
systems.

I am not interested in adding MBOX support myself, but I will be happy to review and eventually accept a PR that adds
this functionality.
