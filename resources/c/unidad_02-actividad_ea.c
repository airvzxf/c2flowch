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
    // Ciclo principal del programa
    do {
        printf("------------------------------------------------------------\n");
        printf("| Compañía eléctrica: ABC                                  |\n");
        printf("------------------------------------------------------------\n");
        printf("Menú de opciones\n");
        printf("1. Calcular tarifa\n");
        printf("2. Salir\n");
        printf("Seleccione una opción: ");
        scanf("%hd", &opcion);

        if (opcion == 1) {
            short unsigned consumo;
            char cliente[50];

            // Captura de datos del cliente y consumo
            printf("  Cliente: ");
            // El espacio inicial consume saltos de línea previos; [^\n] permite capturar espacios
            scanf(" %49[^\n]", cliente);
            printf("  Consumo: ");
            scanf("%hd", &consumo);

            double tarifa;
            char *tarifa_tipo;

            // Determinación del tipo de tarifa según el consumo en kWh
            if (consumo <= 150) {
                tarifa = 2.10;
                tarifa_tipo = "Reducida";
            } else if (consumo <= 300) {
                tarifa = 2.94;
                tarifa_tipo = "Básica";
            } else {
                tarifa = 3.65;
                tarifa_tipo = "Alto Consumo";
            }

            // Cálculo del monto total a pagar
            const double PAGO = consumo * tarifa;

            printf("» Tarifa %s de $%.2f por kWh\n", tarifa_tipo, tarifa);
            printf("\n");

            // Impresión detallada de la factura
            printf("-------------------------------------------\n");
            printf("| FACTURA GENERADA                        |\n");
            printf("-------------------------------------------\n");
            printf("| Cliente: %s\n", cliente);
            printf("| Consumo: %hd kWh\n", consumo);
            printf("| Tarifa: %s\n", tarifa_tipo);
            printf("| Total a pagar: $%.2f\n", PAGO);
            printf("-------------------------------------------\n");
        } else if (opcion == 2) {
            printf("Gracias por su preferencia.\n");
        } else {
            printf("Selección incorrecta, intente de nuevo.\n");
        }
        printf("\n\n");
    } while (opcion != 2); // El programa termina cuando se elige la opción 2
    return 0;
}
