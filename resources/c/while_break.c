#include <stdio.h>

int main() {
  int n = 0;
  while (1) {
    scanf("%d", &n);
    if (n < 0)
      break;
    if (n == 0) {
      puts("zero");
      continue;
    }
    printf("%d\n", n * 2);
  }
  return 0;
}
