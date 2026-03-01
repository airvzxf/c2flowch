#include <stdio.h>

void print_menu() {
    printf("1. Add\n");
    printf("2. Subtract\n");
    printf("3. Exit\n");
}

int add(int a, int b) {
    return a + b;
}

int subtract(int a, int b) {
    return a - b;
}

int main() {
    int choice = 0;
    int x, y, res;

    do {
        print_menu();
        printf("Enter choice: ");
        scanf("%d", &choice);

        if (choice == 1 || choice == 2) {
            printf("Enter two numbers: ");
            scanf("%d %d", &x, &y);
        }

        switch (choice) {
            case 1:
                res = add(x, y);
                printf("Result: %d\n", res);
                break;
            case 2:
                res = subtract(x, y);
                printf("Result: %d\n", res);
                break;
            case 3:
                printf("Exiting...\n");
                break;
            default:
                printf("Invalid choice!\n");
        }
    } while (choice != 3);

    return 0;
}
