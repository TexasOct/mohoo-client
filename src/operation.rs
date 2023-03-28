use rust_uci::Uci;
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::process::exit;
use std::process::Command;
use std::str::FromStr;
use std::string::ToString;
use wireguard_control::backends::kernel::delete_interface;
use wireguard_control::{Backend, DeviceUpdate, Key, KeyPair};
use once_cell::sync::Lazy;
use serde_json::{json, Value};

pub(crate) static mut DEVICE: Lazy<Peer> = Lazy::new(||{
    Peer::init(
        IpAddr::from_str("10.10.1.2").unwrap(),
        "10.1.1.1:8889".to_string(),
        "11111111111111111111111111111111".to_string()
    )
});

pub struct Peer {
    peer_ip: IpAddr,
    server_socket: SocketAddr,
    server_pubkey: Key,
    peer_keypair: KeyPair,
    peer_ssid: String,
    peer_passwd: String,
}

fn boot_wireguard_device(
    peer_ip: &IpAddr,
    peer_keypair: &KeyPair,
    server_socket: &SocketAddr,
    server_pubkey: &Key,
) -> std::io::Result<()> {
    DeviceUpdate::new()
        .set_keypair(peer_keypair.clone())
        .replace_peers()
        .add_peer_with(server_pubkey, |peer| {
            peer.set_endpoint(server_socket.clone())
                .replace_allowed_ips()
                .add_allowed_ip(peer_ip.clone(), 0)
        })
        .apply(&"mosquitto-wg".parse().unwrap(), Backend::Kernel)
}

fn str2raw(key: String) -> [u8; 32] {
    let mut raw = [0; 32];
    if key.len() > 32 || key.len() == 0 {
        exit(-1)
    }
    for (tag, c) in key.chars().enumerate() {
            raw[tag] = c as u8
    }
    raw
}

pub(crate) fn raw2str(key: &[u8]) -> String {
    let mut str = String::from("");
    if key.len() > 32 || key.len() == 0 {
        exit(-1)
    }
    for num in key.iter() {
        str.push(*num as char)
    }
    str
}

impl Peer {
    pub fn init(peer_ip: IpAddr, server_socket: String, server_pubkey: String) -> Self {
        let peer_keypair = KeyPair::generate();
        println!("The device keypair: {:?}", peer_keypair);
        Peer {
            peer_ip,
            server_socket: server_socket
                .parse()
                .expect("Wrong socket address, please check again!"),
            server_pubkey: Key::from_raw(str2raw(server_pubkey)),
            peer_keypair,
            peer_ssid: String::from("mosquitto-ap"),
            peer_passwd: String::from("12345678"),
        }
    }

    pub fn update_server_pubkey(&mut self, key: String) {
        self.server_pubkey = Key::from_raw(str2raw(key))
    }

    pub fn update_server_socket(&mut self, socket: String) {
        self.server_socket = socket.parse().expect("failed to update socket!")
    }

    pub fn update_peer_ip(&mut self, ip: IpAddr) {
        self.peer_ip = ip;
    }

    pub fn update_peer_passwd(&mut self, passwd: String) {
        self.peer_passwd = passwd;
    }

    pub fn update_peer_ssid(&mut self, ssid: String) {
        self.peer_ssid = ssid;
    }

    pub fn generate_new_key(&mut self) -> KeyPair {
        let new_keypair = KeyPair::generate();
        println!("New keypair: {:?}", new_keypair);
        self.peer_keypair = new_keypair.clone();
        new_keypair
    }

    pub fn update_new_keypair(&mut self, private_key: String, _pubkey: String) {
        self.peer_keypair = KeyPair::from_private(
            Key::from_raw(str2raw(private_key)));
    }

    pub fn init_ap(&self) -> Result<(), Box<dyn Error>> {
        let mut uci = Uci::new()?;
        uci.set("firewall.@zone[1].network", "wwan")?;
        uci.commit("firewall")?;
        Command::new("/etc/init.d/firewall").arg("restart");

        uci.set("network.lan.ipaddr", "192.168.2.1")?;
        uci.set("network.wwan", "interface")?;
        uci.set("network.wwan.proto", "dhcp")?;
        uci.commit("network")?;
        Command::new("/etc/init.d/network").arg("restart");

        uci.set("wireless.wwan", "wifi-iface")?;
        uci.set("wireless.wwan.device", "radio0")?;
        uci.set("wireless.wwan.network", "wlan")?;
        uci.set("wireless.wwan.mode", "sta")?;
        uci.set("wireless.wwan.encryption", "none")?;
        uci.set("wireless.wwan.key", &self.peer_passwd)?;
        uci.set("wireless.wwan.ssid", &self.peer_ssid)?;
        uci.commit("wireless")?;
        Command::new("wifi").arg("reload");

        Ok(())
    }

    pub fn start_wireguard_device(&self) {
        delete_interface(&"mosquitto-wg".parse().unwrap()).expect("No wireguard Interface!");
        boot_wireguard_device(
            &self.peer_ip,
            &self.peer_keypair,
            &self.server_socket,
            &self.server_pubkey,
        )
            .expect("Failed to build device!");
        // init_ap(&self.peer_ssid, &self.peer_passwd).expect("Failed to start ap!");
        println!("update wireguard device success!")
    }

/*    pub fn restart_device(&self) {
        delete_interface(&"mosquitto-wg".parse().unwrap()).expect("No wireguard Interface!");
        boot_device(
            &self.peer_ip,
            &self.peer_keypair,
            &self.server_socket,
            &self.server_pubkey,
        )
            .expect("Failed to renew device!");
        println!("restart device success!")
    }*/

    /// Rewrite the config file
    pub fn overwrite_config(&self) -> &'static str {
        self.start_wireguard_device();
        match self.init_ap() {
            Ok(_) => {
                let value;
                unsafe {
                    value = DEVICE.get_existing_value()
                }
                std::fs::write(
                    "./",
                    serde_json::to_string_pretty(&value).unwrap())
                    .unwrap();
                "success!"
            },
            Err(_) => "failed"
        }
    }

    /// read value from ram
    pub fn get_existing_value(&self) -> Value {
        let peer_ip = self.peer_ip.to_string();
        let server_socket = self.server_socket.to_string();
        let server_pubkey = raw2str(self.server_pubkey.as_bytes());
        let peer_private_key = raw2str(self.peer_keypair.private.as_bytes());
        let peer_pubkey = raw2str(self.peer_keypair.public.as_bytes());
        let peer_ssid = self.peer_ssid.clone();
        let peer_passwd = self.peer_passwd.clone();

        let value: Value = json!({
            "peer_ip": peer_ip,
            "server_socket": server_socket,
            "server_pubkey": server_pubkey,
            "peer_private_key": peer_private_key,
            "peer_pubkey": peer_pubkey,
            "peer_ssid": peer_ssid,
            "peer_passwd": peer_passwd
        });
        value
    }
}