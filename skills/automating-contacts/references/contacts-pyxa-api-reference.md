# PyXA Contacts Module API Reference

> **New in PyXA version 0.0.2** - Control macOS Contacts using JXA-like syntax from Python.

This reference documents all classes, methods, properties, and enums in the PyXA Contacts module. For practical examples and usage patterns, see [contacts-recipes.md](contacts-recipes.md).

## Contents

- [Class Hierarchy](#class-hierarchy)
- [XAContactsApplication](#xacontactsapplication)
- [XAContactsPerson](#xacontactsperson)
- [XAContactsGroup](#xacontactsgroup)
- [XAContactsEntry](#xacontactsentry)
- [Contact Information Classes](#contact-information-classes)
  - [XAContactsAddress](#xacontactsaddress)
  - [XAContactsEmail](#xacontactsemail)
  - [XAContactsPhone](#xacontactsphone)
  - [XAContactsURL](#xacontactsurl)
  - [XAContactsInstantMessage](#xacontactsinstantmessage)
  - [XAContactsSocialProfile](#xacontactssocialprofile)
  - [XAContactsCustomDate](#xacontactscustomdate)
  - [XAContactsRelatedName](#xacontactsrelatedname)
- [XAContactsDocument](#xacontactsdocument)
- [List Classes](#list-classes)
- [Enumerations](#enumerations)
- [Quick Reference Tables](#quick-reference-tables)

---

## Class Hierarchy

```
XAObject
├── XAContactsApplication (XASBApplication)
│   ├── XAContactsEntry
│   │   ├── XAContactsPerson
│   │   └── XAContactsGroup
│   ├── XAContactsContactInfo
│   │   ├── XAContactsEmail
│   │   ├── XAContactsPhone
│   │   ├── XAContactsURL
│   │   ├── XAContactsCustomDate
│   │   └── XAContactsRelatedName
│   ├── XAContactsAddress
│   ├── XAContactsInstantMessage
│   ├── XAContactsSocialProfile
│   └── XAContactsDocument
```

---

## XAContactsApplication

**Bases:** `XASBApplication`

Main entry point for interacting with Contacts.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Application name ("Contacts") |
| `version` | `str` | Application version |
| `frontmost` | `bool` | Whether Contacts is the frontmost application |
| `my_card` | `XAContactsPerson` | The user's own contact card |
| `selection` | `XAContactsPersonList` | Currently selected contacts |
| `unsaved` | `bool` | Whether there are unsaved changes |
| `default_country_code` | `str` | Default country code for phone formatting |

### Methods

#### `people(filter=None) -> XAContactsPersonList`

Returns a list of contacts matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
import PyXA
contacts = PyXA.Application("Contacts")
all_people = contacts.people()
for person in all_people:
    print(person.first_name, person.last_name)
```

#### `groups(filter=None) -> XAContactsGroupList`

Returns a list of groups matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
groups = contacts.groups()
print(groups.name())
# ['Family', 'Work', 'Friends', ...]
```

#### `documents(filter=None) -> XAContactsDocumentList`

Returns a list of address book documents matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

#### `make(specifier, properties=None, data=None) -> XAObject`

Creates a new element without adding to any list. Use `XAList.push()` to add.

**Parameters:**
- `specifier` (`str | ObjectType`) - Class name to create (e.g., "person", "group")
- `properties` (`dict`) - Properties for the object
- `data` (`Any`) - Initialization data

**Example:**
```python
# Create a new person
new_person = contacts.make("person", {
    "firstName": "Ada",
    "lastName": "Lovelace"
})
contacts.people().push(new_person)
contacts.save()
```

#### `open(file_path)`

Opens and imports contacts from a file (vCard, etc.).

**Parameters:**
- `file_path` (`str | XAPath`) - Path to the contact file

**Example:**
```python
contacts.open("/path/to/contacts.vcf")
```

#### `save()`

Persists all changes to the address book.

**Example:**
```python
person = contacts.people()[0]
person.first_name = "Updated"
contacts.save()
```

---

## XAContactsPerson

**Bases:** `XAContactsEntry`

Represents an individual contact record in the address book.

### Properties

#### Name Properties

| Property | Type | Description |
|----------|------|-------------|
| `first_name` | `str` | First/given name |
| `last_name` | `str` | Last/family name |
| `middle_name` | `str` | Middle name |
| `nickname` | `str` | Nickname |
| `suffix` | `str` | Name suffix (Jr., Sr., III) |
| `title` | `str` | Name prefix/title (Mr., Dr., etc.) |
| `maiden_name` | `str` | Maiden name |

#### Phonetic Properties

| Property | Type | Description |
|----------|------|-------------|
| `phonetic_first_name` | `str` | Phonetic spelling of first name |
| `phonetic_last_name` | `str` | Phonetic spelling of last name |
| `phonetic_middle_name` | `str` | Phonetic spelling of middle name |

#### Organization Properties

| Property | Type | Description |
|----------|------|-------------|
| `company` | `str` | Company/organization name |
| `organization` | `str` | Organization name (alias) |
| `department` | `str` | Department name |
| `job_title` | `str` | Job title/position |

#### Other Properties

| Property | Type | Description |
|----------|------|-------------|
| `birth_date` | `datetime` | Birth date |
| `home_page` | `str` | Personal website URL |
| `image` | `XAImage` | Contact photo |
| `note` | `str` | Notes/comments |
| `vcard` | `str` | vCard representation of the contact |

### Methods

#### `addresses(filter=None) -> XAContactsAddressList`

Returns physical addresses for this contact.

**Example:**
```python
person = contacts.people()[0]
for addr in person.addresses():
    print(f"{addr.street}, {addr.city}, {addr.state} {addr.zip}")
```

#### `emails(filter=None) -> XAContactsEmailList`

Returns email addresses for this contact.

**Example:**
```python
emails = person.emails()
print(emails.value())
# ['john@example.com', 'john.doe@work.com']
```

#### `phones(filter=None) -> XAContactsPhoneList`

Returns phone numbers for this contact.

**Example:**
```python
phones = person.phones()
for phone in phones:
    print(f"{phone.label}: {phone.value}")
```

#### `urls(filter=None) -> XAContactsURLList`

Returns URLs associated with this contact.

#### `instant_messages(filter=None) -> XAContactsInstantMessageList`

Returns instant messaging accounts for this contact.

#### `social_profiles(filter=None) -> XAContactsSocialProfileList`

Returns social media profiles for this contact.

#### `custom_dates(filter=None) -> XAContactsCustomDateList`

Returns custom date fields for this contact.

#### `related_names(filter=None) -> XAContactsRelatedNameList`

Returns related names (spouse, assistant, etc.) for this contact.

#### `groups(filter=None) -> XAContactsGroupList`

Returns groups this contact belongs to.

**Example:**
```python
person_groups = person.groups()
print(person_groups.name())
# ['Family', 'VIP']
```

#### `show() -> XAContactsPerson`

Opens Contacts.app and displays this contact.

**Example:**
```python
person.show()  # Opens contact card in Contacts.app
```

---

## XAContactsGroup

**Bases:** `XAContactsEntry`

Represents a contact group in the address book.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Group name |

### Methods

#### `people(filter=None) -> XAContactsPersonList`

Returns contacts that belong to this group.

**Example:**
```python
family = contacts.groups().by_name("Family")
members = family.people()
print(members.first_name())
# ['John', 'Jane', 'Bob']
```

#### `groups(filter=None) -> XAContactsGroupList`

Returns subgroups within this group.

---

## XAContactsEntry

**Bases:** `XAObject`

Base class for all contact entries (persons and groups).

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `creation_date` | `datetime` | Date the entry was created |
| `modification_date` | `datetime` | Date the entry was last modified |
| `selected` | `bool` | Whether the entry is currently selected |

### Methods

#### `add_to(parent) -> XAContactsPerson`

Adds this entry to a group.

**Parameters:**
- `parent` (`XAContactsGroup`) - The group to add to

**Example:**
```python
person = contacts.people()[0]
vip_group = contacts.groups().by_name("VIP")
person.add_to(vip_group)
contacts.save()
```

#### `remove_from(elem) -> XAContactsPerson`

Removes this entry from a group.

**Parameters:**
- `elem` (`XAContactsGroup`) - The group to remove from

#### `delete()`

Permanently deletes this entry from the address book.

**Example:**
```python
person = contacts.people().by_first_name("Test")
person.delete()
contacts.save()
```

---

## Contact Information Classes

### XAContactsContactInfo

**Bases:** `XAObject`

Base class for contact information items (email, phone, URL, etc.).

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `label` | `str` | Label (e.g., "Home", "Work", "Mobile") |
| `value` | `str` | The actual value |

---

### XAContactsAddress

**Bases:** `XAObject`

Represents a physical/mailing address.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `label` | `str` | Address label ("Home", "Work", etc.) |
| `street` | `str` | Street address |
| `city` | `str` | City name |
| `state` | `str` | State/province |
| `zip` | `str` | Postal/ZIP code |
| `country` | `str` | Country name |
| `country_code` | `str` | ISO country code |
| `formatted_address` | `str` | Full formatted address string |

**Example:**
```python
person = contacts.people()[0]
home_addr = person.addresses().by_label("Home")
print(home_addr.formatted_address)
# "123 Main St\nSan Francisco, CA 94102\nUnited States"
```

---

### XAContactsEmail

**Bases:** `XAContactsContactInfo`

Represents an email address entry.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `label` | `str` | Email label ("Home", "Work", etc.) |
| `value` | `str` | Email address |

**Example:**
```python
# Get work email
work_email = person.emails().by_label("Work")
print(work_email.value)
# "john.doe@company.com"
```

---

### XAContactsPhone

**Bases:** `XAContactsContactInfo`

Represents a phone number entry.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `label` | `str` | Phone label ("Home", "Work", "Mobile", "iPhone", etc.) |
| `value` | `str` | Phone number |

**Example:**
```python
# Get mobile phone
mobile = person.phones().by_label("Mobile")
print(mobile.value)
# "+1 (555) 123-4567"
```

---

### XAContactsURL

**Bases:** `XAContactsContactInfo`

Represents a URL entry (website, social profile, etc.).

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `label` | `str` | URL label ("Home Page", "Work", etc.) |
| `value` | `str` | URL string |

---

### XAContactsInstantMessage

**Bases:** `XAObject`

Represents an instant messaging account.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `label` | `str` | IM label |
| `service_type` | `ServiceType` | Service type (AIM, Jabber, etc.) |
| `service_name` | `str` | Service name |
| `user_name` | `str` | Username/handle |

**Example:**
```python
im_accounts = person.instant_messages()
for im in im_accounts:
    print(f"{im.service_name}: {im.user_name}")
```

---

### XAContactsSocialProfile

**Bases:** `XAObject`

Represents a social media profile.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `service_name` | `str` | Service name (Twitter, LinkedIn, etc.) |
| `user_name` | `str` | Username/handle |
| `user_identifier` | `str` | Unique user ID on the service |
| `url` | `str` | Profile URL |

**Example:**
```python
profiles = person.social_profiles()
for profile in profiles:
    print(f"{profile.service_name}: @{profile.user_name}")
```

---

### XAContactsCustomDate

**Bases:** `XAContactsContactInfo`

Represents a custom date field (anniversary, etc.).

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `label` | `str` | Date label ("Anniversary", custom labels) |
| `value` | `datetime` | The date value |

---

### XAContactsRelatedName

**Bases:** `XAContactsContactInfo`

Represents a related person (spouse, assistant, etc.).

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `label` | `str` | Relationship type ("Spouse", "Assistant", "Manager", etc.) |
| `value` | `str` | Related person's name |

---

## XAContactsDocument

**Bases:** `XAObject`

Represents an address book document.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Document name |
| `file` | `str` | File location on disk |
| `modified` | `bool` | Whether document has unsaved changes |

---

## List Classes

PyXA provides list wrapper classes with fast enumeration and bulk property access.

### XAContactsPersonList

Bulk methods for person lists:

```python
people = contacts.people()
people.first_name()       # -> list[str]
people.last_name()        # -> list[str]
people.company()          # -> list[str]
people.department()       # -> list[str]
people.job_title()        # -> list[str]
people.birth_date()       # -> list[datetime]
people.note()             # -> list[str]
people.home_page()        # -> list[str]
people.image()            # -> list[XAImage]
people.vcard()            # -> list[str]
```

Filter methods:
```python
people.by_first_name("John")
people.by_last_name("Doe")
people.by_company("Acme Corp")
people.by_department("Engineering")
people.by_job_title("Developer")
people.by_birth_date(date)
people.by_nickname("Johnny")
```

### XAContactsGroupList

Bulk methods for group lists:

```python
groups = contacts.groups()
groups.name()             # -> list[str]
groups.id()               # -> list[str]
groups.creation_date()    # -> list[datetime]
groups.modification_date()  # -> list[datetime]
```

Filter methods:
```python
groups.by_name("Family")
groups.by_id("group-uuid")
```

### XAContactsAddressList

Bulk methods for address lists:

```python
addresses = person.addresses()
addresses.street()        # -> list[str]
addresses.city()          # -> list[str]
addresses.state()         # -> list[str]
addresses.zip()           # -> list[str]
addresses.country()       # -> list[str]
addresses.country_code()  # -> list[str]
addresses.formatted_address()  # -> list[str]
addresses.label()         # -> list[str]
```

Filter methods:
```python
addresses.by_label("Home")
addresses.by_city("San Francisco")
addresses.by_state("CA")
addresses.by_zip("94102")
addresses.by_country("United States")
addresses.by_country_code("US")
```

### XAContactsEmailList

```python
emails = person.emails()
emails.label()            # -> list[str]
emails.value()            # -> list[str]
emails.by_label("Work")
emails.by_value("john@example.com")
```

### XAContactsPhoneList

```python
phones = person.phones()
phones.label()            # -> list[str]
phones.value()            # -> list[str]
phones.by_label("Mobile")
phones.by_value("+1555123456")
```

### XAContactsURLList

```python
urls = person.urls()
urls.label()              # -> list[str]
urls.value()              # -> list[str]
urls.by_label("Home Page")
urls.by_value("https://example.com")
```

### XAContactsInstantMessageList

```python
ims = person.instant_messages()
ims.service_type()        # -> list[ServiceType]
ims.service_name()        # -> list[str]
ims.user_name()           # -> list[str]
ims.by_service_type(ServiceType.JABBER)
ims.by_user_name("johndoe")
```

### XAContactsSocialProfileList

```python
profiles = person.social_profiles()
profiles.service_name()   # -> list[str]
profiles.user_name()      # -> list[str]
profiles.user_identifier()  # -> list[str]
profiles.url()            # -> list[str]
profiles.by_service_name("Twitter")
profiles.by_user_name("johndoe")
profiles.by_url("https://twitter.com/johndoe")
```

### XAContactsCustomDateList

```python
dates = person.custom_dates()
dates.label()             # -> list[str]
dates.value()             # -> list[datetime]
dates.by_label("Anniversary")
```

### XAContactsRelatedNameList

```python
related = person.related_names()
related.label()           # -> list[str]
related.value()           # -> list[str]
related.by_label("Spouse")
related.by_value("Jane Doe")
```

### XAContactsDocumentList

```python
docs = contacts.documents()
docs.name()               # -> list[str]
docs.file()               # -> list[str]
docs.modified()           # -> list[bool]
docs.by_name("Address Book")
docs.by_modified(True)
```

---

## Enumerations

### Format

Archive format options.

| Value | Description |
|-------|-------------|
| `ARCHIVE` | Native Address Book archive format |

### ObjectType

Creatable object types for `make()`.

| Value | Description |
|-------|-------------|
| `DOCUMENT` | Address book document |
| `GROUP` | Contact group |
| `PERSON` | Contact person |
| `URL` | URL entry |

### ServiceType

Instant messaging service types.

| Value | Service |
|-------|---------|
| `AIM` | AOL Instant Messenger |
| `FACEBOOK` | Facebook Messenger |
| `GADU_GADU` | Gadu-Gadu |
| `GOOGLE_TALK` | Google Talk/Hangouts |
| `ICQ` | ICQ |
| `JABBER` | Jabber/XMPP |
| `MSN` | MSN Messenger |
| `QQ` | QQ |
| `SKYPE` | Skype |
| `YAHOO` | Yahoo Messenger |

---

## Quick Reference Tables

### Common Operations

| Task | Code |
|------|------|
| Get Contacts app | `contacts = PyXA.Application("Contacts")` |
| Get all contacts | `people = contacts.people()` |
| Get contact by name | `person = contacts.people().by_first_name("John")` |
| Get user's card | `me = contacts.my_card` |
| Get all groups | `groups = contacts.groups()` |
| Get group by name | `group = contacts.groups().by_name("Family")` |
| Get group members | `members = group.people()` |
| Create new person | `person = contacts.make("person", {"firstName": "Ada"})` |
| Create new group | `group = contacts.make("group", {"name": "VIP"})` |
| Add to group | `person.add_to(group)` |
| Remove from group | `person.remove_from(group)` |
| Delete contact | `person.delete()` |
| Save changes | `contacts.save()` |
| Show contact | `person.show()` |
| Import vCard | `contacts.open("/path/to/contact.vcf")` |

### Contact Multi-Value Fields

| Field | Access Method | Example |
|-------|---------------|---------|
| Emails | `person.emails()` | `person.emails().by_label("Work").value` |
| Phones | `person.phones()` | `person.phones().by_label("Mobile").value` |
| Addresses | `person.addresses()` | `person.addresses().by_label("Home").city` |
| URLs | `person.urls()` | `person.urls().by_label("Home Page").value` |
| IMs | `person.instant_messages()` | `person.instant_messages().user_name()` |
| Social | `person.social_profiles()` | `person.social_profiles().by_service_name("Twitter")` |
| Dates | `person.custom_dates()` | `person.custom_dates().by_label("Anniversary").value` |
| Related | `person.related_names()` | `person.related_names().by_label("Spouse").value` |

### Property Access Patterns

```python
# Single object
person = contacts.people()[0]
print(person.first_name)
print(person.company)

# Bulk access on lists
people = contacts.people()
print(people.first_name())    # Returns list[str]
print(people.company())       # Returns list[str]

# Filtering
work_contacts = people.by_company("Acme Corp")
engineers = people.by_department("Engineering")

# Chained filtering (get first work email)
work_email = person.emails().by_label("Work")[0].value
```

### Creating Contacts with Multi-Value Fields

```python
import PyXA

contacts = PyXA.Application("Contacts")

# Create person
person = contacts.make("person", {
    "firstName": "Ada",
    "lastName": "Lovelace",
    "company": "Analytical Engine Inc.",
    "jobTitle": "Chief Mathematician"
})
contacts.people().push(person)

# Add email
email = contacts.make("email", {
    "label": "Work",
    "value": "ada@analyticalengine.com"
})
person.emails().push(email)

# Add phone
phone = contacts.make("phone", {
    "label": "Mobile",
    "value": "+1 (555) 123-4567"
})
person.phones().push(phone)

# Add address
address = contacts.make("address", {
    "label": "Work",
    "street": "123 Innovation Way",
    "city": "London",
    "country": "United Kingdom"
})
person.addresses().push(address)

# Save all changes
contacts.save()
```

---

## See Also

- [PyXA Contacts Documentation](https://skaplanofficial.github.io/PyXA/reference/apps/contacts.html) - Official PyXA documentation
- [contacts-basics.md](contacts-basics.md) - JXA fundamentals for Contacts
- [contacts-recipes.md](contacts-recipes.md) - Common automation patterns
- [contacts-advanced.md](contacts-advanced.md) - Advanced techniques and troubleshooting
- [contacts-dictionary.md](contacts-dictionary.md) - AppleScript dictionary reference
