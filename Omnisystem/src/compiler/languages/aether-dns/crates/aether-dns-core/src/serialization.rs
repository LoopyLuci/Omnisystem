/// DNS Wire Format Serialization
/// RFC 1035 compliant DNS message serialization and deserialization

use crate::protocol::{DNSMessage, DNSHeader, DNSFlags, DNSQuestion, DNSRecord, RecordType, DomainName, QueryClass};
use bytes::{BytesMut, BufMut, Buf};
use std::io::Cursor;

pub struct DNSSerializer;

impl DNSSerializer {
    pub fn serialize(message: &DNSMessage) -> anyhow::Result<Vec<u8>> {
        let mut buf = BytesMut::with_capacity(512);

        // Serialize header (12 bytes fixed)
        Self::serialize_header(&message.header, &mut buf)?;

        // Serialize questions
        for question in &message.questions {
            Self::serialize_question(question, &mut buf)?;
        }

        // Serialize answers
        for record in &message.answers {
            Self::serialize_record(record, &mut buf)?;
        }

        // Serialize authorities
        for record in &message.authorities {
            Self::serialize_record(record, &mut buf)?;
        }

        // Serialize additionals
        for record in &message.additionals {
            Self::serialize_record(record, &mut buf)?;
        }

        Ok(buf.to_vec())
    }

    fn serialize_header(header: &DNSHeader, buf: &mut BytesMut) -> anyhow::Result<()> {
        buf.put_u16(header.id);

        let flags_byte = Self::encode_flags(&header.flags);
        buf.put_u16(flags_byte);

        buf.put_u16(header.qdcount as u16);
        buf.put_u16(header.ancount as u16);
        buf.put_u16(header.nscount as u16);
        buf.put_u16(header.arcount as u16);

        Ok(())
    }

    fn encode_flags(flags: &DNSFlags) -> u16 {
        let mut byte = 0u16;
        if flags.qr { byte |= 0x8000; }
        byte |= ((flags.opcode as u16) & 0x0F) << 11;
        if flags.aa { byte |= 0x0400; }
        if flags.tc { byte |= 0x0200; }
        if flags.rd { byte |= 0x0100; }
        if flags.ra { byte |= 0x0080; }
        if flags.ad { byte |= 0x0020; }
        if flags.cd { byte |= 0x0010; }
        byte |= (flags.rcode as u16) & 0x0F;
        byte
    }

    fn serialize_question(question: &DNSQuestion, buf: &mut BytesMut) -> anyhow::Result<()> {
        Self::serialize_domain_name(&question.name, buf)?;
        buf.put_u16(question.qtype as u16);
        buf.put_u16(question.qclass as u16);
        Ok(())
    }

    fn serialize_record(record: &DNSRecord, buf: &mut BytesMut) -> anyhow::Result<()> {
        Self::serialize_domain_name(&record.name, buf)?;
        buf.put_u16(record.rtype as u16);
        buf.put_u16(record.rclass as u16);
        buf.put_u32(record.ttl);

        let rddata = Self::serialize_rdata(&record.rtype, &record.rdata)?;
        buf.put_u16(rddata.len() as u16);
        buf.put_slice(&rddata);

        Ok(())
    }

    fn serialize_domain_name(name: &DomainName, buf: &mut BytesMut) -> anyhow::Result<()> {
        for label in &name.labels {
            buf.put_u8(label.len() as u8);
            buf.put_slice(label.as_bytes());
        }
        buf.put_u8(0); // Root label
        Ok(())
    }

    fn serialize_rdata(_rtype: &RecordType, rdata: &[u8]) -> anyhow::Result<Vec<u8>> {
        // For now, just return the rdata as-is
        // In production, would parse based on record type
        Ok(rdata.to_vec())
    }
}

pub struct DNSDeserializer;

impl DNSDeserializer {
    pub fn deserialize(data: &[u8]) -> anyhow::Result<DNSMessage> {
        let mut cursor = Cursor::new(data);

        let header = Self::deserialize_header(&mut cursor)?;

        let mut questions = Vec::new();
        for _ in 0..header.qdcount {
            questions.push(Self::deserialize_question(&mut cursor)?);
        }

        let mut answers = Vec::new();
        for _ in 0..header.ancount {
            answers.push(Self::deserialize_record(&mut cursor)?);
        }

        let mut authorities = Vec::new();
        for _ in 0..header.nscount {
            authorities.push(Self::deserialize_record(&mut cursor)?);
        }

        let mut additionals = Vec::new();
        for _ in 0..header.arcount {
            additionals.push(Self::deserialize_record(&mut cursor)?);
        }

        Ok(DNSMessage {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
    }

    fn deserialize_header(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<DNSHeader> {
        let id = cursor.get_u16();
        let flags_byte = cursor.get_u16();
        let qdcount = cursor.get_u16();
        let ancount = cursor.get_u16();
        let nscount = cursor.get_u16();
        let arcount = cursor.get_u16();

        let flags = Self::decode_flags(flags_byte);

        Ok(DNSHeader {
            id,
            flags,
            qdcount,
            ancount,
            nscount,
            arcount,
        })
    }

    fn decode_flags(byte: u16) -> DNSFlags {
        DNSFlags {
            qr: (byte & 0x8000) != 0,
            opcode: ((byte >> 11) & 0x0F) as u8,
            aa: (byte & 0x0400) != 0,
            tc: (byte & 0x0200) != 0,
            rd: (byte & 0x0100) != 0,
            ra: (byte & 0x0080) != 0,
            z: false,
            ad: (byte & 0x0020) != 0,
            cd: (byte & 0x0010) != 0,
            rcode: (byte & 0x0F) as u8,
        }
    }

    fn deserialize_question(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<DNSQuestion> {
        let name = Self::deserialize_domain_name(cursor)?;
        let qtype = RecordType::from_u16(cursor.get_u16());
        let qclass_u16 = cursor.get_u16();
        let qclass = match qclass_u16 {
            1 => QueryClass::IN,
            3 => QueryClass::CH,
            4 => QueryClass::HS,
            254 => QueryClass::NONE,
            255 => QueryClass::ANY,
            _ => QueryClass::IN, // Default to IN
        };

        Ok(DNSQuestion {
            name,
            qtype,
            qclass,
        })
    }

    fn deserialize_record(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<DNSRecord> {
        let name = Self::deserialize_domain_name(cursor)?;
        let rtype = RecordType::from_u16(cursor.get_u16());
        let rclass_u16 = cursor.get_u16();
        let rclass = match rclass_u16 {
            1 => QueryClass::IN,
            3 => QueryClass::CH,
            4 => QueryClass::HS,
            254 => QueryClass::NONE,
            255 => QueryClass::ANY,
            _ => QueryClass::IN,
        };
        let ttl = cursor.get_u32();
        let rdlen = cursor.get_u16() as usize;

        let rdata = if rdlen > 0 && cursor.remaining() >= rdlen {
            let data_bytes = &cursor.get_ref()[cursor.position() as usize..cursor.position() as usize + rdlen];
            cursor.advance(rdlen);
            data_bytes.to_vec()
        } else {
            vec![]
        };

        Ok(DNSRecord {
            name,
            rtype,
            rclass,
            ttl,
            rdlen: rdlen as u16,
            rdata,
        })
    }

    fn deserialize_domain_name(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<DomainName> {
        let mut labels = Vec::new();

        loop {
            let len = cursor.get_u8() as usize;
            if len == 0 {
                break;
            }

            if cursor.remaining() < len {
                return Err(anyhow::anyhow!("Invalid domain name"));
            }

            let label_bytes = &cursor.get_ref()[cursor.position() as usize..cursor.position() as usize + len];
            cursor.advance(len);

            labels.push(String::from_utf8_lossy(label_bytes).to_string());
        }

        Ok(DomainName { labels })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut message = DNSMessage::new();
        message.header.id = 1234;
        message.header.flags.rd = true;

        let serialized = DNSSerializer::serialize(&message).unwrap();
        let deserialized = DNSDeserializer::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.header.id, 1234);
        assert!(deserialized.header.flags.rd);
    }

    #[test]
    fn test_encode_decode_flags() {
        let mut flags = DNSFlags::default();
        flags.qr = true;
        flags.rd = true;
        flags.ra = true;

        let encoded = DNSSerializer::encode_flags(&flags);
        let decoded = DNSDeserializer::decode_flags(encoded);

        assert_eq!(decoded.qr, flags.qr);
        assert_eq!(decoded.rd, flags.rd);
        assert_eq!(decoded.ra, flags.ra);
    }
}
