## 0.3.0 · 2026-11-27

- La liste donne le total des erreurs non résolues des projets suivis, et se trie par dernière occurrence, plus récentes, plus d’occurrences ou plus d’utilisateurs.
- Chaque erreur montre ses occurrences, son âge et ses utilisateurs, avec Résoudre et Ignorer à droite, et son détail d’un clic : occurrences, utilisateurs, première et dernière occurrence datées, statut, niveau, projet.
- Des cases à cocher sélectionnent des erreurs, ou toutes, pour les résoudre, les ignorer ou les fusionner d’un coup ; fusionner demande deux erreurs d’un même projet Sentry et un second clic.
- Le bouton d’un groupe du tri coche maintenant ses erreurs, pour les fusionner depuis la sélection.

## 0.2.0 · 2026-11-27

- Tri par Haiku : les erreurs d’un même projet de même cause regroupées, avec **Fusionner dans Sentry**.
- Les erreurs qui n’ont rien à corriger (une mise en production, un environnement local, un test) mises de côté avec leur raison, à résoudre ou ignorer une à une ou toutes d’un coup ; elles ne sont pas analysées automatiquement.
- **Sans occurrence depuis [n] jours › Lister** : les erreurs non résolues sans événement depuis ce nombre de jours, retenu par projet, à résoudre quand vous le décidez.
- L’analyse écrit maintenant un diagnostic, un plan numéroté et ce qu’il faut vérifier. **Analyser et planifier** la lance sur n’importe quelle erreur quand l’analyse automatique est coupée.
- **Corriger avec un agent** demande à l’agent de présenter son plan et d’attendre votre accord avant de modifier un fichier.
- Fusionner et tout résoudre demandent un second clic.

## 0.1.0 · 2026-11-24

- Première version : les erreurs Sentry non résolues des projets que vous suivez, dans un panneau à côté de vos agents.
- Chaque erreur nouvelle ou revenue arrive avec une notification et une correction proposée par Claude Code, qui lit le dépôt sans le modifier : un diagnostic, un diff et ce qu’il faut vérifier.
- Corriger avec un agent ouvre Nouvel agent avec l’erreur, la proposition et la stack trace ; Résoudre et Ignorer agissent dans Sentry.
- Les agents peuvent lire les erreurs et leurs stack traces avec `sentry_issues` et `sentry_issue`.
