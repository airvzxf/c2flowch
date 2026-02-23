// Librerías
#include <stdio.h>
#include <string.h>

// Definiciones del compilador
#define LINEAS_SIZE 100
#define CURP_SIZE 19
#define NOMBRE_SIZE 50
#define NUMERO_SIZE 11
#define OPERADORA_SIZE 50

// Estructura de datos para la línea telefónica
struct linea_telefonica {
  short unsigned id;
  char curp[CURP_SIZE];
  char nombre[NOMBRE_SIZE];
  char numero[NUMERO_SIZE];
  char operadora[OPERADORA_SIZE];
} lineas_telefonicas[LINEAS_SIZE];

// Prototipos de las funciones
void mostrar_encabezado();

void seleccionar_opcion_menu(short unsigned *);

void registrar_linea(short unsigned *, struct linea_telefonica *);

void mostrar_informacion_linea(struct linea_telefonica *);

struct linea_telefonica buscar_linea_por_id(short unsigned *, struct linea_telefonica *);

struct linea_telefonica buscar_linea_por_curp(char [CURP_SIZE], struct linea_telefonica *);

void buscar_linea(struct linea_telefonica *);

void eliminar_linea(short unsigned *, struct linea_telefonica *);

void eliminar_linea_por_id(short unsigned *, struct linea_telefonica *);

void mostrar_registros_disponibles(short unsigned *);

void mostrar_opcion_invalida();

void capturar_entrada_usuario_numero(char *, short unsigned *);

void capturar_entrada_usuario_texto(char *, char *);

// Muestra el encabezado del programa con información del alumno.
void mostrar_encabezado() {
  puts("# ======================================================== #");
  puts("# Alumno: Israel Alberto Roldán Vega                       #");
  puts("# Matrícula: ES251105693                                   #");
  puts("# Grupo: DS-DFPR-2601-B1-022                               #");
  puts("# ======================================================== #");
  puts("");
}

// Muestra el menú principal y obtiene la opción del usuario.
void seleccionar_opcion_menu(short unsigned *opcion) {
  puts("=== Menú principal ===");
  puts("1. Registrar línea");
  puts("2. Buscar línea");
  puts("3. Eliminar línea");
  puts("4. Salir");
  puts("------------------------");
  printf("Seleccione una opción: ");
  capturar_entrada_usuario_numero("%hu", opcion);
  puts("");
}

// Solicita al usuario los datos para registrar una nueva línea telefónica.
void registrar_linea(short unsigned *linea_id, struct linea_telefonica *linea) {
  if (*linea_id >= LINEAS_SIZE) {
    puts("!!! Error en el registro, no hay más líneas disponibles");
    mostrar_registros_disponibles(linea_id);
    puts("");
    return;
  }

  char curp[CURP_SIZE];
  char nombre[NOMBRE_SIZE];
  char numero[NUMERO_SIZE];
  char operadora[OPERADORA_SIZE];

  puts("--- Registrar línea telefónica ---");
  printf("CURP (18 caracteres alfanuméricos): ");
  capturar_entrada_usuario_texto(" %18s", curp);
  printf("Nombre: ");
  capturar_entrada_usuario_texto(" %49[^\n]", nombre);
  printf("Número telefónico: ");
  capturar_entrada_usuario_texto(" %10s", numero);
  printf("Operadora (Telcel, AT&T, Movistar, etc): ");
  capturar_entrada_usuario_texto(" %49[^\n]", operadora);

  if (strlen(curp) != 18) {
    puts("");
    puts("!!! Error en el registro, la CURP debe ser de 18 caracteres exactos");
    puts("");
    return;
  } else if (strlen(numero) != 10) {
    puts("");
    puts("!!! Error en el registro, el número telefónico debe ser de 10 dígitos exactos");
    puts("");
    return;
  }

  (*linea_id)++;
  linea->id = *linea_id;
  strcpy(linea->curp, curp);
  strcpy(linea->nombre, nombre);
  strcpy(linea->numero, numero);
  strcpy(linea->operadora, operadora);
  puts("*****************************");
  puts("*   Se registró la línea!   *");
  puts("*****************************");

  mostrar_informacion_linea(linea);

  mostrar_registros_disponibles(linea_id);
  puts("");
}

// Muestra en pantalla la información detallada de una línea telefónica.
void mostrar_informacion_linea(struct linea_telefonica *linea) {
  if (linea->id > 0) {
    puts("");
    puts("~~~ Información de la línea ~~~");
    printf("ID: %hu\n", linea->id);
    printf("CURP: %s\n", linea->curp);
    printf("Nombre: %s\n", linea->nombre);
    printf("Número: %s\n", linea->numero);
    printf("Operadora: %s\n", linea->operadora);
  }
}

// Busca una línea telefónica por su ID en el arreglo de líneas.
struct linea_telefonica buscar_linea_por_id(short unsigned *linea_id, struct linea_telefonica *lineas) {
  struct linea_telefonica linea_vacia = {0};
  if (*linea_id < 1 || *linea_id > 100) {
    puts("");
    puts("!!! Error, el ID está fuera del rango");
    return linea_vacia;
  }

  if (lineas[*linea_id - 1].id == 0) {
    puts("");
    puts("^^^ No se encontró el registro con ese ID");
    return linea_vacia;
  }

  return lineas[*linea_id - 1];
}

// Busca una línea telefónica por su CURP en el arreglo de líneas.
struct linea_telefonica buscar_linea_por_curp(char curp[CURP_SIZE], struct linea_telefonica *lineas) {
  for (short unsigned indice = 0; indice < 100; indice++) {
    if (strcmp(lineas[indice].curp, curp) == 0) {
      return lineas[indice];
    }
  }

  puts("");
  puts("^^^ No se encontraron registros con ese CURP");
  struct linea_telefonica linea_vacia = {0};
  return linea_vacia;
}

// Gestiona la búsqueda de una línea, permitiendo al usuario elegir un criterio.
void buscar_linea(struct linea_telefonica *lineas) {
  short unsigned opcion, linea_id;
  char curp[CURP_SIZE];
  puts("--- Buscar línea telefónica ---");
  puts("Criterios de búsqueda");
  puts("1. ID");
  puts("2. CURP");
  puts("-------------------------------------");
  printf("Seleccione el criterio de búsqueda: ");
  capturar_entrada_usuario_numero("%hu", &opcion);
  puts("");

  struct linea_telefonica linea;
  switch (opcion) {
    case 1:
      puts("----------------");
      printf("Ingrese el ID: ");
      capturar_entrada_usuario_numero("%hu", &linea_id);
      linea = buscar_linea_por_id(&linea_id, lineas);
      mostrar_informacion_linea(&linea);
      break;
    case 2:
      puts("-----------------------------------");
      printf("Ingrese la CURP: ");
      capturar_entrada_usuario_texto(" %18s", curp);
      linea = buscar_linea_por_curp(curp, lineas);
      mostrar_informacion_linea(&linea);
      break;
    default:
      puts("!!! Error al seleccionar el criterio de búsqueda");
      break;
  }
  puts("");
}

// Elimina una línea del registro, identificada por su ID.
void eliminar_linea_por_id(short unsigned *linea_id, struct linea_telefonica *lineas) {
  if (*linea_id < 1 || *linea_id > LINEAS_SIZE) {
    return;
  }

  char opcion;
  puts("----------------------------------");
  printf("¿Desea eliminar la línea? [s/N]: ");
  capturar_entrada_usuario_texto(" %c", &opcion);
  if (opcion != 's' && opcion != 'S') {
    puts("");
    puts("^^^ Usted seleccionó la opción de no borrar la línea.");
    return;
  }

  short unsigned indice = *linea_id - 1;
  lineas[indice].id = 0;
  strcpy(lineas[indice].curp, "");
  strcpy(lineas[indice].nombre, "");
  strcpy(lineas[indice].numero, "");
  strcpy(lineas[indice].operadora, "");
  puts("****************************");
  puts("*   Se eliminó la línea!   *");
  puts("****************************");
}

// Gestiona la eliminación de una línea, buscándola primero por ID o CURP.
void eliminar_linea(short unsigned *linea_id, struct linea_telefonica *lineas) {
  short unsigned opcion, linea_id_entrada;
  char curp[CURP_SIZE];
  puts("--- Eliminar línea telefónica ---");
  puts("Criterios de búsqueda");
  puts("1. ID");
  puts("2. CURP");
  puts("-------------------------------------");
  printf("Seleccione el criterio de búsqueda: ");
  capturar_entrada_usuario_numero("%hu", &opcion);
  puts("");

  struct linea_telefonica linea;
  switch (opcion) {
    case 1:
      puts("----------------");
      printf("Ingrese el ID: ");
      capturar_entrada_usuario_numero("%hu", &linea_id_entrada);
      linea = buscar_linea_por_id(&linea_id_entrada, lineas);
      mostrar_informacion_linea(&linea);
      eliminar_linea_por_id(&linea.id, lineas);
      mostrar_registros_disponibles(linea_id);
      break;
    case 2:
      puts("-----------------------------------");
      printf("Ingrese la CURP: ");
      capturar_entrada_usuario_texto(" %18s", curp);
      linea = buscar_linea_por_curp(curp, lineas);
      mostrar_informacion_linea(&linea);
      eliminar_linea_por_id(&linea.id, lineas);
      mostrar_registros_disponibles(linea_id);
      break;
    default:
      puts("!!! Error al seleccionar el criterio de búsqueda");
      break;
  }
  puts("");
}

// Muestra cuántos registros de líneas nuevos se pueden crear.
void mostrar_registros_disponibles(short unsigned *linea_id) {
  puts("");
  printf("Líneas restantes: %hu", LINEAS_SIZE - *linea_id);
  puts("");
}

// Muestra un mensaje de despedida al salir del programa.
void mostrar_finalizar() {
  puts("Gracias por su preferencia.");
}

// Informa al usuario que la opción seleccionada no es válida.
void mostrar_opcion_invalida() {
  puts("Opción inválida, intente de nuevo.");
  puts("");
}

// Captura un valor numérico del usuario y limpia el búfer de entrada.
void capturar_entrada_usuario_numero(char *opciones, short unsigned *valor) {
  int c;
  scanf(opciones, valor);
  while ((c = getchar()) != '\n' && c != EOF);
}

// Captura una cadena de texto del usuario y limpia el búfer de entrada.
void capturar_entrada_usuario_texto(char *opciones, char *valor) {
  int c;
  scanf(opciones, valor);
  while ((c = getchar()) != '\n' && c != EOF);
}

// Función principal que ejecuta el ciclo del menú de opciones.
int main(void) {
  mostrar_encabezado();

  short unsigned linea_id = 0;
  short unsigned opcion;
  do {
    seleccionar_opcion_menu(&opcion);

    switch (opcion) {
      case 1:
        registrar_linea(&linea_id, &lineas_telefonicas[linea_id]);
        break;
      case 2:
        buscar_linea(lineas_telefonicas);
        break;
      case 3:
        eliminar_linea(&linea_id, lineas_telefonicas);
        break;
      case 4:
        mostrar_finalizar();
        break;
      default:
        mostrar_opcion_invalida();
        break;
    }

    puts("");
  } while (opcion != 4);

  // Devuelve cero, indicando que la ejecución fue exitosa
  return 0;
}
