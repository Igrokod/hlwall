use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use bytes::BytesMut;
use std::convert::TryInto;
use std::io::{self, Cursor, Read, Write};
use tokio_util::codec::{Decoder, Encoder};
use crate::util::ReadStringZExt;

#[derive(Debug)]
pub(crate) enum GoldSourceQuery {
    A2sInfoRequest,
    A2sInfoResponse {
        protocol_version: u8,
        server_name: String,
        map: String,
        folder: String,
        game: String,
        game_steamid: u16,
        players: u8,
        max_players: u8,
        bots: u8,
        server_type: ServerType,
        server_platform: ServerPlatform,
        password_status: PasswordStatus,
        vac_status: VacProtection,
        version: String
    }
}

// 'd' for a dedicated server
// 'l' for a non-dedicated server
// 'p' for a SourceTV relay (proxy)
#[derive(Debug)]
pub(crate) enum ServerType {
    DedicatedServer,
    ListenServer,
    SourceTvRelay
}

impl ServerType {
    fn from_u8(raw: u8) -> Option<ServerType> {
        match raw {
            b'd' => Some(ServerType::DedicatedServer),
            b'l' => Some(ServerType::ListenServer),
            b'p' => Some(ServerType::SourceTvRelay),
            _ => None
        }
    }
}

// 'l' for Linux
// 'w' for Windows
// 'm' or 'o' for Mac (the code changed after L4D1)
#[derive(Debug)]
pub(crate) enum ServerPlatform {
    Linux,
    Windows,
    Mac
}

impl ServerPlatform {
    fn from_u8(raw: u8) -> Option<ServerPlatform> {
        match raw {
            b'l' => Some(ServerPlatform::Linux),
            b'w' => Some(ServerPlatform::Windows),
            b'm' => Some(ServerPlatform::Mac),
            b'o' => Some(ServerPlatform::Mac),
            _ => None
        }
    }
}

#[derive(Debug)]
pub(crate) enum PasswordStatus {
    Public,
    Private
}

impl PasswordStatus {
    fn from_u8(raw: u8) -> Option<PasswordStatus> {
        match raw {
            0 => Some(PasswordStatus::Public),
            1 => Some(PasswordStatus::Private),
            _ => None
        }
    }
}

#[derive(Debug)]
pub(crate) enum VacProtection {
    Unsecured,
    Secured
}

impl VacProtection {
    fn from_u8(raw: u8) -> Option<VacProtection> {
        match raw {
            0 => Some(VacProtection::Unsecured),
            1 => Some(VacProtection::Secured),
            _ => None
        }
    }
}

#[derive(Default)]
pub(crate) struct GoldSourceQueryCodec {}

#[macro_export]
macro_rules! await_bytes {
    ($read_expr:expr) => {
        match $read_expr {
            Ok(v) => v,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };
    };
}

const SINGLE_PACKET_HEADER: i32 = -1;
const SPLIT_PACKET_HEADER: i32 = -2;

const A2S_INFO_REQUEST_HEADER: u8 = b'T';
const A2S_INFO_REQUEST_SIGNATURE: &[u8] = b"Source Engine Query\x00";
const A2S_INFO_RESPONSE_HEADER: u8 = b'I';

fn decode_a2s_info_request<R: Read>(mut reader: R) -> Result<Option<GoldSourceQuery>, io::Error> {
    let mut signature_buf = [0u8; A2S_INFO_REQUEST_SIGNATURE.len()];
    await_bytes!(reader.read_exact(&mut signature_buf));

    if signature_buf != A2S_INFO_REQUEST_SIGNATURE {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Invalid A2S_INFO_REQUEST signature",
        ));
    }

    Ok(Some(GoldSourceQuery::A2sInfoRequest))
}

fn decode_a2s_info_response<R: Read + std::fmt::Debug>(mut reader: R) -> Result<Option<GoldSourceQuery>, io::Error> {
    let protocol_version = await_bytes!(reader.read_u8());
    let server_name = await_bytes!(reader.read_stringz());
    let map = await_bytes!(reader.read_stringz());
    let folder = await_bytes!(reader.read_stringz());
    let game = await_bytes!(reader.read_stringz());
    let game_steamid = await_bytes!(reader.read_u16::<LittleEndian>());
    let players = await_bytes!(reader.read_u8());
    let max_players = await_bytes!(reader.read_u8());
    let bots = await_bytes!(reader.read_u8());
    let raw_server_type = await_bytes!(reader.read_u8());
    let server_type = match ServerType::from_u8(raw_server_type) {
        Some(v) => v,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Invalid Server type received"))
    };
    let raw_server_platform = await_bytes!(reader.read_u8());
    let server_platform = match ServerPlatform::from_u8(raw_server_platform) {
        Some(v) => v,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Invalid Server platform received"))
    };
    let raw_password_status = await_bytes!(reader.read_u8());
    let password_status = match PasswordStatus::from_u8(raw_password_status) {
        Some(v) => v,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Invalid Server password status received"))
    };
    let raw_vac_status = await_bytes!(reader.read_u8());
    let vac_status = match VacProtection::from_u8(raw_vac_status) {
        Some(v) => v,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Invalid Server vac status received"))
    };
    let version = await_bytes!(reader.read_stringz());

    let extra_data_flag = match reader.read_u8() {
        Ok(v) => Some(v),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
        Err(e) => return Err(e)
    };

    dbg!(extra_data_flag);

    if let Some(d) = extra_data_flag {
        dbg!(d & 0x80);
        dbg!(d & 0x10);
        dbg!(d & 0x40);
        dbg!(d & 0x20);
    }

    Ok(Some(GoldSourceQuery::A2sInfoResponse {
        protocol_version,
        server_name,
        map,
        folder,
        game,
        game_steamid,
        players,
        max_players,
        bots,
        server_type,
        server_platform,
        password_status,
        vac_status,
        version
    }))
}

impl Decoder for GoldSourceQueryCodec {
    type Item = GoldSourceQuery;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        dbg!(&src);
        let mut reader = Cursor::new(src);
        let protocol_header = await_bytes!(reader.read_i32::<LittleEndian>());

        match protocol_header {
            SINGLE_PACKET_HEADER => {}
            SPLIT_PACKET_HEADER => bail!("Split packets are not implemented"),
            _ => bail!("Unknown packet header: {}", protocol_header),
        };

        let decode_result = match await_bytes!(reader.read_u8()) {
            A2S_INFO_REQUEST_HEADER => decode_a2s_info_request(&mut reader),
            A2S_INFO_RESPONSE_HEADER => decode_a2s_info_response(&mut reader),
            other => bail!("Request kind {} is not supported", other),
        };

        let packet = match decode_result {
            Ok(Some(v)) => v,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // TODO: Remove panic
        let pos: usize = reader.position().try_into().unwrap();
        let src = reader.into_inner();
        src.split_at(pos);
        Ok(Some(packet))
    }
}

impl Encoder for GoldSourceQueryCodec {
    type Item = GoldSourceQuery;
    type Error = anyhow::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            GoldSourceQuery::A2sInfoRequest => {}
            #[allow(unreachable_patterns)]
            other => bail!("Unsupported item: {:?}", other),
        };

        let buf = vec![];
        let mut writer = Cursor::new(buf);

        let _protocol_header = writer.write_i32::<LittleEndian>(SINGLE_PACKET_HEADER)?;
        let _request_header = writer.write_u8(A2S_INFO_REQUEST_HEADER)?;
        let _request_signature = writer.write(A2S_INFO_REQUEST_SIGNATURE);

        dst.extend_from_slice(&writer.into_inner());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::GoldSourceQuery;
    use super::GoldSourceQueryCodec;
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    const A2S_INFO_REQUEST: &[u8] = b"\xff\xff\xff\xffTSource Engine Query\x00";
    const A2S_INFO_RESPONSE: &[u8] = b"\xff\xff\xff\xffI0Counter-Strike 1.6 Server\0de_dust2\0cstrike\0Counter-Strike\0\n\0\0\x02\0dl\0\x011.1.2.7/Stdio\0\x80\x87i";

    #[test]
    fn test_decoder_empty_request() {
        const EMPTY_REQUEST: &[u8] = b"";
        let mut codec = GoldSourceQueryCodec::default();
        let result = codec.decode(&mut BytesMut::from(EMPTY_REQUEST));
        match result {
            Ok(None) => {}
            _ => panic!("Failed to decode empty request {:?}", result),
        };
    }

    #[test]
    fn test_decoder_invalid_header() {
        const INVALID_HEADER_REQUEST: &[u8] = b"\xff\xff\xff\xfa";
        let mut codec = GoldSourceQueryCodec::default();
        let result = codec.decode(&mut BytesMut::from(INVALID_HEADER_REQUEST));
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_a2s_info_request() {
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

    #[test]
    fn test_decoder_a2s_info_response() {
        let mut codec = GoldSourceQueryCodec::default();
        let result = codec
            .decode(&mut BytesMut::from(A2S_INFO_RESPONSE))
            .expect("Failed to decode A2S_INFO_RESPONSE")
            .expect("Failed to decode A2S_INFO_RESPONSE");

        match result {
            GoldSourceQuery::A2sInfoRequest => {}
            #[allow(unreachable_patterns)]
            other => panic!(
                "Failed to decode A2S_INFO_REQUEST, got instead: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_encoder_a2s_info_request() {
        let mut codec = GoldSourceQueryCodec::default();
        let mut buf = BytesMut::new();

        codec
            .encode(GoldSourceQuery::A2sInfoRequest, &mut buf)
            .expect("Failed to encode A2S_INFO_REQUEST");

        assert_eq!(buf, A2S_INFO_REQUEST);
    }
}
