use std::{env, fs, thread, time};
use unixsockmon::SockMonitor;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        // one argument passed
        2 => {
            server(&args[1]);
        },
        // two argument passed
        3 => {
            client(&args[1], &args[2]);
        }
        _ => eprintln!("Usage: {} <sock> [message]", args[0])
    }
}

fn client(sock: &str, msg: &str) {
    while !fs::metadata(sock).is_ok() {
        thread::sleep(time::Duration::from_millis(500));
    }        
    let client = SockMonitor::new(sock);
    let resp = client.send_string(&format!("{}\n", msg));
    assert!(resp.is_ok());
    assert_eq!(resp.unwrap(), "OK");
}

fn server(sock: &str) {
    if fs::metadata(sock).is_ok() {
        fs::remove_file(sock).unwrap();
    }

    let mon = SockMonitor::new(sock);
    mon.serve(SockMonitor::read_line, move |req| {
        println!("Server: {}", req);
        Ok("OK".to_string())
    }).unwrap();
}