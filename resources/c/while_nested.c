#include <stdio.h>

int main() {
  printf("Inicio\n");

  int i = 0;
  while (i < 5) {
    printf("Iterar i");
    int j = 0;
    while (j < 5) {
      printf("Iterar j");
      printf("While: %d, %d\n", i, j);
      j++;
    }
    i++;
  }

  printf("Fin\n");
  return 0;
}
