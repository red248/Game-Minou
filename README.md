# Minou 🐱 — jeux pour apprendre (4 à 7 ans)

Application web en **Rust + WebAssembly** ([Leptos](https://leptos.dev)), installable sur iPhone comme une vraie app (PWA, fonctionne hors ligne, sans pub, sans compte).

## Les jeux
- 📖 **Lecture** : lire un mot ou une phrase et toucher la bonne image (chaque mot se lit à voix haute quand on le touche)
- 🔤 **Les lettres** : reconnaître les lettres, majuscules/minuscules, première lettre d'un mot
- 🍎 **Calculs** : compter, additions, soustractions avec des fruits
- 🃏 **Les paires** : jeu de mémoire
- 🤖 **Robot jardinier** : premiers pas en programmation (suite de flèches)
- 🏴‍☠️ **Labyrinthes** : pirates, chevaliers, dinos (flèches ou glisser du doigt)
- 🚀 **Astéroïdes** et 🦕 **Le Dino** : jeux d'action, précédés d'un **mur de questions** au niveau de l'enfant
- ♟️ **Échecs** contre un petit robot (indice 💡 et retour arrière ↩)

## Espace parents
Protégé par un petit calcul : prénom, niveau (1 à 5), nombre de questions avant les jeux d'action, limite de temps par jour, son, jeux affichés, statistiques (temps, réussite par domaine). Tout reste sur le téléphone (localStorage).

## Installer sur iPhone
1. Ouvrir l'adresse GitHub Pages dans **Safari**
2. Bouton **Partager** → **Sur l'écran d'accueil**
3. Lancer « Minou » depuis l'écran d'accueil (plein écran, marche hors ligne)

## Développer
```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100
./build.sh                       # produit dist/
python3 -m http.server -d dist   # http://localhost:8000
```
Chaque push sur `main` déploie automatiquement sur GitHub Pages (Settings → Pages → Source : **GitHub Actions**).
