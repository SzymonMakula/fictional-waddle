pub fn get_http_body(data: &str) -> String {
    let body: Vec<_> = data
        .lines()
        .skip_while(|line| !line.is_empty())
        .map(|line| line.trim_matches(char::from(0)))
        .collect();
    String::from_iter(body.into_iter())
}

pub fn get_http_headers(data: &str) -> Vec<Header> {
    let headers: Vec<_> = data
        .lines()
        .skip(1)
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let header_vector: Vec<_> = line.split(":").collect();
            return Header {
                key: String::from(header_vector[0].to_lowercase()),
                value: String::from(header_vector[1].trim_start()),
            };
        })
        .collect();
    headers
}

#[derive(Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
}

pub fn get_socket_key(headers: Vec<Header>) -> Option<String> {
    let socket_key = headers
        .iter()
        .find(|header| header.key.eq("sec-websocket-key"))
        .map(|header| header.value.to_owned());
    let socket_version = headers
        .iter()
        .find(|header| header.key.eq("sec-websocket-version"))
        .map(|header| header.value.to_owned());

    if socket_key.is_none() || socket_version.is_none() {
        return None;
    }
    let is_valid_socket_version = socket_version.unwrap().eq("13");

    if !is_valid_socket_version {
        return None;
    }
    return socket_key;
}
