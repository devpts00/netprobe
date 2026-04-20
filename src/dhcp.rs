use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use byteorder::WriteBytesExt;
use pnet::packet::dhcp::{DhcpHardwareTypes, DhcpOperations, DhcpPacket, MutableDhcpPacket};
use pnet::packet::{MutablePacket, PacketSize};
use socket2::{Domain, Protocol, Type};
use tracing::debug;
use crate::error::NetprobeError;
use crate::util::{find_iface_and_ipv4, ipv4_all, ipv4_zero};

#[inline]
fn write_options(mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let size = buf.len();
    // magic cookie
    buf.write_u8(99)?;
    buf.write_u8(130)?;
    buf.write_u8(83)?;
    buf.write_u8(99)?;
    // message type: discover
    buf.write_u8(53)?;
    buf.write_u8(1)?;
    buf.write_u8(1)?;
    // parameter request list:
    buf.write_u8(55)?;
    buf.write_u8(1)?;
    buf.write_u8(6)?;
    // end
    buf.write_u8(255)?;
    buf.write_u8(0)?;
    Ok(size - buf.len())
}

pub fn discover(ip_trg: Ipv4Addr) -> Result<Ipv4Addr, NetprobeError> {

    let (iface, _ip_snd) = find_iface_and_ipv4(ip_trg)
        .ok_or(NetprobeError::Unexpected("cannot find interface connected to this ip"))?;
    let mac = iface.mac
        .ok_or(NetprobeError::Unexpected("failed to obtain mac address of interface"))?;

    let sk_snd: socket2::Socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    sk_snd.set_reuse_address(true)?;
    sk_snd.set_broadcast(true)?;
    sk_snd.bind(&SocketAddr::new(IpAddr::V4(ipv4_zero()), 68).into())?;
    let sk_snd: UdpSocket = sk_snd.into();

    let mut buf = [0x0u8; 1500];
    let mut dhcp_snd = MutableDhcpPacket::new(&mut buf)
        .ok_or(NetprobeError::Packet("dhcp", "create"))?;

    dhcp_snd.set_op(DhcpOperations::Request);
    dhcp_snd.set_htype(DhcpHardwareTypes::Ethernet);
    dhcp_snd.set_hlen(6);
    dhcp_snd.set_hops(0);
    dhcp_snd.set_xid(0x42);
    dhcp_snd.set_secs(0);
    // respond with broadcast
    dhcp_snd.set_flags(0x8000); 
    dhcp_snd.set_chaddr(mac);
    
    let opt_size = write_options(dhcp_snd.payload_mut())?;
    debug!(">> dhcp: {:?}", dhcp_snd);
    
    let dhcp_size = dhcp_snd.packet_size() + opt_size;
    let dhcp_buf = &buf[..dhcp_size];
    sk_snd.send_to(dhcp_buf, SocketAddr::V4(SocketAddrV4::new(ipv4_all(), 67)))?;

    let mut buf = [0x0u8; 1500];
    let (size_rsp, _sock_rsp) = sk_snd.recv_from(&mut buf)?;
    let dhcp_rsp = DhcpPacket::new(&buf[0..size_rsp])
        .ok_or(NetprobeError::Packet("dhcp", "create"))?;
    debug!("<< dhcp: {:?}", dhcp_rsp);

    let ip = dhcp_rsp.get_yiaddr();
    Ok(ip)
}