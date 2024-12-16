use std::io::{self, ErrorKind};
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use std::net::{SocketAddr, Ipv4Addr, IpAddr, SocketAddrV4};
use std::mem::MaybeUninit;
pub struct DiscoveryClient{
    local_port: u16,
    socket: Socket,
}

impl DiscoveryClient {
    pub fn new() -> Result<Self, io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

        socket.set_reuse_address(true)?;

        // Bind the socket to an address
        socket.bind(&SocketAddr::from(([0, 0, 0, 0], 0)).into())?;


        //let local_addr = socket.local_addr()?;
        let local_addr = socket.local_addr()?;
        let mut local_port=0;
        if let Some(socket_addr) = local_addr.as_socket() {
            let ip: IpAddr = socket_addr.ip();
            local_port = socket_addr.port();

            println!("Local IP address: {}", &ip);
            println!("Local port: {}", &local_port);
        } else {
            println!("The socket address could not be converted to a standard SocketAddr.");
        }
    


        Ok(DiscoveryClient { socket, local_port })
    }


    pub fn discover_server(&self,server_adress_ip:IpAddr) -> Result<(String,i32), io::Error> {
        let broadcast_ip =  Ipv4Addr::new(192, 168, 1, 255); // L'indirizzo del server

        
        //let serverAdress = SocketAddrV4::new(serverAdressIp, 9000);
        //#[cfg(target_os = "linux")]
        let server_adress = SocketAddr::new(server_adress_ip, 9000);

        let sock_addr = SockAddr::from(server_adress);

        println!("Sending DISCOVERY message to {:?}", server_adress_ip);

        self.socket.set_broadcast(true)?;

        let discovery_message = "DISCOVERY";
        self.socket.send_to(discovery_message.as_bytes(), &sock_addr)?;
        println!("Sent DISCOVERY message with local port: {}", self.local_port);


        let mut buf = [MaybeUninit::uninit(); 1024];
        let mut ris: Result<(String, i32), io::Error>=Result::Ok(("mdjisf".to_string(),4));

    // Use a while loop to wait until we get the first successful response
     while {
        ris = match self.socket.recv_from(&mut buf) {
            Ok((amt, src)) => {

                let initialized_buf = unsafe {
                    std::slice::from_raw_parts(buf.as_ptr() as *const u8, amt)
                };
            
                let server_response = String::from_utf8_lossy(initialized_buf).to_string();

                println!("Received response: {} from {:?}", server_response, src.as_socket_ipv4().unwrap().ip());
                let ip_addr: IpAddr = src.as_socket().expect("no as socket works").ip();
                println!("the Server IP is {}",ip_addr.to_string());
                let client_ip = server_response.trim().to_string();
                return Ok((client_ip, self.local_port as i32));
            }
            Err(e) => {
                if !matches!(e.kind(), ErrorKind::WouldBlock) {
                    eprintln!("{}", e);
                } else {
                    println!("Failed to receive response: {}", e);
                }
                Err(e)
            }
        };
        

        ris.is_err()
    } {}

    // If the loop exits without a successful result, return an error
    ris
    }

    pub fn notify_disconnection(&self) -> Result<(), io::Error> {
        let server_addr =  Ipv4Addr::new(192, 168, 1, 255); // L'indirizzo del server


        //let broadcast_addr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), 9000);
        let broadcast_addr = SocketAddrV4::new(server_addr, 9000);
        #[cfg(target_os = "linux")]
        let broadcast_addr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), 9000);
        let server_addr = SockAddr::from(broadcast_addr);
        self.socket.set_broadcast(true)?;

        println!("Sending DISCONNECT message to {:?}", server_addr.as_socket_ipv4().unwrap().ip());

        let disconnect_message = format!("DISCONNECT:{}", self.local_port);
        self.socket.send_to(disconnect_message.as_bytes(), &server_addr)?;
        println!("Sent DISCONNECT message with local port: {}", self.local_port);

        Ok(())
    }
}





impl Drop for DiscoveryClient {
        fn drop(&mut self) {
            // Perform cleanup actions when the DiscoveryClient is dropped
            println!("Dropping DiscoveryClient and closing socket bound to port {}", self.local_port);
            self.notify_disconnection().unwrap();
    
            // Explicitly set the socket to None to close it
            // This is not strictly necessary because Rust automatically drops the socket
            // when the struct is dropped, but it helps to indicate the intention.
            // The socket is automatically closed when it goes out of scope, so no need for extra code.
        }
    }
