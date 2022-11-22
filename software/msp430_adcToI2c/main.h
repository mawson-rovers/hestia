#ifndef MAIN_H
#define MAIN_H

// GPIO PINS
#define LED_GREEN 0x04
#define LED_YELLOW 0x08
#define HEATER_PIN 0x80

// I2C commands
#define COMMAND_SENSOR_LOW          0x01
#define COMMAND_SENSOR_HIGH         0x08
#define COMMAND_WRITE_HEATER_MODE   0x40
#define COMMAND_TARGET_TEMP         0x41
#define COMMAND_TARGET_SENSOR       0x42
#define COMMAND_WRITE_PWM_FREQ      0x43
#define COMMAND_READ_HEATER_MODE    0x48
#define COMMAND_READ_PWM_FREQ       0x49
#define COMMAND_RESET               0x50

// heater modes
#define HEATER_MODE_OFF 0x00
#define HEATER_MODE_PID 0x01
#define HEATER_MODE_PWM 0x02

#define HEATER_PWM_FREQ_DEFAULT 50

// ADC
#define ADC_MIN_VALUE 0x10
#define ADC_UNKNOWN_VALUE 0xffff


/* Initialized the software state machine according to the received cmd
 *
 * cmd: The command/register address received
 * */
void heater_proccess();
void process_cmd_tx(unsigned char cmd);
void I2C_Slave_ProcessCMD(unsigned char *message, uint16_t length);
void initGPIO();
void set_adc_channel(int channel);
void initClockTo16MHz();
void initADC();



#endif
