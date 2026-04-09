# Contacts JXA Basics (specifiers, reads, writes)

## Initialize
```javascript
const app = Application.currentApplication();
app.includeStandardAdditions = true;
const Contacts = Application('Contacts');
if (!Contacts.running()) Contacts.activate();
```

## Specifier vs data
- `Contacts.people` → specifier (zero IPC).
- `Contacts.people()` → fetches every person (high cost).
- `Contacts.people.name()` → one IPC returning array of names.
- Always call methods to read: `person.firstName()` not `person.firstName`.

## Read collections efficiently
- Prefer server-side filter: `Contacts.people.whose({ emails: { value: { _contains: 'acme.com' }}})`.
- Access properties on the filtered specifier: `.name()` or `.emails.value()` to batch read.
- If `.whose` is too complex, coarse-filter server-side then refine in JS.

## Create & save
- Create root object with proxy + make:
```javascript
const p = Contacts.Person().make();
p.firstName = "Ada";
p.lastName = "Lovelace";
Contacts.save();
```
- Multi-value fields are elements; create and push:
```javascript
const workEmail = Contacts.Email({ label: "Work", value: "ada@engine.co.uk" });
p.emails.push(workEmail);
Contacts.save();
```

## Groups
- Groups are containers; membership is many-to-many.
- Robust add: `Contacts.add(person, { to: group });`
- Alternative sugar: `group.people.push(person);` (watch for duplicates).

## Common operators for `.whose`
- `_equals`, `_contains`, `_beginsWith`, `_endsWith`, `_greaterThan`, `_lessThan`, `_not`.
- Nested collections: `{ emails: { value: { _endsWith: 'university.edu' }}}`.
