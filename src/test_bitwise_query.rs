use crate::{
    bitwise_query::{get_arguments, get_query_type},
    expect,
    query::QueryType,
};

#[test]
fn test_bitwise_query_type() {
    let query = "    0 || New query identifier";
    expect!(Some(QueryType::New), get_query_type(query.to_string()))
}

#[test]
fn test_bitwise_query_invalid_type() {
    let query = "Invalid query identifier";
    expect!(None, get_query_type(query.to_string()));
}

#[test]
fn test_bitwise_query_arguments() {
    let bytes = b"0hello word\x00bake some eggs\x000";
    let query = String::from_utf8(bytes.to_vec()).unwrap();
    let query_type = get_query_type(query.to_string()).unwrap();
    let arguments = get_arguments(query_type, query);
    expect!(Ok(_), arguments.clone());
    assert_eq!(arguments.unwrap().len(), 3);
}
