# Contacts JXA Recipes

## Query contacts by email domain (coarse filter + refine)
```javascript
const matches = Contacts.people.whose({
  emails: { value: { _endsWith: "acme.com" } }
})();
// refine in JS if needed
const tagged = matches.filter(p => (p.note() || "").includes("VIP"));
```

## Create a person with multi-value fields
```javascript
const p = Contacts.Person().make();
p.firstName = "Grace";
p.lastName = "Hopper";
p.organization = "US Navy";

p.emails.push(Contacts.Email({ label: "Work", value: "grace@navy.mil" }));
p.phones.push(Contacts.Phone({ label: "Mobile", value: "+1-555-123-4567" }));
p.addresses.push(Contacts.Address({
  label: "Office",
  street: "1 Programming Way",
  city: "Arlington",
  state: "VA",
  zip: "22202",
  country: "USA",
  countryCode: "us"
}));

// Dates: set to noon to avoid TZ shifts
p.dates.push(Contacts.CustomDate({ label: "Anniversary", value: new Date("2015-08-14T12:00:00") }));
Contacts.save();
```

## Add to group (defensive)
```javascript
function ensureGroup(name) {
  const g = Contacts.groups.whose({ name: { _equals: name } })();
  if (g.length) return g[0];
  const created = Contacts.Group().make();
  created.name = name;
  Contacts.save();
  return created;
}

const group = ensureGroup("System Architects");
const person = Contacts.people.byName("Grace Hopper");

// Avoid duplicate membership
const already = group.people.whose({ id: { _equals: person.id() } })().length > 0;
if (!already) {
  Contacts.add(person, { to: group }); // or group.people.push(person)
  Contacts.save();
}
```

## Hybrid filter when `.whose` is fragile
```javascript
const bayArea = Contacts.people.whose({
  addresses: { city: { _equals: "San Francisco" } }
})();

const projectAlpha = bayArea.filter(p => /Project Alpha/i.test(p.note() || ""));
```
