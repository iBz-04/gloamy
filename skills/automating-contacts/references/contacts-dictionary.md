# Contacts Dictionary & Type Map (JXA view)

## Core classes (specifiers)
- `Application('Contacts')`
- `people` (element): `Person` objects
- `groups` (element): `Group` objects
- `Person` properties: `firstName`, `lastName`, `middleName`, `name`, `organization`, `jobTitle`, `note`, `birthday`, `id`
- `Person` elements (multi-value):
  - `emails` → `Email { label, value }`
  - `phones` → `Phone { label, value }`
  - `addresses` → `Address { label, street, city, state, zip, country, countryCode }`
  - `dates` → `CustomDate { label, value (Date) }`
  - `socialProfiles` → `SocialProfile { service, userName, url }`
  - `instantMessages` → `InstantMessage { service, userName, handle }`
- `Group` properties: `name`, `id`
- `Group` elements: `people` (members)

## Commands
- `make` / `.make()`: create `Person` or `Group`.
- `add <person> to <group>` → JXA: `Contacts.add(person, { to: group })` (preferred) or `group.people.push(person)`.
- `save` → `Contacts.save()` to persist mutations.

## Access patterns
- By name: `Contacts.people.byName("Ada Lovelace")`
- By ID (stable): `Contacts.people.byId("<UUID>")` (preferred for long-lived refs)
- Batch read: `Contacts.people.whose({ organization: {_contains: "Acme"} }).name()`

## Operator map for `.whose`
- `_equals`, `_contains`, `_beginsWith`, `_endsWith`, `_greaterThan`, `_lessThan`, `_not`
- Nested multi-value: `{ emails: { value: { _endsWith: "edu" }}}`

## Localization & labels
- Standard labels ("Home", "Work", "Mobile") are accepted as plain strings; Contacts maps to localized internal constants.
- Custom labels are created automatically when you pass a new string (e.g., `"Secure Line"`).
