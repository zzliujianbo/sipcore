use nom::{
    bytes::complete::{tag, take_while1},
    character::{complete, is_alphabetic},
    sequence::tuple,
};

use core::{str, u8};

pub struct Request<'a> {
    // The request line
    pub rl: Option<RequestLine<'a>>,
    //The request headers.
    // pub headers: &'a [Header<'a>],
}

impl<'a> Request<'a> {
    // fn new() -> Request<'a> {
    //     Request {
    //         // headers: []{Header{na}},
    //         rl : None,
    //     }
    // }

    // pub fn parse(buf_input: &'buf [u8]) -> nom::IResult<&[u8], Request> {
    //     let new_request = Request::new();
    //     new_request.request_buffer = Some(buf_input);
    //     let (_, rl) = RequestLine::parse(buf_input)?;
    //     Ok((buf_input, new_request))
    // }
}

/// SIP-Version
/// ex. `SIP/2.0 -> SipVersion(2, 0)`
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SipVersion(pub u8, pub u8);

/// Ex: INVITE sip:user@example.com SIP/2.0
/// The Request line and u8 buffer shoud have the same life time
pub struct RequestLine<'a> {
    pub method: Method,
    pub uri: &'a str,
    pub sip_version: SipVersion,
}

impl<'a> RequestLine<'a> {
    pub fn parse(rl: &[u8]) -> nom::IResult<&[u8], RequestLine> {
        let method = take_while1(is_alphabetic);
        let uri = take_while1(|c| c != b' ' as u8);
        let (input, (method, _, uri, _, _, major_version, _, minor_version, _)) = tuple((
            method,
            complete::space1,
            uri,
            complete::space1,
            tag("SIP/"),
            complete::digit1,
            complete::char('.'),
            complete::digit1,
            complete::crlf,
        ))(rl)?;

        let sip_version = SipVersion(
            u8::from_str_radix(str::from_utf8(major_version).unwrap(), 10).unwrap(),
            u8::from_str_radix(str::from_utf8(minor_version).unwrap(), 10).unwrap(),
        );

        match RequestLine::parse_method(method) {
            Some(m) => Ok((
                input,
                RequestLine {
                    method: m,
                    uri: str::from_utf8(uri).unwrap(),
                    sip_version: sip_version,
                },
            )),
            None => Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
                rl,
                nom::error::ErrorKind::IsA,
            ))),
        }
    }

    fn parse_method(method: &[u8]) -> Option<Method> {
        match str::from_utf8(method) {
            Ok(s) => Method::from_str(s),
            Err(_) => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Method {
    ACK,
    BYE,
    CANCEL,
    INFO,
    INVITE,
    MESSAGE,
    NOTIFY,
    OPTIONS,
    PRACK,
    PUBLISH,
    REFER,
    REGISTER,
    SUBSCRIBE,
    UPDATE,
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            &Method::ACK => "ACK",
            &Method::BYE => "BYE",
            &Method::CANCEL => "CANCEL",
            &Method::INFO => "INFO",
            &Method::INVITE => "INVITE",
            &Method::MESSAGE => "MESSAGE",
            &Method::NOTIFY => "NOTIFY",
            &Method::OPTIONS => "OPTIONS",
            &Method::PRACK => "PRACK",
            &Method::PUBLISH => "PUBLISH",
            &Method::REFER => "REFER",
            &Method::REGISTER => "REGISTER",
            &Method::SUBSCRIBE => "SUBSCRIBE",
            &Method::UPDATE => "UPDATE",
        }
    }

    pub fn from_str(s: &str) -> Option<Method> {
        match s {
            "ACK" => Some(Method::ACK),
            "BYE" => Some(Method::BYE),
            "CANCEL" => Some(Method::CANCEL),
            "INFO" => Some(Method::INFO),
            "INVITE" => Some(Method::INVITE),
            "MESSAGE" => Some(Method::MESSAGE),
            "NOTIFY" => Some(Method::NOTIFY),
            "OPTIONS" => Some(Method::OPTIONS),
            "PRACK" => Some(Method::PRACK),
            "PUBLISH" => Some(Method::PUBLISH),
            "REFER" => Some(Method::REFER),
            "REGISTER" => Some(Method::REGISTER),
            "SUBSCRIBE" => Some(Method::SUBSCRIBE),
            "UPDATE" => Some(Method::UPDATE),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::request::Method;
    use crate::request::RequestLine;
    use crate::request::SipVersion;

    fn parse_rl_test(
        rl: &str,
        expected_method: Method,
        expected_uri: &str,
        expected_sip_version: SipVersion,
    ) {
        match RequestLine::parse(rl.as_bytes()) {
            Ok((_b, rl)) => {
                assert_eq!(rl.method, expected_method);
                assert_eq!(rl.sip_version, expected_sip_version);
                assert_eq!(rl.uri, expected_uri);
            }
            Err(_e) => panic!(),
        }
    }

    #[test]
    fn get_method_type() {
        parse_rl_test(
            "OPTIONS sip:user@example.com SIP/2.0\r\n",
            Method::OPTIONS,
            "sip:user@example.com",
            SipVersion(2, 0),
        );
        parse_rl_test(
            "INVITE sip:vivekg@chair-dnrc.example.com;unknownparam SIP/2.0\r\n",
            Method::INVITE,
            "sip:vivekg@chair-dnrc.example.com;unknownparam",
            SipVersion(2, 0),
        );
        parse_rl_test(
            "REGISTER sip:[2001:db8::10] SIP/3.1\r\n",
            Method::REGISTER,
            "sip:[2001:db8::10]",
            SipVersion(3, 1),
        );
    }

    #[test]
    fn get_method_type_fail() {
        match RequestLine::parse("OPTI2ONS sip:user@example.com SIP/2.0\r\n".as_bytes()) {
            Ok((_, _)) => panic!(),
            Err(_e) => (),
        }
    }
}