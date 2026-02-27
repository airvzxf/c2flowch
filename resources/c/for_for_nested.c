#include <stdio.h>

int main() {
  printf("Inicio\n");

  for (int i = 0; i < 3; i++) {
    printf("Iterar i");
    for (int j = 0; j < 3; j++) {
      printf("Iterar j");
      printf("For: %d, %d\n", i, j);
    }
  }

  printf("Fin\n");
  return 0;
}
