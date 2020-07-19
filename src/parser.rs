use std;
use num::*;
use std::collections::*;
use std::rc::Rc;

pub type Int = i128;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum E {
	Ap(Rc<E>, Rc<E>),
	Num(Int),
	Pair(Rc<E>, Rc<E>),
	Nil,
	T,
	F,
	Cons,
	Other(String),
	Cloned(Rc<E>, usize),
}

pub fn parse(ss: &[&str], i: usize) -> (E, usize) {
	if ss[i] == "ap" {
		let (first, j) = parse(ss, i + 1);
		let (second, k) = parse(ss, j);
		(E::Ap(Rc::new(first), Rc::new(second)), k)
	// } else if ss[i] == "(" {
	// 	let mut list = vec![];
	// 	let mut i = i + 1;
	// 	while ss[i] != ")" {
	// 		let (e, j) = parse(ss, i);
	// 		list.push(e);
	// 		i = j;
	// 		if ss[i] == "," {
	// 			i += 1;
	// 		}
	// 	}
	// 	(E::List(list), i + 1)
	} else if let Ok(a) = ss[i].parse::<Int>() {
		(E::Num(a), i + 1)
	} else {
		(match ss[i] {
			"nil" => E::Nil,
			"t" => E::T,
			"f" => E::F,
			"cons" => E::Cons,
			_ => E::Other(ss[i].to_owned()),
		}, i + 1)
	}
}

impl Into<(Int, Int)> for &E {
	fn into(self) -> (Int, Int) {
		if let E::Pair(x, y) = self {
			if let (E::Num(x), E::Num(y)) = (x.as_ref(), y.as_ref()) {
				return (x.clone(), y.clone());
			}
		}
		panic!("expected coords but got {:?}", self)
	}
}

#[derive(Clone, Default)]
pub struct Data {
	pub count: BTreeMap<String, usize>,
	pub cache: Vec<Option<E>>,
	pub cache2: Vec<Option<E>>,
}

#[deprecated(note="Please use Evaluator.eval")]
pub fn eval(e: &E, map: &BTreeMap<String, E>, eval_tuple: bool, data: &mut Data) -> E {
	let mut ev = Evaluator::default();
	ev.map = map.clone();  // fixme: maybe slow
	ev.data = std::mem::take(data);
	let out = ev.eval(e, eval_tuple);
	*data = ev.data;
	out
}

#[derive(Clone, Default)]
pub struct Evaluator {
	pub map: BTreeMap<String, E>,
	pub data: Data,
}

impl Evaluator {
	pub fn insert_function(&mut self, name: String, exp: E) {
		self.map.insert(name, exp);
	}

pub fn eval(&mut self, e: &E, eval_tuple: bool) -> E {
	match e {
		E::Cloned(a, id) => {
			if !eval_tuple {
				if let Some(ref b) = self.data.cache[*id] {
					b.clone()
				} else {
					let b = self.eval(a.as_ref(), eval_tuple);
					self.data.cache[*id] = Some(b.clone());
					b
				}
			} else {
				if let Some(ref b) = self.data.cache2[*id] {
					b.clone()
				} else {
					let b = self.eval(a.as_ref(), eval_tuple);
					self.data.cache2[*id] = Some(b.clone());
					b
				}
			}
		}
		E::Ap(x1, y1) => {
			let x1 = self.eval(&x1, eval_tuple);
			match &x1 {
				E::Ap(x2, y2) => match x2.as_ref() {
					E::Cons => {
						if eval_tuple {
							E::Pair(
								self.eval(y2, eval_tuple).into(),
								self.eval(y1, eval_tuple).into(),
							)
						} else {
							E::Pair(y2.clone(), y1.clone().into())
						}
					}
					E::Other(name) if name == "eq" => {
						let y1 = self.eval(&y1, eval_tuple);
						let y2 = self.eval(&y2, eval_tuple);
						match (&y1, &y2) {
							(E::Num(y1), E::Num(y2)) => {
								if y1 == y2 {
									E::T
								} else {
									E::F
								}
							}
							_ => panic!("eq with {} and {} is invalid", y2, y1),
						}
					}
					E::T => self.eval(&y2, eval_tuple),
					E::F => self.eval(&y1, eval_tuple),
					E::Other(name) if name == "add" => {
						let y1 = self.eval(&y1, eval_tuple);
						let y2 = self.eval(&y2, eval_tuple);
						match (y1, y2) {
							(E::Num(y1), E::Num(y2)) => E::Num(y1 + y2),
							(y1, y2) => panic!("add with {} and {} is invalid", y2, y1),
						}
					}
					E::Other(name) if name == "mul" => {
						let y1 = self.eval(&y1, eval_tuple);
						let y2 = self.eval(&y2, eval_tuple);
						match (&y1, &y2) {
							(E::Num(y1), E::Num(y2)) => E::Num(y1 * y2),
							_ => panic!("mul with {} and {} is invalid", y2, y1),
						}
					}
					E::Other(name) if name == "div" => {
						let y1 = self.eval(&y1, eval_tuple);
						let y2 = self.eval(&y2, eval_tuple);
						match (&y1, &y2) {
							(E::Num(y1), E::Num(y2)) => E::Num(y2 / y1),
							_ => panic!("div with {} and {} is invalid", y2, y1),
						}
					}
					E::Other(name) if name == "lt" => {
						let y1 = self.eval(&y1, eval_tuple);
						let y2 = self.eval(&y2, eval_tuple);
						match (&y1, &y2) {
							(E::Num(y1), E::Num(y2)) => {
								if y2 < y1 {
									E::T
								} else {
									E::F
								}
							}
							_ => panic!("lt with {} and {} is invalid", y2, y1),
						}
					}
					E::Ap(x3, y3) => match x3.as_ref() {
						E::Other(name) if name == "b" => self.eval(
							&E::Ap(y3.clone(), Rc::new(E::Ap(y2.clone(), y1.clone()))),
							eval_tuple,
						),
						E::Other(name) if name == "c" => self.eval(
							&E::Ap(Rc::new(E::Ap(y3.clone(), y1.clone())), y2.clone()),
							eval_tuple,
						),
						E::Other(name) if name == "s" => {
							let id = self.data.cache.len();
							self.data.cache.push(None);
							self.data.cache2.push(None);
							self.eval(
								&E::Ap(
									Rc::new(E::Ap(y3.clone(), Rc::new(E::Cloned(y1.clone(), id)))),
									Rc::new(E::Ap(y2.clone(), Rc::new(E::Cloned(y1.clone(), id)))),
								),
								eval_tuple,
							)
						}
						E::Other(name) if name == "if0" => {
							if let E::Num(a) = self.eval(y3, eval_tuple) {
								if a.is_zero() {
									self.eval(y2, eval_tuple)
								} else {
									self.eval(y1, eval_tuple)
								}
							} else {
								panic!("if0 with {}, {} and {} is invalid", y3, y2, y1)
							}
						}
						_ => E::Ap(Rc::new(x1), y1.clone()),
					},
					_ => E::Ap(x1.clone().into(), y1.clone().into()),
				},
				E::Pair(a, b) => self.eval(
					&E::Ap(Rc::new(E::Ap(y1.clone(), a.clone())), b.clone()),
					eval_tuple,
				),
				E::Other(name) if name == "inc" => {
					if let E::Num(a) = self.eval(y1, eval_tuple) {
						E::Num(a + 1)
					} else {
						panic!("inc with {} is invalid", y1);
					}
				}
				E::Other(name) if name == "dec" => {
					if let E::Num(a) = self.eval(y1, eval_tuple) {
						E::Num(a - 1)
					} else {
						panic!("dec with {} is invalid", y1);
					}
				}
				E::Other(name) if name == "neg" => {
					if let E::Num(a) = self.eval(y1, eval_tuple) {
						E::Num(-a)
					} else {
						panic!("neg with {} is invalid", y1);
					}
				}
				E::Other(name) if name == "car" => {
					if let E::Pair(a, _) = self.eval(y1, eval_tuple) {
						self.eval(&a, eval_tuple)
					} else {
						panic!("car with {} is invalid", y1);
					}
				}
				E::Other(name) if name == "cdr" => {
					if let E::Pair(_, a) = self.eval(y1, eval_tuple) {
						self.eval(&a, eval_tuple)
					} else {
						panic!("cdr with {} is invalid", y1);
					}
				}
				E::Other(name) if name == "isnil" => {
					let y1 = self.eval(y1, eval_tuple);
					match y1 {
						E::Nil => E::T,
						E::Pair(_, _) => E::F,
						E::T | E::F | E::Cons | E::Other(_) => {
							eprintln!("warning: isnil {}", &y1);
							E::F
						}
						_ => panic!("isnil with {} is invalid", y1),
					}
				},
				E::Other(name) if name == "i" => self.eval(y1.as_ref(), eval_tuple),
				E::Nil => E::T,
				_ => E::Ap(Rc::new(x1), y1.clone().into()),
			}
		}
		E::Other(name) if name.starts_with(":") => {
			*self.data.count.entry(name.clone()).or_insert(0) += 1;
			if let Some(func_ref) = self.map.get(name) {
				let func = func_ref.clone();
				let func_ref = &func;  // fixme
				self.eval(func_ref, eval_tuple)
			} else {
				panic!("no such function: {}", name)
			}
		}
		E::Pair(a, b) if eval_tuple => E::Pair(
			self.eval(a, eval_tuple).into(),
			self.eval(b, eval_tuple).into(),
		),
		e => e.clone(),
	}
}
}

pub fn simplify(e: &E) -> E {
	match e {
		E::Ap(x1, y1) => {
			let x1 = simplify(x1);
			let y1 = simplify(y1);
			match &x1 {
				E::Other(name) if name == "i" => y1,
				E::Ap(x2, y2) => match x2.as_ref() {
					E::Ap(x3, y3) => match x3.as_ref() {
						E::Other(name) if name == "b" => {
							E::Ap(y3.clone(), Rc::new(E::Ap(y2.clone(), Rc::new(y1))))
						}
						E::Other(name) if name == "c" => {
							E::Ap(Rc::new(E::Ap(y3.clone(), Rc::new(y1))), y2.clone())
						}
						E::Other(name) if name == "s" => E::Ap(
							Rc::new(E::Ap(y3.clone(), Rc::new(y1.clone()))),
							Rc::new(E::Ap(y2.clone(), Rc::new(y1))),
						),
						_ => E::Ap(Rc::new(x1), Rc::new(y1)),
					},
					_ => E::Ap(Rc::new(x1), Rc::new(y1)),
				},
				_ => E::Ap(Rc::new(x1), Rc::new(y1)),
			}
		}
		_ => e.clone(),
	}
}

pub fn get_list(e: &E) -> Option<Vec<Rc<E>>> {
	let mut list = vec![];
	let mut e = e;
	while let E::Pair(a, b) = e {
		list.push(a.clone());
		e = b;
	}
	if e == &E::Nil {
		Some(list)
	} else {
		None
	}
}

impl std::fmt::Display for E {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			E::Ap(a, b) => {
				write!(f, "(")?;
				a.fmt(f)?;
				write!(f, " ")?;
				b.fmt(f)?;
				write!(f, ")")?;
			}
			E::Cloned(a, _) => {
				write!(f, "{}", a)?;
			}
			E::Num(a) => write!(f, "{}", a)?,
			E::Pair(a, b) => {
				if let Some(list) = get_list(&self) {
					write!(f, "[")?;
					for i in 0..list.len() {
						if i > 0 {
							write!(f, ", ")?;
						}
						write!(f, "{}", list[i])?;
					}
					write!(f, "]")?;
				} else {
					write!(f, "<{}, {}>", a, b)?
				}
			},
			E::Other(name) => write!(f, "{}", name)?,
			E::Nil => write!(f, "[]")?,
			E::T => write!(f, "t")?,
			E::F => write!(f, "f")?,
			E::Cons => write!(f, "cons")?,
		}
		Ok(())
	}
}

fn consume_space(s: &str) -> &str {
	return s.trim_start_matches(char::is_whitespace);
}

pub fn to_text(e: &E) -> String {
	match e {
		E::Ap(a, b) => format!("ap {} {}", to_text(a), to_text(b)),
		E::Cloned(a, _) => to_text(a),
		E::Num(a) => format!("{}", a),
		E::Pair(a, b) => format!("ap ap cons {} {}", to_text(a), to_text(b)),
		e => format!("{}", e),
	}
}

pub fn parse_lisp(s: &str) -> (E, &str) {
	let mut s = consume_space(s);
	if s.starts_with("(") {
		let r1 = parse_lisp(&s[1..]);
		s = consume_space(r1.1);
		if s.starts_with(",") {
			s = &s[1..];
		}
		let r2 = parse_lisp(s);
		s = consume_space(r2.1);
		if !s.starts_with(")") {
			panic!("')' is expected, but {}", s);
		}
		return (E::Ap(Rc::new(r1.0), Rc::new(r2.0)), &s[1..]);
	}
	if s.starts_with("<") {
		let r1 = parse_lisp(&s[1..]);
		s = consume_space(r1.1);
		if s.starts_with(",") {
			s = &s[1..];
		}
		let r2 = parse_lisp(s);
		s = consume_space(r2.1);
		if !s.starts_with(">") {
			panic!("'>' is expected, but {}", s);
		}
		return (E::Pair(Rc::new(r1.0), Rc::new(r2.0)), &s[1..]);
	}
	if s.starts_with("[") {
		s = &s[1..];
		let mut es = Vec::new();
		while !s.is_empty() {
			s = consume_space(s);
			if s.starts_with("]") {
				let mut ee = E::Nil;
				es.reverse();
				for e in es {
					ee = E::Pair(Rc::new(e), Rc::new(ee));
				}
				return (ee, &s[1..]);
			}
			let r = parse_lisp(&s);
			es.push(r.0);
			s = consume_space(r.1);
			if s.starts_with(",") {
				s = &s[1..];
			}
		}
		panic!("']' is missing");
	}
	let p = match s.find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '+' && c != ':') {
		Some(p) => p,
		_ => s.len(),
	};
	if p != 0 {
		return (
			if let Ok(a) = s[..p].parse::<Int>() {
				E::Num(a)
			} else {
				E::Other(s[..p].to_owned())
			},
			&s[p..],
		);
	}
	panic!("Unexpected literal: {}", s);
}

// iterate as list
impl<'a> IntoIterator for &'a E {
	type Item = &'a E;
	type IntoIter = EIterator<'a>;
	fn into_iter(self) -> Self::IntoIter {
		EIterator(&self)
	}
}

#[derive(Debug)]
pub struct EIterator<'a>(&'a E);

impl<'a> Iterator for EIterator<'a> {
	type Item = &'a E;
	fn next(&mut self) -> Option<Self::Item> {
		match &self.0 {
			E::Nil => None,
			E::Pair(head, tail) => {
				self.0 = tail.as_ref();
				Some(head.as_ref())
			}
			_ => panic!(),
		}
	}
}