#!/usr/bin/env bash
#
# pipeline.bash - Procesa todos los archivos C a diagramas de flujo ASCII
#
set -euo pipefail

# Obtener el directorio donde reside el script y cambiar al directorio raíz del proyecto
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Configuración de rutas (ahora relativas a la raíz del proyecto)
C_DIR="resources/c"
YAML_DIR="resources/yaml"
ASCII_DIR="resources/ascii"

# Crear directorios de salida si no existen
mkdir -p "$YAML_DIR"
mkdir -p "$ASCII_DIR"

echo "=== Iniciando Pipeline de C2Flowch ==="

# Buscar todos los archivos .c en el directorio de recursos
for c_file in "$C_DIR"/*.c; do
    # Verificar si existen archivos (manejo de caso glob vacío)
    [ -e "$c_file" ] || continue

    # Extraer el nombre base sin extensión
    base_name=$(basename "$c_file" .c)

    yaml_output="$YAML_DIR/$base_name.yaml"
    ascii_output="$ASCII_DIR/$base_name.txt"

    echo "------------------------------------------------"
    echo "Procesando: $base_name"

    # 1. Parseo: C -> YAML
    if time cargo run --bin c2flowch --quiet -- parse "$c_file" --output "$yaml_output"; then
        echo "  [OK] Parseo completado -> $yaml_output"
    else
        echo "  [ERROR] Falló el parseo de $c_file"
        continue
    fi

    # 2. Renderizado: YAML -> ASCII
    if time cargo run --bin c2flowch --quiet -- render "$yaml_output" --output "$ascii_output"; then
        echo "  [OK] Renderizado completado -> $ascii_output"
    else
        echo "  [ERROR] Falló el renderizado de $yaml_output"
        continue
    fi
done

echo "------------------------------------------------"
echo "=== Proceso finalizado con éxito ==="
