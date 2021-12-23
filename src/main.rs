#![feature(backtrace)]

#[cfg(all(windows, not(debug_assertions)))]
mod win32;

#[macro_use]
mod printutil;

use dr_downloader::{
	converter::Converter, downloader::Downloader, error::Result, saver::Saver,
};
use std::io::stdin;

const FFMPEG: &[u8] = include_bytes!("../ffmpeg-win32.exe");

macro_rules! do_while {
	(($cond:expr)$body:block) => {
		loop {
			let res = $body;
			if !$cond {
				break res;
			}
		}
	};
}

fn clear_console() {
	fprint!("\x1B[2J\x1B[1;1H");
}

fn log_error(err: impl AsRef<dyn std::error::Error>) {
	fprintln!("\x1B[91mError!\x1B[0m {}", err.as_ref());
	let trace = err.as_ref().backtrace();
	let mut content = match trace {
		Some(val) => val.to_string(),
		None => err.as_ref().to_string(),
	};
	content.push('\n');
	let mut file = if let Ok(f) =
		std::fs::File::open("error.txt").or_else(|_| std::fs::File::create("error.txt"))
	{
		f
	} else {
		return;
	};
	use std::io::Write;
	file.write_all(content.as_bytes()).ok();
}

fn create_ffmpeg() -> Result<String> {
	let dir = std::env::temp_dir().join("ffmpeg.exe");
	let dir_str = dir.to_string_lossy().into_owned();
	std::fs::write(&dir_str, FFMPEG)?;
	Ok(dir_str)
}

#[tokio::main]
async fn main() -> Result<()> {
	#[cfg(all(windows, not(debug_assertions)))]
	win32::set_virtual_console_mode();

	let mut downloader = Downloader::default();
	downloader
		.download_event
		.sub(&|x| fprintln!("Downloading {}...", x));
	downloader
		.finished_event
		.sub(&|x| fprintln!("Finished downloading {}", x));
	downloader
		.failed_event
		.sub(&|x| fprintln!("Failed downloading {}", x));

	let mut converter = Converter::new(create_ffmpeg()?);
	converter
		.on_convert
		.sub(&|x| fprintln!("Converting {}...", x));
	converter
		.on_done
		.sub(&|x| fprintln!("Finished converting {}", x));
	let saver = Saver::new(downloader).with_converter(converter, ".mp4");

	let mut args = std::env::args();
	let input_mode = args.len() <= 1;

	let inp = stdin();
	let mut input_buffer = if input_mode {
		String::new()
	} else {
		args.nth(1).unwrap()
	};

	do_while!((input_mode) {
		if input_mode {
			clear_console();
			fprint!("\x1B[1mEnter url:\x1B[0m ");
			inp.read_line(&mut input_buffer)?;
		}

		let result = saver.save(&input_buffer, "./").await;

		if let Err(val) = result {
			log_error(val);
			const CLEAR_TIME: u16 = 5000;
			fprint!("Clearing in {}s", CLEAR_TIME / 1000);
			std::thread::sleep(std::time::Duration::from_millis(CLEAR_TIME as u64));
			continue;
		}

		fprintln!("\x1B[92mDone!\x1B[0m");
		std::thread::sleep(std::time::Duration::from_millis(2000));
	});
	Ok(())
}
