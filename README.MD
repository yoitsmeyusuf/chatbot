# Chatbot Project  

This project is a multi-component chatbot system that integrates a Python-based language model, a Rust-based WebSocket server, and a web-based client interface. The system facilitates real-time communication between users and an AI-powered chatbot.  

## Project Structure  

The project is organized into three main components:  

### 1. **Python LLM**  
- **Description**: Hosts the AI chatbot logic using a pre-trained language model (Mistral 7B).  
- **Files**:  
    - `main.py`: Main chatbot logic and ZMQ communication.  
    - `sender.py`: Sends messages to the Rust server via ZMQ.  
    - `reciever.py`: Receives messages from the Rust server via ZMQ.  

### 2. **Rust Server**  
- **Description**: Acts as a WebSocket server to bridge communication between the web client and the Python backend.  
- **Files**:  
    - `src/main.rs`: Implements WebSocket server and ZMQ communication.  
    - `Cargo.toml`: Rust project dependencies.  
    - `.gitignore`: Ignores build artifacts.  

### 3. **Web Client**  
- **Description**: A simple HTML/JavaScript-based user interface for interacting with the chatbot.  
- **Files**:  
    - `index.html`: Frontend for sending and receiving messages.  

## How It Works  

1. **Web Client**:  
     - Users send messages via the web interface.  
     - Messages are sent to the Rust WebSocket server.  

2. **Rust Server**:  
     - Forwards user messages to the Python backend using ZMQ.  
     - Receives responses from the Python backend and sends them back to the web client.  

3. **Python LLM**:  
     - Processes user messages using the Mistral 7B language model.  
     - Sends responses back to the Rust server via ZMQ.  

## Prerequisites  

- Python 3.8+  
- Rust (with Cargo)  
- Node.js (optional, for additional frontend tooling)  
- PyTorch (with GPU support for optimal performance)  

## Installation  

1. Clone the repository:  
     ```bash  
     git clone <repository-url>  
     cd chatbot  
     ```  

2. Install Python dependencies:  
     ```bash  
     pip install -r requirements.txt  
     ```  

3. Build the Rust server:  
     ```bash  
     cd rust-server  
     cargo build --release  
     ```  

4. Configure the web client:  
     - Update `WS_SERVER_IP` in `index.html` to match your server's IP address.  

## Running the Project  

1. Start the Python backend:  
     ```bash  
     python python-llm/main.py  
     ```  

2. Start the Rust WebSocket server:  
     ```bash  
     cd rust-server  
     cargo run  
     ```  

3. Open the web client:  
     - Open `web-client/index.html` in a browser.  

## Features  

- Real-time communication between users and the chatbot.  
- Scalable architecture with separate components for frontend, backend, and server.  
- GPU-accelerated language model for fast response generation.  

## Acknowledgments  

- [Mistral 7B](https://huggingface.co/mistralai/Mistral-7B-Instruct-v0.1) for the language model.  
- [ZeroMQ](https://zeromq.org/) for inter-process communication.  
- [Tokio](https://tokio.rs/) for asynchronous Rust programming.  
