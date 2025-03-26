import zmq
import time

class Receiver:
    def __init__(self, address="tcp://localhost:5555", timeout=5000):
        self.context = zmq.Context()
        self.sub = self.context.socket(zmq.SUB)
        self.sub.setsockopt(zmq.RCVTIMEO, timeout)
        self.sub.setsockopt(zmq.SUBSCRIBE, b"")
        self.sub.connect(address)
        time.sleep(1)  # Slow joiner problemi için bekle

    def receive_messages(self):
        while True:
            try:
                msg = self.sub.recv(flags=zmq.NOBLOCK)
                print("Alındı:", len(msg), "byte")
            except zmq.Again:
                print("⏳ Bekleniyor... Veri gelmedi")
                time.sleep(0.1)

    def close(self):
        self.sub.close()
        self.context.term()
        print("Bağlantı kapatıldı")

    def set_timeout(self, timeout):
        self.sub.setsockopt(zmq.RCVTIMEO, timeout)
        print(f"Yeni zaman aşımı süresi: {timeout} ms")

    def set_subscription(self, topic):
        self.sub.setsockopt(zmq.SUBSCRIBE, topic.encode('utf-8'))
        print(f"Yeni abone olunan konu: {topic}")

