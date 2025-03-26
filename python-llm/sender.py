import zmq
import time

class Sender:
    def __init__(self, address="tcp://localhost:5556"):
        self.context = zmq.Context()
        self.pub_socket = self.context.socket(zmq.PUB)
        self.pub_socket.bind(address)

    def send_messages_loops(self, message="Merhaba Rust!", interval=1):
        while True:
            self.pub_socket.send_string(message)
            time.sleep(interval)
    
    def send_messages(self, message="Merhaba Rust!"):
        self.pub_socket.send_string(message)

    def close(self):
        self.pub_socket.close()
        self.context.term()

    def send_multiple_messages(self, messages, interval=1):
        for message in messages:
            self.pub_socket.send_string(message)
            time.sleep(interval)

    def send_message_with_timestamp(self, message="Merhaba Rust!"):
        timestamped_message = f"{message} - {time.time()}"
        self.pub_socket.send_string(timestamped_message)
