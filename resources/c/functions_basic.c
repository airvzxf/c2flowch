#include <stdio.h>

void greet() {
    printf("Hello, World!\n");
}

int add(int a, int b) {
    return a + b;
}

int main() {
    greet();
    int sum = add(5, 7);
    printf("Sum: %d\n", sum);
    return 0;
}
