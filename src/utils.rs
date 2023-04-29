use std::process::exit;

pub fn str2raw(key: String) -> [u8; 32] {
    let mut raw = [0; 32];
    if key.len() > 32 || key.len() == 0 {
        exit(-1)
    }
    for (tag, c) in key.chars().enumerate() {
        raw[tag] = c as u8
    }
    raw
}

pub fn raw2str(key: &[u8]) -> String {
    let mut str = String::from("");
    if key.len() > 32 || key.len() == 0 {
        exit(-1)
    }
    for num in key.iter() {
        str.push(*num as char)
    }
    str
}