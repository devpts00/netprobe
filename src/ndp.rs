use crate::error::NetprobeError;
use crate::util::{eth_channel, find_iface_and_ipv6, merge_by_prefix};
use pnet::datalink::Config;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::icmpv6::ndp::{MutableNdpOptionPacket, MutableNeighborSolicitPacket, NdpOptionTypes, NeighborAdvertPacket};
use pnet::packet::icmpv6::{checksum, Icmpv6Code, Icmpv6Packet, Icmpv6Types};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv6::{Ipv6Packet, MutableIpv6Packet};
use pnet::packet::{MutablePacket, Packet, PacketSize};
use pnet::util::MacAddr;
use std::net::Ipv6Addr;
use tracing::debug;

const NEIGHBOR_SOLICIT_IPV6: [u8; 16] = [0xFF, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0xFF, 0, 0, 0];
const NEIGHBOR_SOLICIT_MAC: [u8; 6] = [0x33, 0x33, 0xFF, 0, 0, 0];

#[inline]
fn neighbor_solicit_ipv6(ip: Ipv6Addr) -> Ipv6Addr {
    merge_by_prefix(Ipv6Addr::from(NEIGHBOR_SOLICIT_IPV6), ip, 104)
}

#[inline]
fn neighbor_solicit_mac(ip: Ipv6Addr) -> MacAddr {
    let mut octets: [u8; 6] = [0; 6];
    octets[0..3].copy_from_slice(&NEIGHBOR_SOLICIT_MAC[0..3]);
    octets[3..6].copy_from_slice(&ip.octets()[13..16]);
    MacAddr::from(octets)
}

pub fn request(trg_ip: Ipv6Addr) -> Result<MacAddr, NetprobeError> {

    let (src_if, src_ip) = find_iface_and_ipv6(trg_ip)
        .ok_or(NetprobeError::Unexpected("cannot find interface connected to this ip"))?;
    let src_mc = src_if.mac
        .ok_or(NetprobeError::Unexpected("failed to obtain mac address of interface"))?;
    let dst_ip = neighbor_solicit_ipv6(trg_ip);
    let dst_mc = neighbor_solicit_mac(trg_ip);

    debug!("src, iface: {}", src_if.name);
    debug!("src, mac/unicast: {}", src_mc);
    debug!("src, ip/unicast: {}", src_ip);
    debug!("dst, mac/multicast: {}", dst_mc);
    debug!("dst, ip/multicast: {}", dst_ip);
    debug!("trg, ip/unicast: {}", trg_ip);

    let mut buf = [0u8; 1500];
    let mut eth_snd = MutableEthernetPacket::new(&mut buf)
        .ok_or(NetprobeError::Packet("ethernet", "create"))?;
    eth_snd.set_ethertype(EtherTypes::Ipv6);
    eth_snd.set_destination(dst_mc);
    eth_snd.set_source(src_mc);
    let eth_snd_size = eth_snd.packet_size();
    debug!(">> eth, size: {}, data: {:?}", eth_snd_size, eth_snd);

    let mut ip6_snd = MutableIpv6Packet::new(eth_snd.payload_mut())
        .ok_or(NetprobeError::Packet("ethernet", "create"))?;
    ip6_snd.set_version(6);
    ip6_snd.set_traffic_class(0);
    ip6_snd.set_flow_label(0xb8d66);
    ip6_snd.set_payload_length(32); // TODO: calculate dynamically
    ip6_snd.set_next_header(IpNextHeaderProtocols::Icmpv6);
    ip6_snd.set_hop_limit(255);
    ip6_snd.set_source(src_ip);
    ip6_snd.set_destination(dst_ip);
    let ip6_snd_size = ip6_snd.packet_size();
    debug!(">> ip6, size: {}, data: {:?}", ip6_snd_size, ip6_snd);

    let mut ndp_snd = MutableNeighborSolicitPacket::new(ip6_snd.payload_mut())
        .ok_or(NetprobeError::Packet("ndp", "create"))?;
    ndp_snd.set_icmpv6_type(Icmpv6Types::NeighborSolicit);
    ndp_snd.set_icmpv6_code(Icmpv6Code(0));
    ndp_snd.set_checksum(0);
    ndp_snd.set_reserved(0);
    ndp_snd.set_target_addr(trg_ip);

    let mut opt_snd = MutableNdpOptionPacket::new(ndp_snd.get_options_raw_mut())
        .ok_or(NetprobeError::Packet("options", "create"))?;
    opt_snd.set_option_type(NdpOptionTypes::SourceLLAddr);
    opt_snd.set_length(1);
    opt_snd.set_data(&src_mc.octets());

    let icmpv6_snd = Icmpv6Packet::new(ndp_snd.packet_mut())
        .ok_or(NetprobeError::Packet("icmpv6", "create"))?;
    let checksum = checksum(&icmpv6_snd, &src_ip, &dst_ip);
    ndp_snd.set_checksum(checksum);
    debug!(">> ndp, size: {}, : {:?}", ndp_snd.packet_size(), ndp_snd);

    let (mut snd, mut rcv) = eth_channel(&src_if, Config::default())?;
    snd.send_to(&eth_snd.packet()[0..eth_snd_size + ip6_snd_size], None)
        .ok_or(NetprobeError::Packet("ethernet", "send"))??;

    loop {
        let buf = rcv.next()?;
        let eth_rsp = EthernetPacket::new(buf)
            .ok_or(NetprobeError::Packet("ethernet", "read"))?;
        debug!("<< eth: {:?}", eth_rsp);
        if eth_rsp.get_ethertype() == EtherTypes::Ipv6 && eth_rsp.get_destination() == src_mc {
            let buf = eth_rsp.payload();
            let ipv6_rsp = Ipv6Packet::new(buf)
                .ok_or(NetprobeError::Packet("ipv6", "read"))?;
            debug!("<< ip6: {:?}", ipv6_rsp);
            if ipv6_rsp.get_next_header() == IpNextHeaderProtocols::Icmpv6 {
                let buf = ipv6_rsp.payload();
                let icmpv6_rsp = Icmpv6Packet::new(buf)
                    .ok_or(NetprobeError::Packet("icmpv6", "read"))?;
                if icmpv6_rsp.get_icmpv6_type() == Icmpv6Types::NeighborAdvert {
                    let ndp_rsp = NeighborAdvertPacket::new(buf)
                        .ok_or(NetprobeError::Packet("neighbor", "read"))?;
                    debug!("<< ndp: {:?}", ndp_rsp);
                    for opt in ndp_rsp.get_options().iter() {
                        if opt.option_type == NdpOptionTypes::TargetLLAddr {
                            if opt.data.len() >= 6 {
                                let b = &opt.data[0..6];
                                return Ok(MacAddr::new(b[0], b[1], b[2], b[3], b[4], b[5]));
                            } else {
                                return Err(NetprobeError::Unexpected("TLLA is too short"))
                            }
                        }
                    }
                }
            }
        }
    }
}
