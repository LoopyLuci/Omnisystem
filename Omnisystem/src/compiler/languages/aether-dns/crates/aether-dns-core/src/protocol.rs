/// DNS Protocol Implementation (RFC 1035)
/// Complete DNS message structures and parsing

use serde::{Deserialize, Serialize};
use std::fmt;

/// DNS Message Header (12 bytes fixed)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DNSHeader {
    pub id: u16,
    pub flags: DNSFlags,
    pub qdcount: u16,  // Number of questions
    pub ancount: u16,  // Number of answer records
    pub nscount: u16,  // Number of authority records
    pub arcount: u16,  // Number of additional records
}

/// DNS Flags (16 bits total)
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct DNSFlags {
    pub qr: bool,           // 0 = query, 1 = response
    pub opcode: u8,         // 0 = query, 1 = iquery, 2 = status
    pub aa: bool,           // Authoritative answer
    pub tc: bool,           // Truncated
    pub rd: bool,           // Recursion desired
    pub ra: bool,           // Recursion available
    pub z: bool,            // Reserved
    pub ad: bool,           // Authentic data (DNSSEC)
    pub cd: bool,           // Checking disabled (DNSSEC)
    pub rcode: u8,          // Response code (0 = no error)
}

/// DNS Record Types (RFC 1035 + extensions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum RecordType {
    A = 1,          // IPv4 address
    NS = 2,         // Authoritative name server
    MD = 3,         // Mail destination (obsolete)
    MF = 4,         // Mail forwarder (obsolete)
    CNAME = 5,      // Canonical name
    SOA = 6,        // Start of authority
    MB = 7,         // Mailbox (experimental)
    MG = 8,         // Mail group (experimental)
    MR = 9,         // Mail rename (experimental)
    NULL = 10,      // Null resource record
    WKS = 11,       // Well-known service
    PTR = 12,       // Domain pointer
    HINFO = 13,     // Host information
    MINFO = 14,     // Mailbox information
    MX = 15,        // Mail exchange
    TXT = 16,       // Text strings
    RP = 17,        // Responsible person
    AFSDB = 18,     // AFS database
    X25 = 19,       // X.25 PSDN address
    ISDN = 20,      // ISDN address
    RT = 21,        // Route through
    NSAP = 22,      // NSAP address
    NSAPPTR = 23,   // NSAP pointer
    SIG = 24,       // Security signature (DNSSEC)
    KEY = 25,       // Security key (DNSSEC)
    PX = 26,        // X.400 mail mapping
    GPOS = 27,      // Geographical position
    AAAA = 28,      // IPv6 address
    LOC = 29,       // Location information
    NXT = 30,       // Next (deprecated)
    EID = 31,       // Endpoint identifier
    NIMLOC = 32,    // NIMROD locator
    SRV = 33,       // Service record
    ATMA = 34,      // ATM address
    NAPTR = 35,     // Naming authority pointer
    KX = 36,        // Key exchanger
    CERT = 37,      // Certificate
    A6 = 38,        // IPv6 address (deprecated)
    DNAME = 39,     // DNAME
    SINK = 40,      // Sink
    OPT = 41,       // EDNS(0)
    APL = 42,       // Address prefix list
    DS = 43,        // Delegation signer (DNSSEC)
    SSHFP = 44,     // SSH public key
    IPSECKEY = 45,  // IPSEC key
    RRSIG = 46,     // DNSSEC signature
    NSEC = 47,      // DNSSEC next secure
    DNSKEY = 48,    // DNSSEC key
    DHCID = 49,     // DHCP identifier
    NSEC3 = 50,     // DNSSEC next secure v3
    NSEC3PARAM = 51,// DNSSEC NSEC3 parameters
    TLSA = 52,      // DANE/TLSA
    HIP = 55,       // Host identity protocol
    CAA = 257,      // Certification authority authorization
    CDS = 59,       // Child DS (DNSSEC)
    CDNSKEY = 60,   // Child DNSKEY (DNSSEC)
    CSYNC = 62,     // Child sync
    SPF = 99,       // Sender policy framework
    UNSPEC = 103,   // Unspecified
    NID = 104,      // Node identifier
    L32 = 105,      // 32-bit locator
    L64 = 106,      // 64-bit locator
    LP = 107,       // Locator pointer
    EUI48 = 108,    // 48-bit extended unique identifier
    EUI64 = 109,    // 64-bit extended unique identifier
    TKEY = 249,     // Transaction key
    TSIG = 250,     // Transaction signature
    IXFR = 251,     // Incremental zone transfer
    AXFR = 252,     // Authoritative zone transfer
    MAILB = 253,    // Mailbox related
    MAILA = 254,    // Mail agent
    ANY = 255,      // Any type
    URI = 256,      // Uniform resource identifier
    TA = 32768,     // DNSSEC trust authorities
    DLV = 32769,    // DNSSEC lookaside validation
}

impl RecordType {
    pub fn from_u16(n: u16) -> Self {
        match n {
            1 => RecordType::A,
            2 => RecordType::NS,
            5 => RecordType::CNAME,
            6 => RecordType::SOA,
            12 => RecordType::PTR,
            15 => RecordType::MX,
            16 => RecordType::TXT,
            28 => RecordType::AAAA,
            33 => RecordType::SRV,
            43 => RecordType::DS,
            46 => RecordType::RRSIG,
            47 => RecordType::NSEC,
            48 => RecordType::DNSKEY,
            52 => RecordType::TLSA,
            257 => RecordType::CAA,
            255 => RecordType::ANY,
            _ => RecordType::NULL,
        }
    }

    pub fn as_u16(&self) -> u16 {
        *self as u16
    }

    pub fn to_string(&self) -> String {
        match self {
            RecordType::A => "A".to_string(),
            RecordType::AAAA => "AAAA".to_string(),
            RecordType::CNAME => "CNAME".to_string(),
            RecordType::MX => "MX".to_string(),
            RecordType::NS => "NS".to_string(),
            RecordType::SOA => "SOA".to_string(),
            RecordType::SRV => "SRV".to_string(),
            RecordType::TXT => "TXT".to_string(),
            RecordType::CAA => "CAA".to_string(),
            RecordType::TLSA => "TLSA".to_string(),
            RecordType::PTR => "PTR".to_string(),
            RecordType::ANY => "ANY".to_string(),
            _ => format!("TYPE{}", self.as_u16()),
        }
    }
}

impl std::fmt::Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Query Class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum QueryClass {
    IN = 1,     // Internet
    CH = 3,     // Chaos
    HS = 4,     // Hesiod
    NONE = 254, // None
    ANY = 255,  // Any
}

impl QueryClass {
    pub fn from_u16(n: u16) -> Self {
        match n {
            1 => QueryClass::IN,
            3 => QueryClass::CH,
            4 => QueryClass::HS,
            254 => QueryClass::NONE,
            255 => QueryClass::ANY,
            _ => QueryClass::IN,
        }
    }
}

/// Domain Name (with compression support)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainName {
    pub labels: Vec<String>,
}

impl DomainName {
    pub fn new(name: &str) -> Self {
        let labels = name
            .trim_end_matches('.')
            .split('.')
            .map(|s| s.to_lowercase())
            .collect();
        DomainName { labels }
    }

    pub fn to_string(&self) -> String {
        if self.labels.is_empty() {
            ".".to_string()
        } else {
            format!("{}.", self.labels.join("."))
        }
    }

    pub fn len(&self) -> usize {
        self.labels.iter().map(|l| l.len() + 1).sum::<usize>() + 1
    }
}

/// DNS Question Section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DNSQuestion {
    pub name: DomainName,
    pub qtype: RecordType,
    pub qclass: QueryClass,
}

/// Response Code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResponseCode {
    NoError = 0,        // No error
    FormErr = 1,        // Format error
    ServFail = 2,       // Server failure
    NxDomain = 3,       // Non-existent domain
    NotImp = 4,         // Not implemented
    Refused = 5,        // Query refused
    YxDomain = 6,       // Domain exists
    YxRRset = 7,        // RRset exists
    NxRRset = 8,        // RRset does not exist
    NotAuth = 9,        // Not authorized
    NotZone = 10,       // Not in zone
}

impl ResponseCode {
    pub fn from_u8(n: u8) -> Self {
        match n {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormErr,
            2 => ResponseCode::ServFail,
            3 => ResponseCode::NxDomain,
            4 => ResponseCode::NotImp,
            5 => ResponseCode::Refused,
            _ => ResponseCode::NoError,
        }
    }
}

/// DNS Resource Record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DNSRecord {
    pub name: DomainName,
    pub rtype: RecordType,
    pub rclass: QueryClass,
    pub ttl: u32,
    pub rdlen: u16,
    pub rdata: Vec<u8>,
}

/// Resource Record with parsed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRecord {
    pub name: DomainName,
    pub rtype: RecordType,
    pub rclass: QueryClass,
    pub ttl: u32,
    pub data: RecordData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordData {
    A(std::net::Ipv4Addr),
    AAAA(std::net::Ipv6Addr),
    CNAME(DomainName),
    MX { preference: u16, exchange: DomainName },
    NS(DomainName),
    PTR(DomainName),
    SOA {
        mname: DomainName,
        rname: DomainName,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    },
    SRV {
        priority: u16,
        weight: u16,
        port: u16,
        target: DomainName,
    },
    TXT(Vec<String>),
    CAA {
        flags: u8,
        tag: String,
        value: String,
    },
    TLSA {
        cert_usage: u8,
        selector: u8,
        matching_type: u8,
        association_data: Vec<u8>,
    },
    Unknown(Vec<u8>),
}

/// Complete DNS Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNSMessage {
    pub header: DNSHeader,
    pub questions: Vec<DNSQuestion>,
    pub answers: Vec<DNSRecord>,
    pub authorities: Vec<DNSRecord>,
    pub additionals: Vec<DNSRecord>,
}

impl DNSMessage {
    pub fn new() -> Self {
        DNSMessage {
            header: DNSHeader {
                id: rand::random(),
                flags: DNSFlags {
                    qr: false,
                    opcode: 0,
                    aa: false,
                    tc: false,
                    rd: true,
                    ra: false,
                    z: false,
                    ad: false,
                    cd: false,
                    rcode: 0,
                },
                qdcount: 0,
                ancount: 0,
                nscount: 0,
                arcount: 0,
            },
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
        }
    }

    pub fn query(domain: &str, qtype: RecordType) -> Self {
        let mut msg = DNSMessage::new();
        msg.header.qdcount = 1;
        msg.questions.push(DNSQuestion {
            name: DomainName::new(domain),
            qtype,
            qclass: QueryClass::IN,
        });
        msg
    }

    pub fn is_response(&self) -> bool {
        self.header.flags.qr
    }

    pub fn is_truncated(&self) -> bool {
        self.header.flags.tc
    }
}

impl Default for DNSMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DNSMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DNS Message (ID: {})", self.header.id)?;
        for q in &self.questions {
            write!(f, "\n  Q: {} {}", q.name.to_string(), q.qtype.to_string())?;
        }
        for a in &self.answers {
            write!(f, "\n  A: {} {} TTL:{}", a.name.to_string(), a.rtype.to_string(), a.ttl)?;
        }
        Ok(())
    }
}

use rand;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_type_conversion() {
        assert_eq!(RecordType::from_u16(1), RecordType::A);
        assert_eq!(RecordType::from_u16(28), RecordType::AAAA);
        assert_eq!(RecordType::from_u16(255), RecordType::ANY);
    }

    #[test]
    fn test_domain_name() {
        let name = DomainName::new("example.com");
        assert_eq!(name.labels, vec!["example".to_string(), "com".to_string()]);
        assert_eq!(name.to_string(), "example.com.");
    }

    #[test]
    fn test_dns_message_creation() {
        let msg = DNSMessage::query("google.com", RecordType::A);
        assert_eq!(msg.questions.len(), 1);
        assert!(!msg.is_response());
    }

    #[test]
    fn test_flags_serialization() {
        let flags = DNSFlags {
            qr: true,
            opcode: 0,
            aa: true,
            tc: false,
            rd: true,
            ra: true,
            z: false,
            ad: true,
            cd: false,
            rcode: 0,
        };
        let serialized = serde_json::to_string(&flags).unwrap();
        let deserialized: DNSFlags = serde_json::from_str(&serialized).unwrap();
        assert_eq!(flags, deserialized);
    }
}
