use std::process::exit;
use std::str::from_utf8;

// pub fn str2raw(key: String) -> [u8; 32] {
//     let key_bytes = key.as_bytes();
//     if key_bytes.len() > 44 || key.is_empty() {
//         exit(-1)
//     }
//
//     let mut raw = [0;32];
//     raw[..key_bytes.len()].copy_from_slice(key_bytes);
//     raw
// }

pub fn raw2str(key: &[u8]) -> String {
    if key.len() > 44 || key.len() == 0 {
        exit(-1)
    }
    let res = from_utf8(&key[..key.len()]).map(|s| s.to_string());
    match res {
        Ok(s) => s,
        Err(_) => "null".to_string()
    }
}