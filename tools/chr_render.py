#!/usr/bin/env python3
"""CHR tile renderer (S0/S2 forensics aid). Projection only; ROM is truth.
Usage: chr_render.py <rom> <file_off_hex> <len_hex> <cols> <out.png>
NOTE: <file_off_hex> is a FILE offset. CHR 1KB bank n = file 0x20010 + n*0x400.
Renders 2bpp NES tiles to a PNG grid (grey palette)."""
import sys, zlib, struct
def png(path, w, h, rgb):
    raw=b''.join(b'\x00'+rgb[y*w*3:(y+1)*w*3] for y in range(h))
    def chunk(t,d):
        c=struct.pack('>I',len(d))+t+d
        return c+struct.pack('>I',zlib.crc32(t+d)&0xffffffff)
    open(path,'wb').write(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b''))
rom=open(sys.argv[1],'rb').read()
off=int(sys.argv[2],16); ln=int(sys.argv[3],16); cols=int(sys.argv[4])
data=rom[off:off+ln]
ntiles=len(data)//16
rows=(ntiles+cols-1)//cols
tw,th=8,8
w,h=cols*(tw+1)+1, rows*(th+1)+1
rgb=bytearray(w*h*3)
for y in range(h):
    for x in range(w):
        i=(y*w+x)*3
        v=30 if (x%(tw+1)==0 or y%(th+1)==0) else 0
        rgb[i]=rgb[i+1]=rgb[i+2]=v
for t in range(ntiles):
    tx,ty=(t%cols)*(tw+1)+1,(t//cols)*(th+1)+1
    for py in range(8):
        p0,p1=data[t*16+py],data[t*16+py+8]
        for px in range(8):
            c=((p0>>(7-px))&1)|(((p1>>(7-px))&1)<<1)
            g=(0,85,170,255)[c]
            i=((ty+py)*w+tx+px)*3
            rgb[i]=rgb[i+1]=rgb[i+2]=g
png(sys.argv[5],w,h,bytes(rgb))
print('wrote',sys.argv[5],w,'x',h)
