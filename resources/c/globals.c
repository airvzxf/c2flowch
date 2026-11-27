#include <stdio.h>

#define LIMIT 3

int counter = 0;
static double rate = 1.5, total;
const char *name = "c2flowch";
int values[LIMIT];
void report(void);

void report(void) {
  printf("%s: %d\n", name, counter);
}

int main(void) {
  int i;
  for (i = 0; i < LIMIT; i++) {
    counter++;
    total = total + rate;
  }
  report();
  return 0;
}
