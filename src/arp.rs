use std::net::Ipv4Addr;
use pnet::datalink::Config;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::Packet;
use pnet::util::MacAddr;
use tracing::info;
use crate::error::NetprobeError;
use crate::util::{eth_channel, find_iface_and_ipv4};

pub fn request(ip_trg: Ipv4Addr) -> Result<MacAddr, NetprobeError> {

    let (iface, ip_snd) = find_iface_and_ipv4(ip_trg)
        .ok_or(NetprobeError::Unexpected("cannot find interface connected to this ip"))?;
    let mac = iface.mac
        .ok_or(NetprobeError::Unexpected("failed to obtain mac address of interface"))?;

    info!("iface: {}, ip: {}, mac: {:?}", iface.name, ip_snd, mac);

    let mut eth_buf = [0u8; 42];
    let mut eth_snd = MutableEthernetPacket::new(&mut eth_buf[..])
        .ok_or(NetprobeError::Unexpected("failed to create ethernet packet"))?;
    eth_snd.set_ethertype(EtherTypes::Arp);
    eth_snd.set_destination(MacAddr::broadcast());
    eth_snd.set_source(mac);

    let mut arp_buf = [0u8; 28];
    let mut arp_snd = MutableArpPacket::new(&mut arp_buf[..])
        .ok_or(NetprobeError::Unexpected("failed to create arp packet"))?;

    arp_snd.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_snd.set_hw_addr_len(6);
    arp_snd.set_protocol_type(EtherTypes::Ipv4);
    arp_snd.set_proto_addr_len(4);
    arp_snd.set_operation(ArpOperations::Request);
    arp_snd.set_sender_hw_addr(mac);
    arp_snd.set_sender_proto_addr(ip_snd);
    arp_snd.set_target_hw_addr(MacAddr::zero());
    arp_snd.set_target_proto_addr(ip_trg);

    info!(">> arp: {:?}", arp_snd);

    eth_snd.set_payload(arp_snd.packet());

    info!(">> eth: {:?}", eth_snd);

    let x = pnet::datalink::channel(&iface, Config::default())?;

    let (mut snd, mut rcv) = eth_channel(&iface, Config::default())?;

    snd.send_to(eth_snd.packet(), None)
        .ok_or(NetprobeError::Unexpected("cannot send eth packet"))??;

    let buf = rcv.next()?;
    let eth_rsp = EthernetPacket::new(buf)
        .ok_or(NetprobeError::Unexpected("failed to read ethernet frame"))?;
    info!("<< eth: {:?}", eth_rsp);

    let buf = eth_rsp.payload();
    let arp_rsp = ArpPacket::new(buf)
        .ok_or(NetprobeError::Unexpected("failed to read arp packet"))?;
    info!("<< arp: {:?}", arp_rsp);

    Ok(arp_rsp.get_sender_hw_addr())
}
