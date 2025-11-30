use clap::Parser;
use std::collections::VecDeque;
use std::io;
use std::io::Write;

const BASE64: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];

enum State {
    State0,
    State1,
    State2,
    State3,
}

struct Base64Queue {
    state: State,
    data: VecDeque<u8>,
}

impl Base64Queue {
    fn new() -> Self {
        Self {
            state: State::State0,
            data: VecDeque::new(),
        }
    }

    fn push(&mut self, value: u8) {
        self.data.push_back(value)
    }

    fn pop(&mut self) -> Option<u8> {
        let (state, value) = match self.state {
            State::State0 => (State::State1, {
                let byte0 = self.data.front()?;
                Some(byte0 >> 2)
            }),
            State::State1 => (State::State2, {
                let byte0 = self.data.pop_front().expect("State should be unreachable");
                let byte1 = self.data.front().unwrap_or(&0);
                Some(((byte0 << 4) & 0x3f) | (byte1 >> 4))
            }),
            State::State2 => (State::State3, {
                self.data.pop_front().map(|byte1| {
                    let byte2 = self.data.front().unwrap_or(&0);
                    ((byte1 << 2) & 0x3f) | (byte2 >> 6)
                })
            }),
            State::State3 => (State::State0, {
                self.data.pop_front().map(|byte2| byte2 & 0x3f)
            }),
        };
        self.state = state;
        Some(value.map_or(b'=', |idx| BASE64[idx as usize]))
    }
}

const SEED: [u8; 3] = [b'V', b'm', b'0'];

fn base64_fixpoint<Writer: Write>(writer: &mut Writer) -> io::Result<()> {
    let mut base64 = Base64Queue::new();
    for byte in SEED {
        base64.push(byte);
        let popped = base64.pop();
        assert!(popped == Some(byte));
        writer.write_all(&[byte])?;
    }
    loop {
        let popped = base64.pop();
        let byte = popped.expect("Queue should never be empty");
        base64.push(byte);
        writer.write_all(&[byte])?;
    }
}

#[derive(Parser)]
struct Args {}

fn main() -> io::Result<()> {
    let _args = Args::parse();
    match base64_fixpoint(&mut io::stdout().lock()) {
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}
