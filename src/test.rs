#[cfg(test)]
mod test {
    use wireguard_control::Key;

    #[test]
    fn test_get_value() {
        use std::net::IpAddr;
        use std::str::FromStr;
        use crate::operation::Peer;
        let peer =
            Peer::init(
                IpAddr::from_str("10.10.1.2").unwrap(),
                "223.129.127.2:8889".to_string(),
                "L9pVwwThBs1gGczwGsgUFXROFUkyTFoXEVp5MBkBbkc=".to_string()
            );

        println!("{:?}",peer.get_existing_value());
    }

    #[test]
    fn test_to_base64() {
        use wireguard_control::Key;
        let key = Key::from_base64("QC+HObWWhTztQVVlEyimn7PlQEIpi8/7IKWb9r8n7Vc=");

        let res = key.unwrap().to_base64();

        assert_eq!("QC+HObWWhTztQVVlEyimn7PlQEIpi8/7IKWb9r8n7Vc=", res)
    }
}