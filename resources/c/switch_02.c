#include <stdio.h>

int main() {
  printf("Inicio\n");

  short unsigned opcion = 0;
  switch (opcion) {
    case 1:
        printf("Caso 1\n");
        break;
    case 2:
        printf("Caso 2\n");
        break;
    case 35:
        printf("Caso 35\n");
        break;
    default:
        printf("Caso default\n");
  }

  printf("Fin\n");
  return 0;
}
