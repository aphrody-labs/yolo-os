# Transfert Claude : Linux, Windows et systèmes YOLO

Consolidation du 4 novembre 2026, depuis les documents locaux et les travaux de cette conversation. C'est un instantané de reprise; vérifier l'état courant avant toute action. Les notes historiques sont des rapports de session, pas une nouvelle exécution indépendante.

## Lecture initiale

1. `C:\src\yolo\docs\CODEX-SESSIONS.md` : chronologie et coordination intersessions; ses états de processus et dates peuvent être périmés.
2. `C:\src\yolo\docs\LLM-TX-AGENT.md` : inventaire de la flotte au 4 novembre 2026; le SHA et la file de patchs Bun décrits dans cet ancien instantané ont évolué.
3. `C:\src\yolo-os\docs\bunos-lfs-integration-plan.md`, `lfs-rust-method.md`, `bun-package-orchestration.md` et `awesome-rust-lfs.md` : méthode LFS, étapes et limites.
4. `C:\src\yolo-bun\docs\OS-BOOT.md`, `BUNOS-PORT.md`, `UPSTREAM.md`, `PACKAGE-MANAGER.md`, `PERFORMANCE.md`, et `docs\receipts\` : sources, ports, état du démarrage et reçus.
5. `C:\src\yolo-cross\docs\toolchain-2026.md`, `cross-targets.md`, `BUNCC-LINUX-DESKTOP.md` et `C:\src\yolo-cross\packages\windows\README.md` : cibles, build et adaptateurs Windows.

## Décisions de propriété

| Projet | Responsabilité |
| --- | --- |
| `yolo-os` | Distribution, LFS, choix de paquets, configuration noyau, initramfs, image et qualification de démarrage; `cargo-lfs` orchestre les étapes Rust autour du livre. |
| `yolo-rs/bun` | Fork miroir fidèle à `oven-sh/bun`; synchronisation fast-forward, sans patch YOLO. |
| `yolo-bun` | Monorepo d'intégration, file de patchs SHA-verrouillée, adaptateurs Bun, outils de paquets et prototype PID 1 en espace utilisateur. |
| `yolo-cross` | Toolchains et builds Linux/Windows multiplateformes, MSVC, sysroots, interop et adaptateurs FFI. Une API native essentielle à Bun peut être examinée dans `yolo-bun`. |
| `yolo-web` | Chromium, `bun:chromium`, Obscura, CEF et intégrations navigateur. |
| `yolo-ui` / Bun UI | Tauri, interfaces, frontends, frameworks et plugins UI. |
| `yolo-plugins` / `yolo-standards` | Catalogue et distribution des plugins inter-hôtes / schémas neutres. |

## Linux et méthode LFS

- Choisir une version du livre LFS et conserver ses recettes, sources, correctifs, sommes et errata ensemble. Le Rust de `cargo-lfs` aide à inventorier, planifier et journaliser; il ne réécrit pas les recettes upstream en Rust. Le dépôt contient des changements importants non commités; inspecter `git status` avant toute intervention.
- Le relevé du 4 novembre a identifié Linux `7.4-rc1`; la référence demandée `v7.4.1-rc` n'avait pas été trouvée. Revérifier la version réelle à la reprise : [Makefile Linux](https://github.com/torvalds/linux/blob/master/Makefile). Rust-for-Linux utilise des crates noyau `no_std` et Kbuild.
- Bun s'exécute en espace utilisateur. Le profil prévu construit un initramfs avec `/init` et `rdinit=/init`; aucune insertion de JavaScriptCore dans le noyau n'est démontrée nécessaire. Voir [démarrage Linux](https://github.com/torvalds/linux/blob/master/init/main.c) et `C:\src\yolo-bun\factory\linux`.
- `packages/bunos-init/src/init.ts`, attribué à BunOS sous Apache-2.0, est un prototype. Les montages, la collecte des processus enfants et l'arrêt sont couverts par son code; découverte/validation de la racine, switch-root, transmission aux services et qualification de l'image restent à faire. Les rapports BunOS historiques mentionnent un banc QEMU; les reçus correspondants n'ont pas été retrouvés dans l'archive locale lors de l'audit. Ne pas appeler un boot complet réussi sans preuve actuelle.
- Les remplacements uutils/coreutils et sudo-rs/PAM sont des étapes après la base LFS : conserver les replis GNU et une récupération root jusqu'aux contrôles de compatibilité et privilèges en VM. Le plan détaillé est dans `docs/bunos-lfs-integration-plan.md`.
- Asterinas reste une cible expérimentale séparée : initramfs et chargement ELF existent, mais la couverture des syscalls est partielle. Aucun démarrage Bun sur Asterinas n'est qualifié dans les preuves consultées. Ne proposer un correctif noyau que pour un écart d'ABI reproduit.

## Bun et Windows

- Le miroir `yolo-rs/bun` a été synchronisé au commit upstream `e57d89ba63a7504957faa2ccc8f416ce2fb15453` lors de la session; vérifier les HEAD avant de supposer que c'est toujours le dernier. La version stable relevée était 1.4.4. Bun est un workspace Rust avec JavaScriptCore/C++ et des sources JS/TS; préserver son système de build et ses licences.
- `C:\src\yolo-bun\vendor\bun-e57d89ba63a7` est un checkout généré ignoré; `upstream\bun` est un ancien checkout conservé. Les sources préparées, la file de patchs, les gardes et les fichiers non suivis peuvent appartenir à une session active : ne pas les réinitialiser.
- Contrairement à l'ancien rapport à file vide, `C:\src\yolo-bun\patches\queue.json` contient maintenant `windows-codegen-relative-paths.patch`, SHA-256 `14b6eb55ef423c81175a55822c9420030ace0703c061422b8d9f355ca9e6410e`, destiné à une limite de longueur de commande Windows pendant la génération AOT. Le reçu `docs\receipts\windows-codegen-patch.json` rapporte une comparaison locale de sorties et des contrôles ciblés; il ne démontre pas la performance ni un build release complet.
- Le rapport intersession mentionne une tentative Windows x64 release avec LTO et un processus 92243. Cet identifiant n'était plus accessible depuis l'outil de cette session; l'état final du build est donc **inconnu**. Vérifier les processus, journaux et artefacts avant toute nouvelle compilation. Ne pas démarrer de build concurrent sur le même répertoire.
- Bun upstream contient déjà des implémentations cookie, dotenv/environnement, Windows système et compatibilité V8/Node. Auditer un écart précis avant de porter du code de l'ancien fork `aphrody-labs/bun` (`7c57ac563e12c35be817697c4b5488b0fd64cc09`). Son ancienne API `bun:windows` est un candidat à un port ciblé; les surfaces UI restent à leur propriétaire. L'adaptateur FFI existant sous `yolo-cross/packages/windows` a un reçu limité à des lectures Windows; ConPTY et Job objects ne sont pas qualifiés par ce reçu.
- Bun 1.4 `check`, `lint`, `fmt`, AOT et ses API natives sont les outils de référence. AOT pour les applications JS/TS et LTO du binaire runtime sont deux mesures différentes. Comparer les binaires upstream et candidat de même révision, toolchain, options et machine; l'auto-comparaison de benchmark existante n'établit pas la parité.
- Les rapports `CODEX-SESSIONS.md` décrivent aussi WSL 2/Alpine, Docker Desktop et des vérifications Cargo/Bun de `yolo-cross`. Ce sont des observations historiques de l'hôte, à revérifier avant usage. Les bibliothèques `yolo-capture`, `yolo-process` et `yolo-windows-napi` y ont des vérifications ciblées rapportées; elles ne qualifient pas toutes les cibles.

## Paquets et liaison à l'image

Le gestionnaire Bun orchestre les backends sans remplacer leurs résolveurs, bases ou signatures : Bun pour JS/TS; pacman pour un profil Arch/CachyOS/Omarchy; Flatpak/Flathub pour applications; Cargo, uv et vcpkg pour leurs domaines; WinGet pour Windows; F-Droid pour Android. `yolo-cross` fournit les toolchains. Voir `docs/bun-package-orchestration.md` et `C:\src\yolo-bun\docs\PACKAGE-MANAGER.md`.

`yolo-bun` fournit `packages:image-plan` pour assembler les artefacts déjà résolus et `packages:verify-image-plan` pour comparer les fichiers locaux aux SHA-256 et vérifier le digest du plan. Cela ne vérifie ni la signature ni l'authenticité du plan et ne résout pas les dépendances. La connexion à `cargo-lfs`, la préparation de racine staging, les reçus de signatures et l'image restent à faire. Les tests ciblés, `bun check` et `bun lint` ont passé lors du dernier travail sur ces commandes; voir les commits `ab723af` et `bc030b6` de `yolo-rs/yolo-bun`.

## Sources et état à préserver

- `C:\src\bun-os-archive` est fortement modifié : source historique à consulter, jamais à nettoyer ou réinitialiser. Son image de production observée utilisait systemd; l'expérience Bun PID 1 était distincte.
- Les listes Awesome sont forkées sous `yolo-rs`; la branche `yolo-lfs-2026` de `yolo-rs/awesome-rust` préserve `e2e91e71`. Les licences et limites des listes sont documentées dans `C:\src\yolo-rs\README.md`.
- La prochaine étape est d'inspecter les modifications et processus courants, consolider la file de patchs Bun sans écraser la session Windows, puis raccorder un plan de paquets vérifié à un staging `cargo-lfs` avec des reçus indépendants. Les builds runtime, comparaisons de performance et boots restent des qualifications distinctes.
