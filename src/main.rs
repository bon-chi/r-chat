extern crate mio;
use mio::*;
use std::net::SocketAddr;
use mio::tcp::*;
use std::collections::HashMap;

extern crate sha1;
extern crate rustc_serialize;

use rustc_serialize::base64::{ToBase64, STANDARD};

fn gen_key(key: &String) -> String {
    let mut m = sha1::Sha1::new();
    let mut buf = [0u8; 20];

    m.update(key.as_bytes());
    m.update("258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes());

    m.outpu(&mut buf);

    return buf.to_base64(STANDARD);
}

struct WebSocketServer {
    socket: TcpListener,
    clients: HashMap<Token, WebSocketClient>,
    token_counter: usize,
}

const SERVER_TOKEN: Token = Token(0);

impl Handler for WebSocketServer {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self,
             event_loop: &mut EventLoop<WebSocketServer>,
             token: Token,
             events: EventSet) {
        match token {
            SERVER_TOKEN => {
                let client_socket = match self.socket.accept() {
                    Err(e) => {
                        println!("Accept error: {}", e);
                        return;
                    }
                    Ok(None) => unreachable!(),
                    Ok(Some((sock, addr))) => sock,
                };
                let new_token = Token(self.token_counter);
                self.clients.insert(new_token, WebSocketClient::new(client_socket));
                self.token_counter += 1;
                event_loop.register(&self.clients[&new_token].socket,
                                    new_token,
                                    EventSet::readable(),
                                    PollOpt::edge() | PollOpt::oneshot())
                          .unwrap();
            }
            token => {
                let mut client = self.clients.get_mut(&token).unwrap();
                client.read();
                event_loop.reregister(&client.socket,
                                      token,
                                      EventSet::readable(),
                                      PollOpt::edge() | PollOpt::oneshot())
                          .unwrap();
            }
        }

    }
}

extern crate http_muncher;
use http_muncher::{Parser, ParserHandler};
use std::cell::RefCell;
use std::rc::Rc;

struct HttpParser {
    current_key: Option<String>,
    headers: Rc<RefCell<HashMap<String, String>>>,
}

impl ParserHandler for HttpParser {
    fn on_header_field(&mut self, s: &[u8]) -> bool {
        self.current_key = Some(std::str::from_utf8(s).unwrap().to_string());
        true
    }

    fn on_header_value(&mut self, s: &[u8]) -> bool {
        self.headers.borrow_mut().insert(self.current_key.clone().unwrap(),
                                         std::str::from_utf8(s).unwrap().to_string());
        true
    }

    fn on_headers_complete(&mut self) -> bool {
        false
    }
}

struct WebSocketClient {
    socket: TcpStream,
    http_parser: Parser<HttpParser>,
}

impl WebSocketClient {
    fn new(socket: TcpStream) -> WebSocketClient {
        WebSocketClient {
            socket: socket,
            http_parser: Parser::request(HttpParser),
        }
    }

    fn read(&mut self) {
        loop {
            let mut buf = [0; 2048];
            match self.socket.try_read(&mut buf) {
                Err(e) => {
                    println!("Error while readingg socket: {:?}", e);
                    return;
                }
                Ok(None) => break,
                Ok(Some(len)) => {
                    self.http_parser.parse(&buf[0..len]);
                    if self.http_parser.is_upgrade() {
                        break;
                    }
                }

            }
        }
    }
}

fn main() {
    let mut event_loop = EventLoop::new().unwrap();
    let address = "0.0.0.0:10000".parse::<SocketAddr>().unwrap();
    let server_socket = TcpListener::bind(&address).unwrap();
    let mut server = WebSocketServer {
        token_counter: 1,
        clients: HashMap::new(),
        socket: server_socket,
    };
    event_loop.register(&server.socket,
                        SERVER_TOKEN,
                        EventSet::readable(),
                        PollOpt::edge())
              .unwrap();
    event_loop.run(&mut server).unwrap();
}
