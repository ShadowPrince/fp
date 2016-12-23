use std::io::Read;
use std::iter::Iterator;

pub struct Stream<'a, T> {
    buf: Box<Vec<u8>>,
    last_check_position: usize,
    pub stream: T,
    pub token: &'a str,
}

impl<'a, T: Read> Stream<'a, T> {
    pub fn new(token: &str, stream: T) -> Stream<T> {
        let buf = Box::new(Vec::<u8>::new());

        Stream {
            buf: buf,
            last_check_position: 0,
            stream: stream,
            token: token,
        }
    }

    fn buffer_fill(&mut self) -> bool {
        let mut bytes = [0u8; 1024];
        match self.stream.read(&mut bytes) {
            Ok(n) if n > 0 => {
                self.buf.append(&mut bytes[..n].to_vec());
                true
            },
            _ => {
                false
            },
        }
    }

    fn buffer_token_position(&mut self) -> Option<usize> {
        let match_bytes = self.token.as_bytes();

        for i in self.last_check_position..self.buf.len() {
            let slice = &self.buf[i..(i+match_bytes.len())];
            if slice == match_bytes {
                self.last_check_position = 0;
                return Some(i);
            }
        }

        self.last_check_position = self.buf.len();
        None
    }
}

impl<'a, T: Read> Iterator for Stream<'a, T> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.buffer_token_position() {
                Some(index) => {
                    let mut result = Vec::new();

                    // TODO: faster
                    for _ in 0..index {
                        result.push(self.buf[0]);
                        self.buf.remove(0);
                    }

                    // remove token
                    self.buf.remove(0);
                    return Some(result);
                },
                _ => {
                    if !self.buffer_fill() {
                        if self.buf.len() > 0 {
                            let mut result = Vec::new();
                            result.append(&mut *self.buf);
                            return Some(result);
                        } else {
                            return None;
                        }
                    }
                },
            }
        }
    }
}
