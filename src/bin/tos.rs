use app::client::*;
// use std;
use std::convert::*;

fn run() {
	let server_url = std::env::args().nth(1).unwrap();
	let mut client = Client::new(server_url);
	if std::env::args().len() == 2 {
		client.send("[1, 0]");
		return;
	}
	let player_key = std::env::args().nth(2).unwrap();
	let mut resp = client.join(&player_key);
	let power = 64;
	let cool = 8;
	let life = 1;
	resp = client.start(resp.info.ability.potential - power * 4 - cool * 12 - life * 2, power, cool, life);

	let mut dx: i32 = std::env::var("DX").unwrap().parse().unwrap();
	let dy: i32 = std::env::var("DY").unwrap().parse().unwrap();
	while resp.stage != 2 {
		let mut myship = None;
		for ship in resp.state.ships.iter() {
			if ship.role != resp.info.role {
				continue;
			}
			myship = Some(ship.clone());
		}
		let myship = myship.unwrap();

		let mut commands = vec![];
		{
			// anti gravity
			let (x, y) = myship.pos;
			println!("{}, {}", x, y);
			// assert_eq!(myship.v, (0, 0));
			println!("{:?}", myship.v);
			let (gx, gy) = gravity(x, y);
			println!("{}, {}", gx, gy);
			Command::Accelerate(myship.id, (-gx, -gy));
		}
		
		let shoot_now = myship.heat -myship.status.cool <= myship.max_heat;
		if shoot_now {
			commands.push(Command::Shoot(1, (myship.pos.0 + dx, myship.pos.1 + dy), 64, None));
			dx+=1;
		}

		// !!!
		resp = client.command(&commands);
		// !!!

		if shoot_now {
			// println!("{}", resp);
			println!("shoot!!!!!!!");
			for ship in resp.state.ships.iter() {
				for cmd in ship.commands.iter() {
					match cmd {
						Command::Shoot(_, (x,y), _, Some((impact, four))) => {
							println!("({},{}) impact={}, four={}", x - ship.pos.0,y - ship.pos.1, impact, four);
						},
						_=> {},
					}
				}
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

fn gravity(x: i32, y: i32) -> (i32, i32) {
	(
		if x.abs() >= y.abs() {
			if x < 0 {
				1
			} else {
				-1
			}
		} else { 0 },

		if x.abs() <= y.abs() {
			if y < 0 {
				1
			} else {
				-1
			}
		} else { 0 },
	)
}