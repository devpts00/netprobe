use clap::Parser;
use tracing::info;
use crate::args::{Cmd, Proto};
use crate::util::{init_tracing, log};

mod util;
mod args;
mod error;
mod arp;

fn run(cmd: Cmd) {
    info!("cmd: {:?}", cmd);
    match cmd.proto {
        Proto::Arp { ip: ip_trg } => {
            log(arp::request(ip_trg))
        }
        Proto::Dhcp { .. } => {

        }
    }
}

fn main() {
    let _guard = init_tracing();
    let cmd = Cmd::parse();
    run(cmd);
}
