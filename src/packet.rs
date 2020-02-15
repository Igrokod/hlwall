use byteorder::{ByteOrder, LittleEndian};
use std::convert::TryFrom;

#[derive(Debug)]
pub(crate) enum GoldSrcPacket {
    A2sInfoRequest,
}

const SINGLE_PACKET: i32 = -1;

const A2S_INFO_REQUEST: u8 = b'T';

#[derive(thiserror::Error, Debug)]
pub enum PacketParseError {
    #[error("Mailformed packet")]
    MailformedPacket,
    #[error("Unsupported packet split status. Only single packets are supported")]
    UnsupportedSplitStatus,
    #[error("Unsupported packet type: {0:x}")]
    UnsupportedPacketType(u8),
}

impl TryFrom<&[u8]> for GoldSrcPacket {
    type Error = PacketParseError;

    fn try_from(body: &[u8]) -> Result<Self, Self::Error> {
        if body.len() <= 4 {
            return Err(PacketParseError::MailformedPacket);
        }

        let packet_split_status = LittleEndian::read_i32(&body[0..4]);
        if packet_split_status != SINGLE_PACKET {
            return Err(PacketParseError::UnsupportedSplitStatus);
        }

        let request_kind = body[4];
        match request_kind {
            A2S_INFO_REQUEST => Ok(GoldSrcPacket::A2sInfoRequest),
            _ => Err(PacketParseError::UnsupportedPacketType(request_kind)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GoldSrcPacket;
    use std::convert::TryFrom;

    #[test]
    fn test_mailformed_packet() {
        const MAILFORMED_PACKETS: &[&[u8]] = &[
            b"",
            b"\xff\xff\xff",
            b"\xff\xff\xff\xff",
            b"\xff\xff\xff\xffZ",
        ];

        for mailformed_packet in MAILFORMED_PACKETS {
            match GoldSrcPacket::try_from(*mailformed_packet) {
                Err(_) => {}
                other => panic!("Expected parse error, got: {:?}", other),
            }
        }
    }

    #[test]
    fn test_a2s_info_request() {
        const A2S_INFO_REQUEST: &[u8] = b"\xff\xff\xff\xffT";

        match GoldSrcPacket::try_from(A2S_INFO_REQUEST) {
            Ok(GoldSrcPacket::A2sInfoRequest) => {}
            other => panic!("A2S_INFO_REQUEST Deserialization fail: {:?}", other),
        }
    }
}