pub struct Header {
    msg_type: MsgType,
    length: u16,
    transaction_id: TransactionId,
}

impl Header {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(self.msg_type.to_bytes());
        bytes.extend(&self.length.to_be_bytes());
        bytes.extend(&MAGIC_COOKIE.to_be_bytes());
        bytes.extend(self.transaction_id.to_bytes());

        bytes
    }
}

pub struct MsgType {
    method: Method,
    class: Class,
}

impl MsgType {
    pub fn to_bytes(&self) -> Vec<u8> {
        let method_5bit = self.method.0 & 0b111110000000 << 2;
        let method_3bit = self.method.0 & 0b000001110000 << 1;
        let method_4bit = self.method.0 & 0b000000001111;

        let class_msb = ((self.class.0 & 0b10) as u16) << 7;
        let class_mlb = ((self.class.0 & 0b01) as u16) << 4;

        let msg_type = method_5bit | class_msb | method_3bit | class_mlb | method_4bit;

        let mut bytes = vec![];
        bytes.extend(&msg_type.to_be_bytes());

        bytes
    }
}

pub struct Method(u16);

impl Method {
    pub const BINDING: Self = Self(0b000000000001);
}

pub struct Class(u8);

impl Class {
    pub const REQUEST: Self = Self(0b00);
    pub const INDICATION: Self = Self(0b01);
    pub const SUCCESS_RESPONSE: Self = Self(0b10);
    pub const ERROR_RESPONSE: Self = Self(0b11);
}

const MAGIC_COOKIE: u32 = 0x2112A442;

pub struct TransactionId {
    data: [u8; 12],
}

impl TransactionId {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header() {
        let msg_type = MsgType {
            method: Method::BINDING,
            class: Class::ERROR_RESPONSE,
        };

        let length = 1000;
        let transaction_id = TransactionId {
            data: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        };

        let header = Header {
            msg_type,
            length,
            transaction_id,
        };

        let bytes = header.to_bytes();

        assert_eq!(bytes[0..2], 0b0000000100010001_u16.to_be_bytes());
        assert_eq!(bytes[2..4], 1000_u16.to_be_bytes());
        assert_eq!(bytes[4..8], MAGIC_COOKIE.to_be_bytes());
        assert_eq!(bytes[8..], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    }
}
