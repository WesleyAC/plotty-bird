// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate serialport;
extern crate rand;

use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufRead, Error};
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};

use rand::Rng;

use serialport::prelude::*;

struct PipePair {
    x: u32,
    y1: u32,
    y2: u32,
}

fn draw_board(board: &Vec<PipePair>) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let pipe_body_width = 800;
    let pipe_top_width = 1000;
    let pipe_top_height = 200;
    let height = 7650;

    out.push("SP1;".to_string());
    for p in board {
        // bottom pipe
        out.push("PU;".to_string());
        out.push(format!("PA{},{};", p.x - pipe_body_width/2, 0));
        out.push("PD;".to_string());
        out.push(format!("PA{},{};", p.x - pipe_body_width/2, p.y1 - pipe_top_height));
        out.push(format!("PA{},{};", p.x + pipe_body_width/2, p.y1 - pipe_top_height));
        out.push(format!("PA{},{};", p.x + pipe_body_width/2, 0));
        out.push("PU;".to_string());
        out.push(format!("PA{},{};", p.x - pipe_body_width/2, p.y1 - pipe_top_height));
        out.push("PD;".to_string());
        out.push(format!("PA{},{};", p.x - pipe_top_width/2, p.y1 - pipe_top_height));
        out.push(format!("PA{},{};", p.x - pipe_top_width/2, p.y1));
        out.push(format!("PA{},{};", p.x + pipe_top_width/2, p.y1));
        out.push(format!("PA{},{};", p.x + pipe_top_width/2, p.y1 - pipe_top_height));
        out.push(format!("PA{},{};", p.x + pipe_body_width/2, p.y1 - pipe_top_height));
        out.push("PU;".to_string());

        // top pipe
        out.push(format!("PA{},{};", p.x - pipe_body_width/2, height));
        out.push("PD;".to_string());
        out.push(format!("PA{},{};", p.x - pipe_body_width/2, p.y2 + pipe_top_height));
        out.push(format!("PA{},{};", p.x + pipe_body_width/2, p.y2 + pipe_top_height));
        out.push(format!("PA{},{};", p.x + pipe_body_width/2, height));
        out.push("PU;".to_string());
        out.push(format!("PA{},{};", p.x - pipe_body_width/2, p.y2 + pipe_top_height));
        out.push("PD;".to_string());
        out.push(format!("PA{},{};", p.x - pipe_top_width/2, p.y2 + pipe_top_height));
        out.push(format!("PA{},{};", p.x - pipe_top_width/2, p.y2));
        out.push(format!("PA{},{};", p.x + pipe_top_width/2, p.y2));
        out.push(format!("PA{},{};", p.x + pipe_top_width/2, p.y2 + pipe_top_height));
        out.push(format!("PA{},{};", p.x + pipe_body_width/2, p.y2 + pipe_top_height));
        out.push("PU;".to_string());
    }
    out
}

fn gen_board() -> Vec<PipePair> {
    let mut out = vec![];
    let mut rng = rand::thread_rng();
    for i in 1..5 {
        let height = rng.gen_range(1000, 4600);
        let spacing = rng.gen_range(2100, 2600);
        out.push(PipePair {x: i * 2400 - 500, y1: height, y2: height + spacing});
    }
    out
}

fn check_collision(x: i32, y: i32, board: &Vec<PipePair>) -> bool {
    let pipe_body_width = 800;
    let height = 7650;

    if y <= 0 || y >= height { return true; }
    for p in board {
        if x > (p.x - pipe_body_width / 2) as i32 && x < (p.x + pipe_body_width / 2) as i32 && (y < p.y1 as i32 || y > p.y2 as i32) {
            return true;
        }
    }

    false
}

fn main() -> Result<(),Error> {
    let board: Vec<PipePair> = gen_board();

    let board_cmds: Vec<Vec<u8>> = draw_board(&board).iter().map(|x| { x.clone().into_bytes() }).collect();

	let s = SerialPortSettings {
		baud_rate: 9600,
		data_bits: DataBits::Eight,
		flow_control: FlowControl::None,
		parity: Parity::None,
		stop_bits: StopBits::One,
		timeout: Duration::from_millis(1000),
	};

	let args: Vec<_> = std::env::args().collect();

    match serialport::open_with_settings(&args[1], &s) {
        Ok(mut port) => {
			port.write(b"IN;");
            let mut next_cmd = vec![];
			for cmd in board_cmds.iter() {
                if next_cmd.len() + cmd.len() < 57 {
                    next_cmd.append(&mut cmd.clone());
                } else {
                    port.write(&next_cmd);
                    port.write(b"OA;");
                    let mut c = 0;
                    while c != 13 {
                        let mut v = vec![0];
                        port.read(v.as_mut_slice());
                        c = v[0];
                    }
                    port.clear(ClearBuffer::All);
                    next_cmd = cmd.to_vec();
                }
			}
            port.write(&next_cmd);
            port.write(b"PU;SP2;PA0,3825;PD;OA;");
            let mut c = 0;
            while c != 13 {
                let mut v = vec![0];
                port.read(v.as_mut_slice());
                c = v[0];
            }
            port.clear(ClearBuffer::All);

            let pos: Arc<Mutex<(i32, i32, i32)>> = Arc::new(Mutex::new((0, 3825, 0)));
            let ipos = Arc::clone(&pos);
            thread::spawn(move || {
                loop {
                    let mut input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(n) => {
                            let mut p = ipos.lock().unwrap();
                            p.2 = 300;
                        }
                        Err(error) => println!("error: {}", error),
                    }
                }
            });
            loop {
                {
                    let mut p = pos.lock().unwrap();
                    port.write(format!("PA{},{};", p.0, p.1).as_bytes());
                    p.0 += 100;
                    p.1 += p.2;
                    p.2 -= 30;
                    if check_collision(p.0, p.1, &board) {
                        port.write(b"PU;PR-270,-300;PD;PR-40,200;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"PR-120,80;PR160,110;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"PR-40,200;PR160,-110;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"PR20,200;PR120,-180;PR140,120;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"PR20,-140;PR170,30;PR-140,-100;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"PR160,-70;PR-160,-70;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"PR140,-120;PR-160,-20;PR20,-150;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"PR-140,120;PR-100,-120;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"PR-90,100;PR-120,-80;PU;");
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"SP0;PA-5000,0;");
                        thread::sleep(Duration::from_millis(180));
                        ::std::process::exit(0);
                    }
                    if p.0 > 9600 {
                        let bird_cmds: Vec<Vec<u8>> = vec!["PU;", "PR-500,-200;", "PR246,577;", "PD;", "PR-4,-19;", "PR-22,-43;", "PR4,-1;", "PR2,-58;", "PR2,0;", "PR16,-53;", "PR4,2;", "PR22,-28;", "PR-14,1;", "PR-54,-41;", "PR4,-2;", "PR-26,-49;", "PR5,-1;", "PR-1,-31;", "PR3,0;", "PR12,-34;", "PR4,1;", "PR29,-33;", "PR2,3;", "PR45,-24;", "PR1,3;", "PR58,-13;", "PR0,2;", "PR68,0;", "PR-1,3;", "PR73,16;", "PR-1,2;", "PR74,35;", "PR-1,1;", "PR72,55;", "PR-2,2;", "PR66,77;", "PR-13,0;", "PR-113,129;", "PR-2,-2;", "PR-111,89;", "PR-1,-2;", "PR-61,29;", "PR-1,-2;", "PR-60,11;", "PR0,-4;", "PR-45,-8;", "PR1,-3;", "PR-43,-25;", "PR5,-4;", "PR-18,-35;", "PU;", "PR555,-82;", "PD;", "PR11,1;", "PR18,-21;", "PR4,7;", "PR29,-2;", "PR-3,7;", "PR22,19;", "PR-7,3;", "PR2,29;", "PR-8,-3;", "PR-18,22;", "PR-4,-7;", "PR-29,2;", "PR3,-7;", "PR-22,-19;", "PR7,-4;", "PR-2,-28;", "PR8,2;", "PR18,-21;", "PR4,7;", "PR11,-1;", "PU;", "PR137,80;", "PD;", "PR-15,-7;", "PR-11,-17;", "PR7,-1;", "PR7,-35;", "PR3,1;", "PR17,-31;", "PR5,5;", "PR21,-7;", "PR-2,10;", "PR43,42;", "PR-1,1;", "PR22,30;", "PR-16,-6;", "PR-98,18;", "PR0,-2;", "PR-2,0;", "PR5,-8;", "PR-11,-17;", "PR7,-1;", "PR4,-20;", "PU;", "PR-277,21;", "PD;", "PR-10,4;", "PR-208,-154;", "PR-1,0;", "PR-176,-121;", "PR-2,7;", "PR-271,42;", "PR10,-11;", "PR2,-110;", "PR10,10;", "PR269,-3;", "PR-3,-7;", "PR71,-58;", "PR1,2;", "PR97,-53;", "PR1,3;", "PR117,-27;", "PR0,3;", "PR65,3;", "PR0,2;", "PR68,18;", "PR-1,2;", "PR89,42;", "PR-1,1;", "PR68,51;", "PR-1,1;", "PR52,55;", "PR-2,1;", "PR37,55;", "PR-2,1;", "PR38,91;", "PR-1,1;", "PR8,38;", "PR-2,0;", "PR-3,128;", "PR-9,-5;", "PR-47,26;", "PR-1,-1;", "PR-53,18;", "PR-1,-2;", "PR-67,8;", "PR1,-3;", "PR-75,-11;", "PR1,-3;", "PR-79,-38;", "PR1,-2;", "PR-32,-24;", "SP0;", "PR-626,-463;"].iter().map(|x| { Vec::from(x.clone().as_bytes()) }).collect();
                        let mut next_cmd = vec![];
                        for cmd in bird_cmds.iter() {
                            if next_cmd.len() + cmd.len() < 57 {
                                next_cmd.append(&mut cmd.clone());
                            } else {
                                port.write(&next_cmd);
                                port.write(b"OA;");
                                let mut c = 0;
                                while c != 13 {
                                    let mut v = vec![0];
                                    port.read(v.as_mut_slice());
                                    c = v[0];
                                }
                                port.clear(ClearBuffer::All);
                                next_cmd = cmd.to_vec();
                            }
                        }
                        port.write(&next_cmd);
                        thread::sleep(Duration::from_millis(180));
                        port.write(b"SP0;PA-5000,0;");
                        thread::sleep(Duration::from_millis(180));
                        ::std::process::exit(0);
                    }
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
        Err(e) => {
            ::std::process::exit(1);
        }
    };

	Ok(())
}