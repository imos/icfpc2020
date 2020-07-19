use std::io::prelude::*;

use app::parser::*;
use app::sender::*;
use std::rc::Rc;
use structopt::StructOpt;

use app::*;

#[derive(structopt::StructOpt, Debug)]
struct Args {
	#[structopt(long, default_value = "")]
	init_state: String,
}

fn prepare_init_state(args: Args) -> E {
	if args.init_state.is_empty() {
		parser::parse(&["nil"], 0).0
	} else {
		let mut init_state = std::fs::File::open(args.init_state).unwrap();
		let mut state = String::new();
		init_state.read_to_string(&mut state).expect("ini_state read error");
		parser::parse_lisp(&state).0
	}
}

fn run() {
	let args = Args::from_args();
	println!("Args: {:?}", &args);

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

	let mut state = prepare_init_state(args);

	let mut stack = vec![];
	let stdin = std::io::stdin();
	let mut stdin = stdin.lock();
	let mut current_data = E::Num(0.into());
	for iter in 0.. {
		let (x, y) = if iter == 0 {
			(9999, 9999)
		} else {
			let mut line = String::new();
			let _ = stdin.read_line(&mut line).unwrap();
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
		let xy = E::Pair(Rc::new(E::Num(x.into())), Rc::new(E::Num(y.into())));
		let exp = E::Ap(
			Rc::new(E::Ap(Rc::new(E::Etc(":1338".to_owned())), state.clone().into())),
			xy.into(),
		);
		let mut data = app::parser::Data::default();
		let f = eval(&exp, &functions, false, &mut data);
		let f = eval(&f, &functions, true, &mut data);
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
		if flag || state != new_state || iter == 0 {
			stack.push((state.clone(), current_data.clone()));
			state = new_state;
			current_data = data.clone();
			eprintln!("flag = {}", flag);
			eprintln!("state: {}", state);
			while flag {
				let modulated = app::modulation::modulate(&data);
				eprintln!("send: {}", &modulated);
				let resp = send(&modulated);
				eprintln!("resp: {}", &resp[0..resp.len().min(50)]);
				let resp = app::modulation::demodulate(&resp);
				let exp = E::Ap(
					Rc::new(E::Ap(Rc::new(E::Etc(":1338".to_owned())), state.clone().into())),
					resp.into(),
				);
				let mut parser_data = app::parser::Data::default();
				let f = eval(&exp, &functions, false, &mut parser_data);
				let f = eval(&f, &functions, true, &mut parser_data);
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
