#![no_std]

//! # Introduction
//!
//! Library for parsing/editing/constructing SIP requests and responses.
//!
//! This is the very first version where only simple parsing is support.
//!
//! ## Example
//! ```rust
//!
//! use sipmsg::{SipMessage, SipMethod, SipRequestUriScheme,
//!              SipRFCHeader, SipHeaderTagType, SipVersion};
//! use unicase::Ascii;
//!
//! let invite_msg_buf = "\
//! INVITE sip:bob@biloxi.com;user=phone?to=alice%40atlanta.com&priority=urgent SIP/2.0\r\n\
//! Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKkjshdyff\r\n\
//! Via: SIP/2.0/UDP 192.168.1.111\r\n\
//! To: Bob <sip:bob@biloxi.com>\r\n\
//! From: Alice <sip:alice@atlanta.com>;tag=88sja8x\r\n\
//! Contact: Caller <sip:alice@client.atlanta.example.com;transport=tcp>\r\n\
//! Max-Forwards: 70\r\n\
//! Call-ID: f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com\r\n\
//! Extention-Header: extention header value;param=123;without_value\r\n\
//! CSeq: 986759 INVITE\r\n\r\nbody_stuff"
//! .as_bytes();
//!
//! // First parameter not realized yet.
//! // It should consist be residue if Content-Length is less then actual body length.
//! let (_, sip_msg) = SipMessage::parse(invite_msg_buf).unwrap();
//! let request = sip_msg.request().unwrap();
//! assert_eq!(request.rl.method, SipMethod::INVITE);
//! assert_eq!(request.rl.sip_version, SipVersion(2, 0));
//!
//! // RURI
//! assert_eq!(request.rl.uri.scheme, SipRequestUriScheme::SIP);
//! assert_eq!(request.rl.uri.user_info().unwrap().value, "bob");
//! assert_eq!(request.rl.uri.hostport.host, "biloxi.com");
//! assert_eq!(request.rl.uri.params().unwrap().get(&"user"), Some(&Some("phone")));
//! assert_eq!(request.rl.uri.headers().unwrap().get(&"to"), Some(&"alice%40atlanta.com"));
//! assert_eq!(request.rl.uri.headers().unwrap().get(&"priority"), Some(&"urgent"));
//!
//! let call_id_header = request.headers.get_rfc_s(SipRFCHeader::CallID).unwrap();
//! assert_eq!(call_id_header.value.vstr, "f81d4fae-7dec-11d0-a765-00a0c91e6bf6@foo.bar.com");
//! assert_eq!(call_id_header.value.tags().unwrap()[&SipHeaderTagType::ID],
//!           "f81d4fae-7dec-11d0-a765-00a0c91e6bf6".as_bytes());
//! assert_eq!(call_id_header.value.tags().unwrap()[&SipHeaderTagType::Host], b"foo.bar.com");
//!
//! // Via Header
//! let via_headers = request.headers.get_rfc(SipRFCHeader::Via).unwrap();
//! assert_eq!(via_headers[0].value.vstr, "SIP/2.0/UDP pc33.atlanta.com");
//! assert_eq!(
//!     via_headers[0].params().unwrap().get(&"branch"),
//!     Some(&Some("z9hG4bKkjshdyff"))
//! );
//! assert_eq!(
//!     via_headers[0].value.tags().unwrap()[&SipHeaderTagType::ProtocolName],
//!     b"SIP"
//! );
//! assert_eq!(
//!     via_headers[0].value.tags().unwrap()[&SipHeaderTagType::ProtocolVersion],
//!     b"2.0"
//! );
//! assert_eq!(
//!     via_headers[0].value.tags().unwrap()[&SipHeaderTagType::ProtocolTransport],
//!     b"UDP"
//! );
//! assert_eq!(
//!     via_headers[0].value.tags().unwrap()[&SipHeaderTagType::Host],
//!     b"pc33.atlanta.com"
//! );
//! assert_eq!(via_headers[1].value.vstr, "SIP/2.0/UDP 192.168.1.111");
//! assert_eq!(
//!     via_headers[1].params(),
//!     None
//! );
//!
//! // Contact header
//! let contact_header = request.headers.get_rfc_s(SipRFCHeader::Contact).unwrap();
//! assert_eq!(
//!            contact_header.value.tags().unwrap()[&SipHeaderTagType::DisplayName],
//!            b"Caller"
//! );
//! assert_eq!(
//!            contact_header.value.sip_uri().unwrap().user_info().unwrap().value,
//!            "alice"
//! );
//! assert_eq!(
//!            contact_header.value.sip_uri().unwrap().hostport.host,
//!            "client.atlanta.example.com"
//! );
//!
//! assert_eq!(
//!    contact_header.value.sip_uri().unwrap().params().unwrap().get(&"transport"),
//!    Some(&Some("tcp"))
//! );
//! assert_eq!(
//!    contact_header.value.sip_uri().unwrap().params().unwrap().get(&"non-exists-param"),
//!    None
//! );
//!
//! // Extention Header
//! let extention_header = request.headers.get_ext_s("extention-header").unwrap();
//! assert_eq!(extention_header.name, "extention-header");
//! assert_eq!(extention_header.value.vstr, "extention header value;param=123;without_value");
//!
//! // Body
//! assert_eq!(request.body.unwrap(), b"body_stuff");
//! ```
//!
extern crate alloc;
extern crate nom;

#[macro_use]
pub mod common;
pub use common::errorparse;
pub use common::sip_method::SipMethod;

mod message;
pub use message::get_message_type as get_sip_message_type;
pub use message::MessageType as SipMessageType;
pub use message::SipVersion;
pub use message::SipMessage;

mod userinfo;

mod request;
pub use request::Request as SipRequest;
pub use request::RequestLine as SipRequestLine;

mod response;
pub use response::Response as SipResponse;
pub use response::StatusCode as SipResponseStatusCode;
pub use response::StatusLine as SipResponseStatusLine;

mod headers;
pub use headers::sipuri::RequestUriScheme as SipRequestUriScheme;
pub use headers::*;

mod serializer;

pub use unicase::Ascii as SipAscii;
