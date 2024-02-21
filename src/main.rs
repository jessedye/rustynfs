use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use std::io;

// Basic NFS server struct
struct NFSServer {
    // Simulated file system. In reality, you would implement more sophisticated logic.
    file_system: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl NFSServer {
    // Method to handle NFS read request
    async fn handle_read_request(&self, filename: String) -> Option<Vec<u8>> {
        let file_system = self.file_system.lock().await;
        file_system.get(&filename).cloned()
    }

    // Method to handle NFS write request
    async fn handle_write_request(&self, filename: String, content: Vec<u8>) {
        let mut file_system = self.file_system.lock().await;
        file_system.insert(filename, content);
    }
}

// Function to handle NFS client connections
async fn handle_client(mut stream: TcpStream, server: Arc<NFSServer>) -> io::Result<()> {
    // Print a debug log when a client connects
    println!("Client connected: {:?}", stream.peer_addr());

    // Read the request from the client
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await?;

    // Basic parsing logic for demonstration purposes
    let request = String::from_utf8_lossy(&buf[..n]);
    let parts: Vec<&str> = request.split(',').collect();

    // Determine the type of request and handle it accordingly
    match parts[0] {
        "READ" => {
            let filename = parts[1].to_string();
            if let Some(content) = server.handle_read_request(filename).await {
                stream.write_all(&content).await?;
            }
        }
        "WRITE" => {
            let filename = parts[1].to_string();
            let content = parts[2].as_bytes().to_vec();
            server.handle_write_request(filename, content).await;
        }
        _ => {} // Handle other types of requests as needed
    }

    Ok(())
}

// Function to start the NFS server
async fn start_server(addr: String, server: Arc<NFSServer>) -> io::Result<()> {
    let listener = TcpListener::bind(format!("{}:2049", addr)).await?;
    println!("NFS server listening on {}", addr);

    // Accept incoming client connections and handle them asynchronously
    while let Ok((stream, _)) = listener.accept().await {
        let server = server.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, server).await {
                eprintln!("Error handling client: {}", err);
            }
        });
    }

    Ok(())
}

// Function to run the NFS server
async fn run_server(addr: String) -> io::Result<()> {
    // Initialize the file system with some initial data
    let file_system: HashMap<String, Vec<u8>> = HashMap::new();
    let server = Arc::new(NFSServer {
        file_system: Arc::new(Mutex::new(file_system)),
    });

    // Start the NFS server
    start_server(addr, server).await
}

// Main function
#[tokio::main]
async fn main() -> io::Result<()> {
    // Define the address and port to listen on
    let addr = "0.0.0.0".to_string(); // Listen on all available interfaces
    run_server(addr).await
}
