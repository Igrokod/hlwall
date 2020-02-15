use std::io;

pub(crate) trait ReadStringZExt: io::Read {
    fn read_stringz(&mut self) -> io::Result<String> {
        let mut string_vec = vec![];
        let mut buf = [0u8; 1];

        loop {
            match self.read(&mut buf) {
                Ok(n) if n == 0 => return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "No null byte found in reader",
                )),
                Ok(_) => {
                    if buf[0] == b'\x00' {
                        break;
                    } else {
                        string_vec.push(buf[0])
                    }
                }
                Err(e) => return Err(e),
            };
        }

        Ok(String::from_utf8_lossy(&string_vec).to_string())
    }
}

impl<R: io::Read + ?Sized> ReadStringZExt for R {}

#[cfg(test)]
mod tests {
    use super::ReadStringZExt;
    use std::io;

    #[test]
    fn test_read_empty_string() {
        let mut reader = io::Cursor::new(b"");

        match reader.read_stringz() {
            Ok(some) => panic!("found stringz where it should not be: {}", some),
            Err(e) => {
                if e.kind() != io::ErrorKind::InvalidInput && e.to_string() != "No null byte found in reader" {
                    panic!("Wrong error received: {}", e)
                }
            }
        };
    }

    #[test]
    fn test_read_string_without_zero() {
        let mut reader = io::Cursor::new(b"Some data");

        match reader.read_stringz() {
            Ok(some) => panic!("found stringz where it should not be: {}", some),
            Err(e) => {
                if e.kind() != io::ErrorKind::InvalidInput && e.to_string() != "No null byte found in reader" {
                    panic!("Wrong error received: {}", e)
                }
            }
        };
    }

    #[test]
    fn test_read_stringz() {
        let mut reader = io::Cursor::new(b"Some data \x00skipped");

        match reader.read_stringz() {
            Ok(some) => assert_eq!(some, "Some data "),
            Err(e) => panic!("Received error when should have not: {}", e)
        };
    }
}
