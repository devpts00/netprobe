use clap::Parser;
use tracing::debug;
use crate::args::{Cmd, Proto};
use crate::util::{init_tracing, log};

mod util;
mod args;
mod error;
mod arp;
mod ndp;
mod dhcp;

fn main() {
    let _guard = init_tracing();
    let cmd = Cmd::parse();
    debug!("cmd: {:?}", cmd);
    match cmd.proto {
        Proto::Arp { ip: ip_trg } => {
            log(arp::request(ip_trg))
        }
        Proto::Ndp { ip: ip_trg } => {
            log(ndp::request(ip_trg))
        }
        Proto::Dhcp { ip: ip_trg } => {
            log(dhcp::discover(ip_trg))
        }
    }
}
