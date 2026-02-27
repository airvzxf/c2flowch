#include <stdio.h>

int main() {
  printf("Inicio\n");

  int i = 0;
  do {
    printf("Iterar i");
    i++;

    int j = 0;
    do {
      printf("Iterar j");
      printf("While: %d, %d\n", i, j);
      j++;
    } while (j < 5);
  } while (i < 5);

  printf("Fin\n");
  return 0;
}
