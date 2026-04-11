use std::error::Error;
use std::fmt::{Debug, Display};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use pnet::datalink::{Channel, Config, DataLinkReceiver, DataLinkSender, NetworkInterface};
use tracing::level_filters::LevelFilter;
use tracing::{debug, error};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use crate::error::NetprobeError;

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .pretty()
            .with_file(false)
            .with_line_number(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env()
                    .unwrap()
            )
        )
        .init();
}

#[inline]
pub fn find_iface_and_ip(ip: IpAddr) -> Option<(NetworkInterface, IpAddr)> {
    pnet::datalink::interfaces().into_iter()
        .filter_map(|iface| {
            iface.ips.iter()
                .find(|ipn| { ipn.contains( ip) })
                .map(|ipn| ipn.to_owned())
                .map(|ipn| { (iface, ipn.ip())})
        }).next()
}

#[inline]
pub fn find_iface_and_ipv4(ip: Ipv4Addr) -> Option<(NetworkInterface, Ipv4Addr)> {
    if let Some((iface, IpAddr::V4(ip))) = find_iface_and_ip(IpAddr::V4(ip)) {
        Some((iface, ip))
    } else {
        None
    }
}

#[inline]
pub fn find_iface_and_ipv6(ip: Ipv6Addr) -> Option<(NetworkInterface, Ipv6Addr)> {
    if let Some((iface, IpAddr::V6(ip))) = find_iface_and_ip(IpAddr::V6(ip)) {
        Some((iface, ip))
    } else {
        None
    }
}

#[inline]
pub fn eth_channel(iface: &NetworkInterface, cfg: Config) -> Result<(Box<dyn DataLinkSender>, Box<dyn DataLinkReceiver>), NetprobeError> {
    match pnet::datalink::channel(iface, cfg)? {
        Channel::Ethernet(snd, rcv) => Ok((snd, rcv)),
        _ => Err(NetprobeError::Unexpected("unexpected channel type"))
    }
}

#[inline]
pub fn merge_by_prefix(net: Ipv6Addr, host: Ipv6Addr, prefix: u8) -> Ipv6Addr {
    let host_msk = (1 << (128 - prefix)) - 1;
    let net_msk = !host_msk;
    let host = host.to_bits() & host_msk;
    let net = net.to_bits() & net_msk;
    let ip = net | host;
    Ipv6Addr::from(ip)
}

#[inline]
pub fn log<T: Debug + Display, E: Error>(result: Result<T, E>) {
    match result {
        Ok(value) => {
            debug!("result: {:?}", value);
            println!("{}", value);
        },
        Err(err) => {
            error!("error: {}", err)
        }
    }
}

