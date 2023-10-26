#![feature(fs_try_exists)]
#![feature(iter_intersperse)]

#[cfg(all(windows, not(debug_assertions)))]
mod win32;

mod command_handler;
mod marco_utils;

#[macro_use]
mod printutil;

use crate::command_handler::AsyncCommandHandler;
use command_handler::Passthrough;
use dr_downloader::{
    cacher::{get_token, set_token},
    converter::Converter,
    downloader::Downloader,
    error::Result,
    saver::Saver,
};
use marco_utils::do_while;
use std::{
    io::{stdin, Stdin, Write},
    sync::Arc,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn log_error(err: impl AsRef<str>) {
    let err = err.as_ref();
    fprintln!("\x1B[91mError!\x1B[0m {}", err);
    let err_msg = format!("{:?}", err);
    let mut file = if let Ok(f) =
        std::fs::File::open("error.txt").or_else(|_| std::fs::File::create("error.txt"))
    {
        f
    } else {
        return;
    };
    file.write_all(err_msg.as_bytes()).ok();
}

fn create_ffmpeg() -> Result<String> {
    const FFMPEG_PATH: &str = "ffmpeg.exe";
    return Ok(FFMPEG_PATH.to_owned());
    /* if std::fs::try_exists(FFMPEG_PATH)? {

    };
    return Err("FFmpeg not found! Please install FFmpeg and add to PATH or put the executable in the downloader root.".into()); */
}

async fn setup_downloader() -> Result<Downloader<'static>> {
    let mut downloader = Downloader::default_async().await?;
    downloader
        .download_event
        .sub(&|x| fprintln!("Downloading {}...", x));
    downloader
        .finished_event
        .sub(&|x| fprintln!("Finished downloading {}", x));
    downloader
        .failed_event
        .sub(&|x| fprintln!("Failed downloading {}", x));
    Ok(downloader)
}

async fn setup_converter() -> Result<Converter<'static>> {
    let mut converter = Converter::new(create_ffmpeg()?);
    converter
        .on_convert
        .sub(&|x| fprintln!("Converting {}...", x));
    converter
        .on_done
        .sub(&|x| fprintln!("Finished converting {}", x));
    Ok(converter)
}

async fn setup_saver() -> Result<Saver<'static>> {
    let downloader = setup_downloader().await?;
    let converter = setup_converter().await?;
    let saver = Saver::new(downloader).with_converter(converter);
    Ok(saver)
}

async fn setup_input() -> Result<(bool, Stdin, String)> {
    let args = std::env::args();
    let input_mode = args.len() <= 1;
    let inp = stdin();
    let input_buffer = if input_mode {
        String::new()
    } else {
        args.skip(1)
            .intersperse(String::from_utf8_lossy(&[b' ']).into_owned())
            .collect()
    };
    Ok((input_mode, inp, input_buffer))
}

fn print_header() {
    fprint!(
        "\x1B[1m\x1B[47m\x1B[31m DR \x1B[49m\x1B[39m Downloader CLI\x1B[0m v{}\n\n",
        VERSION
    );
}

async fn clear() -> command_handler::Result<()> {
    std::io::stdout().flush()?;
    fprint!("\x1B[2J\x1B[1;1H");
    Ok(())
}

async fn version(args: Vec<String>) -> command_handler::Result<()> {
    let mut arg_iter = args.iter();
    if arg_iter.any(|x| x == "no-newline") {
        fprint!("{}", VERSION);
    } else {
        fprintln!("{}", VERSION);
    }
    Ok(())
}

async fn download<'a>(args: Vec<String>, saver: Passthrough) -> command_handler::Result<()> {
    let mut arg_iter = args.into_iter();
    let url = arg_iter.next().ok_or("URL as first argument required.")?;
    let format = arg_iter
        .next()
        .map(|x| dr_downloader::format::Format::new(x));

    saver
        .save(url, "./", format)
        .await
        .map_err(|e| e.to_string())?;

    fprintln!("\x1B[92mDone!\x1B[0m");
    Ok(())
}

async fn token<'a>(args: Vec<String>, _saver: Passthrough) -> command_handler::Result<()> {
    let mut args_iter = args.iter();
    let subcommand = match args_iter.next() {
        Some(x) => x,
        None => return Err("Expected sub command!".into()),
    };
    match subcommand.as_str() {
        "set" => {
            let value = match args_iter.next() {
                Some(x) => x,
                None => return Err("Expected new value!".into()),
            };
            set_token(value)?;
            fprintln!("Token successfully updated!");
        }
        "get" => {
            let value = get_token()?;
            fprintln!("{}", value);
        }
        _ => return Err("Unknown subcommand.".into()),
    }
    Ok(())
}

macro_rules! async_handler_call {
    ($func_call:expr) => {{
        let val = $func_call;
        std::boxed::Box::pin(val)
    }};
}

#[tokio::main]
async fn main() -> Result<()> {
    //let is_github_ci = std::env::var("GITHUB_ACTIONS").is_ok();

    let (input_mode, inp, mut input_buffer) = setup_input().await?;

    let mut cmds = AsyncCommandHandler::new();
    cmds.register("clear", |_, _| async_handler_call!(clear()));
    cmds.register("download", |x, y| {
        async_handler_call!(download(x.to_owned(), y))
    });
    cmds.register("token", |x, y| async_handler_call!(token(x, y)));
    cmds.register("version", |x, _| async_handler_call!(version(x)));

    let saver = Arc::new(setup_saver().await?);

    if input_mode {
        #[cfg(all(windows, not(debug_assertions)))]
        win32::set_virtual_console_mode();
        print_header();
    }
    do_while!((input_mode) {
        if input_mode {
            input_buffer.clear();
            inp.read_line(&mut input_buffer)?;
        }

        let result = cmds.handle(&input_buffer, saver.clone()).await;

        if let Err(val) = result {
            log_error(val.to_string());
        }
    });
    Ok(())
}
