#include <stdio.h>

int main() {
  printf("Inicio\n");

  int i = 0;
  do {
    printf("While: %d\n", i);
    i++;
  } while (i < 5);

  printf("Fin\n");
  return 0;
}
