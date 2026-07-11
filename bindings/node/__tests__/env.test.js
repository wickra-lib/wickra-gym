const test = require('node:test')
const assert = require('node:assert')
const { Env } = require('..')

const SPEC = JSON.stringify({
  dataset_ref: 'smoke',
  symbol: 'TEST',
  observation: { features: [{ kind: 'price', field: 'close' }] },
  action_space: { type: 'discrete', n: 3 },
  reward: 'pnl',
  episode: { max_steps: 100, warmup: 0 },
})

const CANDLES = Array.from({ length: 5 }, (_, i) => ({
  ts: i,
  open: 100 + i,
  high: 100 + i,
  low: 100 + i,
  close: 100 + i,
}))

test('load, reset and step', () => {
  const env = new Env(SPEC)
  assert.strictEqual(env.command(JSON.stringify({ cmd: 'load', candles: CANDLES })), '{"ok":true}')
  const reset = JSON.parse(env.command(JSON.stringify({ cmd: 'reset' })))
  assert.deepStrictEqual(reset.observation, [100.0])
  const step = JSON.parse(env.command(JSON.stringify({ cmd: 'step', action: 2 })))
  assert.strictEqual(step.reward, 1.0)
  assert.strictEqual(step.terminated, false)
  assert.strictEqual(step.info.step, 1.0)
})

test('version is a string', () => {
  const env = new Env(SPEC)
  assert.strictEqual(typeof env.version(), 'string')
})

test('a bad spec throws', () => {
  assert.throws(() => new Env(JSON.stringify({ not: 'a spec' })))
})
