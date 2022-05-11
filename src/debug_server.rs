use crate::c64::C64;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};

use std::net::{TcpListener, TcpStream};
use std::io::*;


pub struct DebuggerServer {
    thread_handle: std::thread::JoinHandle<()>,
    req_rx: Receiver<Request>,
    res_tx: Sender<String>,
}

impl DebuggerServer {
    pub fn new () -> Self {
        let (req_tx, req_rx): (Sender<Request>, Receiver<Request>) = mpsc::channel();
        let (res_tx, res_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        let thread_handle = std::thread::spawn(move || {
            let address = "127.0.0.1:7878";
            let listener = TcpListener::bind(address).unwrap();

            info!("Debug server opened at http://{address}");

            for stream in listener.incoming() {
                let mut stream = stream.unwrap();

                info!("Connection established!");
                //loop {
                    // TODO: buffer size?
                    let mut buffer = [0; 1024];

                    stream.read(&mut buffer).unwrap();

                    let mut headers = [httparse::EMPTY_HEADER; 16];
                    let mut req = httparse::Request::new(&mut headers);
                    let _ = req.parse(&buffer).unwrap();
                    debug!("\n\nReq path is: {}", req.path.unwrap());

                    req_tx.send(Request::CpuState{}).unwrap();

                    let body = res_rx.recv().unwrap();
                    let response = format!("HTTP/1.1 200 OK\r\n{}\r\n\r\n{}", body.len(), body);
                    stream.write(response.as_bytes()).unwrap();
                    stream.flush().unwrap();
                //}
            }

        });

        Self {
            thread_handle,
            req_rx,
            res_tx,
        }
    }

    pub fn get_request(&mut self) -> Option<Request> {
        if let Ok(req) = self.req_rx.try_recv() {
            Some(req)
        } else {
            None
        }
    }

    pub fn set_responce(&mut self, resp: String) {
        self.res_tx.send(resp).unwrap();
    }

}

pub enum Request {
    CpuState,
}
