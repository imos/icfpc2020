use std::io::prelude::*;

use app::parser::*;
use app::sender::*;
use app;
use std::rc::Rc;
use std;
use structopt::StructOpt;
use std::env;

use app::*;

#[derive(structopt::StructOpt, Debug)]
struct Args {
	#[structopt(long, default_value = "")]
	init_state: String,
	#[structopt(long, default_value = "")]
	input_file: String,
	#[structopt(long)]
	recognize: bool,
	#[structopt(long)]
	performance_test: bool,
}

fn prepare_init_state(args: &Args) -> E {
	if args.init_state.is_empty() {
		parser::parse(&["nil"], 0).0
	} else {
		let mut init_state = std::fs::File::open(&args.init_state).unwrap();
		let mut state = String::new();
		init_state.read_to_string(&mut state).expect("ini_state read error");
		parser::parse_lisp(&state).0
	}
}

fn run() {
	let args = Args::from_args();
	println!("Args: {:?}", &args);
	let recognizer = recognize::Recognizer::new();
	let mut recognition_result = recognize::RecognitionResult::new_empty();
	let mut ev = parser::Evaluator::new(std::fs::File::open("data/galaxy.txt").unwrap());
	// FOR PERFORMANCE TEST.
	let mut expected_requests = vec![
		"11011000011101000",
		"11011000101101111111111111111100010100110100111101100000000110001010100010001110110011101101100110000",
	];
	expected_requests.reverse();
	let mut expected_responses = vec![
		"11011000011111110101101111111111111111100010100110100111101100000000110001010100010001110110011101101100001111011000011101111111111111111100111000101100111111101000101101000111101010001010010110101111011000000",
	];
	expected_responses.reverse();

	let mut state = prepare_init_state(&args);

	let mut stack = vec![];
	let stdin = std::io::stdin();
	let mut stdin: Box<dyn BufRead> = if args.input_file.len() > 0 {
		Box::new(std::io::BufReader::new(std::fs::File::open(args.input_file).unwrap()))
	} else {
		Box::new(stdin.lock())
	};
	let mut current_data = E::Num(0.into());
	for iter in 0.. {
		let (x, y) = if iter == 0 {
			(9999, 9999)
		} else {
			let mut line = String::new();
			if let Ok(_) = std::env::var("GUI") {
				print!("###GUI###\t");
				let list_of_list_of_coords = app::visualize::collect_list_of_list_of_coords(&current_data);
				let ((w, h), offset) = app::draw::range_vv(&list_of_list_of_coords);
				print!("w:{}\th:{}\tx:{}\ty:{}", w, h, offset.0, offset.1);
				println!();
			}
			let bytes = stdin.read_line(&mut line).unwrap();
			if bytes == 0 {
				eprintln!("EOF");
				break;
			}
			let mut line = recognition_result.filter_command(line.trim());
			if line.starts_with("!") {
				stack.push((state.clone(), current_data.clone()));
				state = parser::parse_lisp(&line[1..]).0;
				line = "9999 9999".to_owned();
			}

			let ss = line.trim().split_whitespace().collect::<Vec<_>>();
			if ss.len() == 1 && ss[0] == "undo" {
				let (prev_state, prev_data) = stack.pop().unwrap();
				state = prev_state;
				current_data = prev_data;
				if let Err(_) = env::var("JUDGE_SERVER") {
					app::visualize::multidraw_stacked_from_e_to_file_scale(&current_data, "out/cui.png", 8);
				}
				if let Ok(p) = env::var("IMAGE_OUTPUT") {
					app::visualize::multidraw_stacked_from_e_to_file(&current_data, &p);
				}
				if args.recognize {
					recognition_result = recognizer.recognize(&current_data);
					recognition_result.pretty_print();
				}

				continue;
			} else if ss.len() == 1 && ss[0] == "stat" {
				for i in 1029..1495 {
					eprintln!("{}: {}", i, ev.count[i]);
				}
				continue;
			} else if ss.len() != 2 {
				if let Err(_) = env::var("JUDGE_SERVER") {
					app::visualize::multidraw_stacked_from_e_to_file_scale(&current_data, "out/cui.png", 8);
				}
				if let Ok(p) = env::var("IMAGE_OUTPUT") {
					app::visualize::multidraw_stacked_from_e_to_file(&current_data, &p);
				}
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
			Rc::new(E::Ap(Rc::new(E::Function(1338)), state.clone().into())),
			xy.into(),
		);
		ev.clear_cache();
		let f = ev.eval(&exp, false);
		let f = ev.eval(&f, true);
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
				eprintln!("send(lisp): {}", &data);
				let resp = if args.performance_test {
					let expected = expected_requests.pop().unwrap();
					if expected != modulated {
						panic!("Unexpected input: expected={}, actual={}", expected, modulated);
					}
					match expected_responses.pop() {
						Some(x) => x.to_owned(),
						_ => {
							println!("Successfully, the response stack has become empty.");
							std::process::exit(0);
						},
					}
				} else {
					send(&modulated)
				};
				eprintln!("resp: {}", &resp[0..resp.len().min(50)]);
				let resp = app::modulation::demodulate(&resp);
				eprintln!("resp(lisp): {}", &resp);
				let exp = E::Ap(
					Rc::new(E::Ap(Rc::new(E::Function(1338)), state.clone().into())),
					resp.into(),
				);
				ev.clear_cache();
				let f = ev.eval(&exp, false);
				let f = ev.eval(&f, true);
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
			if let Err(_) = env::var("JUDGE_SERVER") {
				app::visualize::multidraw_stacked_from_e_to_file_scale(&data, "out/cui.png", 8);
			}
			if let Ok(p) = env::var("IMAGE_OUTPUT") {
				app::visualize::multidraw_stacked_from_e_to_file(&data, &p);
			}
			if args.recognize {
				recognition_result = recognizer.recognize(&current_data);
				recognition_result.pretty_print();
			}
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
