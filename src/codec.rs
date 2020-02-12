use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt};
use bytes::BytesMut;
use std::io::{self, Cursor, Read};
use tokio_util::codec::Decoder;

#[derive(Debug)]
pub(crate) enum GoldSourceQuery {
    A2sInfoRequest,
}

#[derive(Default)]
pub(crate) struct GoldSourceQueryCodec {}

#[macro_export]
macro_rules! await_bytes {
    ($read_expr:expr) => {
        match $read_expr {
            Ok(v) => v,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };
    };
}

const SINGLE_PACKET_HEADER: i32 = -1;
const SPLIT_PACKET_HEADER: i32 = -2;

const A2S_INFO_REQUEST_HEADER: u8 = b'T';
const A2S_INFO_REQUEST_SIGNATURE: &[u8] = b"Source Engine Query";

impl Decoder for GoldSourceQueryCodec {
    type Item = GoldSourceQuery;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut reader = Cursor::new(src);
        let protocol_header = await_bytes!(reader.read_i32::<LittleEndian>());

        match protocol_header {
            SINGLE_PACKET_HEADER => {}
            SPLIT_PACKET_HEADER => bail!("Split packets are not implemented"),
            _ => bail!("Unknown packet header: {}", protocol_header),
        };

        let request_header = await_bytes!(reader.read_u8());

        if request_header != A2S_INFO_REQUEST_HEADER {
            bail!("Request kind {} is not supported", request_header);
        }

        let mut signature_buf = [0u8; A2S_INFO_REQUEST_SIGNATURE.len()];
        await_bytes!(reader.read_exact(&mut signature_buf));

        if signature_buf != A2S_INFO_REQUEST_SIGNATURE {
            bail!("Invalid A2S_INFO_REQUEST signature");
        }

        Ok(Some(GoldSourceQuery::A2sInfoRequest))
    }
}

#[cfg(test)]
mod tests {
    use super::GoldSourceQuery;
    use super::GoldSourceQueryCodec;
    use bytes::BytesMut;
    use tokio_util::codec::Decoder;

    #[test]
    fn test_empty_request() {
        const EMPTY_REQUEST: &[u8] = b"";
        let mut codec = GoldSourceQueryCodec::default();
        let result = codec.decode(&mut BytesMut::from(EMPTY_REQUEST));
        match result {
            Ok(None) => {}
            _ => panic!("Failed to decode empty request {:?}", result),
        };
    }

    #[test]
    fn test_invalid_header() {
        const INVALID_HEADER_REQUEST: &[u8] = b"\xff\xff\xff\xfa";
        let mut codec = GoldSourceQueryCodec::default();
        let result = codec.decode(&mut BytesMut::from(INVALID_HEADER_REQUEST));
        assert!(result.is_err());
    }

    #[test]
    fn test_a2s_info_request() {
        const A2S_INFO_REQUEST: &[u8] = b"\xff\xff\xff\xffTSource Engine Query\x00";
        let mut codec = GoldSourceQueryCodec::default();
        let result = codec
            .decode(&mut BytesMut::from(A2S_INFO_REQUEST))
            .expect("Failed to decode A2S_INFO_REQUEST")
            .expect("Failed to decode A2S_INFO_REQUEST");

        match result {
            GoldSourceQuery::A2sInfoRequest => {}
            #[allow(unreachable_patterns)]
            other => panic!(
                "Failed to decode A2S_INFO_REQUEST, got instead: {:?}",
                other
            ),
        }
    }
}
