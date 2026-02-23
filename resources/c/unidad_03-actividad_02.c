// Librerías
#include <stdio.h>

// Prototipos de las funciones
float calcularArea(float, float);
float calcularPresupuesto(float);

// Función: calcula el área: base por altura
float calcularArea(float base, float altura) {
  return base * altura;
}

// Función: calcula el presupuesto y agrega el IVA
float calcularPresupuesto(float area) {
  return area * 150 * 1.16;
}

// Función principal
int main(void) {
  // Encabezado del programa con información del alumno
  printf("# ======================================================== #\n");
  printf("# Alumno: Israel Alberto Roldan Vega                       #\n");
  printf("# Matrícula: ES251105693                                   #\n");
  printf("# Grupo: DS-DFPR-2601-B1-022                               #\n");
  printf("# ======================================================== #\n");
  printf("\n");

  // Información de la tarifa
  printf("=== Cotizador: Pinturas Rápidas ===\n");
  printf("Tarifa por m2: $150.00\n");
  printf("\n");

  // Entrada de datos por parte del usuario
  float base, altura;
  printf("Ingrese los siguientes datos.\n");
  printf("Base de la pared (metros): ");
  scanf("%f", &base);
  printf("Altura de la pared (metros): ");
  scanf("%f", &altura);
  printf("\n");

  // Llama a la función para calcular el área
  const float AREA = calcularArea(base, altura);
  // Llama a la función para calcular el presupuesto
  const float PRESUPUESTO = calcularPresupuesto(AREA);

  // Muestra al usuario la información del presupuesto
  printf("--- Recibo del presupuesto ---\n");
  printf("Medidas: %.2f m x %.2f m\n", base, altura);
  printf("Area total: %.2f m2\n", AREA);
  printf("------------------------------\n");
  printf("Total con IVA: $%.2f MXN\n", PRESUPUESTO);
  printf("\n");

  // Regresa cero, indicando que se ejecutó exitosamente
  return 0;
}
