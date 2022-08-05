use crate::query::{get_arguments, get_query_type, parse_query, QueryType};

static QUERY_NEW: &'static str =
    "NEW 'hi' VALUE 'hello there \"general kenobi\"'WITH TTL '604800';";

static QUERY_GET: &'static str = "GET 'hi';";
static QUERY_GETTTL: &'static str = "GET TTL 'hi';";

macro_rules! expect {
    ($expected:pat, $exec:expr) => {{
        match $exec {
            $expected => assert!(true),
            got => {
                println!("Got: {:?}", got);
                assert!(false);
            }
        }
    }};
}

#[test]
fn test_query_type_parse() {
    expect!(Some(QueryType::New), get_query_type(QUERY_NEW.to_string()));
    expect!(Some(QueryType::Get), get_query_type(QUERY_GET.to_string()));
    expect!(
        Some(QueryType::GetTTL),
        get_query_type(QUERY_GETTTL.to_string())
    );
}

#[test]
fn test_query_get_arguments() {
    expect!(Ok(_), get_arguments(QUERY_NEW.to_string()));
}

#[test]
fn test_query_parse() {
    expect!(Ok(_), parse_query(QUERY_NEW.to_string()));
}

#[test]
fn test_query_parse_failure() {
    expect!(Err(_), parse_query("".to_string()));
}

#[test]
fn test_non_ascii() {
    expect!(Err(_), parse_query("うずまき ナルト".to_string()));
}
