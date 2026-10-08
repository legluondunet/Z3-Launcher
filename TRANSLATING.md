# Translating Z3-Launcher

English, French, Italian (Italiano), Spanish (Español) and German (Deutsch) are embedded in the executable. They remain available without external locale files. All currently contain 341 messages, including help and installation dialogs.

No Rust changes or recompilation are needed to add another language. External JSON files can also override a bundled language for corrections.

1. Copy locales/en.json to locales/es.json (replace es with your language code).
2. In language, set code to es and name to the native name, for example Español.
3. Translate the VALUES in messages. Keep every message KEY unchanged.
4. Keep named placeholders such as {path}, {error}, {section}, {slot} exactly
   as they are. Their order may change. Keep newline escapes (\n) when needed.
5. Validate and restart the launcher. Select the new language in General.

Example of a small, incomplete translation:

```json
{
  "language": { "code": "es", "name": "Español" },
  "messages": {
    "general.interface_language": "Idioma de la interfaz",
    "text.general": "General",
    "text.options": "Opciones",
    "status.error": "Error: {error}"
  }
}
```

Missing messages fall back to embedded English. Invalid placeholder entries are
also ignored in favor of English. Unknown keys are ignored. Invalid JSON files
are skipped with a diagnostic in the launcher log. English and French are built
into the executable so they remain available when external files are absent.
Even an incomplete external English override cannot remove embedded defaults.

## Where to install translations

The launcher reads these directories at startup, in increasing priority:

1. locales/ in the current working directory (convenient during development).
2. locales/ next to the executable.
3. $XDG_CONFIG_HOME/Z3-Launcher/locales/ or
   ~/.config/Z3-Launcher/locales/ (user translations).

Only JSON files are loaded. Use a filename matching language.code. Files with
the same language code in a higher-priority directory replace the lower-priority
catalog, with English fallback for any omitted entry. Restart after adding or
changing files. Avoid introducing a second file with the same code in one directory.
Keep UI labels short and test long translations in the actual window.

## Validate

From the project directory:

```bash
python3 tools/check-translations.py
python3 tools/check-translations.py --allow-missing
```

You can also pass another directory containing en.json and your translations.
The script detects malformed JSON, duplicate keys/codes, missing or unknown
identifiers, empty strings, and changed placeholders. Normal validation expects
complete catalogs; --allow-missing allows incremental translation work.

## Scope

Catalogs cover tabs, buttons, setting labels, shortcut names, launcher-generated
errors, CLI help and dependency reports. INI keys, SDL button/key names, commands,
paths and file extensions are deliberately not translated. Native dialogs use
the desktop's own language. Output from Git, Python, make, the operating system,
SDL and the game is not translated by the launcher. Existing log entries are not
rewritten after switching language.

The interface defaults to English on every OS locale. The selected language is
stored separately in language.txt in the application configuration directory;
it does not change the game language or zelda3.ini. A missing saved language
falls back to English. Switching is immediate; no restart is needed for catalogs
already loaded. The selector is disabled while an operation is running.

The bundled fonts cover common Latin scripts. For languages requiring additional
glyphs (for example Chinese/Japanese), a suitable font will need to be integrated
before their rendering can be guaranteed. This version does not load custom
fonts from translation files.
