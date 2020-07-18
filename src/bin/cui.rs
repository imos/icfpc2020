use std::io::prelude::*;

use app::parser::*;
use app::sender::*;
use std::rc::Rc;

type E = app::parser::E;
fn eval1(exp: &E, functions: &::std::collections::BTreeMap<String, E>) -> E {
	let now = ::std::time::Instant::now();
	let mut data = app::parser::Data::default();
	let f = eval(&exp, &functions, true, &mut data);
	let dur = now.elapsed();
	eprintln!("{:.6} sec", dur.as_secs_f64());
	f
}

fn run() {
	let f = std::fs::File::open("data/galaxy.txt").unwrap();
	let f = std::io::BufReader::new(f);
	let mut functions = std::collections::BTreeMap::new();
	for line in f.lines() {
		let line = line.unwrap();
		let ss = line.split_whitespace().collect::<Vec<_>>();
		let name = ss[0].to_owned();
		let (exp, n) = parse(&ss[2..], 0);
		assert_eq!(n, ss.len() - 2);
		functions.insert(name, exp);
	}
	let mut state = E::Etc("nil".to_owned());
	let mut stack = vec![];
	let stdin = std::io::stdin();
	let mut stdin = stdin.lock();
	let mut current_data = E::Num(0.into());
	for iter in 0.. {
		let (x, y) = if iter == 0 {
			(0, 0)
		} else {
			let mut line = String::new();
			let bytes = stdin.read_line(&mut line).unwrap();
			if bytes == 0 {
				eprintln!("EOF");
				return;
			}
			let ss = line.trim().split_whitespace().collect::<Vec<_>>();
			if ss.len() == 1 && ss[0] == "undo" {
				let (prev_state, prev_data) = stack.pop().unwrap();
				state = prev_state;
				current_data = prev_data;
				app::visualize::multidraw_stacked_from_e_to_file_scale(&current_data, "out/cui.png", 8);
				continue;
			} else if ss.len() != 2 {
				eprintln!("illegal input");
				continue;
			} else if let (Ok(x), Ok(y)) = (ss[0].parse(), ss[1].parse()) {
				(x, y)
			} else {
				eprintln!("illegal input");
				continue;
			}
		};
		let s = format!("ap ap cons {} {}", x, y);
		let xy = parse(&s.split_whitespace().collect::<Vec<_>>(), 0).0;
		let exp = E::Ap(
			Rc::new(E::Ap(Rc::new(E::Etc(":1338".to_owned())), state.clone().into())),
			xy.into(),
		);
		let f = eval1(&exp, &functions);
		let (mut flag, new_state, mut data) = if let E::Pair(flag, a) = f {
			if let E::Pair(a, b) = a.as_ref() {
				if let E::Pair(data, _) = b.as_ref() {
					(flag.as_ref() != &E::Num(0.into()), a.as_ref().clone(), data.as_ref().clone())
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		};
		if flag || state != new_state {
			stack.push((state.clone(), current_data.clone()));
			state = new_state;
			current_data = data.clone();
			eprintln!("flag = {}", flag);
			eprintln!("state: {}", state);
			while flag {
				eprintln!("send: {}", app::modulation::modulate(&data));
				let resp = send(&app::modulation::modulate(&data));
				eprintln!("resp: {}", &resp[0..resp.len().min(50)]);
				let resp = app::modulation::demodulate(&resp);
				let exp = E::Ap(
					Rc::new(E::Ap(Rc::new(E::Etc(":1338".to_owned())), state.clone().into())),
					resp.into(),
				);
				let f = eval1(&exp, &functions);
				let (new_flag, new_state, new_data) = if let E::Pair(flag, a) = f {
					if let E::Pair(a, b) = a.as_ref() {
						if let E::Pair(data, _) = b.as_ref() {
							(flag.as_ref() != &E::Num(0.into()), a.as_ref().clone(), data.as_ref().clone())
						} else {
							panic!();
						}
					} else {
						panic!();
					}
				} else {
					panic!();
				};
				flag = new_flag;
				state = new_state;
				data = new_data;
				current_data = data.clone();
				eprintln!("flag = {}", flag);
				eprintln!("state: {}", state);
			}
			app::visualize::multidraw_stacked_from_e_to_file_scale(&data, "out/cui.png", 8);
		} else {
			eprintln!("orz");
		}
	}
}

fn main() {
	let _ = ::std::thread::Builder::new()
		.name("run".to_string())
		.stack_size(32 * 1024 * 1024)
		.spawn(run)
		.unwrap()
		.join();
}
