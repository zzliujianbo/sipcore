use crate::{
    common::{
        bnfcore::*, errorparse::SipParseError, nom_wrappers::from_utf8_nom, take_sws_token,
        traits::NomParser,
    },
    headers::{
        parsers::ExtensionParser,
        traits::{HeaderValueParserFn, SipHeaderParser},
        GenericParams, SipRFCHeader,
    },
};
use alloc::collections::{BTreeMap, VecDeque};
use core::str;
use nom::{bytes::complete::take_while1, character::complete, sequence::tuple};
use unicase::Ascii;

// All possible types of value
#[derive(PartialEq, Debug)]
pub enum HeaderValueType {
    EmptyValue,   // SIP header with empty value
    SimpleString, // Haven't tags
}

#[derive(PartialEq, Debug)]
pub enum HeaderValueTags {}

#[derive(PartialEq, Debug)]
pub struct HeaderValue<'a> {
    pub vstr: &'a str,
    pub vtype: HeaderValueType,
    pub vtags: Option<BTreeMap<HeaderValueTags, &'a str>>,
}

impl<'a> HeaderValue<'a> {
    pub fn create_empty_value() -> HeaderValue<'a> {
        HeaderValue {
            vstr: "",
            vtype: HeaderValueType::EmptyValue,
            vtags: None,
        }
    }

    pub fn new(
        val: &'a [u8],
        vtype: HeaderValueType,
        vtags: Option<BTreeMap<HeaderValueTags, &'a str>>,
    ) -> nom::IResult<&'a [u8], HeaderValue<'a>, SipParseError<'a>> {
        let (_, vstr) = from_utf8_nom(val)?;

        Ok((
            val,
            HeaderValue {
                vstr: vstr,
                vtype: vtype,
                vtags: vtags,
            },
        ))
    }
}

#[derive(PartialEq, Debug)]
/// [rfc3261 section-7.3](https://tools.ietf.org/html/rfc3261#section-7.3)
pub struct Header<'a> {
    /// SIP header name
    pub name: Ascii<&'a str>,
    /// SIP header value
    pub value: HeaderValue<'a>,
    /// SIP parameters
    parameters: Option<GenericParams<'a>>,
}

impl<'a> Header<'a> {
    pub fn new(
        name: &'a str,
        value: HeaderValue<'a>,
        parameters: Option<GenericParams<'a>>,
    ) -> Header<'a> {
        Header {
            name: { Ascii::new(name) },
            value: value,
            parameters: parameters,
        }
    }

    pub fn params(&self) -> Option<&GenericParams<'a>> {
        self.parameters.as_ref()
    }

    pub fn find_parser(header_name: &'a str) -> (Option<SipRFCHeader>, HeaderValueParserFn) {
        match SipRFCHeader::from_str(&header_name) {
            Some(rfc_header) => (Some(rfc_header), rfc_header.get_parser()),
            None => (None, ExtensionParser::take_value),
        }
    }

    pub fn take_name(input: &'a [u8]) -> nom::IResult<&[u8], &'a str, SipParseError> {
        let (input_rest, (header_name, _, _, _)) = tuple((
            take_while1(is_token_char),
            complete::space0,
            complete::char(':'),
            complete::space0,
        ))(input)?;
        match str::from_utf8(header_name) {
            Ok(hdr_str) => Ok((input_rest, hdr_str)),
            Err(_) => sip_parse_error!(1, "Bad header name"),
        }
    }

    // pub fn long_header_value_parser_wrapper(
    //     input: &[u8],
    //     parser: HeaderValueParserFn,
    // ) -> nom::IResult<&[u8], &[u8], SipParseError> {
    //     let mut offset = 0;
    //     loop {
    //         let (rest, val) = parser(&input[offset..])?;
    //         offset += vallen();
    //         if !rest.is_empty() && is_wsp(rest[0]) {
    //             let (_, (sp, _, sp2)) =
    //                 tuple((complete::space1, tag("\r\n"), complete::space0))(rest)?;
    //             offset += sp.len() + 2 /* \r\n */ + sp2.len();
    //             continue;
    //         }
    //         break;
    //     }
    //     Ok((&input[offset..], &input[..offset]))
    // }

    /// Should return COMMA, SEMI or '\r\n' in first argument
    pub fn take_value(
        input: &'a [u8],
        parser: HeaderValueParserFn,
    ) -> nom::IResult<&'a [u8], (HeaderValue<'a>, Option<GenericParams<'a>>), SipParseError<'a>>
    {
        // skip whitespaces before take value
        let (input, _) = complete::space0(input)?;
        if is_crlf(input) {
            return Ok((input, (HeaderValue::create_empty_value(), None))); // This is header with empty value
        }

        // add long_header_value_parser_wrapper?
        let (inp, value) = parser(input)?;
        // let (_, value) = from_utf8_nom(value)?;

        // skip whitespaces after take value
        let (inp, _) = complete::space0(inp)?;
        if inp.is_empty() {
            return sip_parse_error!(1, "Error parse header value");
        }
        if inp[0] != b',' && inp[0] != b';' && inp[0] != b' ' && !is_crlf(inp) {
            return sip_parse_error!(2, "Error parse header value");
        }

        if inp[0] == b';' {
            let (inp, params) = Header::try_take_parameters(inp)?;
            return Ok((inp, (value, params)));
        }
        Ok((inp, (value, None)))
    }

    fn try_take_parameters(
        input: &'a [u8],
    ) -> nom::IResult<&'a [u8], Option<GenericParams<'a>>, SipParseError<'a>> {
        if input.is_empty() || input[0] != b';' {
            return Ok((input, None));
        }
        let (input, parameters) = GenericParams::parse(input)?;
        Ok((input, Some(parameters)))
    }
}

impl<'a> NomParser<'a> for Header<'a> {
    type ParseResult = (Option<SipRFCHeader>, VecDeque<Header<'a>>);
    fn parse(input: &'a [u8]) -> nom::IResult<&[u8], Self::ParseResult, SipParseError> {
        let mut headers = VecDeque::new();
        let (input, header_name) = Header::take_name(input)?;
        let (rfc_type, value_parser) = Header::find_parser(header_name);
        let mut inp = input;
        loop {
            // TODO remember about long headers
            // let (input, value) = Header::long_header_value_parser_wrapper(input, value_parser)?;
            let (input, (value, params)) = Header::take_value(inp, value_parser)?;
            headers.push_back(Header::new(header_name, value, params));
            if input[0] == b',' {
                let (input, _) = take_sws_token::comma(input)?;
                inp = input;
                continue;
            }
            inp = input;
            break;
        }
        Ok((inp, (rfc_type, headers)))
    }
}
