#!/usr/bin/env python3
"""
Export Contacts to CSV Script - PyXA Implementation
Exports contacts from macOS Contacts to a CSV file

Usage: python export_contacts_to_csv.py [output.csv]
"""

import sys
import csv
import PyXA

def export_contacts_to_csv(output_file="contacts_export.csv"):
    """Export all contacts to CSV file"""
    try:
        contacts_app = PyXA.Application("Contacts")

        contacts = contacts_app.contacts()
        exported_count = 0

        with open(output_file, 'w', newline='', encoding='utf-8') as f:
            fieldnames = ['first_name', 'last_name', 'emails', 'phones', 'company', 'job_title']
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()

            for contact in contacts:
                try:
                    # Extract contact information
                    first_name = contact.first_name() or ""
                    last_name = contact.last_name() or ""
                    company = contact.organization() or ""
                    job_title = contact.job_title() or ""

                    # Extract emails
                    emails = []
                    try:
                        for email in contact.emails():
                            emails.append(f"{email.label()}: {email.value()}")
                    except:
                        pass

                    # Extract phones
                    phones = []
                    try:
                        for phone in contact.phones():
                            phones.append(f"{phone.label()}: {phone.value()}")
                    except:
                        pass

                    # Write to CSV
                    writer.writerow({
                        'first_name': first_name,
                        'last_name': last_name,
                        'emails': '; '.join(emails),
                        'phones': '; '.join(phones),
                        'company': company,
                        'job_title': job_title
                    })

                    exported_count += 1

                    if exported_count % 50 == 0:
                        print(f"Exported {exported_count} contacts...")

                except Exception as e:
                    print(f"Error exporting contact {contact.id()}: {e}")
                    continue

        print(f"\nExport complete: {exported_count} contacts exported to {output_file}")
        return exported_count

    except Exception as e:
        print(f"Error in export process: {e}")
        return 0

if __name__ == "__main__":
    output_file = sys.argv[1] if len(sys.argv) > 1 else "contacts_export.csv"
    exported = export_contacts_to_csv(output_file)
    sys.exit(0 if exported > 0 else 1)