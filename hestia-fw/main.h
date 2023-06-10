#ifndef MAIN_H
#define MAIN_H

// GPIO PINS
#define LED_YELLOW BIT2  // P5.2
#define LED_GREEN  BIT3  // P5.3
#define LED_BLUE   BIT4  // P5.4
#define HEATER_PIN BIT7  // P1.7

// I2C commands
#define COMMAND_READ_SENSOR_LOW      0x01
#define COMMAND_READ_SENSOR_HIGH     0x08
#define COMMAND_READ_HEATER_MODE     0x20
#define COMMAND_READ_TARGET_TEMP     0x21
#define COMMAND_READ_TARGET_SENSOR   0x22
#define COMMAND_READ_PWM_FREQ        0x23
#define COMMAND_WRITE_HEATER_MODE    0x40
#define COMMAND_WRITE_TARGET_TEMP    0x41
#define COMMAND_WRITE_TARGET_SENSOR  0x42
#define COMMAND_WRITE_PWM_FREQ       0x43
#define COMMAND_RESET                0x50

// heater modes
#define HEATER_MODE_OFF 0x00
#define HEATER_MODE_PID 0x01
#define HEATER_MODE_PWM 0x02

#define HEATER_PWM_FREQ_DEFAULT 255

// ADC
#define ADC_SENSOR_COUNT 8
#define ADC_MIN_VALUE 0x0010
#define ADC_MAX_VALUE 0x0FFF
#define ADC_UNKNOWN_VALUE 0xffff

// ADC values for TH1 - thermistor model NB21K00103
#define TEMP_80C 3561
#define TEMP_70C 3406
#define TEMP_60C 3204
#define TEMP_50C 2947
#define TEMP_40C 2629
#define TEMP_25C 2030
#define TEMP_0C  1012

void initGPIO();
void initClockTo16MHz();
void initADC();
void initTimer();

void process_cmd_tx(unsigned char cmd);
void transmit_uint(unsigned int value);

void I2C_Slave_ProcessCMD(unsigned char *message, uint16_t length);
void heater_process();

#endif
