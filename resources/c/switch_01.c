#include <stdio.h>

int main() {
  printf("Inicio\n");

  short unsigned opcion = 0;
  switch (opcion) {
    case 1:
        printf("Caso 1\n");
    case 2:
        printf("Caso 2\n");
        break;
    case 3:
        printf("Caso 3\n");
    default:
        printf("Caso default\n");
  }

  printf("Fin\n");
  return 0;
}
