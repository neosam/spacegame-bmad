<!-- .claude/commands/bmad-run-sprint.md -->
Lies den Sprint Backlog aus docs/sprint-backlog.md.

Für jede Story die den Status "todo" hat, führe folgende Pipeline aus:

### Schritt 1: Story erstellen
Delegiere an den bmad-story-creator Subagent.
Übergib die Backlog-Beschreibung als Kontext.

### Schritt 2: Story implementieren
Delegiere an den bmad-developer Subagent.
Übergib die erstellte Story-Datei als Kontext.

### Schritt 3: Code Review + Fix Loop
Delegiere an den bmad-reviewer Subagent.
Er reviewt und fixt bis alles clean ist.

### Nach jeder Story:
- `git add -A && git commit -m "feat: [Story-Titel]"`
- Setze den Status in sprint-backlog.md auf "done"
- Aktualisiere architecture.md falls nötig

### Parallelisierung:
Stories OHNE Abhängigkeiten dürfen parallel laufen 
(mehrere Subagents gleichzeitig). 
Stories MIT Abhängigkeiten müssen sequentiell laufen.

### Am Ende:
Erstelle einen Sprint-Report in docs/sprint-report.md.
