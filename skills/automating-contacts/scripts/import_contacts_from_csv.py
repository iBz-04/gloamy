#!/usr/bin/env python3
"""
Import Contacts from CSV Script - PyXA Implementation
Imports contacts from a CSV file into macOS Contacts

Usage: python import_contacts_from_csv.py contacts.csv
"""

import sys
import csv
import PyXA

def import_contacts_from_csv(csv_file):
    """Import contacts from CSV file"""
    try:
        contacts_app = PyXA.Application("Contacts")

        imported_count = 0
        skipped_count = 0

        with open(csv_file, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)

            for row in reader:
                try:
                    # Extract contact information
                    first_name = row.get('first_name', row.get('First Name', ''))
                    last_name = row.get('last_name', row.get('Last Name', ''))
                    email = row.get('email', row.get('Email', ''))
                    phone = row.get('phone', row.get('Phone', ''))

                    # Skip if no basic information
                    if not (first_name or last_name or email):
                        print(f"Skipping contact: insufficient information in row")
                        skipped_count += 1
                        continue

                    # Check if contact already exists
                    existing_contacts = contacts_app.contacts()
                    contact_exists = False

                    for contact in existing_contacts:
                        if (contact.first_name() == first_name and
                            contact.last_name() == last_name):
                            contact_exists = True
                            break

                    if contact_exists:
                        print(f"Contact {first_name} {last_name} already exists")
                        skipped_count += 1
                        continue

                    # Create new contact
                    new_contact = contacts_app.contacts().push({
                        "first_name": first_name,
                        "last_name": last_name
                    })

                    # Add email if provided
                    if email:
                        new_contact.emails().push({
                            "label": "work",
                            "value": email
                        })

                    # Add phone if provided
                    if phone:
                        new_contact.phones().push({
                            "label": "mobile",
                            "value": phone
                        })

                    imported_count += 1
                    print(f"Imported: {first_name} {last_name}")

                except Exception as e:
                    print(f"Error importing contact: {e}")
                    skipped_count += 1
                    continue

        print(f"\nImport complete:")
        print(f"Imported: {imported_count} contacts")
        print(f"Skipped: {skipped_count} contacts")

        return imported_count

    except Exception as e:
        print(f"Error in import process: {e}")
        return 0

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python import_contacts_from_csv.py contacts.csv")
        print("CSV should have columns: first_name, last_name, email, phone")
        sys.exit(1)

    csv_file = sys.argv[1]
    imported = import_contacts_from_csv(csv_file)
    sys.exit(0 if imported > 0 else 1)