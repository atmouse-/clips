#!/usr/bin/python3
# coding: utf-8

import socket
import struct
import select
from io import BytesIO
from PIL import Image
import zlib

HOST = "127.0.0.1"
PORT = 9091

class clipc():
    def __init__(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    
    def outfile(self, selection):
        outfp = open("/tmp/1.png", 'wb')
        # selection to bmap
        tempBuff = BytesIO()
        tempBuff.write(selection)
        tempBuff.seek(0)
        image = Image.open(tempBuff)

        image.convert("RGB").save(outfp, "PNG")
        outfp.close()

    def conn(self):
        self.s.connect((HOST, PORT))
        magic = bytearray([16, 1, 24, 3])
        self.s.sendall(magic)
        # recv: type, content_length
        rsock = self.s.makefile('rb', -1)
        while 1:
            ready = select.select([rsock], [], [], 1)
            if ready[0]:
                # st = rsock.read(8)
                # st_type, st_length = struct.unpack('=il', st)
                # print(st_type, st_length)
                # selection = rsock.read(st_length)
                selection = rsock.read()
                st_type = 2
                if not selection:
                    print("rsock closed")
                    return
                # print("got clipboard: {}".format(selection))
                self.outfile(selection)

if __name__ == "__main__":
    c = clipc()
    c.conn()
