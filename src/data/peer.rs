#[allow(dead_code)]

#[derive(Debug)]
pub struct Peer {
    pub address: String,
    pub retry_count: i8,
}

impl Peer {
    pub fn new(address:String) -> Self {
        Self {
            address,
            retry_count: 0,
        }
    }
}
