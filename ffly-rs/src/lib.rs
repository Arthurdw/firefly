// TODO: Write tests
use core::fmt;
use std::{
    error::Error,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// Catch-all error type
pub type GenericError = Box<dyn Error + Send + Sync + 'static>;

pub type FireflyResult<T> = Result<T, GenericError>;
pub type OptResult = FireflyResult<()>;
pub type StringResult = FireflyResult<String>;

pub struct FireflyStream {
    /// The stream to the Firefly server.
    tcp_stream: Arc<Mutex<TcpStream>>,

    /// The maximum length of the response. (size for response buffer)
    max_buffer_size: usize,

    /// The default TTL for new records.
    /// If this value is not zero, it will be added to the current timestamp.
    /// So this is the TTL from when the new is executed.
    pub default_ttl: usize,
}

#[derive(Debug)]
pub enum FireflyError {
    /// The server returned a value which was not in the expected format.
    UnexpectedResponseError,
}

impl Error for FireflyError {}
impl fmt::Display for FireflyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Get the amount of seconds since the UNIX epoch.
fn current_epoch() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

impl FireflyStream {
    /// Instantiate a new TCP connection with a Firefly server.
    /// Fails if the connection cannot be established. The expected buffer
    /// size is set to 512.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the Firefly server. (e.g. "127.0.0.1:46600")
    pub async fn connect(address: &str) -> FireflyResult<Self> {
        Self::connect_with_max_buffer(address, 512).await
    }

    /// Same as `FireflyStream::connect`, but with a custom buffer size.
    /// The buffer size is the maximum expected response size.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the Firefly server. (e.g. "127.0.0.1:46600")
    /// * `max_buffer_size` - The maximum expected response size.
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

    /// Send a slice of bytes to the Firefly server.
    ///
    /// # Arguments
    ///
    /// * `data` - The slice of bytes to send.
    async fn send_no_check(&self, data: &[u8]) -> StringResult {
        let mut stream = self.tcp_stream.lock().unwrap();
        stream.write(data).await?;

        let mut buffer = vec![0; self.max_buffer_size];
        let response_size = stream.read(&mut buffer).await?;
        drop(stream);

        // The server will ALWAYS return valid utf8
        Ok(String::from_utf8(buffer[..response_size].to_vec()).unwrap())
    }

    /// The same as the `FireflyStream::send_no_check` method, but check the response.
    ///
    /// # Arguments
    ///
    /// * `data` - The slice of bytes to send.
    /// * `expected` - A closure predicate that returns true if the response is valid.
    async fn send(&self, data: &[u8], expected: fn(&str) -> bool) -> StringResult {
        let response = self.send_no_check(data).await?;

        if expected(&response) {
            return Ok(response);
        }

        Err(FireflyError::UnexpectedResponseError.into())
    }

    /// Same as send, but checks if the response contains "Ok" or doesn't
    /// contain "Error".
    ///
    /// # Arguments
    ///
    /// * `data` - The slice of bytes to send.
    async fn send_ok(&self, data: &[u8]) -> StringResult {
        self.send(data, |expected| {
            expected == "Ok" || !expected.contains("Error")
        })
        .await
    }

    /// Create a new record with the default TTL.
    /// The default TTL is 0. (record lasts for ever)
    ///
    /// # Arguments
    ///
    /// * `key` - Your unique key for the record.
    /// * `value` - The value of the record.
    pub async fn new(&self, key: &str, value: &str) -> OptResult {
        let mut ttl = self.default_ttl;

        if ttl != 0 {
            ttl += current_epoch();
        }

        self.new_with_ttl(key, value, ttl).await
    }

    /// Same as `FireflyStream::new`, but with a custom TTL.
    /// The TTL is the timestamp since the UNIX epoch.
    ///
    /// # Arguments
    ///
    /// * `key` - Your unique key for the record.
    /// * `value` - The value of the record.
    /// * `ttl` - The timestamp since the UNIX epoch for the data to expire. (0 = never)
    pub async fn new_with_ttl(&self, key: &str, value: &str, ttl: usize) -> OptResult {
        let query = format!("0{key}\0{value}\0{ttl}");
        self.send_ok(query.as_bytes()).await?;

        Ok(())
    }

    /// Get a record from the Firefly server. If you only need the value or ttl
    /// use the specific methods for those purposes. As this returns both values.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the record.
    pub async fn get(&self, key: &str) -> FireflyResult<(String, String)> {
        let query = format!("1{key}");
        let data = self
            .send(query.as_bytes(), |response| response.contains(0 as char))
            .await?;

        match data.split_once(0 as char) {
            Some((value, ttl)) => Ok((value.to_string(), ttl.to_string())),
            None => Err(FireflyError::UnexpectedResponseError.into()),
        }
    }

    /// Same as `FireflyStream::get`, but only returns the value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the record.
    pub async fn get_value(&self, key: &str) -> StringResult {
        self.send_ok(format!("2{key}").as_bytes()).await
    }

    /// Same as `FireflyStream::get`, but only returns the ttl.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the record.
    pub async fn get_ttl(&self, key: &str) -> FireflyResult<usize> {
        let ttl = self.send_ok(format!("3{key}").as_bytes()).await?;
        Ok(ttl.parse()?)
    }

    /// Remove a record from the Firefly server.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the record.
    pub async fn drop(&self, key: &str) -> OptResult {
        self.send_ok(format!("4{key}").as_bytes()).await?;
        Ok(())
    }

    /// Remove ALL records that have a certain value.
    /// Using this method is generally discouraged. As it is a heavy operation.
    ///
    /// # Arguments
    ///
    /// * `value` - The valy of ANY record that should be removed.
    pub async fn drop_values(&self, value: &str) -> OptResult {
        self.send_ok(format!("5{value}").as_bytes()).await?;
        Ok(())
    }
}
