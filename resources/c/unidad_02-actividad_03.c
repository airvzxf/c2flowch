#include <stdio.h>

int main(void) {
    // Encabezado del programa con información del alumno
    printf("# ======================================================== #\n");
    printf("# Alumno: Israel Alberto Roldan Vega                       #\n");
    printf("# Matrícula: ES251105693                                   #\n");
    printf("# Grupo: DS-DFPR-2601-B1-022                               #\n");
    printf("# ======================================================== #\n");
    printf("\n");

    short unsigned opcion; // Almacena la opción seleccionada del menú
    // Ciclo principal del sistema despachador
    do {
        // Mostrar opciones de productos y sus precios
        printf("------------------------------------------------------------\n");
        printf("Sistema despachador de productos con monedas\n");
        printf("------------------------------------------------------------\n");
        printf("Menú de productos\n");
        printf("1. Ensalada    ← $82\n");
        printf("2. Agua        ← $12\n");
        printf("3. Desodorante ← $74\n");
        printf("4. Peine       ← $34\n");
        printf("5. RTX 5090    ← $45,325\n");
        printf("6. Salir\n");
        printf("Seleccione una opción: ");
        scanf("%hd", &opcion);

        // Procesar la selección de un producto (opciones 1 a 5)
        if (opcion > 0 && opcion < 6) {
            short unsigned precio;
            // Asignar el precio correspondiente al producto elegido
            switch (opcion) {
                case 1:
                    precio = 82;
                    break;
                case 2:
                    precio = 12;
                    break;
                case 3:
                    precio = 74;
                    break;
                case 4:
                    precio = 34;
                    break;
                case 5:
                    precio = 45325;
                    break;
                default:
                    precio = 0;
            }

            short unsigned monto = 0; // Acumulador del dinero total ingresado
            // Solicitar monedas hasta completar o superar el precio
            do {
                short unsigned moneda;
                printf("Ingrese una moneda ($1, $2, $5 o $10): $");
                scanf("%hu", &moneda);
                monto += moneda;
                printf("Total ingresado: $%hu\n", monto);
            } while (monto < precio);

            // Calcular y mostrar el cambio resultante
            const short unsigned cambio = monto - precio;
            if (cambio > 0) {
                printf("Su cambio es: $%hu\n", cambio);
            } else {
                printf("Pago exacto.\n");
            }

            printf("Producto entregado.\n");
        } else if (opcion == 6) {
            // Mensaje de salida del sistema
            printf("Gracias por su preferencia.\n");
        } else {
            // Notificar selección no válida
            printf("Selección incorrecta. Intente de nuevo.\n");
        }
        printf("\n\n");
    } while (opcion != 6); // El programa termina cuando se elige la opción 6
    return 0;
}
