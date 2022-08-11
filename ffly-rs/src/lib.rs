use core::fmt;
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub type GenericError = Box<dyn Error + Send + Sync + 'static>;
pub type FireflyResult<T> = Result<T, GenericError>;
pub type OptResult = FireflyResult<()>;
pub type Data = FireflyResult<String>;

pub struct FireflyStream {
    tcp_stream: Arc<Mutex<TcpStream>>,
    max_buffer_size: usize,
    default_ttl: usize,
}

#[derive(Debug)]
pub enum FireflyError {
    UnexpectedResponseError,
}

impl Error for FireflyError {}
impl fmt::Display for FireflyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FireflyStream {
    // TODO: Document this
    pub async fn connect(address: &str) -> FireflyResult<Self> {
        Self::connect_with_max_buffer(address, 512).await
    }

    // TODO: Document this
    pub async fn connect_with_max_buffer(
        address: &str,
        max_buffer_size: usize,
    ) -> FireflyResult<Self> {
        let tcp_stream = Arc::new(Mutex::new(TcpStream::connect(address).await?));
        let client = Self {
            max_buffer_size,
            tcp_stream,
            default_ttl: 0,
        };

        client.send_ok("QUERY TYPE BITWISE;".as_bytes()).await?;

        Ok(client)
    }

    async fn send_no_check(&self, data: &[u8]) -> Data {
        let mut stream = self.tcp_stream.lock().unwrap();
        stream.write(data).await?;

        let mut buffer = vec![0; self.max_buffer_size];
        let response_size = stream.read(&mut buffer).await?;
        drop(stream);

        // The server will ALWAYS return valid utf8
        Ok(String::from_utf8(buffer[..response_size].to_vec()).unwrap())
    }

    // TODO: Document this
    async fn send(&self, data: &[u8], expected: fn(&str) -> bool) -> Data {
        let response = self.send_no_check(data).await?;

        if expected(&response) {
            return Ok(response);
        }

        Err(FireflyError::UnexpectedResponseError.into())
    }

    // TODO: Document this
    async fn send_ok(&self, data: &[u8]) -> Data {
        self.send(data, |expected| {
            expected == "Ok" || !expected.contains("Error")
        })
        .await
    }

    // TODO: Document this
    pub async fn new(&self, key: &str, value: &str) -> OptResult {
        let ttl = self.default_ttl;

        self.new_with_ttl(key, value, ttl).await
    }

    // TODO: Document this
    pub async fn new_with_ttl(&self, key: &str, value: &str, ttl: usize) -> OptResult {
        let query = format!("0{key}\0{value}\0{ttl}");
        self.send_ok(query.as_bytes()).await?;

        Ok(())
    }

    // TODO: Document this
    pub async fn get(&self, key: &str) -> FireflyResult<(String, String)> {
        let query = format!("1{key}");
        let data = self
            .send(query.as_bytes(), |response| response.contains(','))
            .await?;

        match data.split_once(',') {
            Some((value, ttl)) => Ok((value.to_string(), ttl.to_string())),
            None => Err(FireflyError::UnexpectedResponseError.into()),
        }
    }

    // TODO: Document this
    pub async fn get_value(&self, key: &str) -> Data {
        self.send_ok(format!("2{key}").as_bytes()).await
    }

    // TODO: Document this
    pub async fn get_ttl(&self, key: &str) -> FireflyResult<usize> {
        let ttl = self.send_ok(format!("3{key}").as_bytes()).await?;
        Ok(ttl.parse()?)
    }

    // TODO: Document this
    pub async fn drop(&self, key: &str) -> OptResult {
        self.send_ok(format!("4{key}").as_bytes()).await?;
        Ok(())
    }

    // TODO: Document this
    pub async fn drop_values(&self, value: &str) -> OptResult {
        self.send_ok(format!("5{value}").as_bytes()).await?;
        Ok(())
    }
}
