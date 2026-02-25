#include <stdio.h>

int main() {
  printf("Inicio\n");

  int i = 0;
  while (i < 5) {
    printf("While: %d\n", i);
    i++;
  }

  for (int i = 0; i < 3; i++) {
    printf("For: %d\n", i);
  }

  printf("Fin\n");
  return 0;
}
