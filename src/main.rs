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
use tokio::sync::Mutex;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const FFMPEG: &[u8] = include_bytes!("../ffmpeg-win32.exe");

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
    use std::io::Write;
    file.write_all(err_msg.as_bytes()).ok();
}

fn create_ffmpeg() -> Result<String> {
    const FFMPEG_PATH: &str = "ffmpeg.exe";
    if std::fs::try_exists(FFMPEG_PATH)? {
        return Ok(FFMPEG_PATH.to_owned());
    };
    let dir = std::env::temp_dir().join(FFMPEG_PATH);
    let dir_str = dir.to_string_lossy().into_owned();
    std::fs::write(&dir_str, FFMPEG)?;
    Ok(dir_str)
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
    print_header();
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

async fn download(args: Vec<String>, passthrough: Passthrough) -> command_handler::Result<()> {
    let saver = match passthrough {
        Some(x) => Arc::downcast::<Mutex<Saver>>(x)
            .map_err(|_| "Invalid passthrough (internal error)".to_owned())?,
        None => return Err("Invalid passthrough (internal error)".into()),
    };

    let mut arg_iter = args.into_iter();
    let url = arg_iter.next().ok_or("URL as first argument required.")?;
    let format = arg_iter
        .next()
        .map(|x| dr_downloader::format::Format::new(x));

    let saver = saver.lock().await;
    saver
        .save(url, "./", format)
        .await
        .map_err(|e| e.to_string())?;

    fprintln!("\x1B[92mDone!\x1B[0m");
    Ok(())
}

async fn token(args: Vec<String>, _passthrough: Passthrough) -> command_handler::Result<()> {
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

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(all(windows, not(debug_assertions)))]
    win32::set_virtual_console_mode();

    let saver = setup_saver().await?;
    let (input_mode, inp, mut input_buffer) = setup_input().await?;

    let mut cmds = AsyncCommandHandler::new();
    cmds.register("clear", |_, _| Box::pin(clear()));
    cmds.register("download", |x, y| Box::pin(download(x.to_owned(), y)));
    cmds.register("token", |x, y| Box::pin(token(x, y)));
    cmds.register("version", |x, _| Box::pin(version(x)));

    let shared_saver = Arc::new(Mutex::new(saver));

    if input_mode {
        print_header();
    }
    do_while!((input_mode) {
        if input_mode {
            input_buffer.clear();
            inp.read_line(&mut input_buffer)?;
        }

        let result = cmds.handle(&input_buffer, Some(shared_saver.clone())).await;
        if let Err(val) = result {
            log_error(val.to_string());
        }
    });
    Ok(())
}
