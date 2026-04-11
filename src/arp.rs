use std::net::Ipv4Addr;
use pnet::datalink::Config;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use pnet::util::MacAddr;
use tracing::debug;
use crate::error::NetprobeError;
use crate::util::{eth_channel, find_iface_and_ipv4};

pub fn request(ip_trg: Ipv4Addr) -> Result<MacAddr, NetprobeError> {

    let (iface, ip_snd) = find_iface_and_ipv4(ip_trg)
        .ok_or(NetprobeError::Unexpected("cannot find interface connected to this ip"))?;
    let mac = iface.mac
        .ok_or(NetprobeError::Unexpected("failed to obtain mac address of interface"))?;

    debug!("iface: {}, ip: {}, mac: {:?}", iface.name, ip_snd, mac);

    let mut buf = [0u8; 42];
    let mut eth_snd = MutableEthernetPacket::new(&mut buf)
        .ok_or(NetprobeError::Packet("ethernet", "create"))?;
    eth_snd.set_ethertype(EtherTypes::Arp);
    eth_snd.set_destination(MacAddr::broadcast());
    eth_snd.set_source(mac);
    debug!(">> eth: {:?}", eth_snd);
    
    let mut arp_snd = MutableArpPacket::new(eth_snd.payload_mut())
        .ok_or(NetprobeError::Packet("arp", "create"))?;
    arp_snd.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_snd.set_hw_addr_len(6);
    arp_snd.set_protocol_type(EtherTypes::Ipv4);
    arp_snd.set_proto_addr_len(4);
    arp_snd.set_operation(ArpOperations::Request);
    arp_snd.set_sender_hw_addr(mac);
    arp_snd.set_sender_proto_addr(ip_snd);
    arp_snd.set_target_hw_addr(MacAddr::zero());
    arp_snd.set_target_proto_addr(ip_trg);
    debug!(">> arp: {:?}", arp_snd);

    let (mut snd, mut rcv) = eth_channel(&iface, Config::default())?;
    snd.send_to(eth_snd.packet(), None)
        .ok_or(NetprobeError::Packet("ethernet", "send"))??;

    let buf = rcv.next()?;
    let eth_rsp = EthernetPacket::new(buf)
        .ok_or(NetprobeError::Packet("ethernet", "read"))?;
    debug!("<< eth: {:?}", eth_rsp);

    let buf = eth_rsp.payload();
    let arp_rsp = ArpPacket::new(buf)
        .ok_or(NetprobeError::Packet("arp", "read"))?;
    debug!("<< arp: {:?}", arp_rsp);

    Ok(arp_rsp.get_sender_hw_addr())
}
