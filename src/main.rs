extern crate mio;
use mio::*;
use std::net::SocketAddr;
use mio::tcp::*;
use std::collections::HashMap;

struct WebSocketServer {
    socket: TcpListener,
    clients: HashMap<Token, TcpStream>,
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
            }
        }

    }
}

fn main() {
    let mut event_loop = EventLoop::new().unwrap();
    let mut handler = WebSocketServer;
    let address = "0.0.0.0:10000".parse::<SocketAddr>().unwrap();
    let server_socket = TcpListener::bind(&address).unwrap();
    event_loop.register(&server_socket,
                        Token(0),
                        EventSet::readable(),
                        PollOpt::edge())
              .unwrap();
    event_loop.run(&mut handler).unwrap();
}
