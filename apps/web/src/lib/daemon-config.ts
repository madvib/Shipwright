const DAEMON_PORT = import.meta.env.VITE_SHIP_DAEMON_PORT ?? '9315'
export const DAEMON_BASE_URL = `http://localhost:${DAEMON_PORT}`
