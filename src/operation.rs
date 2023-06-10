use rust_uci::Uci;
use serde_json::{json, Value};
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::process::Command;
use wireguard_control::{Backend, DeviceUpdate, Key, KeyPair};

pub struct Peer {
    peer_ip: IpAddr,
    server_socket: SocketAddr,
    server_pubkey: Key,
    peer_keypair: KeyPair,
    peer_ssid: String,
    peer_passwd: String,
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
            server_pubkey: Key::from_base64(&*server_pubkey).unwrap(),
            peer_keypair,
            peer_ssid: String::from("mohoo-ap"),
            peer_passwd: String::from("12345678"),
        }
    }

    pub fn update_server_pubkey(&mut self, key: String) {
        self.server_pubkey = Key::from_base64(&*key).unwrap()
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

    pub fn update_new_keypair(&mut self, private_key: String, pubkey: String) {
        let pub_key = Key::from_base64(pubkey.as_str());
        match pub_key {
            Ok(value)=> {
                self.peer_keypair = KeyPair {
                    private: Key::from_base64(private_key.as_str()).unwrap(),
                    public: value,
                }
            }
            Err(_) => {
                self.peer_keypair = KeyPair::from_private(Key::from_base64(&*private_key).unwrap());
            }
        }
    }

    pub fn init_ap(&self) -> Result<(), Box<dyn Error>> {
        println!("Start ap init");
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
        uci.set("wireless.wwan.encryption", "wpa2")?;
        uci.set("wireless.wwan.key", &self.peer_passwd)?;
        uci.set("wireless.wwan.ssid", &self.peer_ssid)?;
        uci.commit("wireless")?;
        Command::new("wifi").arg("down");
        Command::new("wifi").arg("up");
        Ok(())
    }


    pub fn reload_ap(&self) -> Result<(), Box<dyn Error>> {
        println!("AP reloading");
        let mut uci = Uci::new()?;
        uci.set("wireless.wwan.key", &self.peer_passwd)?;
        uci.set("wireless.wwan.ssid", &self.peer_ssid)?;
        uci.commit("wireless")?;
        Command::new("wifi").arg("reload");
        Ok(())
    }


    pub fn start(&self) {
        println!("Start wg init");
        self.boot_wireguard_device()
        .expect("Failed to build device!");
        self.init_ap().expect("Failed to start ap!");
        println!("update wireguard device success!")
    }


    /// Rewrite the config file
    pub fn overwrite_config(&self) -> &'static str {
        self.boot_wireguard_device()
        .expect("Failed to build device!");

        match self.reload_ap() {
            Ok(_) => {
                let value = self.get_existing_value();
                println!("start writing");
                std::fs::write(
                    "./config.json",
                    serde_json::to_string_pretty(&value).unwrap(),
                ).unwrap();
                "success!"
            }
            Err(_) => "failed",
        }
    }


    fn boot_wireguard_device( &self, ) -> std::io::Result<()> {
        DeviceUpdate::new()
            .set_keypair(self.peer_keypair.clone())
            .replace_peers()
            .add_peer_with(&self.server_pubkey, |peer| {
                peer.set_endpoint(self.server_socket.clone())
                    .replace_allowed_ips()
                    .add_allowed_ip(self.peer_ip.clone(), 0)
            })
            .apply(&"mosquitto-wg".parse().unwrap(), Backend::Kernel)
    }


    /// read value from ram
    pub fn get_existing_value(&self) -> Value {
        json!({
            "peer_ip": self.peer_ip,
            "server_socket": self.server_socket,
            "server_pubkey": self.server_pubkey.to_base64(),
            "peer_private_key": self.peer_keypair.private.to_base64(),
            "peer_pubkey": self.peer_keypair.public.to_base64(),
            "peer_ssid": &self.peer_ssid,
            "peer_passwd": &self.peer_passwd
        })
    }
}
