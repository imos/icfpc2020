use app::*;
use draw::*;
use modulation::demodulate;
use std::io::*;

// Usage: draw output.png <<< 111101001000
fn main() {
	let args: Vec<_> = std::env::args().collect();
	if args.len() < 2 {
		eprintln!("1 arg required");
		std::process::exit(1);
	}
	let output = &args[1];
	let stdin = std::io::stdin();
	let stdin = stdin.lock();
	for line in stdin.lines() {
		let line = line.unwrap();
		let result = demodulate(&line);
		let img = {
			let vv = translate_to_vecvec(&result);
			if vv.len() != 0 {
				multidraw(&vv)
			} else {
				let v = translate_to_vec(&result);
				draw(&v)
			}
		};
		img.save(output).unwrap();
	}
}
