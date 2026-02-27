#include <stdio.h>

int main() {
  printf("Inicio\n");

  short unsigned i = 0;
  if(i == 0) {
    printf("i: %d\n", i);
    short unsigned j = 0;
    if (j == 0) {
      printf("j: %d\n", j);
    }
  }

  printf("Fin\n");
  return 0;
}
