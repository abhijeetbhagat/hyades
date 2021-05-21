use rand::{thread_rng, Rng};

#[derive(Clone, Debug)]
pub struct Cookie {
    internal: Vec<u8>,
}

impl PartialEq for Cookie {
    fn eq(&self, other: &Self) -> bool {
        self.internal == other.internal
    }
}

impl Cookie {
    pub fn new() -> Self {
        let mut buf = [0u8; 4];
        thread_rng().fill(&mut buf);

        Self {
            internal: buf.to_vec(),
        }
    }

    pub fn len(&self) -> usize {
        self.internal.len()
    }
}

impl From<&Cookie> for Vec<u8> {
    fn from(cookie: &Cookie) -> Self {
        cookie.internal.clone()
    }
}

impl From<Vec<u8>> for Cookie {
    fn from(buf: Vec<u8>) -> Self {
        Self { internal: buf }
    }
}

impl From<&Vec<u8>> for Cookie {
    fn from(buf: &Vec<u8>) -> Self {
        Self {
            internal: buf.clone(),
        }
    }
}

impl From<&[u8]> for Cookie {
    fn from(buf: &[u8]) -> Self {
        Self {
            internal: buf.to_vec()
        }
    }
}
/*
pub fn gen_cookie() {
    let rng = rand::SystemRandom::new();

    let key = hmac::Key::generate(hmac::HMAC_SHA256, &rng)?;
}
*/
