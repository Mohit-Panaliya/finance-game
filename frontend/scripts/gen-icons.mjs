import zlib from 'node:zlib'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)))
const outDir = path.join(root, 'public', 'icons')
fs.mkdirSync(outDir, { recursive: true })

const CRC_TABLE = (() => {
  const t = new Uint32Array(256)
  for (let n = 0; n < 256; n++) {
    let c = n
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
    t[n] = c >>> 0
  }
  return t
})()

function crc32(buf) {
  let c = 0xffffffff
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8)
  return (c ^ 0xffffffff) >>> 0
}

function chunk(type, data) {
  const len = Buffer.alloc(4)
  len.writeUInt32BE(data.length, 0)
  const typeBuf = Buffer.from(type, 'ascii')
  const crcBuf = Buffer.alloc(4)
  crcBuf.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0)
  return Buffer.concat([len, typeBuf, data, crcBuf])
}

function makePng(size) {
  const px = Buffer.alloc(size * size * 4)
  const set = (x, y, r, g, b, a = 255) => {
    if (x < 0 || y < 0 || x >= size || y >= size) return
    const i = (y * size + x) * 4
    px[i] = r; px[i + 1] = g; px[i + 2] = b; px[i + 3] = a
  }
  // Match public/icon.svg: dark rounded tile, blue wallet body, green dot.
  const BG = [15, 17, 21]
  const TILE = [23, 26, 32]
  const ACCENT = [76, 141, 255]
  const ACCENT_DARK = [64, 118, 214]
  const SUCCESS = [52, 201, 138]
  const s = size / 512
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const rx = Math.round(x / s), ry = Math.round(y / s)
      let r = BG[0], g = BG[1], b = BG[2]

      // Outer tile with a large corner radius (96/512 of the canvas).
      const tileInset = 36
      const tileR = 96
      if (inTile(rx, ry, tileInset, 512 - tileInset, tileR)) {
        r = TILE[0]; g = TILE[1]; b = TILE[2]

        // Wallet body: rounded rect 112,168 → 288x176, radius 26.
        if (inRect(rx, ry, 112, 168, 288, 176, 26)) {
          r = ACCENT[0]; g = ACCENT[1]; b = ACCENT[2]

          // Top seam highlight (thin darker band).
          if (ry >= 216 && ry <= 222) {
            r = ACCENT_DARK[0]; g = ACCENT_DARK[1]; b = ACCENT_DARK[2]
          }

          // Card slot with a green dot.
          if (inRect(rx, ry, 286, 228, 72, 56, 14)) {
            r = TILE[0]; g = TILE[1]; b = TILE[2]
            const dx = rx - 322, dy = ry - 256
            if (dx * dx + dy * dy <= 12 * 12) {
              r = SUCCESS[0]; g = SUCCESS[1]; b = SUCCESS[2]
            }
          }
        }
      }
      set(x, y, r, g, b)
    }
  }
  const raw = Buffer.alloc(size * (size * 4 + 1))
  for (let y = 0; y < size; y++) {
    raw[y * (size * 4 + 1)] = 0
    px.copy(raw, y * (size * 4 + 1) + 1, y * size * 4, (y + 1) * size * 4)
  }
  const ihdr = Buffer.alloc(13)
  ihdr.writeUInt32BE(size, 0)
  ihdr.writeUInt32BE(size, 4)
  ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', zlib.deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0))
  ])
}

/** Rounded-rectangle hit test with per-corner radius. */
function inRect(x, y, rx, ry, w, h, radius) {
  if (x < rx || y < ry || x >= rx + w || y >= ry + h) return false
  const corners = [
    [rx + radius, ry + radius],
    [rx + w - radius - 1, ry + radius],
    [rx + radius, ry + h - radius - 1],
    [rx + w - radius - 1, ry + h - radius - 1]
  ]
  for (const [cx, cy] of corners) {
    const dx = x < cx ? cx - x : x > cx ? x - cx : 0
    const dy = y < cy ? cy - y : y > cy ? y - cy : 0
    if (dx * dx + dy * dy > radius * radius) return false
  }
  return true
}

function inTile(x, y, inset, max, radius) {
  return inRect(x, y, inset, inset, max - inset * 2, max - inset * 2, radius)
}

for (const size of [192, 512]) {
  const file = path.join(outDir, `icon-${size}.png`)
  fs.writeFileSync(file, makePng(size))
  console.log('wrote', file)
}
