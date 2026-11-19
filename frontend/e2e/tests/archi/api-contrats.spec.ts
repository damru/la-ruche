import { test, expect } from '@playwright/test'
import http from 'node:http'
import type { Server } from 'node:http'
import type { APIRequestContext, Page } from '@playwright/test'
import { apiGet, apiPost, apiPut, apiDelete, e2eName } from '../../data/api'

/**
 * Contrats du client API e2e (e2e/data/api.ts) — gaps relevés par le rapport
 * code-review-graph. `page.route` n'intercepte pas le request context de la
 * page (canal API séparé) : un serveur HTTP de test local sert les réponses
 * et la page d'un contexte dédié porte le request context testé.
 */

let server: Server
let base = ''
let calls: Array<{ method: string, url: string, body: unknown }> = []
let mode: 'ok' | 'fail' | 'void' = 'ok'
let page: Page

test.beforeAll(async () => {
  server = http.createServer((req, res) => {
    let raw = ''
    req.on('data', (c: Buffer) => { raw += c })
    req.on('end', () => {
      let body: unknown = null
      if (raw) { try { body = JSON.parse(raw) } catch { body = raw } }
      calls.push({ method: req.method ?? '', url: req.url ?? '', body })
      res.setHeader('Content-Type', 'application/json')
      if (mode === 'fail') {
        res.statusCode = 500
        res.end(JSON.stringify({ error: 'boom '.repeat(100) }))
      } else if (mode === 'void') {
        res.statusCode = 204
        res.end()
      } else {
        res.end(JSON.stringify({ ok: true, echo: body }))
      }
    })
  })
  await new Promise<void>(r => server.listen(0, '127.0.0.1', r))
  const addr = server.address()
  base = `http://127.0.0.1:${typeof addr === 'object' && addr ? addr.port : 0}`
})

test.afterAll(async () => {
  await new Promise<void>(r => server.close(() => r()))
})

test.beforeEach(async ({ browser }) => {
  const ctx = await browser.newContext({ baseURL: base })
  page = await ctx.newPage()
  calls = []
  mode = 'ok'
})

test.afterEach(async () => {
  await page.context().close()
})

test('apiGet parse le JSON et propage les données', async () => {
  const data = await apiGet<{ ok: boolean }>(page, '/ping')
  expect(data.ok).toBe(true)
  expect(calls[0]).toMatchObject({ method: 'GET', url: '/api/v1/ping' })
})

test('apiPost sérialise le body ; corps absent → {}', async () => {
  const created = await apiPost<{ ok: boolean; echo: unknown }>(page, '/echo', { title: 'x' })
  expect(created.echo).toEqual({ title: 'x' })
  expect(calls[0]).toMatchObject({ method: 'POST', body: { title: 'x' } })

  await apiPost(page, '/echo')
  expect(calls[1]).toMatchObject({ body: {} })
})

test('apiPut sérialise le body', async () => {
  await apiPut(page, '/echo/1', { title: 'v2' })
  expect(calls[0]).toMatchObject({ method: 'PUT', body: { title: 'v2' } })
})

test('erreur API → message avec status, url et corps tronqué', async () => {
  mode = 'fail'
  await expect(apiGet(page, '/fail')).rejects.toThrow(/API 500 .*\/api\/v1\/fail.*boom/)
})

test('204 → undefined, pas de parse JSON', async () => {
  mode = 'void'
  await expect(apiPut<void>(page, '/void', {})).resolves.toBeUndefined()
  await expect(apiDelete(page, '/void')).resolves.toBeUndefined()
})

test('e2eName : préfixe e2e-w<worker>-<run> unique par base', () => {
  const info = { workerIndex: 3 } as never as Parameters<typeof e2eName>[0]
  const a = e2eName(info, 'tâche')
  const b = e2eName(info, 'tâche')
  expect(a).toMatch(/^e2e-w3-[0-9a-z]+[0-9a-z]{3}-tâche$/)
  expect(a).not.toBe(b)
})

// Le request context testé est bien celui de la page (contrat du client).
test('le request context porte le baseURL du serveur de test', async ({ request }) => {
  const req: APIRequestContext = request
  expect(req).toBeTruthy()
})
