import socket
import time
import threading
import argparse

parser = argparse.ArgumentParser(description='Upstream server for testing HTTP fuzzing targets')
parser.add_argument('--host', type=str, default='0.0.0.0', help='Host to listen on')
parser.add_argument('--port', type=int, default=10780, help='Port to listen on')
parser.add_argument('--response', type=str, help='Response to send back (path to file)')

DEFAULT_RESPONSE = b'HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 30\r\n\r\n<html><body>Pong</body></html>'

def handle_connection(conn, addr, response):
    print(f'Connected by {addr[0]}:{addr[1]}')
    while True:
        try:
            data = conn.recv(1024)
            data_to_print = data.decode().replace('\r', '[cr]').replace('\n', '[lf]\n')
            print(f'Received ({len(data)}) from {addr[0]}:{addr[1]}: {data_to_print}')
            time.sleep(0.1)
            conn.sendall(response)
            if b'Connection: close' in data:
                print(f'Connection closed from {addr[0]}:{addr[1]} according to connection header')
                conn.close()
                break
        except Exception as e:
            print(f'Connection closed from {addr[0]}:{addr[1]} due to error: {e}')
            break

if __name__ == '__main__':
    args = parser.parse_args()
    if args.response:
        response = open(args.response, 'rb').read()
    else:
        response = DEFAULT_RESPONSE
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind((args.host, args.port))
        s.listen()
        print(f'Listening on {args.host}:{args.port}...')
        while True:
            conn, addr = s.accept()
            threading.Thread(target=handle_connection, args=(conn, addr, response)).start()
