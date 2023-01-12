use crate::consts::frames::MAGIC_STRING;

pub fn encode_accept_key(socket_key: &str) -> String {
    let mut hasher = sha1_smol::Sha1::new();
    hasher.update(socket_key.as_bytes());
    hasher.update(MAGIC_STRING.as_bytes());

    base64::encode(hasher.digest().bytes())
}

