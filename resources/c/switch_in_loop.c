#include <stdio.h>

int main() {
  int op;
  for (;;) {
    scanf("%d", &op);
    switch (op) {
      case 0:
        return 0;
      case 1:
        if (op > 0)
          break;
        puts("never");
        break;
      case 2:
        continue;
      default:
        puts("?");
    }
    puts("next");
  }
}
