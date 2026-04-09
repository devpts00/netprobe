use std::net::Ipv6Addr;
use pnet::packet::icmpv6::{Icmpv6Code, Icmpv6Types};
use pnet::packet::icmpv6::ndp::MutableNeighborSolicitPacket;
use pnet::util::MacAddr;
use tracing::info;
use crate::error::NetprobeError;
use crate::util::find_iface_and_ipv6;

pub fn request(ip_trg: Ipv6Addr) -> Result<MacAddr, NetprobeError> {
    let (iface, ip_snd) = find_iface_and_ipv6(ip_trg)
        .ok_or(NetprobeError::Unexpected("cannot find interface connected to this ip"))?;
    let mac = iface.mac
        .ok_or(NetprobeError::Unexpected("failed to obtain mac address of interface"))?;

    info!("iface: {}, ip: {}, mac: {:?}", iface.name, ip_snd, mac);

    let mut ndp_buf = [0u8; 128];
    let mut ndp_snd = MutableNeighborSolicitPacket::new(&mut ndp_buf[..])
        .ok_or(NetprobeError::Unexpected("failed to create arp packet"))?;

    // ndp_snd.set_icmpv6_type(Icmpv6Types::NeighborSolicit);
    // ndp_snd.set_icmpv6_code(Icmpv6Code::)
    // ndp_snd.set_target_addr();

    Ok(MacAddr::zero())
}