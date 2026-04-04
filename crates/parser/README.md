


## Interface : 

- lien vers le possibilité de récupérer le html d'une page
    - Utilisation du crate `curl-rust`

- getter : 
- `get_club` : Récupère les informations d'un club à partir de son nom
- `get_match` : Récupère les informations d'un ou plusieurs matchs à partir des informations d'un club
- `get_seat` : R2écupère les informations d'une ou plusieurs places à partir des informations d'un match
## Controller : 



## App : 

## Core : 

### Entities : 
- `club` : Une entité qui contient les information d'un club, avec notamment l'adresse du site de la billeterie, un enum pour définir le type et le nom.
- `match` : Un match contient les informations d'un match, notamment la date et l'éventuelle lien vers la billeterie de ce match et si des places sont disponibles ou pas à la revente.
- `seat` : Les informations qui concerne un place disponible à la revente. Notamment le prix, la catérogie, le nombre de place disponible, etc.
