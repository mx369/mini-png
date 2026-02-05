import { Bench } from 'tinybench'

import { compressPng } from '../index.js'

const PNG_BASE64 =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+X2WQAAAAASUVORK5CYII='
const input = Buffer.from(PNG_BASE64, 'base64')

const b = new Bench()

b.add('Native compressPng', async () => {
  await compressPng(input, { level: 4, strip: 'safe', width: 1 })
})

await b.run()

console.table(b.table())
