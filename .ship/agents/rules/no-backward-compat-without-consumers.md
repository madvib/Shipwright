# No Backward Compatibility Without Consumers

* If there are no downstream consumers to protect, make a hard break in the same change.

* Do not keep compatibility aliases, duplicate command surfaces, or transitional wrappers.

* Apply this rule across CLI, MCP, runtime APIs, and config schemas.

* Only allow temporary compatibility paths for data-safety migrations that prevent data loss or corruption.

* Any data-safety exception must include:

  * explicit scope,

  * removal criteria,

  * a test that fails once the exception is no longer needed.

