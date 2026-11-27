#include <stdio.h>

int main() {
  int tries = 0;
  do {
    tries++;
    if (tries > 3)
      break;
    if (tries == 2)
      continue;
    puts("try");
  } while (tries < 10);
  return 0;
}
