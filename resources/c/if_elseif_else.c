#include <stdio.h>

int main() {
  printf("Inicio\n");

  short unsigned i = 0;
  if(i == 0) {
    printf("i: %d\n", i);
  } else if (i == 1) {
    printf("i == 1: %d\n", i);
  } else {
    printf("i else: %d\n", i);
  }

  printf("Fin\n");
  return 0;
}
