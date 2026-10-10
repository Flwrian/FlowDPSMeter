# Vérifications de livraison — 10/10/2026

- TypeScript : vérification des types et build Vite réussis.
- Rust sans desktop : 145 tests de bibliothèque, 22 tests de groupe/replay et 24 tests traversant le décodeur réseau réussis. Un test amont nécessitant une capture externe est ignoré.
- Rust desktop : 168 tests de bibliothèque réussis, dont sauvegarde atomique, limite de 200 combats, préservation d’un historique illisible, migration des anciens réglages et protection des cellules CSV. Le test de journalisation utilise un exécuteur séquentiel pour éviter une interférence de ses subscribers tracing.
- Les deux variantes de bibliothèque partagent des tests : ces nombres ne s’additionnent pas. Total distinct Rust : **214 tests réussis, 1 ignoré**.
- Fenêtre : six tests ciblés du calcul de position/taille, de la conversion DPI, du maintien des coins opposés et de la taille minimale compacte sont fournis avec `npm test`.
- Exécutable release Windows x64 compilé avec le protocole Tauri de production ; interface embarquée et aucun serveur Vite nécessaire.
- Le contrôle natif a confirmé le chargement de l’interface, l’écran de réglages, le recentrage à 620 × 600 et le diagnostic Npcap absent.
- Le premier geste rapide automatisé n’a pas déplacé la fenêtre avec l’ancienne implémentation. Le déplacement/redimensionnement a été remplacé par une gestion des pointeurs conservant leur position finale après un relâchement rapide. **La dernière vérification visuelle de ce correctif a été interrompue par Échap ; elle reste à effectuer.**
- Lanceur compilé en x64, interface française inspectée. Détection WebView2 présent / Npcap absent contrôlée sur ce PC.
- Installateur WebView2 fourni : signature Authenticode Microsoft valide ; vérification WinVerifyTrust du lanceur réussie sans exécuter l’installateur.
- Le contrôle du lanceur rejette un exécutable qui ne possède pas une signature Microsoft attendue (code de sortie 1). Les deux DLL Visual C++ incluses ont une signature Microsoft valide.
- Téléchargement Npcap : lien réel de la page officielle résolu par le lanceur ; signature Nmap Software LLC validée. L’installateur utilisé pour cette vérification n’est pas redistribué dans le pack.
- Dépendances natives de l’exécutable inspectées : `vcruntime140.dll` et `vcruntime140_1.dll` fournies depuis la distribution x64 redist de Visual Studio. Les autres imports sont des composants Windows.

L’installation des dépendances sur un Windows vierge, le mode verrouillé avec clics traversants, la reprise depuis l’icône de notification, les exports depuis l’interface finale et un combat réel Global / Europe avec soigneur ne sont pas prouvés par ces vérifications. Npcap n’a pas été installé sur ce PC pendant cette tâche. Le replay synthétique ne constitue pas une preuve en jeu.

Le pack fournit `SHA256.json` pour contrôler son contenu. Les sources, exemples, licences et instructions sont livrés avec les archives correspondantes.

## Correctif 0.1.1 — groupe en monde ouvert

Huit régressions supplémentaires vérifient : liste comprimée et fragmentée reconnue avant combat ; premières identités conservées au-delà de trois secondes ; retransmissions ne déclenchant pas la sélection du flux ; éviction bornée signalée ; composition conservée après changement de zone ; dissolution complète ; sélection explicite du soigneur et du membre sans dégâts avec exclusion des extérieurs ; attribution par identité exacte tardive ; reset et replay de la composition manuelle. Certaines vérifications partagent un même test.

Le premier fichier réel fourni (544 segments, environ 27 secondes) a été rejoué : il reproduit une composition absente, ne contient aucune liste de groupe reconnue ni le pseudo du coéquipier attendu, et présente zéro lacune TCP. Le correctif ne peut pas restaurer une identité absente de ces octets. Une nouvelle capture reçue depuis la formation du groupe est requise pour confirmer l'affichage du coéquipier en jeu. Aucun contrôle de l’interface sur le bureau ni action dans le jeu n’a été effectué pour ce correctif.

## Correctif 0.1.2 — preuve sur un enregistrement réel

Le second fichier reçu (458 segments, environ 23 secondes) contient des enregistrements de propriétaire de butin avec une identité explicite de coéquipier, mais aucune liste de groupe reconnue. Le moteur amont retenait son nom pour affichage sans le marquer comme identité confirmée ; notre agrégateur le rejetait donc. Les liaisons exactes sont désormais confirmées ; les recherches plus larges restent exclues.

En replay du fichier réel, avec le coéquipier explicitement sélectionné conformément à la composition donnée par l’utilisateur : 11 470 dégâts locaux, 50 294 dégâts du coéquipier, 3 114 soins et 61 764 dégâts collectifs, durée commune 15,448 secondes. Une entité reste inconnue et exclue. Aucun chiffre n’a été inséré dans le moteur. La capture originale est conservée sans modification et n’est pas redistribuée dans les sources. Ce replay valide l’attribution des événements du fichier, pas la présence d’une composition automatique ni un nouveau combat live avec le binaire final.

Quatre tests supplémentaires vérifient l’identité exacte et la récupération des premiers dégâts, l’exclusion d’un propriétaire extérieur, le refus d’une liaison approximative, et les priorités de configuration d’un ancien replay et des scénarios synthétiques.

## Correctif 0.1.3 — dégâts reçus

Le décodeur rejetait les compétences NPC à sept chiffres avant de lire leurs valeurs. Ce rejet a été supprimé : la reconstruction, les limites de paquets, le décodage de la valeur et l’exclusion des joueurs hors groupe continuent à s’appliquer. Les enregistrements directs à blocage issus des preuves amont sont maintenant traversés jusqu’au snapshot du groupe : 74 dégâts (Shield Block), 116 (Iron Wall), 68 (Regeneration) et 1 (Perfect Block). Ils démarrent le combat, alimentent totaux/compétences/courbes et une retransmission ne les compte pas deux fois. Les dégâts périodiques d’un NPC confirmé vers un joueur confirmé n’exigent plus la liste de DoT des joueurs.

Trois tests supplémentaires couvrent ces quatre impacts, l’identité tardive d’une victime alliée avec exclusion d’une autre victime, et un effet périodique hostile de 68.

Le troisième enregistrement réel reçu (162 segments, 8,001 secondes) contient deux attaques NPC sur l’entité 1481 : 518 et 677. Le replay corrigé les attribue à Yokosoo et obtient 1 195 dégâts reçus, répartis dans les compétences et la courbe. Ce fichier ne contient pas d’identité : les paquets réels des deux captures précédentes du même flux TCP ont fourni le contexte, avec Viktor explicitement sélectionné selon la composition donnée par l’utilisateur. Les intervalles entre enregistrements produisent un avertissement de capture partielle ; le rapport ne représente pas l’ensemble du combat. Le replay autonome du seul troisième fichier ne peut pas identifier sa victime. Les originaux restent inchangés et ne sont pas redistribués. La vérification visuelle en direct avec le nouveau binaire reste à effectuer.

## Correctif 0.1.4 — fluidité, compact et découverte

Le tableau reçoit les totaux et les identités, sans charger les compétences et les courbes de tous les membres à chaque rafraîchissement. L’ouverture d’un détail charge ceux du membre choisi. Le cache reste invalidé par les événements, les identités, la composition, le filtre et les pertes ; DPS et HPS restent recalculés selon la durée commune. Les archives et exports passent par la projection complète. La recherche d’une cible vaincue n’est plus exécutée sur tous les événements à chaque impact ordinaire.

Sur un scénario synthétique de 50 000 événements, la sérialisation du snapshot passe de **61 702 à 1 350 octets**. Dans une exécution debug sur ce PC : 131,8 ms pour la première projection complète, 66,4 ms pour la première projection résumée et 0,245 ms pour 100 lectures résumées sans nouveaux événements. Ce sont des mesures du moteur, pas une mesure de FPS du jeu ou de l’overlay en production.

Six régressions supplémentaires couvrent : égalité des totaux entre projections ; taux évoluant sans nouvel impact et invalidation par identité tardive/perte ; réduction de la taille du snapshot ; ajout explicite préservant les membres existants et excluant les extérieurs ; redémarrage conservant les identités sur le même flux et les supprimant sur un autre ; migration des réglages vers le mode compact. Le redimensionnement compact est aussi testé à 125 % de DPI.

La page `tests/ui-smoke.html`, servie uniquement pendant le développement, utilise une fixture produite par le vrai replay et un adaptateur IPC synthétique. **12 contrôles UI réussis dans le navigateur local** : rafraîchissement, stabilité de la ligne et du focus, arrivée d’une identité dans le panneau ouvert, conservation du brouillon, ajout en un clic, membres conservés, compétences/courbe à la demande, panneau export persistant, compact, colonnes/résumé, onglet Soins, soigneur et membre sans dégâts. La mise en page compacte à 420 pixels a été inspectée. Cette page ne valide pas Npcap, les commandes natives de fenêtre ou un combat en jeu.

Les replays des enregistrements réels donnent toujours 61 764 dégâts et 3 114 soins pour le second fichier avec la sélection explicite de Viktor, puis 1 195 dégâts reçus dans le troisième avec son contexte d’identité antérieur. Les fichiers bruts sont restés inchangés. Aucune nouvelle liste de groupe Global/Europe n’a été décodée : l’ajout rapide résout la sélection et le rafraîchissement, mais une identité absente du trafic reste inconnue. Le contrôle natif du déplacement et du mode compact pendant le jeu reste à effectuer.

## Version 0.1.5 — tableau complet et courbes du groupe

- Rust : 145 tests de bibliothèque sans desktop, 168 avec desktop, 22 tests de groupe et 23 tests réseau réussis. Les tests de bibliothèque communs ne s’additionnent pas : **213 tests distincts réussis, 1 test amont ignoré**.
- Deux nouvelles régressions vérifient l’égalité des totaux entre projections, les soins et dégâts reçus séparés, l’attribution unique d’une invocation, l’exclusion des extérieurs et des actions postérieures au départ, la conservation d’un soigneur et d’un membre inactif, le filtre cible, les combats terminés et une réponse de dashboard sans détails inutiles.
- **12 tests JavaScript réussis**, dont six nouveaux : intervalles vides, dernière seconde partielle, événement à la borne finale, combat long borné à 160 intervalles, sommes collectives sans mutation, tri et identités inconnues, SVG sans identifiant de dégradé partagé et valeurs de survol.
- TypeScript strict et build Vite de production réussis. Compilation release Windows x64 avec `custom-protocol` : interface embarquée, sans serveur de développement.
- **25 contrôles UI réussis** dans le navigateur local avec scénario et IPC explicitement synthétiques. Les 12 contrôles précédents sont conservés ; les nouveaux couvrent dix colonnes et tous les membres, trois graphiques simultanés, les huit cellules numériques « — » d’une identité manquante, soins du membre sans dégâts, tri HPS, stabilité du focus, masquage/rétablissement des graphiques, ouverture des détails depuis le tableau, changement de statistique dans les détails, historique et retour à la vue Dégâts.
- Inspection visuelle à 1120 × 780 : dix colonnes et trois graphiques entièrement visibles avec quatre membres, sans défilement horizontal ou vertical du contenu principal. Les groupes plus grands et petites fenêtres peuvent nécessiter un défilement. Le mode compact à 420 pixels reste couvert par le scénario.
- Replays réels inchangés : 61 764 dégâts et 3 114 soins du groupe avec Viktor explicitement sélectionné ; 1 195 dégâts reçus avec le contexte d’identité antérieur. Les fichiers bruts sont conservés sans modification et ne sont pas livrés.

Ces vérifications ne prouvent pas les commandes natives de déplacement/redimensionnement ni la fluidité ou l’exhaustivité des événements pendant un nouveau combat en jeu. Les courbes proviennent des événements réellement comptés ; des pertes ou des identités absentes restent signalées. La composition automatique Global/Europe reste limitée par l’absence de liste décodable dans les captures reçues.

## Version 0.1.6 — compact simultané, actualisation et suivi du groupe

- **214 tests Rust distincts réussis, 1 ignoré** : 168 bibliothèque desktop, 22 groupe et 24 décodeur/replay. Le nouveau test réseau vérifie le retrait rapide, l’exclusion des événements postérieurs au retrait, la préservation des dégâts antérieurs, le mode solo, les identités en attente et l’indépendance entre groupe courant et dernier combat.
- **16 tests JavaScript réussis**. Les quatre nouveaux couvrent le taux par défaut (10 Hz), le rejet de fréquences non prises en charge, les modes 4/20 Hz, le ralentissement des détails/fenêtres masquées, l’absence de boucle de rattrapage, la distinction entre « vous seul suivi » et solo confirmé, et le nombre d’identités confirmées.
- TypeScript strict, build Vite de production et compilation Windows x64 avec `custom-protocol` réussis. Le pack embarque la nouvelle interface et ne dépend pas du serveur Vite.
- **38 contrôles UI réussis** dans le navigateur local. Le scénario utilise les événements synthétiques du moteur et un adaptateur IPC explicitement synthétique. Vérifications : actualisation observée à 10 Hz avec cet adaptateur ; conservation du focus ; découverte/ajout avec brouillon préservé ; trois cellules simultanées par membre compact ; soigneur sans dégâts et membre inactif ; tri compact ; identités inconnues « — » ; largeur 320 px sans débordement horizontal ; option 20 Hz ; désactivation des animations et montant de la fixture effectivement changé ; vue complète/graphes/détails/historique ; retrait rapide, solo, état du groupe courant indépendant du rapport et retour Auto.
- Inspection visuelle du compact à **400 × 300** avec quatre membres, trois colonnes et totaux communs. Les noms/totaux/taux sont visibles ensemble ; les panneaux restent accessibles par la barre de titre. Une hauteur réduite ou un grand groupe défile dans les lignes, avec les totaux maintenus en bas.
- Les replays réels conservent 61 764 dégâts / 3 114 soins du groupe et 1 195 dégâts reçus dans le fichier avec contexte d’identité antérieur. Les sources brutes restent inchangées et ne sont pas redistribuées.

Le taux d’affichage choisi est un objectif d’actualisation, pas une garantie de fréquence des paquets du serveur ni une mesure de FPS en jeu. L’interface ne crée pas de montants intermédiaires. La capture réelle Npcap et les gestes natifs de fenêtre ne sont pas simulés comme une preuve live par ce scénario. Le confort pendant un combat réel et la liste automatique Global/Europe restent à confirmer ; une sélection explicite reste nécessaire quand la liste manque dans le trafic.


## Version 0.1.7 — affichages et feedback, moteur inchangé

- 24 tests JavaScript réussis, dont huit nouveaux pour les états de capture, erreurs, pertes et identités ; référence initiale et hausse réelle du compteur ; remise à zéro lors d’un changement de flux/mode ; historique isolé des pertes live ; préférences et tailles des affichages ; tri indépendant des panneaux sans mutation des données.
- 59 contrôles UI réussis dans le navigateur local avec fixture et IPC explicitement synthétiques. Les 38 contrôles précédents sont conservés. Nouveaux contrôles : cartes sur deux/une colonnes, préférence mémorisée, géométrie préservant la position, trois panneaux et classements indépendants, résumés, focus, détail Soins au clic, bandeau minimal, résumé visible, aucune commande de groupe/capture/combat lors du changement de disposition, activité sur hausse observée seulement, repos sans faux arrêt, confirmations/pending, badge Partiel sur pertes, erreurs et replay.
- Rendus inspectés : Bandeau 400 × 256, Cartes 460 × 324 et Trois panneaux 780 × 344 avec quatre membres, dont un soigneur sans dégâts et un membre inactif. La petite largeur de 320 pixels fait passer les cartes sur une colonne.
- TypeScript strict, build Vite de production et compilation release Windows x64 avec custom-protocol réussis. L’interface est embarquée dans le binaire.
- 93 fichiers Rust de production et de tests vérifiés identiques par SHA256 à leur état avant intervention. Aucun changement au moteur, à la capture, à la composition automatique, aux invocations ou aux calculs. La suite Rust validée en 0.1.6 reste une preuve précédente, sans nouvelle exécution revendiquée pour ce changement visuel.

Le test navigateur vérifie le rendu et l’IPC synthétique ; il ne prouve pas les gestes natifs ou la fluidité pendant un nouveau combat en jeu. La détection automatique qui fonctionne sur le client utilisateur est conservée. L’activité réseau indique des hausses du compteur observées par l’affichage ; elle n’invente ni paquet ni horodatage précis.

## Version 0.1.8 — lanceur et mises à jour GitHub

- 33 contrôles du lanceur réussis : configuration et désactivation sans requête, comparaison stable/égale/ancienne/préversion, URLs limitées au dépôt, digest obligatoire, traversée et doublons ZIP refusés, hashes de fichiers, installation transactionnelle, conservation de réglages/historique/dépôt, annulation, rollback après échec injecté, restauration au lancement suivant et nettoyage après interruption de commit/rollback. Un processus réel du helper remplace ses fixtures sans ouvrir le meter ni la capture.
- 24 tests JavaScript réussis ; TypeScript strict et build Vite de production réussis.
- Compilation Windows x64 release avec custom-protocol réussie en 0.1.8. Le nouveau lanceur est compilé avec les bibliothèques système Windows, sans SDK supplémentaire pour l’utilisateur.
- 93 fichiers Rust de production/tests identiques par SHA256 à l’état avant les changements visuels 0.1.7. Aucun changement au moteur de capture, à la découverte automatique du groupe ou aux statistiques. La suite Rust 0.1.6 et les 59 contrôles navigateur 0.1.7 sont des vérifications précédentes, non relancées pour ce changement de lanceur.

Les tests de remplacement portent sur un dossier isolé : ils ne mettent pas à jour l’application actuellement ouverte. La diffusion GitHub, le téléchargement d’une future version et le relancement complet en conditions utilisateur nécessitent une release stable publiée et un lancement du pack avec le meter fermé. La version installée reste utilisable lorsque GitHub est indisponible. Aucun nouveau combat en jeu n’est revendiqué.
