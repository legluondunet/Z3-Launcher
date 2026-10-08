# AppImage Linux x86_64 — 0.11

## Construire

Le script compile le launcher, crée un AppDir avec ses traductions, son icône et son fichier desktop, déploie les bibliothèques avec linuxdeploy, puis produit :

```
dist/Z3-Launcher-0.11.15-x86_64.AppImage
dist/Z3-Launcher-0.11.15-x86_64.AppImage.sha256
```

Préférer Ubuntu 22.04 x86_64 pour la construction : un binaire compilé avec une glibc récente (par exemple sur Manjaro) peut ne pas fonctionner sur une distribution plus ancienne. Cette méthode vise Ubuntu 22.04 et des systèmes avec glibc 2.35 ou ultérieure, à valider sur chaque distribution visée. Elle ne couvre pas musl/Alpine ou les anciennes distributions. Les pilotes graphiques restent ceux du système.

Sur Ubuntu 22.04, installer Rust stable avec rustup, puis les paquets de construction :

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config cmake curl file python3 libsdl2-dev libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libgl1-mesa-dev
./build-appimage.sh
```

Le script télécharge linuxdeploy depuis sa publication officielle « continuous » et le conserve dans `tools/appimage`. Pour utiliser une version précise préalablement téléchargée, fournir un chemin absolu :

```bash
LINUXDEPLOY=/chemin/linuxdeploy-x86_64.AppImage ./build-appimage.sh
```

La construction nécessite Internet pour les dépendances Cargo et le téléchargement de linuxdeploy. Pour des versions reproductibles, conserver le Cargo.lock généré et la version exacte de linuxdeploy utilisée. La workflow GitHub `Native builds` inclut désormais un job AppImage sur Ubuntu 22.04 ; elle produit un artefact téléchargeable lorsqu'elle est exécutée dans un dépôt GitHub.

## Utiliser et distribuer

Distribuer l'AppImage et, si souhaité, son fichier SHA256. L'utilisateur n'a pas besoin de Rust pour lancer le launcher :

```bash
chmod +x Z3-Launcher-0.11.15-x86_64.AppImage
./Z3-Launcher-0.11.15-x86_64.AppImage
```

En l'absence de FUSE, essayer le mode extraction du runtime :

```bash
./Z3-Launcher-0.11.15-x86_64.AppImage --appimage-extract-and-run
```

Les bibliothèques du launcher (dont SDL2 et les bibliothèques X11/Wayland chargées dynamiquement) et les traductions sont embarquées. Un environnement de bureau, ses pilotes OpenGL et un portail XDG avec backend pour le sélecteur de fichiers restent nécessaires. Aucune ROM ni ressource Nintendo n'est incluse.

Le jeu continue à être téléchargé et construit sur la machine de l'utilisateur. Git, Python/Pillow/PyYAML, Make, le compilateur C, les fichiers de développement SDL2 et sha256sum restent des dépendances système. Le bouton « Vérifier » affiche les éléments manquants et la commande adaptée à la distribution. Les bibliothèques embarquées du launcher ne sont pas imposées aux outils système ni au jeu lancé : le LD_LIBRARY_PATH d'origine est restauré pour ces processus.

## Données et traductions

Créer un dossier portant le nom complet de l'AppImage suivi de `.home` :

```bash
mkdir -p Z3-Launcher-0.11.15-x86_64.AppImage.home
```

Au lancement, HOME et les variables XDG de données, configuration, cache et état sont redirigées vers ce dossier. Les données du jeu et le journal vont dans `.home/.local/share/Z3-Launcher`, les préférences et traductions personnalisées dans `.home/.config/Z3-Launcher`. Ce mode force le dossier de travail local : les anciens chemins enregistrés et l'option CLI `--dir` ne le remplacent pas. Le dossier `.home` prend priorité sur `portable.txt` s'ils sont tous deux présents. Aucun fichier existant n'est déplacé automatiquement. Déplacer ou renommer l'AppImage avec son dossier `.home` correspondant ; son nom doit toujours correspondre au nom complet de l'AppImage.


Les données restent par défaut dans le répertoire XDG habituel. Un `portable.txt` à côté du fichier **.AppImage** active un répertoire `Z3-Launcher` situé à cet endroit, et non dans le montage temporaire en lecture seule. Redémarrer l'application après changement du marqueur. Ce répertoire doit être accessible en écriture ; les anciennes données ne sont pas déplacées.

Pour ajouter une traduction, placer son JSON dans le dossier `locales` du répertoire de configuration utilisateur, ou dans `Z3-Launcher/config/locales` en mode portable. Les JSON inclus dans l'AppImage sont en lecture seule.

## Validation avant publication

La syntaxe du script, les traductions et le démarrage AppRun avec un exécutable de contrôle ont été vérifiés. Aucun compilateur Rust n'est disponible dans l'environnement où cette archive a été préparée : aucune AppImage réelle n'a été construite ni lancée ici.

Avant publication, construire sur Ubuntu 22.04, puis tester l'AppImage sur Ubuntu/Debian, Fedora et Arch/Manjaro : lancement X11/Wayland, sélection et glisser-déposer de ROM, manettes, langue, installation du jeu, et mode portable dans un chemin avec espaces et accents. Vérifier aussi le lancement sur une machine sans Rust et l'affichage des dépendances manquantes.
