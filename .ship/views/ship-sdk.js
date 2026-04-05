/**
 * Ship View SDK — postMessage bridge between view iframes and Studio host.
 *
 * The host (ViewHost component) proxies these messages to the daemon HTTP API.
 * Views never talk to the daemon directly — the host controls the connection.
 *
 * Usage:
 *   const ship = new ShipSDK()
 *   const jobs = await ship.jobs.list()
 *   await ship.events.emit('job.created', entityId, payload)
 *   ship.events.onStream((envelope) => { ... })
 *   ship.theme.onChange((theme) => { ... })
 */

class ShipSDK {
  constructor() {
    this._pending = new Map()
    this._seq = 0
    this._streamListeners = []
    this._themeListeners = []

    window.addEventListener('message', (e) => {
      const msg = e.data
      if (!msg || !msg.__ship) return

      if (msg.type === 'response') {
        const resolve = this._pending.get(msg.seq)
        if (resolve) {
          this._pending.delete(msg.seq)
          if (msg.error) resolve.reject(new Error(msg.error))
          else resolve.resolve(msg.data)
        }
      } else if (msg.type === 'event') {
        for (const fn of this._streamListeners) fn(msg.envelope)
      } else if (msg.type === 'theme') {
        for (const fn of this._themeListeners) fn(msg.theme)
      }
    })

    this.jobs = {
      list: () => this._request('jobs.list'),
      create: (params) => this._request('jobs.create', params),
    }

    this.events = {
      emit: (eventType, entityId, payload) =>
        this._request('events.emit', { event_type: eventType, entity_id: entityId, payload }),
      onStream: (fn) => {
        this._streamListeners.push(fn)
        return () => {
          this._streamListeners = this._streamListeners.filter((f) => f !== fn)
        }
      },
    }

    this.theme = {
      current: () => document.documentElement.getAttribute('data-theme') || 'dark',
      isDark: () => this.theme.current() === 'dark',
      onChange: (fn) => {
        this._themeListeners.push(fn)
        return () => {
          this._themeListeners = this._themeListeners.filter((f) => f !== fn)
        }
      },
    }

    this.workspace = {
      active: () => this._request('workspace.active'),
    }

    this.files = {
      list: (workspaceId) => this._request('files.list', { workspace_id: workspaceId }),
      read: (workspaceId, path) => this._request('files.read', { workspace_id: workspaceId, path }),
    }
  }

  _request(method, params) {
    return new Promise((resolve, reject) => {
      const seq = ++this._seq
      this._pending.set(seq, { resolve, reject })
      window.parent.postMessage({ __ship: true, type: 'request', method, params, seq }, '*')
      // Timeout after 10s
      setTimeout(() => {
        if (this._pending.has(seq)) {
          this._pending.delete(seq)
          reject(new Error(`ship-sdk: ${method} timed out`))
        }
      }, 10000)
    })
  }
}

// Auto-expose as global
window.ship = new ShipSDK()
