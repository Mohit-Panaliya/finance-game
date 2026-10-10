import zlib from 'node:zlib'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)))
const publicDir = path.join(root, 'public')
const outDir = path.join(publicDir, 'icons')
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

/* The mark is authored in a 512-unit design space and rasterised with 4x4
   supersampling so every icon (192, 180, 48, 16 …) keeps clean anti-aliased
   edges instead of the stair-stepped placeholder PNGs. */

const BG = [15, 17, 21]
const TILE_TOP = [32, 37, 46]
const TILE_BOT = [18, 20, 26]
const BORDER = [43, 49, 60]
const ACCENT_TOP = [96, 160, 255]
const ACCENT_BOT = [58, 110, 210]
const SUCCESS = [52, 201, 138]

function lerp(a, b, t) {
  return [
    Math.round(a[0] + (b[0] - a[0]) * t),
    Math.round(a[1] + (b[1] - a[1]) * t),
    Math.round(a[2] + (b[2] - a[2]) * t)
  ]
}

/** Tile fill: a soft top-to-bottom gradient so the mark reads as a lit surface. */
function tileAt(y) {
  return lerp(TILE_TOP, TILE_BOT, y / 512)
}

/** Wallet fill: same treatment, anchored to the wallet's own vertical span. */
function accentAt(y) {
  return lerp(ACCENT_TOP, ACCENT_BOT, (y - 168) / 176)
}

function inRoundRect(x, y, cx, cy, w, h, r) {
  const dx = Math.max(Math.abs(x - cx) - (w / 2 - r), 0)
  const dy = Math.max(Math.abs(y - cy) - (h / 2 - r), 0)
  return dx * dx + dy * dy <= r * r
}

function inCircle(x, y, cx, cy, r) {
  const dx = x - cx
  const dy = y - cy
  return dx * dx + dy * dy <= r * r
}

/** Wallet body + seam + card slot + dot, scaled about the canvas centre. */
function paintMark(x, y, scale) {
  const sx = (x - 256) / scale + 256
  const sy = (y - 256) / scale + 256
  if (!inRoundRect(sx, sy, 256, 256, 288, 176, 26)) return null
  let c = accentAt(sy)
  if (Math.abs(sy - 219) <= 5) c = accentAt(168)
  if (inRoundRect(sx, sy, 322, 256, 72, 56, 14)) {
    c = tileAt(sy)
    if (inCircle(sx, sy, 322, 256, 12)) c = SUCCESS
  }
  return c
}

function sample(x, y, mode) {
  const tile = tileAt(y)
  if (mode === 'maskable') {
    // Full-bleed tile-coloured canvas, mark centred inside the safe zone.
    return paintMark(x, y, 1.42) || tile
  }
  let c = BG
  if (inRoundRect(x, y, 256, 256, 452, 452, 104)) c = BORDER
  if (inRoundRect(x, y, 256, 256, 436, 436, 96)) c = tile
  const mark = paintMark(x, y, mode === 'apple' ? 1.06 : 1)
  return mark || c
}

function makePng(size, mode) {
  const SS = 4
  const px = Buffer.alloc(size * size * 4)
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      let r = 0
      let g = 0
      let b = 0
      for (let sy = 0; sy < SS; sy++) {
        for (let sx = 0; sx < SS; sx++) {
          const dx = ((x + (sx + 0.5) / SS) * 512) / size
          const dy = ((y + (sy + 0.5) / SS) * 512) / size
          const c = sample(dx, dy, mode)
          r += c[0]
          g += c[1]
          b += c[2]
        }
      }
      const n = SS * SS
      const i = (y * size + x) * 4
      px[i] = Math.round(r / n)
      px[i + 1] = Math.round(g / n)
      px[i + 2] = Math.round(b / n)
      px[i + 3] = 255
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
  ihdr[8] = 8
  ihdr[9] = 6
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', zlib.deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0))
  ])
}

/** Minimal ICO container with PNG-compressed entries (supported by all modern browsers). */
function makeIco(pngs) {
  const count = pngs.length
  const header = Buffer.alloc(6)
  header.writeUInt16LE(0, 0)
  header.writeUInt16LE(1, 2)
  header.writeUInt16LE(count, 4)
  const dir = Buffer.alloc(16 * count)
  let offset = 6 + 16 * count
  pngs.forEach(({ size, data }, i) => {
    const b = i * 16
    dir[b] = size >= 256 ? 0 : size
    dir[b + 1] = size >= 256 ? 0 : size
    dir[b + 2] = 0
    dir[b + 3] = 0
    dir.writeUInt16LE(1, b + 4)
    dir.writeUInt16LE(32, b + 6)
    dir.writeUInt32LE(data.length, b + 8)
    dir.writeUInt32LE(offset, b + 12)
    offset += data.length
  })
  return Buffer.concat([header, dir, ...pngs.map((p) => p.data)])
}

/** Decode one of our own non-interlaced truecolour PNGs for verification. */
function analysePng(buf) {
  let pos = 8
  let width = 0
  let height = 0
  const idat = []
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos)
    const type = buf.toString('ascii', pos + 4, pos + 8)
    const data = buf.subarray(pos + 8, pos + 8 + len)
    if (type === 'IHDR') {
      width = data.readUInt32BE(0)
      height = data.readUInt32BE(4)
    } else if (type === 'IDAT') {
      idat.push(data)
    }
    pos += 12 + len
  }
  const raw = zlib.inflateSync(Buffer.concat(idat))
  const colors = new Set()
  let nonBlank = 0
  const stride = width * 4 + 1
  for (let y = 0; y < height; y++) {
    const row = raw.subarray(y * stride + 1, (y + 1) * stride)
    for (let x = 0; x < width; x++) {
      const i = x * 4
      const r = row[i]
      const g = row[i + 1]
      const b = row[i + 2]
      colors.add((r << 16) | (g << 8) | b)
      if (r !== BG[0] || g !== BG[1] || b !== BG[2]) nonBlank++
    }
  }
  return {
    width,
    height,
    colors: colors.size,
    coverage: nonBlank / (width * height)
  }
}

const outputs = [
  { file: 'icon-192.png', size: 192, mode: 'regular' },
  { file: 'icon-512.png', size: 512, mode: 'regular' },
  { file: 'icon-maskable-512.png', size: 512, mode: 'maskable' },
  { file: 'apple-touch-icon.png', size: 180, mode: 'apple' }
]

let failed = false
for (const { file, size, mode } of outputs) {
  const png = makePng(size, mode)
  fs.writeFileSync(path.join(outDir, file), png)
  const stats = analysePng(png)
  const ok = stats.colors > 50 && stats.coverage > 0.2
  if (!ok) failed = true
  console.log(
    `${ok ? 'ok  ' : 'FAIL'} icons/${file} ${stats.width}x${stats.height} ` +
      `colors=${stats.colors} coverage=${(stats.coverage * 100).toFixed(1)}%`
  )
}

const ico = makeIco([
  { size: 16, data: makePng(16, 'regular') },
  { size: 32, data: makePng(32, 'regular') },
  { size: 48, data: makePng(48, 'regular') }
])
fs.writeFileSync(path.join(outDir, 'favicon.ico'), ico)
fs.copyFileSync(path.join(outDir, 'favicon.ico'), path.join(publicDir, 'favicon.ico'))
console.log(`ok   icons/favicon.ico (16/32/48) + public/favicon.ico`)

if (failed) {
  console.error('icon verification failed')
  process.exit(1)
}
