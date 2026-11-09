# KI-Aufwand / AI effort

Das Badge in der README zeigt, was alle Claude-Code-Sessions dieses Repositorys zusammen zu den Listenpreisen
der Claude-API gekostet hätten, Subagenten und Worktree-Sessions eingeschlossen. Bei Nutzung über ein Abo ist das
kein Rechnungsbetrag, sondern ein Gegenwert.

Ein stündlicher Job auf dem Rechner des Autors rechnet die Zahl aus den lokalen Session-Logs neu aus. Ändert sie
sich, kommt eine Zeile in `history.csv`, `latest.json` wird ersetzt und das Badge aktualisiert. Die Preise liest
das Skript bei jedem Lauf von der [Preisseite der Claude-API](https://platform.claude.com/docs/en/about-claude/pricing).

---

The README badge shows what all Claude Code sessions of this repository, subagents and worktree sessions
included, would have cost at Claude API list prices. With a subscription this is not an invoice amount but an
equivalent.

An hourly job on the author's machine recomputes the number from the local session logs. When it changes, a line
is added to `history.csv`, `latest.json` is replaced and the badge is updated. The script reads the prices from
the [Claude API pricing page](https://platform.claude.com/docs/en/about-claude/pricing) on every run.
