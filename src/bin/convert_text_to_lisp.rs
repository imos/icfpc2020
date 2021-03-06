use std::io::*;

use app::*;

fn main() {
	let stdin = std::io::stdin();
	let stdin = stdin.lock();
	for line in stdin.lines() {
		let line = line.unwrap();
		let ss = line.split_whitespace().collect::<Vec<_>>();
		let (exp, n) = parser::parse(&ss, 0);
		assert_eq!(n, ss.len());
		println!("{}", exp);
	}
}
