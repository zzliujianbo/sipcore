use sipmsg::*;

#[test]
fn parse_header() {
    match SipHeader::parse("Subject:This is a test\r\n".as_bytes()) {
        Ok((input, hdr)) => {
            assert_eq!(hdr.name, "Subject");
            assert_eq!(hdr.value, "This is a test");
            assert_eq!(input.len(), 0)
        }
        Err(_e) => panic!(),
    }

    match SipHeader::parse("Name: Value;parameter=false;param2\r\n".as_bytes()) {
        Ok((input, hdr)) => {
            assert_eq!(hdr.name, "Name");
            assert_eq!(hdr.value, "Value");
            assert_eq!(hdr.params().unwrap().get(&"parameter"), Some(&"false"));
            assert_eq!(hdr.params().unwrap().get(&"param2"), Some(&""));
            assert_eq!(input.len(), 0);
        }
        Err(_e) => panic!(),
    }

    match SipHeader::parse("Max-Forwards: 70\r\n".as_bytes()) {
        Ok((input, hdr)) => {
            assert_eq!(hdr.name, "Max-Forwards");
            assert_eq!(hdr.value, "70");
            assert_eq!(input.len(), 0);
        }
        Err(_e) => panic!(),
    }
}

#[test]
fn parse_header_long_folded() {
    match SipHeader::parse("Max-Forwards: 70\r\n continue header\r\n".as_bytes()) {
        Ok((_, _)) => panic!(),
        Err(_) => {}
    }
}

#[test]
fn parse_headers() {
    let parse_headers_result = sipmsg::parse_sip_headers(
        "To: sip:user@example.com\r\n\
         From: caller<sip:caller@example.com>;tag=323\r\n\
         Max-Forwards: 70\r\n\
         Call-ID: lwsdisp.1234abcd@funky.example.com\r\n\
         CSeq: 60 OPTIONS\r\n\
         Via: SIP/2.0/UDP funky.example.com;branch=z9hG4bKkdjuw\r\n\r\nsomebody"
            .as_bytes(),
    );

    match parse_headers_result {
        Ok((input, hdrs)) => {
            assert_eq!(hdrs.len(), 6);
            assert_eq!(hdrs[0].name, "To");
            assert_eq!(hdrs[0].value, "sip:user@example.com");

            assert_eq!(hdrs[1].name, "From");
            assert_eq!(hdrs[1].value, "caller<sip:caller@example.com>");
            assert_eq!(hdrs[1].params().unwrap().get(&"tag"), Some(&"323"));

            assert_eq!(hdrs[2].name, "Max-Forwards");
            assert_eq!(hdrs[2].value, "70");
            assert_eq!(hdrs[2].params(), None);

            assert_eq!(hdrs[3].name, "Call-ID");
            assert_eq!(hdrs[3].value, "lwsdisp.1234abcd@funky.example.com");
            assert_eq!(hdrs[3].params(), None);

            assert_eq!(hdrs[4].name, "CSeq");
            assert_eq!(hdrs[4].value, "60 OPTIONS");
            assert_eq!(hdrs[4].params(), None);

            assert_eq!(hdrs[5].name, "Via");
            assert_eq!(hdrs[5].value, "SIP/2.0/UDP funky.example.com");
            assert_eq!(
                hdrs[5].params().unwrap().get(&"branch"),
                Some(&"z9hG4bKkdjuw")
            );
            assert_eq!(input, "\r\nsomebody".as_bytes());
        }
        Err(_e) => panic!(),
    }
}
