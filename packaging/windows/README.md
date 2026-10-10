# Pack Windows et releases GitHub

Le dépôt public est [Flwrian/FlowDPSMeter](https://github.com/Flwrian/FlowDPSMeter). Le nom de l’application reste AionPartyMeter. Les mises à jour commencent avec le nouveau lanceur 0.1.8 : installer ce pack complet une fois.

## Compiler et vérifier

Depuis la racine des sources, après installation de Node 22, Rust MSVC stable, Windows SDK et Visual Studio Build Tools C++ :

```powershell
npm ci
npm test
npm run build
cargo build --locked --manifest-path src-tauri/Cargo.toml --release --features custom-protocol --bin AionPartyMeter
powershell.exe -ExecutionPolicy Bypass -File packaging/windows/build-launcher.ps1 -Test
python packaging/windows/make-release.py
```

Le lanceur .NET Framework utilise le compilateur C# de Windows et les bibliothèques système. Pas de SDK .NET à installer. build-launcher.ps1 compile ensemble Launcher.cs, Updater.cs et UpdateStartup.cs. Les tests utilisent des fixtures isolées, sans réseau, capture, installation de dépendance ni lancement du meter.

Le script produit les ZIP Update et Sources dans release. Pour un pack complet, ajouter `--full-pack <dossier du pack précédent>` afin de reprendre les DLL Visual C++ et l’installateur WebView2 après vérification de leurs hashes. Utiliser un dossier de sortie neuf avec `--output` ou déplacer la sortie précédente avant de refaire les archives. Sans `--repository compte/depot`, le script reprend update.json.

## Publier une version

1. Choisir un numéro X.Y.Z supérieur à la version publiée. Modifier package.json, package-lock.json, src-tauri/Cargo.toml, le package aion-party-meter dans Cargo.lock, tauri.conf.json, le numéro affiché dans src/main.ts et les guides. Conserver les sections de validation des versions précédentes.
2. Exécuter les contrôles et produire les archives. Le workflow Build Windows release assets peut aussi être lancé depuis Actions. Un tag vX.Y.Z déclenche sa compilation ; le tag doit correspondre aux métadonnées.
3. Pousser les sources et le tag dans Flwrian/FlowDPSMeter. Créer une release sur ce tag avec des notes de version.
4. Joindre exactement **AionPartyMeter-X.Y.Z-Update-Windows-x64.zip** et **AionPartyMeter-X.Y.Z-Sources.zip**. Ajouter le ZIP **AionPartyMeter-X.Y.Z-Windows-x64.zip** pour les nouvelles installations. Le workflow génère les trois archives avec les dépendances Microsoft vérifiées, conserve un artifact et prépare automatiquement un brouillon sur un tag. Sur lancement manuel, cocher Prepare a draft release.
5. Publier la release comme version stable et la définir comme dernière release. Un brouillon ou une préversion est ignoré. Un push de sources seul ne diffuse pas de mise à jour.

Les hashes fournis par GitHub et le manifeste vérifient le téléchargement ; ils ne constituent pas une signature indépendante du dépôt. Le workflow prépare un brouillon avec ses trois assets ; il ne publie pas la version stable de lui-même. Relire le brouillon puis cliquer Publish release sur GitHub. Les sources GPL correspondantes sont incluses dans l’archive Sources.

## Commandes du lanceur

- `DEMARRER.exe` : vérification de version, mise à jour si nécessaire, vérification des dépendances puis ouverture.
- `--updates` : dépôt et activation des mises à jour, enregistrés dans update.json.
- `--skip-update` : ouverture sans réseau pour ce lancement.
- `--configuration` : assistant des dépendances et bouton Mises à jour GitHub.
- `--check <fichier.json>` : diagnostic des dépendances sans interface, capture ou installation.
- `--update-check <fichier.json>` : consulter la version disponible sans installation ni ouverture du meter. L’absence de release ou de réseau est rapportée dans le fichier.
- `--verify-microsoft <installateur.exe>` : vérification Authenticode Microsoft, sans exécution.

Le remplacement est bloqué quand le meter est ouvert. Un journal durable permet de restaurer la version précédente après interruption. Réglages, historique, captures, configuration GitHub et installateurs de dépendances sont conservés. Le petit ZIP Update n’embarque aucun installateur ni enregistrement du jeu.

WebView2 s’installe après clic avec l’installateur Microsoft fourni. Npcap n’est pas redistribué : le lanceur télécharge auprès de l’éditeur, vérifie Authenticode et ouvre l’installation interactive pour acceptation de licence. Ces fonctions et WinVerifyTrust sont conservés.
