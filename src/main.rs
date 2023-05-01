mod operation;
mod test;
#[macro_use] extern crate rocket;
use std::{net::IpAddr, str::FromStr};
use operation::Peer;
use rocket::serde::{json::Json, Serialize, Deserialize};
use once_cell::sync::Lazy;
use serde_json;

static mut DEVICE: Lazy<Peer> = Lazy::new(||{
    Peer::init(
        IpAddr::from_str("10.10.1.2").unwrap(),
        "223.129.127.2:8889".to_string(),
        "L9pVwwThBs1gGczwGsgUFXROFUkyTFoXEVp5MBkBbkc=".to_string()
    )
});

#[get("/")]
fn index() -> &'static str {
    "hello"
}

#[get("/ping")]
fn ping() -> &'static str {
    "pong!"
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ConfigJson {
    peer_ip: String,
    server_socket: String,
    server_pubkey: String,
    peer_private_key: String,
    peer_pubkey: String,
    peer_ssid: String,
    peer_passwd: String,
}

#[get("/get")]
fn get_config() -> Json<ConfigJson> {
    let answer = match std::fs::File::open("config.json") {
        Ok(mut file) => {
            let mut stdout = std::io::stdout();
            let str = &std::io::copy(&mut file, &mut stdout).unwrap().to_string();
            let data: serde_json::Value = serde_json::from_str(str).unwrap();
            data
        }
        Err(e) => {
            println!("with error: {}, use existing settings", e);
            let res; unsafe { res = DEVICE.get_existing_value() }
            res
        }
    };

    Json(ConfigJson {
        peer_ip: answer["peer_ip"].to_string(),
        server_socket: answer["server_socket"].to_string(),
        server_pubkey: answer["server_pubkey"].to_string(),
        peer_private_key: answer["peer_private_key"].to_string(),
        peer_pubkey: answer["peer_pubkey"].to_string(),
        peer_ssid: answer["peer_ssid"].to_string(),
        peer_passwd: answer["peer_passwd"].to_string(),
    })
}

#[post("/update", format = "json", data = "<config>")]
fn update_config(config: Json<ConfigJson>) -> &'static str {
    unsafe {
        DEVICE.update_peer_ip(
            IpAddr::from_str(&*config.peer_ip).unwrap());
        DEVICE.update_server_socket(
            config.server_socket.clone());
        DEVICE.update_server_pubkey(
            config.peer_pubkey.clone());
        DEVICE.update_peer_ssid(
            config.peer_ssid.clone());
        DEVICE.update_peer_passwd(
            config.peer_passwd.clone());
        DEVICE.update_new_keypair(
            config.peer_private_key.clone(),
            config.peer_pubkey.clone())
    }
    "200"
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct KeyPairConfig {
    pubkey: String,
    private_key: String
}

#[post("/gen")]
fn gen_keypair() -> Json<KeyPairConfig> {
    let keypair;
    unsafe{
        keypair = DEVICE.generate_new_key();
    }
    Json(KeyPairConfig{
        pubkey: keypair.public.to_base64(),
        private_key: keypair.private.to_base64(),
    })
}

#[post("/reload")]
fn reload_config() -> &'static str {
    unsafe {
        DEVICE.overwrite_config()
    }
}

#[launch]
async fn rocket() -> _ {
    println!("Start to read config file to start the service");
    match std::fs::File::open("config.json") {
        Ok(mut file) => {
            let mut stdout = std::io::stdout();
            let str = &std::io::copy(&mut file, &mut stdout).unwrap().to_string();
            let data: serde_json::Value = serde_json::from_str(str).unwrap();
            unsafe {
                DEVICE.update_peer_ip(
                    IpAddr::from_str(&data["peer_ip"].to_string()).unwrap(),);
                DEVICE.update_server_socket(
                    data["server_socket"].to_string());
                DEVICE.update_server_pubkey(
                    data["server_pubkey"].to_string());
                DEVICE.update_new_keypair(
                    data["peer_private_key"].to_string(),
                    data["peer_pubkey"].to_string());
                DEVICE.update_peer_ssid(
                    data["peer_ssid"].to_string());
                DEVICE.update_peer_passwd(
                    data["peer_passwd"].to_string());
                DEVICE.start_wireguard_device();
                DEVICE.init_ap().unwrap()
            }
        }
        Err(e) => {
            println!("with error: {}, use default settings", e);
            unsafe {
                DEVICE.start_wireguard_device();
            }
        }
    }

    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![ping])
        .mount("/config", routes![reload_config])
        .mount("/config", routes![update_config])
        .mount("/config", routes![get_config])
        .mount("/keypair",routes![gen_keypair])
}