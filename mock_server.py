from http.server import BaseHTTPRequestHandler, HTTPServer
import json

hostName = "localhost"
serverPort = 8000

DATA = {
    "peers": [
        {"pubkey": "mmFPYMNnhYmWF7OVW7Zvxfll95mQ634JFZNznrljonA=", "endpoint": "10.32.0.1", "port": 40000, "ip": "10.85.0.34"},
        {"pubkey": "9jWWDhS9Fr4hHXHQNTnQIAIpLIrcBNy4yA2aXbG1jzw=", "endpoint": "10.32.0.2", "port": 40000, "ip": "10.85.0.35"},
    ]
}

class MyServer(BaseHTTPRequestHandler):
    def do_POST(self):
        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        data = json.dumps(DATA)
        self.wfile.write(data.encode('utf-8'))
    
if __name__ == "__main__":        
    webServer = HTTPServer((hostName, serverPort), MyServer)
    print("Server started http://%s:%s" % (hostName, serverPort))

    try:
        webServer.serve_forever()
    except KeyboardInterrupt:
        pass

    webServer.server_close()
    print("Server stopped.")