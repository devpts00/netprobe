use std::net::{Ipv4Addr, Ipv6Addr};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cmd {
    #[command(subcommand)]
    pub proto: Proto,
}

#[derive(Subcommand, Debug)]
pub enum Proto {
    Arp {
        #[arg(long)]
        ip: Ipv4Addr,
    },
    Ndp {
        #[arg(long)]
        ip: Ipv6Addr,
    },
    Dhcp {

    }
}