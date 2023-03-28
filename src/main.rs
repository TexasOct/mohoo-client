mod operation;
#[macro_use]
extern crate rocket;
use std::{net::IpAddr, str::FromStr};
use operation::{init_ap, DEVICE, raw2str};
use rocket::serde::{json::Json, Serialize, Deserialize};
use serde_json;

#[get("/")]
fn index() -> &'static str {
    ""
}

#[get("/ping")]
fn ping() -> &'static str {
    "pong!"
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    peer_ip: String,
    server_socket: String,
    server_pubkey: String,
    peer_private_key: String,
    peer_pubkey: String,
    peer_ssid: String,
    peer_passwd: String,
}

#[get("/get/config")]
fn get_config() -> Json<Config> {
    let answer = match std::fs::File::open("config.json") {
        Ok(mut file) => {
            let mut stdout = std::io::stdout();
            let str = &std::io::copy(&mut file, &mut stdout).unwrap().to_string();
            let data: serde_json::Value = serde_json::from_str(str).unwrap();
            data
        }
        Err(e) => {
            println!("with error: {}, use existing settings", e);
            let res;
            unsafe { res = DEVICE.get_existing_value() }
            res
        }
    };

    Json(Config {
        peer_ip: answer["peer_ip"].to_string(),
        server_socket: answer["server_socket"].to_string(),
        server_pubkey: answer["server_pubkey"].to_string(),
        peer_private_key: answer["server_private_key"].to_string(),
        peer_pubkey: answer["server_pubkey"].to_string(),
        peer_ssid: answer["peer_ssid"].to_string(),
        peer_passwd: answer["peer_passwd"].to_string(),
    })
}

#[post("/update/config", format = "json", data = "<config>")]
fn update_config(config: Json<Config>) -> &'static str {
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

#[post("/update/config")]
fn gen_new_keypair() -> Json<KeyPairConfig> {
    let keypair;
    unsafe{
        keypair = DEVICE.generate_new_key();
    }
    Json(KeyPairConfig{
        pubkey: raw2str(keypair.public.as_bytes()),
        private_key: raw2str(keypair.private.as_bytes()),
    })
}

#[post("/update/reload")]
fn reload_config() -> &'static str {
    unsafe {
        DEVICE.start_device()
    }

    match init_ap("test", "test1") {
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

#[launch]
fn rocket() -> _ {
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
                DEVICE.start_device();
            }
        }
        Err(e) => {
            println!("with error: {}, use default settings", e);
            unsafe {
                DEVICE.start_device()
            }
        }
    }

    rocket::build()
        .mount("/", routes![index])
        .mount("/ping", routes![ping])
        .mount("/update/config", routes![update_config])
        .mount("/update/reload", routes![reload_config])
        .mount("/get/config", routes![get_config])
        .mount("/update/keypair",routes![gen_new_keypair])
}