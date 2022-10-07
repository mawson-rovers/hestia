#ifndef MAIN_H
#define MAIN_H

// GPIO PINS
#define LED_GREEN 0x04
#define LED_YELLOW 0x08
#define HEATER_PIN 0x80

// heater modes
#define HEATER_MODE_OFF 0x00
#define HEATER_MODE_PID 0x01
#define HEATER_MODE_PWM 0x02

#define HEATER_PWM_FREQ 50

// ADC
#define ADC_MIN_VALUE 0x10
#define ADC_UNKNOWN_VALUE 0xffff


/* Initialized the software state machine according to the received cmd
 *
 * cmd: The command/register address received
 * */
void heater_proccess();
void adc_proccess(unsigned char cmd);
void I2C_Slave_ProcessCMD(unsigned char *message, uint16_t length);
void initGPIO();
void set_adc_channel(int channel);
void initClockTo16MHz();
void initADC();



#endif
