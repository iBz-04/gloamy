export interface Language {
  value: string
  label: string
}

export function supportedLanguages(): Language[] {
  return [
    {
      label: 'English',
      value: 'en',
    },
    {
      label: 'Türkçe',
      value: 'tr',
    },
    {
      label: 'Français',
      value: 'fr',
    },
  ]
}

export function getLanguageLabel(value: string): string {
  return supportedLanguages().find(lang => lang.value === value)?.label ?? value
}
