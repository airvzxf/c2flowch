#include <stdio.h>

int main(void) {
    int temperature = 21, humidity = 40, pressure = 1013;
    if (temperature > 18 && temperature < 26 && humidity >= 30 && humidity <= 60) {
        printf("comfortable\n");
    }
    while (pressure > 1000 && pressure < 1030 && temperature != 0 || humidity == 100) {
        pressure--;
    }
    if (pressure_sensor_is_calibrated_and_ready_for_a_new_measurement_cycle) {
        printf("ready\n");
    }
    return 0;
}
