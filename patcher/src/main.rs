use std::{env, fs, thread};
use std::sync::{Arc, atomic::{AtomicU8, Ordering}};
use std::time::Duration;

use ctrlc;
use jack::{Client, ClientOptions, Control, NotificationHandler, Port, PortId, PortSpec};

#[repr(u8)]
pub enum Action {
    None = 0,
    Repatch = 1,
    Stop = 2,
}

impl Into<Action> for u8 {
    fn into(self) -> Action {
        match self {
            1 => Action::Repatch,
            2 => Action::Stop,
            _ => Action::None,
        }
    }
}

struct PortHandler {
    state: Arc<AtomicU8>
}

impl PortHandler {
    fn new(s: &Arc<AtomicU8>) -> PortHandler {
        PortHandler { state: s.clone() }
    }
}

impl NotificationHandler for PortHandler {
    fn client_registration(&mut self, _c: &Client, _p: &str, is_reg: bool) {
        if is_reg { self.state.store(Action::Repatch as u8, Ordering::Relaxed) };
    }

    fn port_registration(&mut self, _c: &Client, _p: PortId, is_reg: bool) {
        if is_reg { self.state.store(Action::Repatch as u8, Ordering::Relaxed) };
    }

    fn port_rename(&mut self, _c: &Client, _p: PortId, _o: &str, _n: &str) -> Control {
        self.state.store(Action::Repatch as u8, Ordering::Relaxed);
        Control::Continue
    }

    fn ports_connected(&mut self, _c: &Client, _a: PortId, _b: PortId, _: bool) {
        self.state.store(Action::Repatch as u8, Ordering::Relaxed);
    }
}

type ParseRes<T> = Result<Vec<T>, &'static str>;

struct Conn {
    out: String,
    inp: String,
}

fn is_letter(s: &str) -> bool {
    s.len() == 1 && s.chars().all(|x| x.is_ascii_alphabetic())
}

fn is_number(s: &str) -> bool {
    s.chars().all(|c| c.is_digit(10))
}

fn parse_range_elt(range: &str, p: &str, s: &str) -> ParseRes<String> {
    let mut v = Vec::new();
    match range.split_once('-') {
        Some((s, e)) => {
            if is_number(s) && is_number(e) {
                let s = s.parse::<usize>().unwrap();
                let e = e.parse::<usize>().unwrap();
                for i in s..=e { v.push(i.to_string()) }
            } else if is_letter(s) && is_letter(e) {
                let s = s.chars().nth(0).unwrap();
                let e = e.chars().nth(0).unwrap();
                for i in s..=e { v.push(i.to_string()) }
            } else {
                return Err("Incorrect range, must be two numbers or two letters")
            }
        }
        None => v.push(range.to_owned())
    }
    Ok(v.iter().map(|x| [p, x, s].concat().to_owned())
        .collect::<Vec<_>>())
}

fn parse_range(s: &str) -> ParseRes<String> {
    let (prefix, rest) = match s.split_once('[') {
        Some((l, r)) => (l, r),
        None => return Ok(vec![s.to_owned()])
    };
    let (range, suffix) = rest.split_once(']').ok_or("Unbalanced brackets")?;
    Ok(range.split(',')
        .map(|x| parse_range_elt(x.trim(), prefix, suffix))
        .collect::<Result<Vec<_>, &str>>()?.concat())
}

fn parse_line(s: &str) -> ParseRes<Conn> {
    let (o, i) = s.split_once('>').ok_or("Malformed connection")?;
    let (o, i) = (parse_range(o.trim())?, parse_range(i.trim())?);
    if o.len() == i.len() {
        Ok(o.into_iter().zip(i.into_iter())
            .map(|(out, inp)| Conn { out, inp })
            .collect::<Vec<_>>())
    } else { Err("Unbalanced connection") }
}

fn check_connection<PS: PortSpec>(c: &Client, out: &Port<PS>, inp: &Port<PS>, conn: &Conn)
    -> Result<(), jack::Error>
{
    let ok = out.is_connected_to(&conn.inp)?
        && out.connected_count()? == 1
        && inp.connected_count()? == 1;
    if !ok {
        let _ = c.disconnect(out)?;
        let _ = c.disconnect(inp)?;
        let _ = c.connect_ports(out, inp)?;
    }
    Ok(())
}

fn check_ports(c: &Client, conns: &Vec<Conn>) {
    for conn in conns {
        if let (Some(out), Some(inp)) = (c.port_by_name(&conn.out), c.port_by_name(&conn.inp)) {
            match check_connection(c, &out, &inp, &conn) {
                Ok(_) => println!("Connected {} to {}", conn.out, conn.inp),
                Err(e) => eprintln!("Error {} occured while processing connection {} > {}", e, conn.out, conn.inp)
            }
        }
    }
}

fn set_quit_handler(s: &Arc<AtomicU8>) {
    let s = s.clone();
    ctrlc::set_handler(move || { 
        println!("Patcher is shutting down");
        s.store(Action::Stop as u8, Ordering::Relaxed);
    }).expect("Unable to create signal handler");
}

fn main() {
    let fname = env::args().last().expect("No filename provided");
    let cfg = fs::read_to_string(fname).expect("Unable to read connections file");
    let mut conns = Vec::new();
    for line in cfg.lines() {
        match parse_line(line) {
            Ok(cs) => conns.extend(cs),
            Err(e) => eprintln!("Error parsing line \"{}\": {}", line, e)
        }
    }

    println!("Connection list:");
    for c in &conns { println!("\t{}, {}", c.out, c.inp) }

    let state = Arc::new((Action::Repatch as u8).into());
    let (client, _) = Client::new("patcher", ClientOptions::NO_START_SERVER)
        .expect("Cannot create JACK client");
    let handler = PortHandler::new(&state);
    let client = client.activate_async(handler, ()).expect("Unable to activate client");

    set_quit_handler(&state);
    loop {
        match state.load(Ordering::Relaxed).into() {
            Action::None => (),
            Action::Repatch => {
                check_ports(&client.as_client(), &conns);
                state.store(Action::None as u8, Ordering::Relaxed);
            },
            Action::Stop => break,
        }
        thread::sleep(Duration::from_millis(200));
    }
    if let Err(e) = &client.deactivate() { 
        eprintln!("Error deactivating client: {}", e);
    }
}
