use smoltcp;
use smoltcp::iface::EthernetInterface;
use smoltcp::socket::{Socket, SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::socket::{UdpPacketMetadata, UdpSocket, UdpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpEndpoint, Ipv4Address};

mod packets;
pub use self::packets::BallPacket;
pub use self::packets::GamestatePacket;
pub use self::packets::InputPacket;
pub use self::packets::RacketPacket;

use alloc::Vec;
use board;
use embedded;
use ethernet;
use system_clock;

const PORT: u16 = 2018;

pub struct Network<'a> {
    ethernet_interface: EthernetInterface<'a, 'a, ethernet::EthernetDevice>,
    sockets: SocketSet<'a, 'a, 'a>,
}

impl<'a> Network<'a> {
    pub fn handle_ethernet_packets(&mut self) {
        // handle new ethernet packets
        match self.ethernet_interface.poll(
            &mut self.sockets,
            Instant::from_millis(system_clock::ticks() as i64),
        ) {
            Err(::smoltcp::Error::Exhausted) => return,
            Err(::smoltcp::Error::Unrecognized) => {}
            Err(e) => hprintln!("Network error: {:?}", e),
            Ok(socket_changed) => if socket_changed {
                for mut socket in self.sockets.iter_mut() {
                    poll_socket(&mut socket).expect("socket poll failed");
                }
            },
        }
    }
}

pub fn init(
    rcc: &mut board::rcc::Rcc,
    syscfg: &mut board::syscfg::Syscfg,
    ethernet_mac: &'static mut board::ethernet_mac::EthernetMac,
    ethernet_dma: &'static mut board::ethernet_dma::EthernetDma,
    gpio: &mut embedded::interfaces::gpio::Gpio,
    ethernet_addr: EthernetAddress,
    ip_addr: Ipv4Address,
) -> Option<Network<'static>> {
    // Ethernet init
    let ethernet_interface = ethernet::EthernetDevice::new(
        Default::default(),
        Default::default(),
        rcc,
        syscfg,
        gpio,
        ethernet_mac,
        ethernet_dma,
        ethernet_addr,
    ).map(|device| device.into_interface(ip_addr));
    if let Err(e) = ethernet_interface {
        hprintln!("ethernet init failed: {:?}", e);
        return None;
    }

    let mut sockets = SocketSet::new(Vec::new());
    let endpoint = IpEndpoint::new(IpAddress::Ipv4(ip_addr), PORT);

    let udp_rx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY; 3], vec![0u8; 256]);
    let udp_tx_buffer = UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY; 1], vec![0u8; 128]);
    let mut example_udp_socket = UdpSocket::new(udp_rx_buffer, udp_tx_buffer);
    example_udp_socket.bind(endpoint).unwrap();
    sockets.add(example_udp_socket);


    Some(Network {
        ethernet_interface: ethernet_interface.unwrap(),
        sockets: sockets,
    })
}

fn poll_socket(socket: &mut Socket) -> Result<(), smoltcp::Error> {
    match socket {
        &mut Socket::Udp(ref mut socket) => match socket.endpoint().port {
            PORT => loop {
                let reply;
                match socket.recv() {
                    Ok((data, remote_endpoint)) => {
                        let mut data = Vec::from(data);
                        let len = data.len() - 1;
                        data[..len].reverse();
                        reply = (data, remote_endpoint);
                    }
                    Err(smoltcp::Error::Exhausted) => break,
                    Err(err) => return Err(err),
                }
                socket.send_slice(&reply.0, reply.1);
            },
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

pub trait Client {
    fn send_input(&mut self, input: &InputPacket);
    fn receive_gamestate(&self) -> GamestatePacket;
}

pub trait Server {
    fn receive_inputs(&self) -> [InputPacket; 2];
    fn send_gamestate(&mut self, gamestate: &GamestatePacket);
}

pub struct LocalClient {
    gamestate: GamestatePacket,
    input: InputPacket,
}

impl LocalClient {
    pub fn new() -> LocalClient {
        LocalClient {
            gamestate: GamestatePacket::new(),
            input: InputPacket::new(),
        }
    }
}

impl Client for LocalClient {
    fn send_input(&mut self, input: &InputPacket) {
        self.input = *input;
    }
    fn receive_gamestate(&self) -> GamestatePacket {
        self.gamestate
    }
}

pub struct LocalServer {
    gamestate: GamestatePacket,
    player_inputs: [InputPacket; 2],
}

impl LocalServer {
    pub fn new() -> LocalServer {
        LocalServer {
            gamestate: GamestatePacket::new(),
            player_inputs: [InputPacket::new(), InputPacket::new()],
        }
    }
}

impl Server for LocalServer {
    fn receive_inputs(&self) -> [InputPacket; 2] {
        self.player_inputs
    }
    fn send_gamestate(&mut self, gamestate: &GamestatePacket) {
        self.gamestate = *gamestate;
    }
}

pub fn handle_local(
    client1: &mut LocalClient,
    client2: &mut LocalClient,
    server: &mut LocalServer,
) {
    client1.gamestate = server.gamestate;
    client2.gamestate = server.gamestate;
    server.player_inputs = [client1.input, client2.input];
}