// BerkeOS — net/mod.rs
// Network core definitions

// Ethernet frame constants
pub const ETH_ALEN: usize = 6;
pub const ETH_ZLEN: usize = 60;
pub const ETH_FRAME_LEN: usize = 1514;

// Ethernet Type values
pub const ETH_P_IP: u16 = 0x0800;
pub const ETH_P_ARP: u16 = 0x0806;
pub const ETH_P_IPV6: u16 = 0x86DD;

// Ethernet Frame Header
#[repr(C)]
pub struct EthHeader {
    pub dst: [u8; ETH_ALEN],
    pub src: [u8; ETH_ALEN],
    pub ethertype: u16,
}

impl EthHeader {
    pub fn new() -> Self {
        EthHeader {
            dst: [0; ETH_ALEN],
            src: [0; ETH_ALEN],
            ethertype: ETH_P_IP,
        }
    }

    pub fn is_ipv4(&self) -> bool {
        self.ethertype == ETH_P_IP.to_be()
    }

    pub fn is_arp(&self) -> bool {
        self.ethertype == ETH_P_ARP.to_be()
    }
}

// IPv4 Header (simplified)
#[repr(C)]
pub struct Ipv4Header {
    pub version_ihl: u8,
    pub tos: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags_fragment: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
}

impl Ipv4Header {
    pub fn new() -> Self {
        Ipv4Header {
            version_ihl: 0x45,
            tos: 0,
            total_length: 0,
            identification: 0,
            flags_fragment: 0,
            ttl: 64,
            protocol: 0,
            checksum: 0,
            src_ip: [0; 4],
            dst_ip: [0; 4],
        }
    }
}

// ARP Header
#[repr(C)]
pub struct ArpHeader {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub hardware_size: u8,
    pub protocol_size: u8,
    pub opcode: u16,
    pub sender_mac: [u8; ETH_ALEN],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; ETH_ALEN],
    pub target_ip: [u8; 4],
}

impl ArpHeader {
    pub fn new() -> Self {
        ArpHeader {
            hardware_type: 1,
            protocol_type: ETH_P_IP.to_be(),
            hardware_size: ETH_ALEN as u8,
            protocol_size: 4,
            opcode: 0,
            sender_mac: [0; ETH_ALEN],
            sender_ip: [0; 4],
            target_mac: [0; ETH_ALEN],
            target_ip: [0; 4],
        }
    }

    pub fn is_request(&self) -> bool {
        self.opcode == 1
    }

    pub fn is_reply(&self) -> bool {
        self.opcode == 2
    }
}

// Network Interface
#[derive(Copy, Clone)]
pub struct NetInterface {
    pub mac: [u8; ETH_ALEN],
    pub ip: [u8; 4],
    pub present: bool,
}

impl NetInterface {
    pub const fn new() -> Self {
        NetInterface {
            mac: [0; ETH_ALEN],
            ip: [0; 4],
            present: false,
        }
    }

    pub fn set_mac(&mut self, mac: [u8; ETH_ALEN]) {
        self.mac = mac;
    }

    pub fn set_ip(&mut self, ip: [u8; 4]) {
        self.ip = ip;
    }

    pub fn is_up(&self) -> bool {
        self.present
    }
}

// Network packet buffer
pub struct PacketBuffer {
    pub data: [u8; ETH_FRAME_LEN],
    pub length: usize,
}

impl PacketBuffer {
    pub const fn new() -> Self {
        PacketBuffer {
            data: [0; ETH_FRAME_LEN],
            length: 0,
        }
    }

    pub fn set_data(&mut self, data: &[u8]) {
        let len = data.len().min(ETH_FRAME_LEN);
        self.data[..len].copy_from_slice(&data[..len]);
        self.length = len;
    }
}
