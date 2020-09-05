use crate::{
    common::{
        bnfcore::is_wsp, errorparse::SipParseError, nom_wrappers::from_utf8_nom, take_sws_token,
    },
    headers:: {
        header::{HeaderValue, HeaderValueType},
        traits::SipHeaderParser
    }
};

use iri_string::{spec::UriSpec, validate::iri};
use nom::{bytes::complete::take_while1, sequence::tuple};

/// Alert-Info   =  "Alert-Info" HCOLON alert-param *(COMMA alert-param)
// alert-param  =  LAQUOT absoluteURI RAQUOT *( SEMI generic-param )
pub struct AlertInfoParser;

impl SipHeaderParser for AlertInfoParser {
    fn take_value(input: &[u8]) -> nom::IResult<&[u8], HeaderValue, SipParseError> {
        let uri = take_while1(|c| !is_wsp(c) && c != b'>');

        let (inp, (_ /*LAQUOT*/, uri, _ /* RAQUOT */)) =
            tuple((take_sws_token::laquot, uri, take_sws_token::raquot))(input)?;
        let (_, uri_str) = from_utf8_nom(uri)?;
        if !iri::<UriSpec>(uri_str).is_ok() {
            return sip_parse_error!(1, "Invalid URI");
        }
        let (_, hdr_val) = HeaderValue::new(uri, HeaderValueType::SimpleString, None)?;
        Ok((inp, hdr_val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn headers_parse_test() {
        match AlertInfoParser::take_value(
            "  < http://www.example.com/sounds/moo.wav  >   \r\n".as_bytes(),
        ) {
            Ok((input, val)) => {
                assert_eq!(input, "\r\n".as_bytes());
                assert_eq!(val.vstr, "http://www.example.com/sounds/moo.wav");
            }
            Err(_) => {
                panic!();
            }
        }
        match AlertInfoParser::take_value("random word\r\n".as_bytes()) {
            Ok((_, _)) => {
                panic!();
            }
            Err(_) => {}
        }
    }
}
