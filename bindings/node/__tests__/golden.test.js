const test = require('node:test')
const assert = require('node:assert')
const fs = require('node:fs')
const path = require('node:path')
const { Env } = require('..')

// bindings/node/__tests__ -> repo root -> golden/
const GOLDEN = path.resolve(__dirname, '..', '..', '..', 'golden')

function cases() {
  if (!fs.existsSync(GOLDEN)) return []
  return fs
    .readdirSync(GOLDEN, { withFileTypes: true })
    .filter((d) => d.isDirectory() && fs.existsSync(path.join(GOLDEN, d.name, 'spec.json')))
    .map((d) => d.name)
}

const CASES = cases()

test('cross-language golden rollouts', { skip: CASES.length === 0 ? 'golden fixtures not present yet' : false }, () => {
  for (const name of CASES) {
    const dir = path.join(GOLDEN, name)
    const spec = fs.readFileSync(path.join(dir, 'spec.json'), 'utf8')
    const candles = JSON.parse(fs.readFileSync(path.join(dir, 'candles.json'), 'utf8'))
    const expected = JSON.parse(fs.readFileSync(path.join(dir, 'expected.json'), 'utf8'))

    const env = new Env(spec)
    env.command(JSON.stringify({ cmd: 'load', candles }))
    const resetCmd = expected.seed != null ? { cmd: 'reset', seed: expected.seed } : { cmd: 'reset' }
    const reset = JSON.parse(env.command(JSON.stringify(resetCmd)))
    assert.deepStrictEqual(reset, expected.reset, `${name}: reset`)

    const trajectory = expected.actions.map((action) =>
      JSON.parse(env.command(JSON.stringify({ cmd: 'step', action })))
    )
    assert.deepStrictEqual(trajectory, expected.trajectory, `${name}: trajectory`)
  }
})
