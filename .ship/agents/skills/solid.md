+++
id = "solid"
name = "SOLID Principles"
source = "custom"
+++

# SOLID Principles

Use this skill when designing or refactoring modules, services, and interfaces.

## S - Single Responsibility
- Each unit should change for one reason.
- Split mixed policy + IO + formatting logic.

## O - Open/Closed
- Extend behavior via composition or new implementations, not conditionals spread across callers.

## L - Liskov Substitution
- Replacements must preserve expected behavior and error contracts.

## I - Interface Segregation
- Prefer narrow interfaces per consumer over one large shared interface.

## D - Dependency Inversion
- Depend on abstractions and pass concrete dependencies at boundaries.

## Refactor Checklist
- Are call sites forced to know concrete types?
- Are interfaces broader than the consumer needs?
- Does adding a variant require editing multiple existing modules?
- Are invariants documented and enforced in tests?
