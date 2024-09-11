use super::{
    BulkString, RespArray, RespEncode, RespMap, RespNull, RespNullArray, RespNullBulkString,
    RespSet, SimpleError, SimpleString,
};

const BUF_CAP: usize = 4096;

impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}

impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self.0);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}

impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        // format!(",{:+e}\r\n",self).into_bytes()
        let mut buf = Vec::with_capacity(64);
        let ret = if self.abs() >= 1e+8 {
            format!(",{:e}\r\n", self)
        } else {
            let sign = if self < 0.0 { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.0.len()).into_bytes());
        for (k, v) in self.0 {
            buf.extend_from_slice(&SimpleString::new(k).encode());
            buf.extend_from_slice(&v.encode());
        }
        buf
    }
}
impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

#[cfg(test)]
mod tests {

    use crate::RespFrame;

    use super::*;
    #[test]
    fn test_simple_string_encode() {
        let frame: RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }
    #[test]
    fn test_simple_error_encode() {
        let frame: RespFrame = SimpleError::new("Error message".to_string()).into();
        assert_eq!(frame.encode(), b"-Error message\r\n");
    }
    #[test]
    fn test_integer_encode() {
        let frame: RespFrame = 123456789.into();
        assert_eq!(frame.encode(), b":+123456789\r\n");
        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n");
    }
    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"Hello".to_vec()).into();
        assert_eq!(frame.encode(), b"$5\r\nHello\r\n");
    }
    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = RespNullBulkString.into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }
    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray(vec![
            SimpleString::new("Set".to_string()).into(),
            SimpleString::new("Hello".to_string()).into(),
            SimpleString::new("World".to_string()).into(),
        ])
        .into();
        assert_eq!(frame.encode(), b"*3\r\n+Set\r\n+Hello\r\n+World\r\n");
    }
    #[test]
    fn test_null_array_encode() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }
    #[test]
    fn test_boolean_encode() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");
        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }
    #[test]
    fn test_double_encode() {
        let frame: RespFrame = 1.23456.into();
        assert_eq!(frame.encode(), b",+1.23456\r\n");
        let frame: RespFrame = (-123.456).into();
        assert_eq!(frame.encode(), b",-123.456\r\n");
        let frame: RespFrame = 1e+8.into();
        assert_eq!(frame.encode(), b",1e8\r\n");
        let frame: RespFrame = (-1e+8).into();
        assert_eq!(frame.encode(), b",-1e8\r\n");
    }
    #[test]
    fn test_map_encode() {
        let mut frame = RespMap::new();
        frame.insert("a".to_string(), (1).into());
        frame.insert("b".to_string(), (1234.567).into());
        // assert_eq!(String::from_utf8_lossy(&frame.encode()),"%2\r\n+a\r\n:+1\r\n+b\r\n,+1234.567\r\n");
        assert_eq!(frame.encode(), b"%2\r\n+a\r\n:+1\r\n+b\r\n,+1234.567\r\n");
    }
    #[test]
    fn test_set_encode() {
        let frame = RespSet::new(vec![1.into(), 2.into(), 3.into()]);
        // assert_eq!(String::from_utf8_lossy(&frame.encode()),"~3\r\n:+1\r\n:+2\r\n:+3\r\n");
        assert_eq!(frame.encode(), b"~3\r\n:+1\r\n:+2\r\n:+3\r\n");
    }
}
