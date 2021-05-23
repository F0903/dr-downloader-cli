#![feature(backtrace)]

#[cfg(all(windows, not(debug_assertions)))]
mod win32;

#[macro_use]
mod printutil;

use dr_downloader::{
	converter::Converter, downloader::Downloader, error::Result, event_subscriber::EventSubscriber,
	requester::Requester,
};
use std::io::stdin;

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
	let content = match trace {
		Some(val) => val.to_string(),
		None => err.as_ref().to_string(),
	};
	std::fs::write("error.txt", content).ok();
}

#[tokio::main]
async fn main() -> Result<()> {
	#[cfg(all(windows, not(debug_assertions)))]
	win32::set_virtual_console_mode();

	let downloader = Downloader::new(Requester::new().await?, Converter::new()?).with_subscriber(
		EventSubscriber::new(
			&|x| fprintln!("Downloading {}", x),
			&|x| fprintln!("Converting {}", x),
			&|x| fprintln!("Finished downloading {}", x),
			&|x| fprintln!("Failed downloading {}", x),
		),
	);

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

		let result = downloader.download("./", &input_buffer).await;

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
