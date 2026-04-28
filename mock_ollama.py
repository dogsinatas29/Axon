import sys
from http.server import HTTPServer, BaseHTTPRequestHandler
import json

class SimpleHTTPRequestHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/api/tags':
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b'{"models": [{"name": "llama3"}]}')

    def do_POST(self):
        if self.path == '/api/generate':
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            
            response = {
                "model": "llama3",
                "created_at": "2023-08-04T19:22:45.499127Z",
                "response": "Here is the requested implementation.\n\n[OUTPUT]\nFILE: src/hello.py\n```python\ndef hello():\n    print('Hello from Local LLM!')\n\ndef new_func():\n    return 42\n```\n[/OUTPUT]\n",
                "done": True,
                "total_duration": 5000000,
                "eval_count": 134,
                "eval_duration": 4000000
            }
            self.send_response(200)
            self.end_headers()
            self.wfile.write(json.dumps(response).encode())

print("Starting mock Ollama on port 11434...")
httpd = HTTPServer(('localhost', 11434), SimpleHTTPRequestHandler)
httpd.serve_forever()
