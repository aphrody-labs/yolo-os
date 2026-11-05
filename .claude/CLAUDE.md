# Instructions de reprise : YOLO OS

@../AGENTS.md

Lire [OS-HANDOFF.md](OS-HANDOFF.md) avant de modifier Linux, Windows, les images ou les toolchains. Ce fichier consolide les décisions de plusieurs sessions et renvoie aux preuves et aux travaux en cours.

- Préserver tous les changements existants, y compris les fichiers non suivis, suppressions et zones de travail des autres sessions. Ne pas reset, clean ni écraser une archive.
- `yolo-os` est le projet LFS et image; `yolo-bun` fournit les intégrations Bun et le flux de paquets; `yolo-cross` possède les toolchains, cibles et interop multiplateforme. Voir le transfert pour les autres frontières.
- Utiliser les sources upstream et les fichiers locaux comme autorités; dater les constats et revérifier les versions et les processus avant de les présenter comme actuels. Ne pas qualifier un build, une image, un boot ou une performance sans son reçu.
- Les recettes LFS, leurs versions et sommes proviennent de la révision choisie du livre LFS. Garder les licences et provenances par composant.
- Ne pas copier de secrets, fichiers `.env`, jetons, clés ou bases de production dans les notes ou les sorties.
