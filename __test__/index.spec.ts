import test from 'ava'

import { compressPng } from '../index'

const PNG_BASE64 =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVR4nGNgAAIAAAUAAXpeqz8AAAAASUVORK5CYII='

test('compressPng returns a PNG buffer', async (t) => {
  const input = Buffer.from(PNG_BASE64, 'base64')
  const output = await compressPng(input, { level: 4, strip: 'safe', width: 1 })

  t.true(Buffer.isBuffer(output))
  t.true(output.length > 0)
  t.true(
    output.slice(0, 8).equals(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])),
  )
})
