
use std::io::{Read, Write, BufReader, BufRead};
use std::os::unix::net::{UnixStream, UnixListener};
use std::error::Error;
use std::fs;

/// Generic Unix Named Socket Monitor
///
/// A generic implementation of unix socket monitor which serves
/// commands. It provides sample reader implementations with newline
/// terminated strings and length prepended byte arrays.
///
/// Example send & recv newline terminated strings
/// ```
/// use unixsockmon::SockMonitor;
/// use std::{thread, time, fs};
///
/// //the reciever
/// if fs::metadata("/tmp/mon_ex1.sock").is_ok() {
///     fs::remove_file("/tmp/mon_ex1.sock").unwrap();
/// }
/// thread::spawn(|| {
///     let mon = SockMonitor::new("/tmp/mon_ex1.sock");
///     mon.serve(SockMonitor::read_line, move |req| {
///         println!("{}", req);
///         Ok("OK".to_string())
///     }).unwrap();
/// });
///
/// //the sender
/// while !fs::metadata("/tmp/mon_ex1.sock").is_ok() {
///     thread::sleep(time::Duration::from_millis(500));
/// }
///
/// let client = SockMonitor::new("/tmp/mon_ex1.sock");
/// // string can be with or without newline
/// let resp = client.send_string("a fox jumps over the lazy dog");
/// assert!(resp.is_ok());
/// assert_eq!(resp.unwrap(), "OK");
/// ```
/// 
/// Example send & recv byte arrays
/// ```
/// use unixsockmon::SockMonitor;
/// use std::{thread, time, fs};
///
/// //the reciever
/// if fs::metadata("/tmp/mon_ex2.sock").is_ok() {
///     fs::remove_file("/tmp/mon_ex2.sock").unwrap();
/// }
/// thread::spawn(|| {
///     let mon = SockMonitor::new("/tmp/mon_ex2.sock");
///     mon.serve(SockMonitor::read_bytes, move |req| {
///         println!("{}", req);
///         Ok("OK".to_string())
///     }).unwrap();
/// });
///
/// //the sender
/// while !fs::metadata("/tmp/mon_ex2.sock").is_ok() {
///     thread::sleep(time::Duration::from_millis(500));
/// }
///
/// let client = SockMonitor::new("/tmp/mon_ex2.sock");
/// // message is a byte array with a leading message length
/// let msg = "a fox jumps over the lazy dog";
/// let resp = client.send_bytes(msg.as_bytes());
/// assert!(resp.is_ok());
/// assert_eq!(resp.unwrap(), "OK");
/// ```
///
pub struct SockMonitor {
    sock: String
}

impl SockMonitor {
    /// Create a new named socket monitor
    pub fn new(sock: &str) -> Self {
        SockMonitor { sock: sock.to_string() }
    }

    /// Read a newline terminated string; return string has
    /// the newline stripped.
    pub fn read_line(stream: &mut UnixStream) -> Result<String, std::io::Error> {
        let mut reader = BufReader::new(stream);
        let mut msg = String::new();

        reader.read_line(&mut msg)?;
        if msg.ends_with('\n') {
            msg.pop();
        }
        Ok(msg)
    }

    /// Read a byte array and return as string
    pub fn read_bytes(stream: &mut UnixStream) -> Result<String, std::io::Error> {
        let mut buffer = [0; 4];
        let len;

        // read 4 byte length first
        stream.read_exact(&mut buffer)?;
        len = u32::from_be_bytes(buffer);

        // read the rest of the message
        let mut buffer: Vec<u8> = vec![0; len as usize];
        stream.read_exact(&mut buffer)?;
        let msg = match std::str::from_utf8(&buffer) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "cannot convert bytes!"));
            }
        };
        Ok(msg.to_string())
    }

    /// Serve the named socket
    pub fn serve<H, R>(&self, reader: R, handler: H) -> Result<(), std::io::Error>
        where H: Fn(String) -> Result<String, Box<dyn Error>>,
              H: Send + 'static,
              R: Fn(&mut UnixStream) -> Result<String, std::io::Error>,
              R: Send + 'static
     {
        // cleanup any stale named sockets
        if fs::metadata(&self.sock).is_ok() {
            fs::remove_file(&self.sock)?;
        }

        // create the listener socket
        let listener = UnixListener::bind(&self.sock)?;

        // accept and process each connection
        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    // read message from socket
                    let msg = match reader(&mut s) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("Monitor::serve:read {}", e);
                            continue;
                        }
                    };
                    // process message
                    match handler(msg) {
                        Err(e) => {
                            eprintln!("Monitor::serve:handle {}", e);
                            s.write_all("ERR".to_string().as_bytes()).unwrap_or_else(|e| {
                                eprintln!("Monitor::serve:write:ERR {}", e);
                            });
                        }
                        Ok(r) => {
                            s.write_all(r.as_bytes()).unwrap_or_else(|e| {
                                eprintln!("Monitor::serve:write:{} {}", r, e);
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Monitor::serve:accept {}", e);
                }
            }
        }
        Ok(())
    }

    /// Send a newline terminated string
    pub fn send_string(&self, msg: &str) -> Result<String, std::io::Error>{
        let mut stream = UnixStream::connect(&self.sock)?;
        let mut buf = String::new();

        // send the message string
        stream.write_all(msg.as_bytes())?;
        // if there is no newline, send a newline
        if !msg.ends_with('\n') {
            stream.write_all("\n".as_bytes())?;
        }
        // wait for response
        stream.read_to_string(&mut buf)?;
        // return response
        Ok(buf)
    }

    /// Send a byte array
    pub fn send_bytes(&self, msg: &[u8]) -> Result<String, std::io::Error>{
        let mut stream = UnixStream::connect(&self.sock)?;
        let mut buf = String::new();

        // find the length of message and create a byte
        // array with it
        let mut val = (msg.len() as u32).to_be_bytes().to_vec();
        // append the message bytes to the byte array
        val.append(&mut msg.to_vec());

        // send the byte array
        stream.write_all(&val)?;
        // wait for response
        stream.read_to_string(&mut buf)?;
        // return response
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, thread, time};

    #[test]
    fn test_mon_string() {
        if fs::metadata("/tmp/mon-line.sock").is_ok() {
            fs::remove_file("/tmp/mon-line.sock").unwrap();
        }

        thread::spawn(|| {
            let mon = SockMonitor::new("/tmp/mon-line.sock");
            mon.serve(SockMonitor::read_line, move |req| {
                println!("{}", req);
                assert_eq!(req, "a fox jumps over the lazy dog");
                Ok("OK".to_string())
            }).unwrap();
        });

        while !fs::metadata("/tmp/mon-line.sock").is_ok() {
            thread::sleep(time::Duration::from_millis(500));
        }        
        let client = SockMonitor::new("/tmp/mon-line.sock");
        let resp = client.send_string("a fox jumps over the lazy dog\n");
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), "OK");
        let resp = client.send_string("a fox jumps over the lazy dog");
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), "OK");
    }
    #[test]
    fn test_mon_bytes() {
        if fs::metadata("/tmp/mon-bytes.sock").is_ok() {
            fs::remove_file("/tmp/mon-bytes.sock").unwrap();
        }

        thread::spawn(|| {
            let mon = SockMonitor::new("/tmp/mon-bytes.sock");
            mon.serve(SockMonitor::read_bytes, move |req| {
                println!("{}", req);
                assert_eq!(req, "a fox jumps over the lazy dog");
                Ok("OK".to_string())
            }).unwrap();
        });

        while !fs::metadata("/tmp/mon-bytes.sock").is_ok() {
            thread::sleep(time::Duration::from_millis(500));
        }        
        let client = SockMonitor::new("/tmp/mon-bytes.sock");
        let msg = "a fox jumps over the lazy dog";
        let resp = client.send_bytes(msg.as_bytes());
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), "OK");
    }
}