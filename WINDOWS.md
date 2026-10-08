# Windows x64 — version 0.10

## Compiler le launcher

Installer Rust stable avec la cible MSVC, Visual Studio Build Tools avec le composant « Développement Desktop en C++ », et CMake (accessible dans PATH). SDL2 est compilé et lié statiquement dans le launcher pour les manettes : aucun SDK SDL2 système n'est nécessaire pour compiler le launcher.

Depuis PowerShell dans le dossier du projet :

```powershell
cargo test --no-default-features
cargo build --release
.\target\release\z3-launcher.exe
```

L'exécutable conserve une console pour les diagnostics et les commandes CLI. La workflow `.github/workflows/build.yml` fournit une compilation native Windows et Linux une fois le projet placé dans un dépôt GitHub ; elle n'a pas été exécutée lors de la préparation de cette archive.

## Préparer les outils du jeu

Installer [MSYS2](https://www.msys2.org/) dans `C:\msys64`. Ouvrir son terminal **UCRT64**, effectuer les mises à jour selon les instructions MSYS2, puis :

```bash
pacman -S --needed git make mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-binutils mingw-w64-ucrt-x86_64-python mingw-w64-ucrt-x86_64-python-pillow mingw-w64-ucrt-x86_64-python-yaml mingw-w64-ucrt-x86_64-SDL2
```

Pour une autre installation, définir la variable d'environnement `MSYS2_ROOT` vers sa racine avant de démarrer le launcher :

```powershell
$env:MSYS2_ROOT = 'D:\outils\msys64'
.\target\release\z3-launcher.exe
```

Le bouton **Vérification des dépendances** teste les outils dans cet environnement et affiche les paquets manquants avec une commande `pacman` à lancer dans UCRT64. Chaque test est limité à cinq secondes. Si des dépendances manquent, le bouton **Installer les dépendances** apparaît : après confirmation des paquets, le launcher met MSYS2 entièrement à jour, installe les paquets UCRT64 nécessaires et relance la vérification. Si MSYS2 est absent, son installateur officiel est téléchargé depuis les releases GitHub de MSYS2 et sa somme SHA-256 est vérifiée avant exécution. Le dossier de destination est `C:\msys64`, ou `MSYS2_ROOT` si cette variable est définie. Un dossier existant mais incomplet ne sera jamais écrasé : réparez-le ou choisissez un autre emplacement. Les opérations sont affichées dans le journal ; attendez leur fin avant de fermer le launcher. Après une interruption ou un échec, relancez la vérification avant de réessayer. Le launcher appelle Git, Python et Make via le Bash de MSYS2, transmet les arguments séparément, et construit le jeu avec `CC=gcc`, `OS=Windows_NT` et `TARGET_EXEC=zelda3.exe`. Il lance ensuite le jeu nativement en ajoutant `ucrt64\bin` au PATH de son processus pour les éventuelles DLL du compilateur. Conserver MSYS2 installé ; cette version ne crée pas de paquet autonome du jeu avec ses DLL.

## Fichiers et mode portable

Par défaut, les données vont dans `%LOCALAPPDATA%\Z3-Launcher`, et les préférences du launcher dans son sous-dossier `config`.

Un fichier `portable.txt` à côté de **z3-launcher.exe** active `Z3-Launcher` à côté de l'exécutable, avec `Z3-Launcher\config` pour les préférences. Redémarrer le launcher après ajout ou suppression du marqueur. Le répertoire doit être accessible en écriture. Les fichiers existants ne sont pas déplacés automatiquement. Le mode portable concerne les données du launcher et du jeu ; les outils MSYS2 restent externes.

Sélectionner une ROM US compatible pour installer le jeu ; sélectionner ensuite la ROM de la langue choisie pour importer ses ressources, comme sous Linux.

## Validation restante

Les traductions ont été contrôlées et le code adapté par plateforme. Aucun compilateur Rust ni environnement Windows n'était disponible lors de la préparation : la compilation MSVC, la compilation du jeu, les chemins avec espaces/accents, le glisser-déposer et les manettes doivent encore être validés sur Windows. Aucune ROM ou ressource du jeu n'est incluse.

## Installation des dépendances sous Linux

Dans l’onglet Général, **Vérification des dépendances** affiche les paquets manquants. **Installer les dépendances** demande confirmation puis utilise apt (Debian/Ubuntu et dérivées), pacman (Arch et dérivées), dnf (Fedora) ou zypper (openSUSE). L’authentification administrateur est gérée par le bureau via `pkexec` : aucun mot de passe n’est collecté par le launcher. Si `pkexec` ou son agent d’authentification est absent, utilisez la commande affichée dans le journal. Les distributions non reconnues et les variantes immuables restent en installation manuelle. Les gestionnaires utilisent leurs dépôts et leur vérification habituelle des paquets. Aucun paquet n’est supprimé automatiquement.
