# dr-downloader-cli

[![Rust](https://github.com/F0903/dr-downloader/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/F0903/dr-downloader/actions/workflows/rust.yml)

A downloader that downloads media from DR (Danish Broadcasting Corporation) in parallel, and converts them to convenient MP4 files.

To use, simply paste in an episode or season URL from DRTV.

If you'd rather use it with command-line arguments (for example, from another program), it is possible to pass in the URL as would with any other program from the command line, and the program will gracefully shut down once the download is done.

Release binaries are provided through the build action, and can be accessed through the Actions menu.

Note:
This program will most likely only work in Denmark, as DR has restrictions on who can access their content.
Use of this program assumes you have the right to download the relevant media.
