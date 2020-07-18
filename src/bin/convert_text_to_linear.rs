use std::io::*;

use app::*;
use modulation::modulate;

fn main() {
	let stdin = std::io::stdin();
	let stdin = stdin.lock();
	let functions = std::collections::BTreeMap::new();
	for line in stdin.lines() {
		let line = line.unwrap();
		let ss = line.split_whitespace().collect::<Vec<_>>();
		let (exp, n) = parser::parse(&ss, 0);
		assert_eq!(n, ss.len());
		let e = parser::eval(&exp, &functions, true, &mut app::parser::Data::default());
		let result = modulate(&e);
		println!("{}", result);
	}
}
