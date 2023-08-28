# dr-downloader-cli

[![Rust](https://github.com/F0903/dr-downloader/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/F0903/dr-downloader/actions/workflows/rust.yml)
[![Crates.io](https://shields.io/crates/v/dr-downloader.svg)](https://crates.io/crates/dr-downloader)

A command-line program that downloads media from DR-TV (Danish Broadcasting Corporation) in parallel, and converts them to MP4 files (or any other format you'd like).
Can be used both interactively by launching it normally, or by launching the program with arguments from a terminal.

Release binaries are provided through the build actions, or the releases.

To use as a library, [use the dedicated library instead.](https://github.com/F0903/dr-downloader)

Note:
This program will most likely only work in Denmark, as DR has restrictions on who can access their content.
Use of this program assumes you have the right to download the relevant media.

## Setup

**Before downloading, you will need an authentication token as explained below.**

- First, go to the DR-TV video player.
- Press F12. This should open the developer window on the right of the page.
- Go to the Network tab and press CTRL+R. This will reload the page. You should now see a lot of requests in the window.
- Find the request whose name starts with "account?ff="
- Then scroll down on the right "Headers" section of the request, and find the header called "X-Authorization".
- Copy the value of this header EXCEPT the "Bearer" part. Make sure the token has no spaces or newline characters.
- Start the program, type "token set ", paste your token, and press enter.

You should now be able to download any episode or show.

## Commands

Syntax: **command-name** _required-param_ _(optional-param)_

**download** _url_ _(format)_ -> Downloads media.  
**token** get -> Prints current token.  
**token** set _token_ -> Sets current token.  
**clear** -> Clears terminal.
**version** -> Prints version.
