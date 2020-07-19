use std::io::prelude::*;

use app::parser::*;
use app::sender::*;
use std::rc::Rc;
use rand::prelude::*;
use num::*;

fn run() {
	let stdin = std::io::stdin();
	let stdin = stdin.lock();
	let mut functions = std::collections::BTreeMap::new();
	for line in stdin.lines() {
		let line = line.unwrap();
		let ss = line.split_whitespace().collect::<Vec<_>>();
		let name = ss[0].to_owned();
		let (exp, n) = parse(&ss[2..], 0);
		assert_eq!(n, ss.len() - 2);
		functions.insert(name, exp);
	}
	
	let mut state = E::Nil;
	eprintln!("{}", state);
	let mut rng = rand::thread_rng();
	let mut iter = 0;
	let mut last_data = E::Nil;
	loop {
		let (x, y) = if iter > 15 {
			let es = last_data.into_iter().nth(0).unwrap().into_iter().collect::<Vec<_>>();
			let e = es[rng.gen_range(0, es.len())];
			if let E::Pair(a, b) = e {
				if let (E::Num(a), E::Num(b)) = (a.as_ref(), b.as_ref()) {
					(a.to_i64().unwrap(), b.to_i64().unwrap())
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			(rng.gen_range(-20, 20), rng.gen_range(-20, 20))
		};
		let s = format!("ap ap cons {} {}", x, y);
		let xy = parse(&s.split_whitespace().collect::<Vec<_>>(), 0).0;
		let exp = E::Ap(
			Rc::new(E::Ap(Rc::new(E::Other(":1338".to_owned())), state.clone().into())),
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
		if flag || state != new_state {
			last_data = data.clone();
			state = new_state;
			eprintln!("flag = {}", flag);
			eprintln!("{} {}", x, y);
			eprintln!("state: {}", state);
			if data.to_string().len() >= 30 {
				eprintln!("iter: {}", iter);
				app::visualize::multidraw_stacked_from_e_to_file(&data, &format!("../../ICFPC2020/wata_out/stacked-{}.png", iter));
				iter += 1;
			}
			while flag {
				eprintln!("send: {}", app::modulation::modulate(&data));
				let resp = send(&app::modulation::modulate(&data));
				eprintln!("resp: {}", &resp[0..resp.len().min(50)]);
				let resp = app::modulation::demodulate(&resp);
				let exp = E::Ap(
					Rc::new(E::Ap(Rc::new(E::Other(":1338".to_owned())), state.clone().into())),
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
				last_data = data.clone();
				if data.to_string().len() >= 30 {
					eprintln!("iter: {}", iter);
					app::visualize::multidraw_stacked_from_e_to_file(&data, &format!("../../ICFPC2020/wata_out/stacked-{}.png", iter));
					iter += 1;
				}
				eprintln!("flag = {}", flag);
				eprintln!("state: {}", state);
			}
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
