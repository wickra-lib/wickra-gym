const test = require('node:test')
const assert = require('node:assert')
const native = require('..')

test('Env class is exported', () => {
  assert.strictEqual(typeof native.Env, 'function')
})

test('Env has command and version methods', () => {
  for (const name of ['command', 'version']) {
    assert.strictEqual(typeof native.Env.prototype[name], 'function', `missing method: ${name}`)
  }
})
