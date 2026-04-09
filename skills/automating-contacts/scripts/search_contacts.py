#!/usr/bin/env python3
"""
Search Contacts Script - PyXA Implementation
Searches for contacts by name, email, or phone

Usage: python search_contacts.py "search term" [--email] [--phone]
"""

import sys
import PyXA

def search_contacts(search_term, search_emails=False, search_phones=False):
    """Search contacts by various criteria"""
    try:
        contacts_app = PyXA.Application("Contacts")

        contacts = contacts_app.contacts()
        found_contacts = []

        search_lower = search_term.lower()

        for contact in contacts:
            try:
                # Search in name
                first_name = (contact.first_name() or "").lower()
                last_name = (contact.last_name() or "").lower()
                full_name = f"{first_name} {last_name}".strip()

                name_match = (search_lower in first_name or
                            search_lower in last_name or
                            search_lower in full_name)

                # Search in emails if requested
                email_match = False
                if search_emails:
                    try:
                        for email in contact.emails():
                            if search_lower in (email.value() or "").lower():
                                email_match = True
                                break
                    except:
                        pass

                # Search in phones if requested
                phone_match = False
                if search_phones:
                    try:
                        for phone in contact.phones():
                            if search_lower in (phone.value() or "").lower():
                                phone_match = True
                                break
                    except:
                        pass

                # If any match found, collect contact info
                if name_match or email_match or phone_match:
                    contact_info = {
                        'first_name': contact.first_name() or "",
                        'last_name': contact.last_name() or "",
                        'company': contact.organization() or "",
                        'job_title': contact.job_title() or "",
                        'emails': [],
                        'phones': []
                    }

                    # Collect emails
                    try:
                        for email in contact.emails():
                            contact_info['emails'].append({
                                'label': email.label() or "work",
                                'value': email.value() or ""
                            })
                    except:
                        pass

                    # Collect phones
                    try:
                        for phone in contact.phones():
                            contact_info['phones'].append({
                                'label': phone.label() or "mobile",
                                'value': phone.value() or ""
                            })
                    except:
                        pass

                    found_contacts.append(contact_info)

            except Exception as e:
                print(f"Error processing contact: {e}")
                continue

        # Display results
        if found_contacts:
            print(f"Found {len(found_contacts)} contacts matching '{search_term}':")
            print("=" * 60)

            for i, contact in enumerate(found_contacts, 1):
                print(f"{i}. {contact['first_name']} {contact['last_name']}")
                if contact['company']:
                    print(f"   Company: {contact['company']}")
                if contact['job_title']:
                    print(f"   Title: {contact['job_title']}")

                if contact['emails']:
                    print("   Emails:")
                    for email in contact['emails']:
                        print(f"     {email['label']}: {email['value']}")

                if contact['phones']:
                    print("   Phones:")
                    for phone in contact['phones']:
                        print(f"     {phone['label']}: {phone['value']}")

                print()
        else:
            search_types = []
            if not (search_emails or search_phones):
                search_types.append("name")
            if search_emails:
                search_types.append("email")
            if search_phones:
                search_types.append("phone")

            print(f"No contacts found matching '{search_term}' in {', '.join(search_types)}")

        return found_contacts

    except Exception as e:
        print(f"Error in search process: {e}")
        return []

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python search_contacts.py 'search term' [--email] [--phone]")
        sys.exit(1)

    search_term = sys.argv[1]
    search_emails = "--email" in sys.argv
    search_phones = "--phone" in sys.argv

    results = search_contacts(search_term, search_emails, search_phones)
    sys.exit(0 if results else 1)