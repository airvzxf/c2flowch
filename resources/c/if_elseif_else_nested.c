#include <stdio.h>

int main() {
  printf("Inicio\n");

  short unsigned i = 0;
  if(i == 0) {
    printf("i: %d\n", i);
    short unsigned j = 0;
    if (j == 0) {
      printf("j: %d\n", j);
    } else if (j == 1) {
      printf("j == 1: %d\n", j);
    } else {
      printf("j else: %d\n", j);
    }
  } else if(i == 1) {
    printf("i == 1: %d\n", i);
  } else {
    printf("i else: %d\n", i);
  }

  printf("Fin\n");
  return 0;
}
