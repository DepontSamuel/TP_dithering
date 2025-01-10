# Question 2

## Afin d'ouvrir une image depuis un fichier, on utilise image::open, on obtient alors un DynamicImage qu'on peut passer en RGB8 en utilisant la méthode to_rgb8.

# Question 3

## Lors de la convertion de l'image en RGB8, si l'image initiale avait une couche alpha, la convertion va simplement supprimer la couche. 

# Question 5

## L'image a a juste un filtre blanc.

# Question 6

## Afin de récupérer la luminosité d'un pixel, il faut utiliser la formule suivante : 0.2126 * R + 0.7152 * G + 0.0722 * B.