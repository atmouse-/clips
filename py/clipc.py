#!/usr/bin/python2
# coding: utf-8

import socket
import win32clipboard as wincb
import win32con
import struct
import select
from cStringIO import StringIO
from PIL import Image
import zlib

HOST = "127.0.0.1"
PORT = 9091

class clipc():
    def __init__(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    
    def set_clipboard(self, st_type, selection):
        #selection = selection.replace('\n','\r\n')
        print("start set clipboard")
        if st_type == 1:
            cftype = win32con.CF_TEXT
        elif st_type == 2:
            #selection = zlib.decompress(selection)
            cftype = win32con.CF_DIB
        else:
            raise(Exception("st_type: {} unknow".format(st_type)))
        wincb.OpenClipboard()
        wincb.EmptyClipboard()

        # selection to bmap
        tempBuff = StringIO()
        tempBuff.write(selection)
        tempBuff.seek(0)
        image = Image.open(tempBuff)

        output = StringIO()
        image.convert("RGB").save(output, "BMP")
        data = output.getvalue()[14:]
        output.close()

        wincb.SetClipboardData(cftype, data)
        #print wincb.GetClipboardData(win32con.CF_TEXT)  #'copy token'
        wincb.CloseClipboard()

    def conn(self):
        self.s.connect((HOST, PORT))
        magic = str(bytearray([16, 1, 24, 3]))
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
                self.set_clipboard(st_type, selection)

if __name__ == "__main__":
    c = clipc()
    c.conn()
