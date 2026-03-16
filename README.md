# telescrap-sr
<br>
<p align="center">
	<img src="doc/img/logo.png" width="150">
</p>

Script qui scan le site de la billetterie du Stade Rochelais pour détecter les places qui sont mises à la revente et notifie via Telegram.

## Roadmap

- [ ] Améliorer le scrapping : mode aggressif pour permettre de mettre au panier les billets qui ont été détectés en revente.
- [ ] Ajouter un module de log qui permet de suivre les actions du bot.
- [ ] Permet de définir les intervalles de scrapping via commandes Telegram (par l'interface administrateur).

## Pourquoi ce projet ?

Ce projet fournit un bot qui publie dans un groupe Telegram les billets trouvés en revente. Il est également possible de le configurer et de le superviser via un canal privé administrateur.

## Comment ça marche ?

Le bot analyse la page d'accueil de la billetterie, filtre les mentions liées au Stade Rochelais, puis détecte les boutons de revente. En cas de changement, il notifie les utilisateurs du canal avec un lien permettant d'acheter les billets.

## Configuration

1. Créer un bot Telegram et récupérer son token (voir [cette documentation](https://core.telegram.org/bots#6-botfather)).

2. Récupérer votre chat ID Telegram (voir [cette documentation](https://stackoverflow.com/a/32464976)).

3. Créer un fichier `.env` à la racine du projet avec les variables d'environnement suivantes :
```
TELOXIDE_TOKEN=your_bot_token_here
TELEGRAM_CHAT_ID=your_channel_chat_id_here
TELEGRAM_ADMIN_CHAT_IDS=your_admin_chat_id_here
```

## 🚨 Canal de revente Telegram 🚨

Le bot est actuellement actif sur un canal de revente privé, accessible uniquement sur ajout manuel.
Vous pouvez demander l'accès en contactant l'administrateur du projet en message privé.