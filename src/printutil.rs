macro_rules! fprint {
	($($arg:tt)*) => {{
		use std::io::Write;
		print!($($arg)*);
		::std::io::stdout().flush().ok();
	}};
}

macro_rules! fprintln {
	($($arg:tt)*) => {{
		use std::io::Write;
		println!($($arg)*);
		::std::io::stdout().flush().ok();
	}};
}
