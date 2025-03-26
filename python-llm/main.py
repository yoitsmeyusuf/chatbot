import zmq
import time
import os
from transformers import AutoTokenizer, AutoModelForCausalLM
import torch

# Suppress TensorFlow warnings (since we're using PyTorch)
os.environ['TF_CPP_MIN_LOG_LEVEL'] = '3' 
os.environ['CUDA_LAUNCH_BLOCKING'] = '1'  # For better CUDA error messages

class MistralChatBot:
    def __init__(self, sub_address="tcp://localhost:5555", pub_address="tcp://*:5556"):
        # Initialize ZMQ
        self.context = zmq.Context()
        self.sub = self.context.socket(zmq.SUB)
        self.sub.setsockopt(zmq.SUBSCRIBE, b"")
        self.sub.connect(sub_address)
        self.pub = self.context.socket(zmq.PUB)
        self.pub.bind(pub_address)
        
        # Model loading
        print("⏳ Loading Mistral 7B...")
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        model_name = "mistralai/Mistral-7B-Instruct-v0.1"
        
        # Tokenizer config
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        if self.tokenizer.pad_token is None:
            self.tokenizer.pad_token = self.tokenizer.eos_token
            
        # Model config
        kwargs = {
            "torch_dtype": torch.float16,
            "device_map": "auto",
            "low_cpu_mem_usage": True
        }
        
        if self.device == "cuda":
            kwargs["attn_implementation"] = "flash_attention_2"  # Faster attention
            
        self.model = AutoModelForCausalLM.from_pretrained(model_name, **kwargs)
        print(f"✅ Model loaded on {self.device.upper()}")

    def generate_response(self, prompt):
        try:
            # Prepare messages
            messages = [{"role": "user", "content": prompt}]
            
            # Tokenize
            inputs = self.tokenizer.apply_chat_template(
                messages,
                return_tensors="pt",
                return_attention_mask=True,
                padding=True
            ).to(self.device)
            
            # Generate
            outputs = self.model.generate(
                inputs,
                max_new_tokens=200,
                temperature=0.7,
                do_sample=True,
                pad_token_id=self.tokenizer.pad_token_id,
                eos_token_id=self.tokenizer.eos_token_id
            )
            
            return self.tokenizer.decode(outputs[0], skip_special_tokens=True)
            
        except torch.cuda.OutOfMemoryError:
            torch.cuda.empty_cache()
            return "Error: Out of memory, please try a shorter message"
        except Exception as e:
            print(f"⚠️ Generation error: {str(e)}")
            return "Sorry, I encountered an error processing your message."

    def run(self):
        try:
            print("🚀 Chatbot ready - Waiting for messages...")
            while True:
                try:
                    message = self.sub.recv_string(flags=zmq.NOBLOCK)
                    if '|' in message:
                        client_id, text = message.split('|', 1)
                        response = self.generate_response(text)
                        self.pub.send_string(f"{client_id}|{response}")
                except zmq.Again:
                    time.sleep(0.1)
                except Exception as e:
                    print(f"⚠️ Processing error: {str(e)}")
        except KeyboardInterrupt:
            print("\n🛑 Shutting down...")
        finally:
            self.cleanup()

    def cleanup(self):
        self.sub.close()
        self.pub.close()
        self.context.term()
        if torch.cuda.is_available():
            torch.cuda.empty_cache()
        print("✅ Resources cleaned up")

if __name__ == "__main__":
    bot = MistralChatBot()
    try:
        bot.run()
    except Exception as e:
        print(f"🔥 Fatal error: {str(e)}")
        bot.cleanup()