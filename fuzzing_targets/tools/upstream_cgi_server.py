import argparse
import socket
import threading
import struct

parser = argparse.ArgumentParser(description='Upstream server for testing HTTP fuzzing targets')
parser.add_argument('--host', type=str, default='0.0.0.0', help='Host to listen on')
parser.add_argument('--port', type=int, default=10781, help='Port to listen on')
parser.add_argument('--response', type=str, help='Response to send back (path to file)')
parser.add_argument('protocol', type=str, help='Protocol to use (scgi, fastcgi, uwsgi, ajp)')

SUPPORTED_PROTOCOLS = ['scgi' ,'fastcgi', 'uwsgi', 'ajp']

STATUS_CODE = 200
STATUS_MSG = b'OK'
BODY = b'<html><body>Pong</body></html>'
HEADERS = { b'Content-Type': b'text/html', b'Content-Length': str(len(BODY)).encode() }
JOINED_HEADERS = b''.join([k + b': ' + v + b'\r\n' for k, v in HEADERS.items()])
CGI_STYLE_RESPONSE = b'Status: %d %s\r\n%s\r\n%s' % (STATUS_CODE, STATUS_MSG, JOINED_HEADERS, BODY)
HTTP_STYLE_RESPONSE = b'HTTP/1.1 %d %s\r\n%s\r\n%s' % (STATUS_CODE, STATUS_MSG, JOINED_HEADERS, BODY)

def ajp_encode_string(s):
    """Encodes a string for AJP protocol."""
    if s is None:
        return struct.pack('!h', -1)
    if isinstance(s, str):
        s = s.encode('utf-8')
    length = len(s)
    return struct.pack(f'!H{length}sB', length, s, 0)

def ajp_make_packet(data):
    """Makes an AJP packet."""
    return struct.pack(f'!BBH', 0x41, 0x42, len(data)) + data

def ajp_encode_headers(headers):
    """Encodes headers for AJP protocol."""
    header_data = b''
    for key, value in headers.items():
        header_data += ajp_encode_string(key)
        header_data += ajp_encode_string(value)
    return struct.pack('!H', len(headers)) + header_data

def ajp_encode_body(body, max_chunk_size=8192):
    """Encodes body for AJP protocol."""
    body_chunks = []
    for i in range(0, len(body), max_chunk_size):
        chunk = body[i:i + max_chunk_size]
        chunk_length = len(chunk)
        body_chunks.append(ajp_make_packet(struct.pack(f'!BH{chunk_length}sB', 0x03, chunk_length, chunk, 0)))
    return b''.join(body_chunks)

DEFAULT_RESPONSES = {
    'scgi': CGI_STYLE_RESPONSE,
    'fastcgi':
        struct.pack('!BBHHBx', 1, 6 # FCGI_STDOUT
            , 1, len(CGI_STYLE_RESPONSE), 0) + CGI_STYLE_RESPONSE
        + struct.pack('!BBHHBx', 1, 6 # FCGI_STDOUT
            , 1, 0, 0)
        + struct.pack('!BBHHBx', 1, 3 # FCGI_END_REQUEST
            , 1, 8, 0)
        + struct.pack('!IB3x', 0, 0),
    'uwsgi': HTTP_STYLE_RESPONSE,
    'ajp':
        ajp_make_packet(struct.pack('!BH', 0x04, STATUS_CODE) + ajp_encode_string(STATUS_MSG) + ajp_encode_headers(HEADERS))
        + ajp_encode_body(BODY)
        + ajp_make_packet(struct.pack('!Bb', 0x05, False)),
}

BUFFER_SIZE = 4096

response = b''

def handle_recv_all(conn: socket.socket, addr, response):
    data = b""
    conn.setblocking(False)
    
    while True:
        try:
            part = conn.recv(BUFFER_SIZE)
            data += part
        except Exception:
            break

    conn.sendall(response)
    conn.close()

HANDLERS = {
    'scgi': handle_recv_all,
    'fastcgi': handle_recv_all,
    'uwsgi': handle_recv_all,
    'ajp': handle_recv_all,
}

if __name__ == '__main__':
    args = parser.parse_args()
    if args.protocol not in SUPPORTED_PROTOCOLS:
        print(f'Unsupported protocol: {args.protocol}')
        exit(1)
    if args.response:
        response = open(args.response, 'rb').read()
    else:
        response = DEFAULT_RESPONSES[args.protocol]
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind((args.host, args.port))
        s.listen()
        print(f'Listening on {args.host}:{args.port}...')
        while True:
            conn, addr = s.accept()
            print(f'Connected by {addr[0]}:{addr[1]}')
            threading.Thread(target=HANDLERS[args.protocol], args=(conn, addr, response)).start()
