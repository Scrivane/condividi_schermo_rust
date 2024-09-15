use std::io::{self, ErrorKind};
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use std::net::{SocketAddr, Ipv4Addr,IpAddr};
use std::mem::MaybeUninit;
pub struct DiscoveryClient{
    local_port: u16,
    socket: Socket,
}

impl DiscoveryClient {
    pub fn new() -> Result<Self, io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

       
        // Set the ReusePort option
        #[cfg(target_os = "linux")]
        socket.set_reuse_port(true)?;

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

    pub fn discover_server(&self) -> Result<(String,i32), io::Error> {

        let broadcast_addr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), 9000);
        let sock_addr = SockAddr::from(broadcast_addr);
        //let server_addr = "255.255.255.255:9000"; // Broadcast al server di scoperta

        println!("Sending DISCOVERY message to {:?}", sock_addr);

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

                println!("Received response: {} from {:?}", server_response, src);
                let ipAddr: IpAddr = src.as_socket().expect("no as socket works").ip();
                println!("the ip is {}",ipAddr.to_string());
                let ip = ipAddr.to_string().split(':').next().unwrap().to_string(); // Takes only the IP address
                return Ok((ip, self.local_port as i32));
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
        let server_addr = "255.255.255.255:9000"; // L'indirizzo del server

        let broadcast_addr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), 9000);
        let server_addr = SockAddr::from(broadcast_addr);
        self.socket.set_broadcast(true)?;

        println!("Sending DISCONNECT message to {:?}", server_addr);

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
    
            // Explicitly set the socket to None to close it
            // This is not strictly necessary because Rust automatically drops the socket
            // when the struct is dropped, but it helps to indicate the intention.
            // The socket is automatically closed when it goes out of scope, so no need for extra code.
        }
    }
