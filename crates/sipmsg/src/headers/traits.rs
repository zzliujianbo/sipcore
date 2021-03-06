use crate::{common::errorparse::SipParseError, headers::header::HeaderValue};
use nom;

pub type HeaderValueParserFn = fn(input: &[u8]) -> nom::IResult<&[u8], HeaderValue, SipParseError>;
pub trait SipHeaderParser {
    // It should returns COMMA in first parameter if it header with multiple value
    // or SEMI if it contains perameters
    fn take_value(input: &[u8]) -> nom::IResult<&[u8], HeaderValue, SipParseError>;
}
