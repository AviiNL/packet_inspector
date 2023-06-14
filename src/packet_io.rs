use bytes::BufMut;
use bytes::BytesMut;
use std::io;
use std::io::ErrorKind;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use valence_core::__private::VarInt;
use valence_core::protocol::decode::{PacketDecoder, PacketFrame};
use valence_core::protocol::encode::PacketEncoder;
use valence_core::protocol::Decode;
use valence_core::protocol::Encode;
use valence_core::protocol::Packet;

#[derive(Clone, Debug)]
pub enum Direction {
    Clientbound,
    Serverbound,
}

pub(crate) struct PacketIoReader {
    reader: tokio::io::ReadHalf<tokio::net::TcpStream>,
    dec: PacketDecoder,
    threshold: Option<u32>,
    direction: Direction,
}

impl PacketIoReader {
    pub(crate) async fn recv_packet_raw(&mut self) -> anyhow::Result<PacketFrame> {
        loop {
            if let Some(frame) = self.dec.try_next_packet()? {
                // self.logger
                //     .log("Unknown".to_string(), self.direction.clone(), frame.clone());

                return Ok(frame.clone());
            }

            self.dec.reserve(READ_BUF_SIZE);
            let mut buf = self.dec.take_capacity();

            if self.reader.read_buf(&mut buf).await? == 0 {
                tracing::error!("EOF {:?}", self.direction);
                return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
            }

            // This should always be an O(1) unsplit because we reserved space earlier and
            // the call to `read_buf` shouldn't have grown the allocation.
            self.dec.queue_bytes(buf);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_compression(&mut self, threshold: Option<u32>) {
        self.threshold = threshold;
        self.dec.set_compression(threshold);
    }
}

pub(crate) struct PacketIoWriter {
    writer: tokio::io::WriteHalf<tokio::net::TcpStream>,
    enc: PacketEncoder,
    threshold: Option<u32>,
}

impl PacketIoWriter {
    pub(crate) async fn send_packet_raw(&mut self, frame: &PacketFrame) -> anyhow::Result<()> {
        let id_buf = varint_to_bytes(VarInt(frame.id));

        let packet_length = id_buf.len() + frame.body.len();

        if let Some(threshold) = self.threshold {
            let bytes = self.compress(frame, threshold)?;

            self.enc.append_bytes(&bytes);

            let bytes = self.enc.take();

            self.writer.write_all(&bytes).await?;

            return Ok(());
        }

        let length = varint_to_bytes(VarInt(packet_length as i32));

        // the frame should be uncompressed at this point.
        self.enc.append_bytes(&length);
        self.enc.append_bytes(&id_buf);
        self.enc.append_bytes(&frame.body);

        let bytes = self.enc.take();

        self.writer.write_all(&bytes).await?;

        Ok(())
    }

    fn compress(&mut self, frame: &PacketFrame, threshold: u32) -> anyhow::Result<BytesMut> {
        use std::io::Read;

        use flate2::bufread::ZlibEncoder;
        use flate2::Compression;

        self.enc.clear();

        let id_buf = varint_to_bytes(VarInt(frame.id));
        let packet_length = id_buf.len() + frame.body.len();

        let mut uncompressed = BytesMut::new();
        uncompressed.extend(&id_buf);
        uncompressed.extend(&frame.body);

        if packet_length < threshold as usize {
            // TODO: Im probably doing something very fucking stupid here...
            let mut bytes = BytesMut::new();

            let packet_len = varint_to_bytes(VarInt(frame.body.len() as i32));
            let packet_length = varint_to_bytes(VarInt(0 as i32));

            bytes.extend(&packet_len);
            bytes.extend(&packet_length);
            bytes.extend(&uncompressed);

            return Ok(bytes);
        }

        let mut bytes = BytesMut::new();
        let mut compressed = Vec::new();

        let data_len_size = VarInt(packet_length as i32).written_size();
        let mut z = ZlibEncoder::new(&uncompressed[..], Compression::new(4));

        let packet_len = data_len_size + z.read_to_end(&mut compressed)?;
        drop(z);

        let packet_len = varint_to_bytes(VarInt(packet_len as i32));
        let packet_length = varint_to_bytes(VarInt(packet_length as i32));

        bytes.extend(&packet_len);
        bytes.extend(&packet_length);
        bytes.extend(&compressed);

        return Ok(bytes);
    }

    #[allow(dead_code)]
    pub(crate) fn set_compression(&mut self, threshold: Option<u32>) {
        self.threshold = threshold;
        self.enc.set_compression(threshold);
    }
}

pub(crate) struct PacketIo {
    stream: TcpStream,
    enc: PacketEncoder,
    dec: PacketDecoder,
    threshold: Option<u32>,
    direction: Direction,
}

const READ_BUF_SIZE: usize = 1024;

impl PacketIo {
    pub(crate) fn new(stream: TcpStream, direction: Direction) -> Self {
        Self {
            stream: stream,
            enc: PacketEncoder::new(),
            dec: PacketDecoder::new(),
            threshold: None,
            direction,
        }
    }

    pub fn split(self) -> (PacketIoReader, PacketIoWriter) {
        let (reader, writer) = tokio::io::split(self.stream);

        (
            PacketIoReader {
                reader,
                dec: self.dec,
                threshold: self.threshold,
                direction: self.direction,
            },
            PacketIoWriter {
                writer,
                enc: self.enc,
                threshold: self.threshold,
            },
        )
    }

    #[allow(dead_code)]
    pub(crate) async fn set_compression(&mut self, threshold: Option<u32>) {
        self.threshold = threshold;
        self.enc.set_compression(threshold);
        self.dec.set_compression(threshold);
    }
}

fn varint_to_bytes(i: VarInt) -> BytesMut {
    let mut buf = BytesMut::new();
    let mut writer = (&mut buf).writer();
    i.encode(&mut writer).unwrap();

    buf
}
