use std::io::{self};
use std::time::Duration;
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
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



        let server_adress = SocketAddr::new(server_adress_ip, 9000);

        let sock_addr = SockAddr::from(server_adress);

        println!("Sending DISCOVERY message to {:?}", server_adress_ip);

        let set_broadcast_result = self.socket.set_broadcast(true);
        match set_broadcast_result{
            Ok(_) => {

            },
            Err(e) => {
                println!("Error setting broadcast: {}", e);
            },
        }

        

        let mut count = 13;
        let mut cond = true;

    // Use a while loop to wait until we get a succesfull response or we exceed 26 seconds
     while cond {
        let discovery_message = "DISCOVERY";
        self.socket.send_to(discovery_message.as_bytes(), &sock_addr)?;
        println!("Sent DISCOVERY message with local port: {}", self.local_port);


        let mut buf = [MaybeUninit::uninit(); 1024];
        let ris: Result<(String, i32), io::Error>=Result::Ok(("mdjisf".to_string(),4));

        match ris {
            Ok(_) => {
                println!("Connection...");
            },
            Err(e) => {
                println!("Error in connecting: {}", e);
            },
        }
        let socket_response = receive_with_timeout(&self.socket, &mut buf);
  
        match socket_response {
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
            },
            Err(e) => {
                println!("Error in discovering the server: {}", e);
                count = count - 1;
                match count {
                    0 => {
                        cond = false;
                    }
                    _ => {}
                }
            },
        }}

    // If the loop exits without a successful result, return an error
    Err(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        format!("Unable to connect to server"),
    ))
    }

    pub fn notify_disconnection(&self) -> Result<(), io::Error> {

        
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

pub fn receive_with_timeout(socket: &Socket, buf: &mut [MaybeUninit<u8>]) -> Result<(usize, SockAddr), String> {
    // Imposta il timeout di lettura
    socket
        .set_read_timeout(Some(Duration::new(2, 0)))
        .expect("Failed to set read timeout");

    // Tentativo di ricezione
    match socket.recv_from(buf) {
        Ok((size, src)) => Ok((size, src)), // Restituisce la dimensione dei dati ricevuti
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err("Timeout reached".to_string()),
        Err(e) => Err(format!("Failed to receive data: {:?}", e)),
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
