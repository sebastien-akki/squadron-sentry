## 0.1.0 · 2026-11-24

- Première version : les erreurs Sentry non résolues des projets que vous suivez, dans un panneau à côté de vos agents.
- Chaque erreur nouvelle ou revenue arrive avec une notification et une correction proposée par Claude Code, qui lit le dépôt sans le modifier : un diagnostic, un diff et ce qu’il faut vérifier.
- Corriger avec un agent ouvre Nouvel agent avec l’erreur, la proposition et la stack trace ; Résoudre et Ignorer agissent dans Sentry.
- Les agents peuvent lire les erreurs et leurs stack traces avec `sentry_issues` et `sentry_issue`.
