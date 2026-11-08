# Program texts

Every message the program shows comes from `i18n/<language>/courseplan.ftl`, a Project Fluent file
(ADR 0022). English is the reference: `en/courseplan.ftl` defines every message id, and a missing message in
another language falls back to it.

## Add a language

1. Copy `en/courseplan.ftl` to `i18n/<code>/courseplan.ftl`, with the two-letter code, for example `fr`.
2. Translate the text after each `=`. Keep the message ids and the variable names (`{ $code }`) as they are.
   Move a variable to wherever the grammar of your language wants it.
3. Add the language to `EMBEDDED` in `src/text.rs`.
4. Run `cargo test`. `every_language_has_every_message` fails when an id is missing or extra, or when a
   message does not format.

Parser messages and operating system errors reach the texts as variables and stay in English. Use the domain
words from `docs/glossary.md` in your language.
