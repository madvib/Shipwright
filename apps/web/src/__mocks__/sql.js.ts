// Minimal sql.js stub for test environments where the real package is unavailable
export default async () => ({
  Database: class {
    prepare() { return { step: () => false, getAsObject: () => ({}), free: () => {} } }
    close() {}
  },
})
